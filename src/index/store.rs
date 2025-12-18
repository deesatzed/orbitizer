use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::model::project::ProjectEntry;

#[derive(Debug, Serialize, Deserialize)]
pub struct OrbitIndex {
    pub version: String,
    pub root: String,
    pub generated_at: Option<DateTime<Local>>,
    pub projects: Vec<ProjectEntry>,
}

impl Default for OrbitIndex {
    fn default() -> Self {
        Self {
            version: "0.5".into(),
            root: ".".into(),
            generated_at: None,
            projects: vec![],
        }
    }
}

pub fn index_path(root: &Path) -> std::path::PathBuf {
    root.join(".orbit").join("index.json")
}

pub fn load(root: &Path) -> Result<OrbitIndex> {
    let p = index_path(root);
    if !p.exists() {
        return Ok(OrbitIndex::default());
    }
    let content =
        fs::read_to_string(&p).with_context(|| format!("Failed to read {}", p.display()))?;
    serde_json::from_str(&content).with_context(|| format!("Failed to parse {}", p.display()))
}

pub fn save(root: &Path, idx: &OrbitIndex) -> Result<()> {
    let p = index_path(root);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }
    let content = serde_json::to_string_pretty(idx).context("Failed to serialize index")?;
    atomic_write(&p, &content)
}

/// Write-then-rename for atomic file updates
pub fn atomic_write(path: &Path, content: &str) -> Result<()> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, content).with_context(|| format!("Failed to write {}", tmp.display()))?;
    fs::rename(&tmp, path)
        .with_context(|| format!("Failed to rename {} to {}", tmp.display(), path.display()))?;
    Ok(())
}
