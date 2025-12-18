use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct DiscoveredProject {
    pub root: PathBuf,
    pub markers: Vec<String>,
}

/// Cached glob matcher for project marker files - built once and reused
static MARKER_GLOBSET: Lazy<GlobSet> = Lazy::new(|| {
    let mut b = GlobSetBuilder::new();
    // These patterns are known valid, so expect() is safe
    b.add(Glob::new("**/CLAUDE.md").expect("valid glob"));
    b.add(Glob::new("**/AGENT.md").expect("valid glob"));
    b.add(Glob::new("**/HANDOFF*.md").expect("valid glob"));
    b.add(Glob::new("**/*handoff*.md").expect("valid glob"));
    b.add(Glob::new("**/*export*.md").expect("valid glob"));
    b.add(Glob::new("**/*session*.md").expect("valid glob"));
    b.add(Glob::new("**/README.md").expect("valid glob"));
    b.add(Glob::new("**/Cargo.toml").expect("valid glob"));
    b.add(Glob::new("**/pyproject.toml").expect("valid glob"));
    b.add(Glob::new("**/package.json").expect("valid glob"));
    b.build().expect("valid globset")
});

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

fn should_skip(entry: &walkdir::DirEntry) -> bool {
    entry.file_type().is_dir()
        && entry
            .file_name()
            .to_str()
            .map(|s| SKIP_DIRS.contains(&s))
            .unwrap_or(false)
}

pub fn discover_projects(root: &Path, depth: usize) -> Result<Vec<DiscoveredProject>> {
    let gs = &*MARKER_GLOBSET;
    let mut projects: Vec<DiscoveredProject> = vec![];

    for entry in WalkDir::new(root)
        .follow_links(false)
        .max_depth(depth)
        .into_iter()
        .filter_entry(|e| !should_skip(e))
    {
        let entry = entry.with_context(|| format!("Failed to read entry in {}", root.display()))?;
        let p = entry.path();

        if entry.file_type().is_dir() && entry.file_name() == ".git" {
            upsert(
                &mut projects,
                p.parent().unwrap_or(root).to_path_buf(),
                ".git".into(),
            );
            continue;
        }
        if entry.file_type().is_file() {
            let rel = p.strip_prefix(root).unwrap_or(p);
            if gs.is_match(rel) {
                upsert(
                    &mut projects,
                    p.parent().unwrap_or(root).to_path_buf(),
                    rel.to_string_lossy().to_string(),
                );
            }
        }
    }
    projects.sort_by_key(|d| d.root.clone());
    Ok(projects)
}

fn upsert(projects: &mut Vec<DiscoveredProject>, root: PathBuf, marker: String) {
    if let Some(p) = projects.iter_mut().find(|x| x.root == root) {
        if !p.markers.iter().any(|m| m == &marker) {
            p.markers.push(marker);
        }
    } else {
        projects.push(DiscoveredProject {
            root,
            markers: vec![marker],
        });
    }
}
