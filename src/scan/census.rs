use anyhow::{Context, Result};
use chrono::{DateTime, Local, NaiveDate, TimeZone};
use std::path::Path;
use walkdir::WalkDir;

const SKIP_DIRS: &[&str] = &[
    "node_modules",
    "target",
    ".git",
    "__pycache__",
    "venv",
    ".venv",
    "dist",
    "build",
    ".next",
    "vendor",
];

use crate::index::{focus::load_focus, store};
use crate::model::project::{sync_pinned_flags, ProjectEntry, ProjectKind};
use crate::scan::{artifacts, discover, fingerprint};

type ProjectSummary = (Option<DateTime<Local>>, u64, u32, (bool, bool, bool, bool));

/// Main census entry point - orchestrates the pipeline stages
pub fn run_census(
    root_str: &str,
    depth: usize,
    since: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let root = Path::new(root_str);
    let focus = load_focus(root).unwrap_or_default();
    let cutoff = parse_cutoff(since)?;

    // Pipeline stages
    let discovered = discover::discover_projects(root, depth)?;
    let mut projects = build_project_entries(root, &discovered, cutoff)?;

    // Post-processing
    sync_pinned_flags(&mut projects, &focus.pinned);
    mark_duplicates_by_fingerprint(&mut projects);

    // Persist and output
    let idx = save_index(root, root_str, projects)?;
    output_result(root, &idx, json_output);

    Ok(())
}

/// Build ProjectEntry list from discovered projects
fn build_project_entries(
    root: &Path,
    discovered: &[discover::DiscoveredProject],
    cutoff: Option<DateTime<Local>>,
) -> Result<Vec<ProjectEntry>> {
    let mut projects = Vec::with_capacity(discovered.len());

    for dp in discovered {
        let rel = relpath(root, &dp.root);
        let (latest, size_bytes, artifact_count, hints) = summarize_project(&dp.root, cutoff)?;
        let fp = fingerprint::fingerprint_project(&dp.root)?;
        let kind = classify_project(&rel, latest, cutoff);

        projects.push(ProjectEntry {
            path: rel,
            kind,
            pinned: false, // Will be set by sync_pinned_flags
            latest_mtime: latest,
            size_bytes: Some(size_bytes),
            artifact_count,
            has_git: hints.0,
            has_rust: hints.1,
            has_node: hints.2,
            has_python: hints.3,
            fingerprint: fp,
        });
    }

    Ok(projects)
}

/// Classify project kind based on path and activity
fn classify_project(
    rel_path: &str,
    latest: Option<DateTime<Local>>,
    cutoff: Option<DateTime<Local>>,
) -> ProjectKind {
    let lrel = rel_path.to_lowercase();

    // Path-based classification takes precedence
    if lrel.contains("backup") || lrel.contains("copy") || lrel.contains("old") {
        return ProjectKind::BackupDuplicate;
    }
    if lrel.contains("sandbox") || lrel.contains("experiment") || lrel.contains("try") {
        return ProjectKind::Experimental;
    }

    // Activity-based classification
    if let Some(lm) = latest {
        if cutoff.map(|c| lm >= c).unwrap_or(true) {
            return ProjectKind::ActiveStandalone;
        }
    }

    ProjectKind::Standalone
}

/// Save projects to index file
fn save_index(
    root: &Path,
    root_str: &str,
    projects: Vec<ProjectEntry>,
) -> Result<store::OrbitIndex> {
    let mut idx = store::load(root).unwrap_or_default();
    idx.root = root_str.to_string();
    idx.generated_at = Some(Local::now());
    idx.projects = projects;
    store::save(root, &idx)?;
    Ok(idx)
}

/// Output census result
fn output_result(root: &Path, idx: &store::OrbitIndex, json_output: bool) {
    if json_output {
        println!(
            "{}",
            serde_json::json!({
                "status": "complete",
                "index_path": store::index_path(root).display().to_string(),
                "project_count": idx.projects.len()
            })
        );
    } else {
        println!(
            "Census complete. Updated {}",
            store::index_path(root).display()
        );
    }
}

