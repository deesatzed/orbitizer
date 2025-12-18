use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::index::store;

pub fn export_all(root_str: &str) -> Result<()> {
    let root = Path::new(root_str);
    let idx = store::load(root)?;
    let out = root.join(".orbit").join("exports");
    fs::create_dir_all(&out)
        .with_context(|| format!("Failed to create exports directory {}", out.display()))?;

    let md_path = out.join("summary.md");
    fs::write(&md_path, crate::export::md::render_md(&idx)?)
        .with_context(|| format!("Failed to write {}", md_path.display()))?;

    let json_path = out.join("index.json");
    fs::write(
        &json_path,
        serde_json::to_string_pretty(&idx).context("Failed to serialize index")?,
    )
    .with_context(|| format!("Failed to write {}", json_path.display()))?;

    let mut wtr = csv::Writer::from_writer(vec![]);
    wtr.write_record([
        "path",
        "kind",
        "pinned",
        "latest_mtime",
        "size_bytes",
        "artifact_count",
        "has_git",
        "has_rust",
        "has_node",
        "has_python",
        "fingerprint",
    ])?;
    for p in &idx.projects {
        wtr.write_record([
            p.path.as_str(),
            format!("{:?}", p.kind).as_str(),
            p.pinned.to_string().as_str(),
            p.latest_mtime
                .map(|d| d.to_rfc3339())
                .unwrap_or_default()
                .as_str(),
            p.size_bytes
                .map(|n| n.to_string())
                .unwrap_or_default()
                .as_str(),
            p.artifact_count.to_string().as_str(),
            p.has_git.to_string().as_str(),
            p.has_rust.to_string().as_str(),
            p.has_node.to_string().as_str(),
            p.has_python.to_string().as_str(),
            p.fingerprint.clone().unwrap_or_default().as_str(),
        ])?;
    }
    let csv_path = out.join("index.csv");
    fs::write(&csv_path, wtr.into_inner()?)
        .with_context(|| format!("Failed to write {}", csv_path.display()))?;
    println!("Exported to {}", out.display());
    Ok(())
}
