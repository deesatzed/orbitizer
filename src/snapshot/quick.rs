use anyhow::{Context, Result};
use chrono::Local;
use std::fs;
use std::path::Path;

use crate::export::md::render_md;
use crate::index::{focus::load_focus, store};

pub fn snapshot_pinned(root_str: &str, label: Option<&str>) -> Result<()> {
    let root = Path::new(root_str);
    let focus = load_focus(root).unwrap_or_default();
    let idx = store::load(root).unwrap_or_default();

    let ts = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let lab = label.unwrap_or("snapshot");
    let snap_dir = root
        .join(".orbit")
        .join("snapshots")
        .join(format!("{}_{}", ts, sanitize(lab)));
    fs::create_dir_all(&snap_dir)
        .with_context(|| format!("Failed to create snapshot directory {}", snap_dir.display()))?;

    let focus_path = snap_dir.join("focus.json");
    fs::write(
        &focus_path,
        serde_json::to_string_pretty(&focus).context("Failed to serialize focus")?,
    )
    .with_context(|| format!("Failed to write {}", focus_path.display()))?;

    let index_path = snap_dir.join("index.json");
    fs::write(
        &index_path,
        serde_json::to_string_pretty(&idx).context("Failed to serialize index")?,
    )
    .with_context(|| format!("Failed to write {}", index_path.display()))?;

    let artifacts_dir = snap_dir.join("artifacts");
    fs::create_dir_all(&artifacts_dir).with_context(|| {
        format!(
            "Failed to create artifacts directory {}",
            artifacts_dir.display()
        )
    })?;
    for p in focus.pinned.iter() {
        let pr = root.join(p);
        if pr.is_dir() {
            copy_md_artifacts(&pr, &artifacts_dir, p)?;
        }
    }

    let summary_path = snap_dir.join("summary.md");
    fs::write(&summary_path, render_md(&idx)?)
        .with_context(|| format!("Failed to write {}", summary_path.display()))?;
    println!("Snapshot created: {}", snap_dir.display());
    Ok(())
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

fn copy_md_artifacts(project_root: &Path, out_dir: &Path, rel: &str) -> Result<()> {
    for entry in walkdir::WalkDir::new(project_root)
        .max_depth(6)
        .follow_links(false)
    {
        let entry =
            entry.with_context(|| format!("Failed to read entry in {}", project_root.display()))?;
        if entry.file_type().is_file() {
            let p = entry.path();
            if p.extension()
                .and_then(|s| s.to_str())
                .map(|e| e.eq_ignore_ascii_case("md"))
                .unwrap_or(false)
            {
                let name = p.file_name().unwrap().to_string_lossy().to_string();
                let lname = name.to_lowercase();
                if lname.contains("handoff")
                    || lname.contains("agent")
                    || lname.contains("claude")
                    || lname.contains("export")
                    || lname.contains("plan")
                    || lname.contains("session")
                    || lname.contains("summary")
                    || lname.contains("roadmap")
                    || lname.contains("prompt")
                {
                    let target = out_dir.join(format!("{}__{}", sanitize(rel), name));
                    fs::copy(p, &target).with_context(|| {
                        format!("Failed to copy {} to {}", p.display(), target.display())
                    })?;
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::sanitize;

    #[test]
    fn sanitize_replaces_non_alnum() {
        assert_eq!(sanitize("my label!"), "my_label_");
        assert_eq!(sanitize("abc123"), "abc123");
        assert_eq!(sanitize("a/b\\c"), "a_b_c");
    }
}