fn parse_cutoff(since: Option<&str>) -> Result<Option<DateTime<Local>>> {
    match since {
        None => Ok(None),
        Some(s) => {
            let d = NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .with_context(|| format!("Invalid date '{}': expected YYYY-MM-DD format", s))?;
            let dt = d
                .and_hms_opt(0, 0, 0)
                .ok_or_else(|| anyhow::anyhow!("Invalid time for date {}", s))?;
            let local = Local
                .from_local_datetime(&dt)
                .single()
                .ok_or_else(|| anyhow::anyhow!("Ambiguous local time for date {}", s))?;
            Ok(Some(local))
        }
    }
}

fn should_skip(entry: &walkdir::DirEntry) -> bool {
    entry.file_type().is_dir()
        && entry
            .file_name()
            .to_str()
            .map(|s| SKIP_DIRS.contains(&s))
            .unwrap_or(false)
}

fn relpath(root: &Path, p: &Path) -> String {
    let rel = p.strip_prefix(root).unwrap_or(p);
    let s = rel.to_string_lossy().replace("\\", "/");
    if s.is_empty() {
        ".".into()
    } else {
        s
    }
}

fn summarize_project(
    project_root: &Path,
    cutoff: Option<DateTime<Local>>,
) -> Result<ProjectSummary> {
    let mut latest: Option<DateTime<Local>> = None;
    let mut size_bytes: u64 = 0;
    let mut artifact_count: u32 = 0;
    let mut has_git = false;
    let mut has_rust = false;
    let mut has_node = false;
    let mut has_python = false;

    for entry in WalkDir::new(project_root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !should_skip(e))
    {
        let entry =
            entry.with_context(|| format!("Failed to read entry in {}", project_root.display()))?;
        let p = entry.path();

        if entry.file_type().is_dir() && entry.file_name() == ".git" {
            has_git = true;
        }

        if entry.file_type().is_file() {
            if artifacts::is_artifact_name(p) {
                artifact_count += 1;
            }
            if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                if name == "Cargo.toml" {
                    has_rust = true;
                }
                if name == "package.json" {
                    has_node = true;
                }
                if name == "pyproject.toml" || name == "pytest.ini" {
                    has_python = true;
                }
            }
            let md = entry
                .metadata()
                .with_context(|| format!("Failed to read metadata for {}", p.display()))?;
            size_bytes += md.len();
            if let Ok(m) = md.modified() {
                let dt: DateTime<Local> = m.into();
                if cutoff.map(|c| dt >= c).unwrap_or(true) {
                    latest = Some(match latest {
                        None => dt,
                        Some(prev) => {
                            if dt > prev {
                                dt
                            } else {
                                prev
                            }
                        }
                    });
                }
            }
        }
    }
    Ok((
        latest,
        size_bytes,
        artifact_count,
        (has_git, has_rust, has_node, has_python),
    ))
}

