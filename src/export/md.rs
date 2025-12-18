use crate::index::store::OrbitIndex;
use crate::model::project::ProjectKind;
use anyhow::Result;

pub fn render_md(idx: &OrbitIndex) -> Result<String> {
    let mut s = String::new();
    s.push_str("# Orbit Census Summary\n\n");
    s.push_str(&format!("- Root: `{}`\n", idx.root));
    if let Some(dt) = &idx.generated_at {
        s.push_str(&format!("- Generated: `{}`\n", dt.to_rfc3339()));
    }
    s.push_str(&format!("- Projects: `{}`\n\n", idx.projects.len()));

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
    let pinned = idx.projects.iter().filter(|p| p.pinned).count();
    s.push_str("## Totals\n");
    s.push_str(&format!(
        "- Active: {}\n- Backup/Duplicate: {}\n- Pinned: {}\n\n",
        active, backups, pinned
    ));

    s.push_str("## Projects (most recent)\n");
    let mut ps = idx.projects.clone();
    ps.sort_by_key(|p| p.latest_mtime);
    ps.reverse();
    for p in ps.iter().take(50) {
        s.push_str(&format!(
            "- {} {} — {:?}\n",
            if p.pinned { "★" } else { " " },
            p.path,
            p.kind
        ));
    }
    Ok(s)
}
