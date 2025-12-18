use std::path::Path;

pub const ARTIFACT_KEYWORDS: &[&str] = &[
    "handoff",
    "agent",
    "claude",
    "export",
    "plan",
    "roadmap",
    "decision",
    "prompt",
    "session",
    "conversation",
    "summary",
];

pub fn is_artifact_name(path: &Path) -> bool {
    if let Some(n) = path.file_name().and_then(|s| s.to_str()) {
        let ln = n.to_lowercase();
        return ARTIFACT_KEYWORDS.iter().any(|k| ln.contains(k));
    }
    false
}

#[cfg(test)]
mod tests {
    use super::is_artifact_name;
    use std::path::Path;

    #[test]
    fn detects_known_artifacts() {
        assert!(is_artifact_name(Path::new("HANDOFF_notes.md")));
        assert!(is_artifact_name(Path::new("session_2025-01-01.txt")));
        assert!(is_artifact_name(Path::new("roadmap.md")));
    }

    #[test]
    fn ignores_non_artifacts() {
        assert!(!is_artifact_name(Path::new("Cargo.toml")));
        assert!(!is_artifact_name(Path::new("src/main.rs")));
        assert!(!is_artifact_name(Path::new("notes.txt")));
    }
}