fn mark_duplicates_by_fingerprint(projects: &mut [ProjectEntry]) {
    use std::collections::HashMap;
    let mut map: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, p) in projects.iter().enumerate() {
        if let Some(fp) = &p.fingerprint {
            map.entry(fp.clone()).or_default().push(i);
        }
    }
    for (_fp, idxs) in map {
        if idxs.len() >= 2 {
            for &i in &idxs {
                if !projects[i].pinned && !matches!(projects[i].kind, ProjectKind::Experimental) {
                    projects[i].kind = ProjectKind::BackupDuplicate;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{classify_project, mark_duplicates_by_fingerprint, parse_cutoff, relpath};
    use crate::model::project::{ProjectEntry, ProjectKind};
    use chrono::{Local, TimeZone};
    use std::path::Path;

    #[test]
    fn relpath_handles_root_and_child() {
        let root = Path::new("/tmp/orbit_root");
        assert_eq!(relpath(root, root), ".");
        assert_eq!(
            relpath(root, Path::new("/tmp/orbit_root/project1")),
            "project1"
        );
    }

    #[test]
    fn parse_cutoff_none_returns_none() {
        let result = parse_cutoff(None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn parse_cutoff_valid_date_returns_datetime() {
        let result = parse_cutoff(Some("2024-01-15")).unwrap();
        assert!(result.is_some());
        let dt = result.unwrap();
        assert_eq!(dt.format("%Y-%m-%d").to_string(), "2024-01-15");
    }

    #[test]
    fn parse_cutoff_invalid_date_returns_error() {
        let result = parse_cutoff(Some("not-a-date"));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Invalid date"));
        assert!(err.contains("not-a-date"));
    }

    #[test]
    fn parse_cutoff_wrong_format_returns_error() {
        let result = parse_cutoff(Some("01/15/2024"));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Invalid date"));
    }

    #[test]
    fn duplicates_are_demoted_unless_pinned_or_experimental() {
        let mut projects = vec![
            ProjectEntry {
                path: "a".into(),
                kind: ProjectKind::Standalone,
                pinned: false,
                latest_mtime: None,
                size_bytes: None,
                artifact_count: 0,
                has_git: false,
                has_rust: false,
                has_node: false,
                has_python: false,
                fingerprint: Some("fp".into()),
            },
            ProjectEntry {
                path: "b".into(),
                kind: ProjectKind::Standalone,
                pinned: false,
                latest_mtime: None,
                size_bytes: None,
                artifact_count: 0,
                has_git: false,
                has_rust: false,
                has_node: false,
                has_python: false,
                fingerprint: Some("fp".into()),
            },
            ProjectEntry {
                path: "c".into(),
                kind: ProjectKind::Standalone,
                pinned: true,
                latest_mtime: None,
                size_bytes: None,
                artifact_count: 0,
                has_git: false,
                has_rust: false,
                has_node: false,
                has_python: false,
                fingerprint: Some("fp".into()),
            },
        ];

        mark_duplicates_by_fingerprint(&mut projects);

        assert!(matches!(projects[0].kind, ProjectKind::BackupDuplicate));
        assert!(matches!(projects[1].kind, ProjectKind::BackupDuplicate));
        assert!(matches!(projects[2].kind, ProjectKind::Standalone));
    }

    #[test]
    fn classify_project_by_path_patterns() {
        // Backup patterns
        assert!(matches!(
            classify_project("my_backup", None, None),
            ProjectKind::BackupDuplicate
        ));
        assert!(matches!(
            classify_project("project_old", None, None),
            ProjectKind::BackupDuplicate
        ));
        assert!(matches!(
            classify_project("copy_of_project", None, None),
            ProjectKind::BackupDuplicate
        ));

        // Experimental patterns
        assert!(matches!(
            classify_project("sandbox_test", None, None),
            ProjectKind::Experimental
        ));
        assert!(matches!(
            classify_project("experiment_v2", None, None),
            ProjectKind::Experimental
        ));
        assert!(matches!(
            classify_project("try_something", None, None),
            ProjectKind::Experimental
        ));

        // Regular project defaults to Standalone
        assert!(matches!(
            classify_project("myproject", None, None),
            ProjectKind::Standalone
        ));
    }

    #[test]
    fn classify_project_by_activity() {
        let now = Local::now();
        let old_cutoff = Local.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();

        // Active project (modified after cutoff)
        assert!(matches!(
            classify_project("myproject", Some(now), Some(old_cutoff)),
            ProjectKind::ActiveStandalone
        ));

        // Inactive project (no mtime)
        assert!(matches!(
            classify_project("myproject", None, Some(old_cutoff)),
            ProjectKind::Standalone
        ));

        // Path patterns take precedence over activity
        assert!(matches!(
            classify_project("backup_project", Some(now), Some(old_cutoff)),
            ProjectKind::BackupDuplicate
        ));
    }
}
