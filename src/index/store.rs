use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

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

fn home_index_path() -> Option<PathBuf> {
    env::var("HOME").ok().map(|h| PathBuf::from(h).join(".orbit").join("index.json"))
}

pub fn index_path(root: &Path) -> std::path::PathBuf {
    // Legacy per-root index path (kept for compatibility/tests)
    root.join(".orbit").join("index.json")
}

pub fn load(root: &Path) -> Result<OrbitIndex> {
    // Prefer shared home-level index, fallback to legacy root/.orbit/index.json
    let candidates: Vec<PathBuf> = home_index_path()
        .into_iter()
        .chain(std::iter::once(index_path(root)))
        .collect();

    for p in candidates {
        if !p.exists() {
            continue;
        }
        let content =
            fs::read_to_string(&p).with_context(|| format!("Failed to read {}", p.display()))?;
        if let Ok(parsed) =
            serde_json::from_str::<OrbitIndex>(&content).with_context(|| format!("Failed to parse {}", p.display()))
        {
            return Ok(parsed);
        }
    }
    Ok(OrbitIndex::default())
}

pub fn save(root: &Path, idx: &OrbitIndex) -> Result<()> {
    let content = serde_json::to_string_pretty(idx).context("Failed to serialize index")?;

    // Primary: home-level shared index
    if let Some(home_path) = home_index_path() {
        if let Some(parent) = home_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }
        atomic_write(&home_path, &content)?;
    }

    // Legacy: root/.orbit/index.json (kept for compatibility/tests)
    let legacy = index_path(root);
    if let Some(parent) = legacy.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }
    atomic_write(&legacy, &content)
}

/// Write-then-rename for atomic file updates
pub fn atomic_write(path: &Path, content: &str) -> Result<()> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, content).with_context(|| format!("Failed to write {}", tmp.display()))?;
    fs::rename(&tmp, path)
        .with_context(|| format!("Failed to rename {} to {}", tmp.display(), path.display()))?;
    Ok(())
}
