use orbit::feature;

#[test]
fn env_bool_parsing_defaults_off() {
    // Ensure defaults are off when env unset
    std::env::remove_var("ORBIT_FEATURE_METRICS");
    std::env::remove_var("ORBIT_FEATURE_PROGRESS");
    std::env::remove_var("ORBIT_FEATURE_DRY_RUN");
    std::env::remove_var("ORBIT_FEATURE_EXPERIMENTAL");
    let flags = feature::flags_fresh_for_tests();
    assert!(!flags.metrics);
    assert!(!flags.progress);
    assert!(!flags.dry_run);
    assert!(!flags.experimental);
}

#[test]
fn env_bool_parsing_true_variants() {
    std::env::set_var("ORBIT_FEATURE_PROGRESS", "true");
    std::env::set_var("ORBIT_FEATURE_DRY_RUN", "1");
    let flags = feature::flags_fresh_for_tests();
    assert!(flags.progress);
    assert!(flags.dry_run);
}
