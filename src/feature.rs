use once_cell::sync::Lazy;
use std::env;

/// Centralized feature flags for new Mole↔Orbit integration work.
/// Defaults are conservative (off) to avoid changing behavior until features are ready.
#[derive(Debug, Clone, Copy)]
pub struct FeatureFlags {
    pub metrics: bool,
    pub progress: bool,
    pub dry_run: bool,
    pub experimental: bool,
}

impl FeatureFlags {
    fn from_env() -> Self {
        Self {
            metrics: env_bool("ORBIT_FEATURE_METRICS", false),
            progress: env_bool("ORBIT_FEATURE_PROGRESS", false),
            dry_run: env_bool("ORBIT_FEATURE_DRY_RUN", false),
            experimental: env_bool("ORBIT_FEATURE_EXPERIMENTAL", false),
        }
    }
}

fn env_bool(name: &str, default: bool) -> bool {
    match env::var(name) {
        Ok(v) => matches!(v.as_str(), "1" | "true" | "TRUE" | "on" | "yes" | "enabled"),
        Err(_) => default,
    }
}

static FLAGS: Lazy<FeatureFlags> = Lazy::new(FeatureFlags::from_env);

/// Accessor for current feature flags (cached from environment).
pub fn flags() -> &'static FeatureFlags {
    &*FLAGS
}

/// For tests: produce a fresh snapshot of flags from current env (not cached).
pub fn flags_fresh_for_tests() -> FeatureFlags {
    FeatureFlags::from_env()
}
