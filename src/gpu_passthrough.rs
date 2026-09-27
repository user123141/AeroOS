//! GPU passthrough — проброс реальной GPU в guest.
//!
//! На Windows это требует:
//!   1. Hyper-V GPU-P (GPU Partitioning)
//!   2. DDA (Discrete Device Assignment)
//!   3. WDDM GPU paravirtualization
//!
//! Реализация через WMI/Hyper-V API. Пока каркас.

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct GpuDevice {
    pub name: String,
    pub vendor: String,
    pub pci_id: String,
    pub vram_mb: u64,
}

pub struct GpuPassthrough {
    devices: Vec<GpuDevice>,
}

impl GpuPassthrough {
    pub fn new() -> Result<Self> {
        let devices = detect_gpus()?;
        tracing::info!("GPU passthrough: detected {} device(s)", devices.len());
        for d in &devices {
            tracing::info!("  {} ({})", d.name, d.pci_id);
        }
        Ok(Self { devices })
    }

    pub fn devices(&self) -> &[GpuDevice] {
        &self.devices
    }

    /// Проброс GPU в VM.
    pub fn attach_to_vm(&self, _vm_id: &str, _pci_id: &str) -> Result<()> {
        tracing::warn!("GPU passthrough attach not yet implemented (WDDM/Hyper-V GPU-P API)");
        Ok(())
    }
}

fn detect_gpus() -> Result<Vec<GpuDevice>> {
    #[cfg(windows)]
    {
        use std::process::Command;
        let out = Command::new("wmic")
            .args([
                "path",
                "win32_VideoController",
                "get",
                "Name,AdapterCompatibility,PNPDeviceID,AdapterRAM",
                "/format:csv",
            ])
            .output()?;
        let text = String::from_utf8_lossy(&out.stdout);
        let mut devices = Vec::new();
        for line in text.lines().skip(2) {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 5 {
                devices.push(GpuDevice {
                    name: parts[1].trim().to_string(),
                    vendor: parts[2].trim().to_string(),
                    pci_id: parts[3].trim().to_string(),
                    vram_mb: parts[4].trim().parse::<u64>().unwrap_or(0) / 1024 / 1024,
                });
            }
        }
        Ok(devices)
    }
    #[cfg(not(windows))]
    {
        Ok(Vec::new())
    }
}
