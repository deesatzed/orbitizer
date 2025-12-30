use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::index::store::atomic_write;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Focus {
    pub pinned: Vec<String>,
}

fn home_focus_path() -> Option<PathBuf> {
    env::var("HOME").ok().map(|h| PathBuf::from(h).join(".orbit").join("focus.json"))
}

pub fn load_focus(root: &Path) -> Result<Focus> {
    let candidates: Vec<PathBuf> = home_focus_path()
        .into_iter()
        .chain(std::iter::once(root.join(".orbit").join("focus.json")))
        .collect();

    for p in candidates {
        if !p.exists() {
            continue;
        }
        let content =
            fs::read_to_string(&p).with_context(|| format!("Failed to read {}", p.display()))?;
        if let Ok(parsed) =
            serde_json::from_str::<Focus>(&content).with_context(|| format!("Failed to parse {}", p.display()))
        {
            return Ok(parsed);
        }
    }
    Ok(Focus::default())
}

pub fn save_focus(root: &Path, f: &Focus) -> Result<()> {
    let content = serde_json::to_string_pretty(f).context("Failed to serialize focus")?;

    // Primary: home-level shared file
    if let Some(home_path) = home_focus_path() {
        if let Some(parent) = home_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }
        atomic_write(&home_path, &content)?;
    }

    // Legacy: project-root .orbit/focus.json (best-effort)
    let legacy = root.join(".orbit").join("focus.json");
    if let Some(parent) = legacy.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }
    atomic_write(&legacy, &content)
}

pub fn handle_focus(
    root_str: &str,
    add: Option<String>,
    remove: Option<String>,
    list: bool,
    json_output: bool,
) -> Result<()> {
    let root = Path::new(root_str);
    let mut f = load_focus(root)?;
    if list {
        if json_output {
            println!("{}", serde_json::json!({ "pinned": f.pinned }));
        } else {
            for p in &f.pinned {
                println!("{}", p);
            }
        }
        return Ok(());
    }
    if let Some(a) = add {
        // Validate path exists before adding (P3-2)
        let full_path = root.join(&a);
        if !full_path.exists() {
            anyhow::bail!("Path does not exist: {}", full_path.display());
        }
        if !f.pinned.iter().any(|x| x == &a) {
            f.pinned.push(a);
            f.pinned.sort();
        }
        save_focus(root, &f)?;
        if json_output {
            println!(
                "{}",
                serde_json::json!({ "status": "updated", "pinned": f.pinned })
            );
        } else {
            println!("Pinned updated.");
        }
    }
    if let Some(r) = remove {
        f.pinned.retain(|x| x != &r);
        save_focus(root, &f)?;
        if json_output {
            println!(
                "{}",
                serde_json::json!({ "status": "updated", "pinned": f.pinned })
            );
        } else {
            println!("Pinned updated.");
        }
    }
    Ok(())
}
