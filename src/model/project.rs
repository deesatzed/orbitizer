use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectEntry {
    pub path: String,
    pub kind: ProjectKind,
    /// Derived from focus list - not the source of truth.
    /// Use `sync_pinned_flags()` to update from focus.
    #[serde(default)]
    pub pinned: bool,
    pub latest_mtime: Option<DateTime<Local>>,
    pub size_bytes: Option<u64>,
    pub artifact_count: u32,
    pub has_git: bool,
    pub has_rust: bool,
    pub has_node: bool,
    pub has_python: bool,
    pub fingerprint: Option<String>,
}

/// Sync pinned flags from focus list to project entries.
/// This is the canonical way to update pinned status.
pub fn sync_pinned_flags(projects: &mut [ProjectEntry], pinned_paths: &[String]) {
    for p in projects.iter_mut() {
        p.pinned = pinned_paths.iter().any(|x| x == &p.path);
    }
}

/// Check if a path is pinned without modifying the project entry.
pub fn is_pinned(path: &str, pinned_paths: &[String]) -> bool {
    pinned_paths.iter().any(|x| x == path)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectKind {
    ActiveStandalone,
    Standalone,
    Experimental,
    BackupDuplicate,
    VendorThirdParty,
    Unknown,
}
