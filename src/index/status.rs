use anyhow::Result;
use std::path::Path;

use crate::index::{focus::load_focus, store};
use crate::model::project::ProjectKind;

pub fn print_status(root_str: &str, json_output: bool) -> Result<()> {
    let root = Path::new(root_str);
    let idx = store::load(root)?;
    let focus = load_focus(root).unwrap_or_default();

    let total = idx.projects.len();
    let pinned = focus.pinned.len();
    let active = idx
        .projects
        .iter()
        .filter(|p| matches!(p.kind, ProjectKind::ActiveStandalone))
        .count();
    let backups = idx
        .projects
        .iter()
        .filter(|p| matches!(p.kind, ProjectKind::BackupDuplicate))
        .count();

    if json_output {
        println!(
            "{}",
            serde_json::json!({
                "root": root.display().to_string(),
                "indexed_projects": total,
                "active": active,
                "backup_duplicate": backups,
                "pinned": pinned,
                "index_path": store::index_path(root).display().to_string()
            })
        );
    } else {
        println!("Orbit status");
        println!("  Root: {}", root.display());
        println!("  Indexed projects: {}", total);
        println!("  Active: {}", active);
        println!("  Backup/Duplicate: {}", backups);
        println!("  Pinned: {}", pinned);
        println!("  Index: {}", store::index_path(root).display());
    }
    Ok(())
}
