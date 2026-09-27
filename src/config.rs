use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuConfig {
    #[serde(default = "d_cores")]
    pub cores: u32,
    #[serde(default = "d_model")]
    pub model: String,
}
fn d_cores() -> u32 {
    2
}
fn d_model() -> String {
    "host".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    #[serde(default = "d_ram")]
    pub ram_mb: u64,
}
fn d_ram() -> u64 {
    4096
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuConfig {
    #[serde(default = "d_gpu")]
    pub model: String,
    #[serde(default)]
    pub accel_3d: bool,
}
fn d_gpu() -> String {
    "virtio-gpu".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskConfig {
    pub image_path: Option<String>,
    #[serde(default = "d_disk")]
    pub interface: String,
}
fn d_disk() -> String {
    "virtio".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotConfig {
    #[serde(default = "d_int")]
    pub auto_interval_ms: u64,
    #[serde(default = "d_max")]
    pub max_count: u32,
    #[serde(default = "d_dir")]
    pub dir: String,
    #[serde(default = "d_true")]
    pub dedup: bool,
    #[serde(default = "d_size")]
    pub max_size_mb: u64,
    #[serde(default)]
    pub encrypt: bool,
    #[serde(default)]
    pub ram_only: bool,
}
fn d_int() -> u64 {
    1000
}
fn d_max() -> u32 {
    100
}
fn d_dir() -> String {
    "snapshots".into()
}
fn d_true() -> bool {
    true
}
fn d_size() -> u64 {
    10240
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            auto_interval_ms: d_int(),
            max_count: d_max(),
            dir: d_dir(),
            dedup: d_true(),
            max_size_mb: d_size(),
            encrypt: false,
            ram_only: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AeroConfig {
    pub cpu: CpuConfig,
    pub memory: MemoryConfig,
    pub gpu: GpuConfig,
    pub disk: DiskConfig,
    pub snapshot: SnapshotConfig,
    #[serde(default = "d_mode")]
    pub mode: String,
}
fn d_mode() -> String {
    "embedded".into()
}

impl AeroConfig {
    pub fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            let c = std::fs::read_to_string(path)?;
            Ok(toml::from_str(&c)?)
        } else {
            let cfg = AeroConfig {
                cpu: CpuConfig {
                    cores: 4,
                    model: "host".into(),
                },
                memory: MemoryConfig { ram_mb: 8192 },
                gpu: GpuConfig {
                    model: "virtio-gpu".into(),
                    accel_3d: true,
                },
                disk: DiskConfig {
                    image_path: None,
                    interface: "virtio".into(),
                },
                snapshot: SnapshotConfig::default(),
                mode: "embedded".into(),
            };
            std::fs::write(path, toml::to_string_pretty(&cfg)?)?;
            Ok(cfg)
        }
    }
}
