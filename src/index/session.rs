use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::index::store::atomic_write;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct OrbitSession {
    pub version: u32,
    pub root: String,
    pub lens: Option<String>,
    pub search: Option<String>,
    pub selection: Option<String>,
    pub high_contrast: Option<bool>,
}

fn session_path() -> Option<PathBuf> {
    env::var("HOME").ok().map(|h| PathBuf::from(h).join(".orbit").join("session.json"))
}

pub fn load_session(root: &Path) -> Result<Option<OrbitSession>> {
    let Some(path) = session_path() else {
        return Ok(None);
    };
    if !path.exists() {
        return Ok(None);
    }
    let content =
        fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
    let session: OrbitSession =
        serde_json::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))?;

    // Basic root check to avoid applying sessions from another workspace
    if !session.root.is_empty() && session.root != root.to_string_lossy() {
        return Ok(None);
    }

    Ok(Some(session))
}

pub fn save_session(session: &OrbitSession) -> Result<()> {
    let Some(path) = session_path() else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }
    let content = serde_json::to_string_pretty(session).context("Failed to serialize session")?;
    atomic_write(&path, &content)
}
