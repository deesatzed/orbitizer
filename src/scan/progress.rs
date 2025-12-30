use std::sync::{Arc, Mutex};

/// Minimal progress reporter to surface long-running scan phases.
/// Enabled via feature flag `ORBIT_FEATURE_PROGRESS=1`.
#[derive(Clone)]
pub struct Progress {
    enabled: bool,
    logs: Arc<Mutex<Vec<String>>>,
}

impl Progress {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn note(&self, msg: &str) {
        if self.enabled {
            {
                if let Ok(mut l) = self.logs.lock() {
                    l.push(msg.to_string());
                    if l.len() > 20 {
                        l.remove(0);
                    }
                }
            }
            eprintln!("[progress] {msg}");
        }
    }

    /// Consume and return buffered progress messages.
    pub fn drain(&self) -> Vec<String> {
        if let Ok(mut l) = self.logs.lock() {
            let drained = l.clone();
            l.clear();
            drained
        } else {
            Vec::new()
        }
    }
}
