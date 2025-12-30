use std::fs;
use std::path::Path;

use orbit::export::all::export_all;
use orbit::snapshot::quick::snapshot_pinned;

#[test]
fn snapshot_dry_run_creates_nothing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();

    // dry-run should short-circuit before creating .orbit
    snapshot_pinned(root.to_string_lossy().as_ref(), Some("test"), true).unwrap();
    let orbit_dir = root.join(".orbit");
    assert!(!orbit_dir.exists(), "dry-run should not create .orbit");
}

#[test]
fn export_dry_run_creates_nothing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();

    export_all(root.to_string_lossy().as_ref(), true).unwrap();
    let exports_dir = root.join(".orbit").join("exports");
    assert!(!exports_dir.exists(), "dry-run should not create exports dir");
}

#[test]
fn snapshot_respects_whitelist() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    let proj = root.join("proj");
    fs::create_dir_all(&proj).unwrap();
    fs::write(proj.join("README.md"), "hi").unwrap();

    // create whitelist that protects proj
    let wl_path = root.join(".orbit");
    fs::create_dir_all(&wl_path).unwrap();
    fs::write(
        wl_path.join("whitelist.json"),
        serde_json::json!({"version": "0.1", "paths": ["proj"]}).to_string(),
    )
    .unwrap();

    snapshot_pinned(root.to_string_lossy().as_ref(), Some("test"), false).unwrap();
    let snap_root = find_snapshot_dir(root);
    let artifacts = snap_root.join("artifacts");
    assert!(artifacts.exists());
    // Protected project should not appear
    let copied = fs::read_dir(&artifacts).unwrap().count();
    assert_eq!(copied, 0, "whitelisted project should be skipped");
}

fn find_snapshot_dir(root: &Path) -> std::path::PathBuf {
    let snaps = root.join(".orbit").join("snapshots");
    let mut entries = fs::read_dir(&snaps).unwrap();
    entries
        .next()
        .unwrap()
        .unwrap()
        .path()
}
