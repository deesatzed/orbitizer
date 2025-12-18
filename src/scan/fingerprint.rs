use anyhow::{Context, Result};
use blake3::Hasher;
use std::fs;
use std::path::{Path, PathBuf};

pub fn fingerprint_project(project_root: &Path) -> Result<Option<String>> {
    if !project_root.is_dir() {
        return Ok(None);
    }

    let markers = [
        "README.md",
        "CLAUDE.md",
        "AGENT.md",
        "Cargo.toml",
        "pyproject.toml",
        "package.json",
        "HANDOFF.md",
        "handoff.md",
        "Handoff.md",
    ];
    let mut hasher = Hasher::new();
    let mut used = 0usize;

    for m in markers {
        let p = project_root.join(m);
        if p.is_file() {
            used += 1;
            hash_small(&mut hasher, &p)?;
        }
    }

    // Structural hint: immediate children names (bounded)
    let mut children: Vec<String> = fs::read_dir(project_root)
        .with_context(|| format!("Failed to read directory {}", project_root.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    children.sort();
    for c in children.iter().take(200) {
        hasher.update(c.as_bytes());
        hasher.update(b"\n");
    }

    if used == 0 && children.is_empty() {
        return Ok(None);
    }
    Ok(Some(hasher.finalize().to_hex().to_string()))
}

fn hash_small(hasher: &mut Hasher, p: &PathBuf) -> Result<()> {
    let data = fs::read(p)
        .with_context(|| format!("Failed to read {} for fingerprinting", p.display()))?;
    let take = std::cmp::min(data.len(), 64 * 1024);
    hasher.update(&data[..take]);
    Ok(())
}
