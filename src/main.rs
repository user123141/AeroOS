#![allow(dead_code, unused_imports)]

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;

mod config;
mod hypervisor;
mod virtio;
mod virtio_fs;
mod snapshot;
mod web;
mod ipc;
mod security;
mod data_folder;
mod crypto;
mod boot;
mod boot_vm;
mod terminal;
mod tap;
mod net_smoltcp;
mod net_proxy;
mod vnc;
mod migration;
mod oci;
mod gpu_passthrough;
mod agent;
mod license;
mod hwid;
mod aerofile;
mod multi_vm;
mod sandbox;
mod registry;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();
    tracing::info!("[AeroOS] v4.1.8 starting");

    // Sandbox (базовая изоляция)
    let _ = sandbox::init_sandbox(&["data", "snapshots"]);

    // License
    let lic = license::LicenseState::load(std::path::Path::new("license.key"))
        .unwrap_or_else(|e| {
            tracing::warn!("License: {}", e);
            license::LicenseState { license: None, tier: license::Tier::Community }
        });
    tracing::info!("Tier: {:?}, max_vms={}, max_ram={} MB",
        lic.tier, lic.max_vms(), lic.max_ram_mb());

    // HWID
    if let Ok(hwid) = hwid::compute_hwid() {
        tracing::info!("HWID: {}", hwid);
    }

    // GPU
    if let Ok(gpu) = gpu_passthrough::GpuPassthrough::new() {
        tracing::info!("GPU devices: {}", gpu.devices().len());
    }

    // Registry
    if let Ok(reg) = registry::AeroRegistry::new() {
        tracing::info!("AeroRegistry ready ({} images)", reg.list().len());
    }

    // Config
    let _ = security::verify_artifacts();
    let config = config::AeroConfig::load(&PathBuf::from("aeroos.config.toml"))?;
    tracing::info!("Config: {} CPU, {} MB RAM", config.cpu.cores, config.memory.ram_mb);

    // Data folder
    let df = match data_folder::DataFolder::new(&config) {
        Ok(df) => { let _ = df.prepare(); df }
        Err(e) => { tracing::warn!("data_folder: {}", e); return Ok(()); }
    };

    // Multi-VM manager
    let mut multi = multi_vm::MultiVmManager::new(lic.max_vms());

    // Try to create VM (may fail if hypervisor off)
    let vm_opt: Option<hypervisor::VirtualMachine> =
        match hypervisor::VirtualMachine::new(&config, &df) {
            Ok(vm) => {
                tracing::info!("WHPX OK");
                Some(vm)
            }
            Err(e) => {
                let m = format!("{}", e);
                if m.contains("0xC0351000") || m.contains("WHvCreatePartition") {
                    tracing::error!("=========================================================");
                    tracing::error!(" Windows Hypervisor is NOT running.");
                    tracing::error!(" Run as admin:");
                    tracing::error!("   bcdedit /set hypervisorlaunchtype auto");
                    tracing::error!(" Then REBOOT.");
                    tracing::error!(" UI will still start without VM.");
                    tracing::error!("=========================================================");
                    None
                } else {
                    return Err(e);
                }
            }
        };

    // Session token
    let token = security::generate_session_token();
    tracing::info!("Token: {}...", &token[..8]);

    // Web server (always)
    let web_token = token.clone();
    let web_handle = tokio::spawn(async move {
        if let Err(e) = web::run_server(web_token).await {
            tracing::error!("web: {}", e);
        }
    });

    // VM handle: either real VM, or "keep alive" future
    let vm_handle = if let Some(vm) = vm_opt {
        multi.create("default", "AeroOS Default", vm)?;
        let inst = multi.get("default").unwrap();
        let vm_arc = inst.vm.clone();

        let ipc_state = ipc::IpcState::new(vm_arc.clone(), token.clone());
        tokio::spawn(async move {
            if let Err(e) = ipc::run_websocket_server(ipc_state).await {
                tracing::error!("ipc: {}", e);
            }
        });

        let vm2 = vm_arc.clone();
        tokio::spawn(async move {
            if let Err(e) = vm2.lock().await.run().await {
                tracing::error!("vm: {}", e);
            }
        })
    } else {
        // ВАЖНО: не даём main завершиться. Ждём вечно.
        tokio::spawn(async {
            tracing::info!("VM skipped (hypervisor off). UI is alive.");
            std::future::pending::<()>().await;
        })
    };

    tracing::info!("UI: http://127.0.0.1:8080");
    tracing::info!("VMs: {}", multi.count());
    tracing::info!("Press Ctrl+C to stop");

    tokio::select! {
        _ = web_handle => tracing::info!("web stopped"),
        _ = vm_handle  => tracing::info!("vm stopped"),
    }

    Ok(())
}