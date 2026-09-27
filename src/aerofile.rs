//! Aerofile — декларативная конфигурация VM.
//!
//! Аналог Dockerfile / Vagrantfile, но для AeroOS.
//!
//! Пример `Aerofile`:
//!
//! ```toml
//! [vm]
//! name = "dev"
//! image = "alpine-3.20"
//! cores = 4
//! ram = 4096
//! disk_mb = 2048
//!
//! [network]
//! mode = "user"
//!
//! [provision]
//! run = [
//!   "apk add --no-cache nodejs npm",
//!   "npm install -g pnpm",
//! ]
//!
//! [[share]]
//! host = "C:\\Projects"
//! guest = "/mnt/projects"
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aerofile {
    pub vm: VmConfig,
    #[serde(default)]
    pub network: NetworkConfig,
    #[serde(default)]
    pub provision: ProvisionConfig,
    #[serde(default, rename = "share")]
    pub shares: Vec<ShareConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmConfig {
    pub name: String,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default = "default_cores")]
    pub cores: u32,
    #[serde(default = "default_ram")]
    pub ram: u64,
    #[serde(default = "default_disk")]
    pub disk_mb: u64,
}
fn default_cores() -> u32 {
    2
}
fn default_ram() -> u64 {
    2048
}
fn default_disk() -> u64 {
    1024
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkConfig {
    #[serde(default = "default_net_mode")]
    pub mode: String,
}
fn default_net_mode() -> String {
    "user".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProvisionConfig {
    #[serde(default)]
    pub run: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareConfig {
    pub host: String,
    pub guest: String,
}

impl Aerofile {
    /// Загрузить Aerofile из файла.
    pub fn load(path: &Path) -> Result<Self> {
        let text =
            std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        let af: Aerofile =
            toml::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
        af.validate()?;
        Ok(af)
    }

    /// Проверить конфигурацию.
    pub fn validate(&self) -> Result<()> {
        if self.vm.name.is_empty() {
            anyhow::bail!("vm.name is required");
        }
        if self.vm.cores < 1 || self.vm.cores > 64 {
            anyhow::bail!("vm.cores must be 1..64");
        }
        if self.vm.ram < 128 || self.vm.ram > 262144 {
            anyhow::bail!("vm.ram must be 128..262144 MB");
        }
        match self.network.mode.as_str() {
            "user" | "tap" | "none" => Ok(()),
            other => anyhow::bail!("unknown network.mode: {}", other),
        }
    }

    /// Создать шаблон Aerofile.
    pub fn template(name: &str) -> Self {
        Self {
            vm: VmConfig {
                name: name.to_string(),
                image: Some("alpine-3.20".into()),
                cores: 2,
                ram: 2048,
                disk_mb: 1024,
            },
            network: NetworkConfig {
                mode: "user".into(),
            },
            provision: ProvisionConfig {
                run: vec!["apk add --no-cache curl".into()],
            },
            shares: vec![],
        }
    }

    /// Сохранить в файл.
    pub fn save(&self, path: &Path) -> Result<()> {
        let text = toml::to_string_pretty(self)?;
        std::fs::write(path, text)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_template_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("Aerofile");
        let af = Aerofile::template("dev");
        af.save(&path).unwrap();
        let loaded = Aerofile::load(&path).unwrap();
        assert_eq!(loaded.vm.name, "dev");
        assert_eq!(loaded.vm.cores, 2);
        assert_eq!(loaded.network.mode, "user");
    }

    #[test]
    fn test_validation_bad_cores() {
        let mut af = Aerofile::template("x");
        af.vm.cores = 100;
        assert!(af.validate().is_err());
    }
}
