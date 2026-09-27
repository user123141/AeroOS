#![allow(dead_code, unused_imports)]

use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

mod aerofile;
mod agent;
mod boot;
mod config;
mod crypto;
mod data_folder;
mod hwid;
mod hypervisor;
mod ipc;
mod license;
mod multi_vm;
mod net_smoltcp;
mod sandbox;
mod security;
mod snapshot;
mod tap;
mod terminal;
mod virtio;
mod web;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    tracing::info!("[AeroOS] v3.0.0 Singularity starting");

    let _ = sandbox::init_sandbox(&["data", "snapshots"]);

    let lic =
        license::LicenseState::load(std::path::Path::new("license.key")).unwrap_or_else(|e| {
            tracing::warn!("License: {}", e);
            license::LicenseState {
                license: None,
                tier: license::Tier::Community,
            }
        });
    tracing::info!("Tier: {:?}", lic.tier);

    if let Ok(hwid) = hwid::compute_hwid() {
        tracing::info!("HWID: {}", hwid);
    }

    let _ = security::verify_artifacts();
    let config = config::AeroConfig::load(&PathBuf::from("aeroos.config.toml"))?;
    tracing::info!(
        "Config: {} CPU, {} MB RAM",
        config.cpu.cores,
        config.memory.ram_mb
    );

    let df = match data_folder::DataFolder::new(&config) {
        Ok(df) => {
            let _ = df.prepare();
            df
        }
        Err(e) => {
            tracing::warn!("data_folder: {}", e);
            return Ok(());
        }
    };

    let max_vms = if lic.tier == license::Tier::Community {
        1
    } else {
        16
    };
    let mut multi = multi_vm::MultiVmManager::new(max_vms);

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
                    tracing::error!(" Гипервизор Windows НЕ запущен.");
                    tracing::error!(" bcdedit /set hypervisorlaunchtype auto  → reboot");
                    tracing::error!(" UI работает без VM.");
                    tracing::error!("=========================================================");
                    None
                } else {
                    return Err(e);
                }
            }
        };

    let token = security::generate_session_token();
    tracing::info!("Token: {}...", &token[..8]);

    let web_token = token.clone();
    let web_handle = tokio::spawn(async move {
        if let Err(e) = web::run_server(web_token).await {
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
        tokio::spawn(async { tracing::info!("VM skipped") })
    };

    tracing::info!("UI: http://127.0.0.1:8080");
    tracing::info!("VMs: {}", multi.count());
    tokio::select! {
        _ = web_handle => {}
        _ = vm_handle  => {}
    }
    Ok(())
}
