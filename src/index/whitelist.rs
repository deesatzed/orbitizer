use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::index::store::atomic_write;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Whitelist {
    pub version: String,
    pub paths: Vec<String>,
}

fn whitelist_path(root: &Path) -> PathBuf {
    root.join(".orbit").join("whitelist.json")
}

pub fn load_whitelist(root: &Path) -> Result<Whitelist> {
    let p = whitelist_path(root);
    if !p.exists() {
        return Ok(Whitelist {
            version: "0.1".into(),
            paths: vec![],
        });
    }
    let content =
        fs::read_to_string(&p).with_context(|| format!("Failed to read {}", p.display()))?;
    serde_json::from_str(&content).with_context(|| format!("Failed to parse {}", p.display()))
}

pub fn save_whitelist(root: &Path, wl: &Whitelist) -> Result<()> {
    let p = whitelist_path(root);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }
    let content = serde_json::to_string_pretty(wl).context("Failed to serialize whitelist")?;
    atomic_write(&p, &content)
}

/// Check if a path should be protected (whitelist)
pub fn is_protected(path: &Path, wl: &Whitelist) -> bool {
    wl.paths.iter().any(|p| {
        let wp = PathBuf::from(p);
        path.starts_with(&wp)
    })
}

pub fn handle_whitelist(
    root_str: &str,
    add: Option<String>,
    remove: Option<String>,
    list: bool,
    json_output: bool,
) -> Result<()> {
    let root = Path::new(root_str);
    let mut wl = load_whitelist(root)?;
    if list {
        if json_output {
            println!("{}", serde_json::json!({ "paths": wl.paths }));
        } else {
            for p in &wl.paths {
                println!("{p}");
            }
        }
        return Ok(());
    }
    if let Some(a) = add {
        let full = root.join(&a);
        if !full.exists() {
            anyhow::bail!("Path does not exist: {}", full.display());
        }
        if !wl.paths.iter().any(|p| p == &a) {
            wl.paths.push(a);
            wl.paths.sort();
        }
        save_whitelist(root, &wl)?;
        if json_output {
            println!("{}", serde_json::json!({ "status": "updated", "paths": wl.paths }));
        } else {
            println!("Whitelist updated.");
        }
    }
    if let Some(r) = remove {
        wl.paths.retain(|p| p != &r);
        save_whitelist(root, &wl)?;
        if json_output {
            println!("{}", serde_json::json!({ "status": "updated", "paths": wl.paths }));
        } else {
            println!("Whitelist updated.");
        }
    }
    Ok(())
}
