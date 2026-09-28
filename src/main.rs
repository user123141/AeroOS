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
mod syscontrol;
mod registry;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();
    tracing::info!("[AeroOS] v4.3.0 Integrity starting");

    let _ = sandbox::init_sandbox(&["data", "snapshots"]);

    let lic = license::LicenseState::load(std::path::Path::new("license.key"))
        .unwrap_or_else(|e| {
            tracing::warn!("License: {}", e);
            license::LicenseState { license: None, tier: license::Tier::Community }
        });
    tracing::info!("Tier: {:?}, max_vms={}, max_ram={} MB",
        lic.tier, lic.max_vms(), lic.max_ram_mb());

    if let Ok(hwid) = hwid::compute_hwid() {
        tracing::info!("HWID: {}", hwid);
    }

    if let Ok(gpu) = gpu_passthrough::GpuPassthrough::new() {
        tracing::info!("GPU devices: {}", gpu.devices().len());
    }

    if let Ok(reg) = registry::AeroRegistry::new() {
        tracing::info!("AeroRegistry ready ({} images)", reg.list().len());
    }

    let _ = security::verify_artifacts();
    let config = config::AeroConfig::load(&PathBuf::from("aeroos.config.toml"))?;
    tracing::info!("Config: {} CPU, {} MB RAM", config.cpu.cores, config.memory.ram_mb);

    let df = match data_folder::DataFolder::new(&config) {
        Ok(df) => { let _ = df.prepare(); df }
        Err(e) => { tracing::warn!("data_folder: {}", e); return Ok(()); }
    };

    // AeroBoot preload — kernel + initramfs
    {
        let mut ab = boot::AeroBoot::new();
        let kp = df.kernel_path();
        let ip = df.initramfs_path();
        match ab.load_from_data_folder(&kp, &ip) {
            Ok(_) => tracing::info!("AeroBoot preload complete"),
            Err(e) => tracing::warn!("AeroBoot preload: {}", e),
        }
    }

    let mut multi = multi_vm::MultiVmManager::new(lic.max_vms());

    let vm_opt: Option<hypervisor::VirtualMachine> =
        match hypervisor::VirtualMachine::new(&config, &df) {
            Ok(vm) => { tracing::info!("WHPX OK"); Some(vm) }
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

    let token = security::generate_session_token();
    tracing::info!("Token: {}...", &token[..8]);
    // Crypto-UI binding: derive UI key from license (if present)
    let ui_key: web::UiKey = std::sync::Arc::new(tokio::sync::RwLock::new(
        lic.license.as_ref().map(license::derive_ui_key),
    ));
    if ui_key.blocking_read().is_some() {
        tracing::info!("Crypto-UI: key derived from license");
    } else {
        tracing::info!("Crypto-UI: no license, open mode");
    }


    let web_token = token.clone();
    
    let web_handle = tokio::spawn(async move {
        if let Err(e) = web::run_server(web_token, ui_key).await {
            tracing::error!("web: {}", e);
        }
    });

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