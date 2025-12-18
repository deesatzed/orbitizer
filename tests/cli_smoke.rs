use std::process::Command;

use tempfile::TempDir;

#[test]
fn cli_smoke_census_focus_export_status() {
    let td = TempDir::new().expect("tempdir");
    let root = td.path();

    let proj = root.join("proj1");
    std::fs::create_dir_all(&proj).expect("create proj1");
    std::fs::write(proj.join("README.md"), "hello\n").expect("write readme");

    let bin = env!("CARGO_BIN_EXE_orbit");

    let census = Command::new(bin)
        .arg("--root")
        .arg(root)
        .arg("census")
        .arg("--depth")
        .arg("4")
        .arg("--since")
        .arg("1970-01-01")
        .output()
        .expect("run census");
    assert!(
        census.status.success(),
        "census failed: {}",
        String::from_utf8_lossy(&census.stderr)
    );

    let index_path = root.join(".orbit").join("index.json");
    assert!(
        index_path.is_file(),
        "index.json missing at {}",
        index_path.display()
    );

    let focus = Command::new(bin)
        .arg("--root")
        .arg(root)
        .arg("focus")
        .arg("--add")
        .arg("proj1")
        .output()
        .expect("run focus add");
    assert!(
        focus.status.success(),
        "focus failed: {}",
        String::from_utf8_lossy(&focus.stderr)
    );

    let focus_path = root.join(".orbit").join("focus.json");
    assert!(
        focus_path.is_file(),
        "focus.json missing at {}",
        focus_path.display()
    );
    let focus_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&focus_path).expect("read focus.json"))
            .expect("parse focus.json");
    let pinned = focus_json
        .get("pinned")
        .and_then(|v| v.as_array())
        .expect("pinned array");
    assert!(
        pinned.iter().any(|v| v.as_str() == Some("proj1")),
        "proj1 not pinned: {focus_json}"
    );

    let export = Command::new(bin)
        .arg("--root")
        .arg(root)
        .arg("export")
        .output()
        .expect("run export");
    assert!(
        export.status.success(),
        "export failed: {}",
        String::from_utf8_lossy(&export.stderr)
    );

    let exports_dir = root.join(".orbit").join("exports");
    assert!(exports_dir.join("summary.md").is_file());
    assert!(exports_dir.join("index.json").is_file());
    assert!(exports_dir.join("index.csv").is_file());

    let snap = Command::new(bin)
        .arg("--root")
        .arg(root)
        .arg("snap")
        .arg("--label")
        .arg("e2e")
        .output()
        .expect("run snap");
    assert!(
        snap.status.success(),
        "snap failed: {}",
        String::from_utf8_lossy(&snap.stderr)
    );

    let snaps_dir = root.join(".orbit").join("snapshots");
    assert!(
        snaps_dir.is_dir(),
        "snapshots dir missing at {}",
        snaps_dir.display()
    );
    let mut entries: Vec<std::path::PathBuf> = std::fs::read_dir(&snaps_dir)
        .expect("read snapshots dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_dir())
        .collect();
    entries.sort();
    assert_eq!(entries.len(), 1, "expected 1 snapshot dir, got {entries:?}");
    let snap_dir = &entries[0];
    assert!(snap_dir.join("focus.json").is_file());
    assert!(snap_dir.join("index.json").is_file());
    assert!(snap_dir.join("summary.md").is_file());
    assert!(snap_dir.join("artifacts").is_dir());

    let status = Command::new(bin)
        .arg("--root")
        .arg(root)
        .arg("status")
        .output()
        .expect("run status");
    assert!(
        status.status.success(),
        "status failed: {}",
        String::from_utf8_lossy(&status.stderr)
    );

    let out = String::from_utf8_lossy(&status.stdout);
    assert!(
        out.contains("Orbit status"),
        "unexpected status output: {out}"
    );
}
