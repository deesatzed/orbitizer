use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::index::store::atomic_write;

/// Lightweight system metrics snapshot (pulled from Mole collectors).
/// All fields are optional to allow partial data and forward compatibility.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemMetrics {
    #[serde(default = "default_version")]
    pub version: String,
    pub captured_at: Option<DateTime<Local>>,
    pub cpu: Option<CpuMetrics>,
    pub memory: Option<MemoryMetrics>,
    pub disk: Option<DiskMetrics>,
    pub network: Option<NetworkMetrics>,
    pub battery: Option<BatteryMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CpuMetrics {
    pub usage_pct: Option<f32>,
    pub load1: Option<f32>,
    pub load5: Option<f32>,
    pub load15: Option<f32>,
    pub cores: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryMetrics {
    pub used_gb: Option<f32>,
    pub total_gb: Option<f32>,
    pub swap_used_gb: Option<f32>,
    pub swap_total_gb: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiskMetrics {
    pub used_gb: Option<f32>,
    pub total_gb: Option<f32>,
    pub read_mb_s: Option<f32>,
    pub write_mb_s: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkMetrics {
    pub rx_mb_s: Option<f32>,
    pub tx_mb_s: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BatteryMetrics {
    pub level_pct: Option<f32>,
    pub health: Option<String>,
    pub cycles: Option<u32>,
    pub temperature_c: Option<f32>,
}

fn default_version() -> String {
    "0.1".into()
}

pub fn metrics_path(root: &Path) -> PathBuf {
    root.join(".orbit").join("metrics.json")
}

pub fn load_metrics(root: &Path) -> Result<Option<SystemMetrics>> {
    let p = metrics_path(root);
    if !p.exists() {
        return Ok(None);
    }
    let content =
        fs::read_to_string(&p).with_context(|| format!("Failed to read {}", p.display()))?;
    let metrics: SystemMetrics =
        serde_json::from_str(&content).with_context(|| format!("Failed to parse {}", p.display()))?;
    Ok(Some(metrics))
}

pub fn save_metrics(root: &Path, metrics: &SystemMetrics) -> Result<()> {
    let p = metrics_path(root);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }
    let content = serde_json::to_string_pretty(metrics).context("Failed to serialize metrics")?;
    atomic_write(&p, &content)
}
