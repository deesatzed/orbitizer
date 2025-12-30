use orbit::index::whitelist::{is_protected, load_whitelist, save_whitelist, Whitelist};
use orbit::system::{load_metrics, save_metrics, SystemMetrics};

#[test]
fn whitelist_roundtrip_and_protection() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let wl = Whitelist {
        version: "0.1".into(),
        paths: vec!["protected".into()],
    };
    save_whitelist(root, &wl).unwrap();
    let loaded = load_whitelist(root).unwrap();
    assert_eq!(loaded.paths, wl.paths);

    let protected_path = root.join("protected").join("nested");
    assert!(is_protected(&protected_path, &loaded));
    let other = root.join("other");
    assert!(!is_protected(&other, &loaded));
}

#[test]
fn metrics_roundtrip_tolerates_partial() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let mut m = SystemMetrics::default();
    m.cpu = Some(Default::default());
    save_metrics(root, &m).unwrap();
    let loaded = load_metrics(root).unwrap().unwrap();
    assert!(loaded.cpu.is_some());
    // missing fields should remain None
    assert!(loaded.memory.is_none());
}
