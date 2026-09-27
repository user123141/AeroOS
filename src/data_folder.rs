//! Data Folder mode: load kernel/initramfs from `data/` next to the executable.
//! Falls back to embedded resources if `mode = "embedded"`.

use crate::config::AeroConfig;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub struct DataFolder {
    base: PathBuf,
    embedded: bool,
    kernel_path: Option<PathBuf>,
    initramfs_path: Option<PathBuf>,
}

impl DataFolder {
    pub fn new(config: &AeroConfig) -> Result<Self> {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));
        let base = exe_dir.join("data");
        let embedded = config.mode == "embedded";
        Ok(Self {
            base,
            embedded,
            kernel_path: None,
            initramfs_path: None,
        })
    }

    pub fn prepare(&self) -> Result<()> {
        if self.embedded {
            tracing::info!("Embedded mode: kernel from binary resources");
            return Ok(());
        }
        // Data Folder mode
        std::fs::create_dir_all(&self.base)
            .with_context(|| format!("create {}", self.base.display()))?;

        let k = self.base.join("aeroos-kernel");
        let i = self.base.join("aeroos-initramfs");

        if !k.exists() || !i.exists() {
            anyhow::bail!(
                "Data Folder mode requires {} and {}",
                k.display(),
                i.display()
            );
        }
        tracing::info!("Data Folder mode: kernel at {}", k.display());
        Ok(())
    }

    pub fn kernel_path(&self) -> PathBuf {
        if self.embedded {
            PathBuf::from("kernel/aeroos-kernel")
        } else {
            self.base.join("aeroos-kernel")
        }
    }

    pub fn initramfs_path(&self) -> PathBuf {
        if self.embedded {
            PathBuf::from("kernel/aeroos-initramfs")
        } else {
            self.base.join("aeroos-initramfs")
        }
    }

    pub fn base_dir(&self) -> &Path {
        &self.base
    }
}
