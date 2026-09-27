//! OCI compatibility — запуск Docker-образов как microVM.
//!
//! Каркас: разбор OCI-манифеста, pull слоёв, конвертация в ext4/initramfs.
//! В v3.3 — реальная интеграция с crun/youki.

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciManifest {
    pub schema_version: u32,
    pub media_type: String,
    pub config: OciDescriptor,
    pub layers: Vec<OciDescriptor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciDescriptor {
    pub media_type: String,
    pub digest: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciConfig {
    pub architecture: String,
    pub os: String,
    pub config: OciRuntimeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciRuntimeConfig {
    #[serde(default)]
    pub Cmd: Vec<String>,
    #[serde(default)]
    pub Env: Vec<String>,
}

pub struct OciRuntime {
    image_dir: PathBuf,
}

impl OciRuntime {
    pub fn new(image_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(image_dir).ok();
        Ok(Self {
            image_dir: image_dir.to_path_buf(),
        })
    }

    /// Скачать OCI-образ. Пока заглушка (реальная интеграция — v3.3).
    pub fn pull(&self, image: &str) -> Result<PathBuf> {
        tracing::info!("OCI pull: {}", image);
        bail!("OCI pull not yet implemented (planned for v3.3)")
    }

    /// Разобрать OCI-манифест.
    pub fn parse_manifest(json: &str) -> Result<OciManifest> {
        Ok(serde_json::from_str(json)?)
    }

    /// Конвертировать OCI-образ в initramfs для AeroOS.
    pub fn to_initramfs(&self, _image_dir: &Path, _out: &Path) -> Result<()> {
        bail!("OCI → initramfs conversion planned for v3.3")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_manifest() {
        let json = r#"{
            "schemaVersion": 2,
            "mediaType": "application/vnd.oci.image.manifest.v1+json",
            "config": {"mediaType":"x", "digest":"sha256:abc", "size":123},
            "layers": []
        }"#;
        let m = OciRuntime::parse_manifest(json).unwrap();
        assert_eq!(m.schema_version, 2);
    }
}
