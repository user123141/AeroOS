//! GPU passthrough через Hyper-V GPU-P (GPU Partitioning).
//!
//! Использует PowerShell cmdlets:
//!   Get-VMHostPartitionableGpu
//!   Add-VMGpuPartitionAdapter
//!   Set-VMGpuPartitionAdapter

use anyhow::{Context, Result};
use std::process::Command;

#[derive(Debug, Clone, serde::Serialize)]
pub struct GpuDevice {
    pub name: String,
    pub vendor: String,
    pub pci_id: String,
    pub vram_mb: u64,
    pub partitionable: bool,
}

pub struct GpuPassthrough {
    devices: Vec<GpuDevice>,
    gpu_p_available: bool,
}

impl GpuPassthrough {
    pub fn new() -> Result<Self> {
        let gpu_p_available = check_gpu_p_support();
        let devices = detect_gpus()?;
        tracing::info!("GPU passthrough: {} device(s), GPU-P available: {}",
            devices.len(), gpu_p_available);
        for d in &devices {
            tracing::info!("  {} [{}] {} MB VRAM, partitionable: {}",
                d.name, d.pci_id, d.vram_mb, d.partitionable);
        }
        Ok(Self { devices, gpu_p_available })
    }

    pub fn devices(&self) -> &[GpuDevice] { &self.devices }
    pub fn gpu_p_available(&self) -> bool { self.gpu_p_available }

    /// Пробросить GPU в VM через GPU-P.
    pub fn attach_to_vm(&self, vm_name: &str, pci_id: &str) -> Result<()> {
        if !self.gpu_p_available {
            anyhow::bail!("GPU-P not available on this host");
        }
        let dev = self.devices.iter().find(|d| d.pci_id == pci_id)
            .ok_or_else(|| anyhow::anyhow!("GPU {} not found", pci_id))?;
        if !dev.partitionable {
            anyhow::bail!("GPU {} is not partitionable", dev.name);
        }

        tracing::info!("Attaching GPU {} to VM {}", dev.name, vm_name);
        let out = Command::new("powershell")
            .args([
                "-NoProfile", "-Command",
                &format!("Add-VMGpuPartitionAdapter -VMName '{}'", vm_name),
            ])
            .output()
            .context("PowerShell Add-VMGpuPartitionAdapter")?;

        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            anyhow::bail!("Add-VMGpuPartitionAdapter failed: {}", err);
        }
        tracing::info!("GPU-P adapter added to VM {}", vm_name);
        Ok(())
    }

    /// Отсоединить GPU от VM.
    pub fn detach_from_vm(&self, vm_name: &str) -> Result<()> {
        let out = Command::new("powershell")
            .args([
                "-NoProfile", "-Command",
                &format!("Remove-VMGpuPartitionAdapter -VMName '{}' -Confirm:$false", vm_name),
            ])
            .output()?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            anyhow::bail!("Remove-VMGpuPartitionAdapter failed: {}", err);
        }
        tracing::info!("GPU-P adapter removed from VM {}", vm_name);
        Ok(())
    }
}

fn check_gpu_p_support() -> bool {
    #[cfg(windows)]
    {
        match Command::new("powershell")
            .args(["-NoProfile", "-Command",
                   "if (Get-Command Get-VMHostPartitionableGpu -ErrorAction SilentlyContinue) { 'YES' } else { 'NO' }"])
            .output()
        {
            Ok(out) => String::from_utf8_lossy(&out.stdout).trim() == "YES",
            Err(_) => false,
        }
    }
    #[cfg(not(windows))]
    { false }
}

fn detect_gpus() -> Result<Vec<GpuDevice>> {
    #[cfg(windows)]
    {
        // Partitionable GPUs via Hyper-V
        let partitionable_ids: Vec<String> = Command::new("powershell")
            .args(["-NoProfile", "-Command",
                   "(Get-VMHostPartitionableGpu -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Name) -join ','"])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.trim().to_string())
            .collect();

        // Все GPU через WMI
        let out = Command::new("wmic")
            .args(["path", "win32_VideoController", "get",
                   "Name,AdapterCompatibility,PNPDeviceID,AdapterRAM", "/format:csv"])
            .output()?;
        let text = String::from_utf8_lossy(&out.stdout);
        let mut devices = Vec::new();
        for line in text.lines().skip(2) {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 5 {
                let pci_id = parts[3].trim().to_string();
                let partitionable = partitionable_ids.iter().any(|id| pci_id.contains(id) || id.contains(&pci_id));
                devices.push(GpuDevice {
                    name: parts[1].trim().to_string(),
                    vendor: parts[2].trim().to_string(),
                    pci_id,
                    vram_mb: parts[4].trim().parse::<u64>().unwrap_or(0) / 1024 / 1024,
                    partitionable,
                });
            }
        }
        Ok(devices)
    }
    #[cfg(not(windows))]
    { Ok(Vec::new()) }
}