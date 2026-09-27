#![allow(dead_code, unused_imports)]

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;

mod config;
mod hypervisor;
mod virtio;
mod snapshot;
mod web;
mod ipc;
mod security;
mod data_folder;
mod crypto;
mod boot;
mod terminal;
mod tap;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("[AeroOS] v1.6.0 starting");
    let _ = security::verify_artifacts();

    let config = config::AeroConfig::load(&PathBuf::from("aeroos.config.toml"))?;
    tracing::info!("Config: {} CPU, {} MB RAM", config.cpu.cores, config.memory.ram_mb);

    let df = match data_folder::DataFolder::new(&config) {
        Ok(df) => { let _ = df.prepare(); df }
        Err(e) => { tracing::warn!("data_folder: {}", e); return Ok(()); }
    };

    let vm_opt: Option<Arc<Mutex<hypervisor::VirtualMachine>>> = match hypervisor::VirtualMachine::new(&config, &df) {
        Ok(vm) => {
            tracing::info!("WHPX initialized");
            Some(Arc::new(Mutex::new(vm)))
        }
        Err(e) => {
            let msg = format!("{}", e);
            if msg.contains("0xC0351000") || msg.contains("WHvCreatePartition") {
                tracing::error!("=========================================================");
                tracing::error!(" Гипервизор Windows НЕ запущен.");
                tracing::error!(" Открой PowerShell от админа и выполни:");
                tracing::error!("   bcdedit /set hypervisorlaunchtype auto");
                tracing::error!(" Затем перезагрузи компьютер.");
                tracing::error!(" UI и API будут работать без VM.");
                tracing::error!("=========================================================");
                None
            } else {
                return Err(e);
            }
        }
    };

    let token = security::generate_session_token();
    tracing::info!("Session token: {}...", &token[..8]);

    let web_token = token.clone();
    let web_handle = tokio::spawn(async move {
        if let Err(e) = web::run_server(web_token).await {
            tracing::error!("web: {}", e);
        }
    });

    let vm_handle = if let Some(vm) = vm_opt.clone() {
        let ipc_state = ipc::IpcState::new(vm.clone(), token.clone());
        tokio::spawn(async move {
            if let Err(e) = ipc::run_websocket_server(ipc_state).await {
                tracing::error!("ipc: {}", e);
            }
        });
        let vm2 = vm.clone();
        tokio::spawn(async move {
            if let Err(e) = vm2.lock().await.run().await {
                tracing::error!("vm: {}", e);
            }
        })
    } else {
        tokio::spawn(async { tracing::info!("VM skipped (no hypervisor)") })
    };

    tracing::info!("UI: http://127.0.0.1:8080");
    tokio::select! {
        _ = web_handle => tracing::info!("web stopped"),
        _ = vm_handle  => tracing::info!("vm stopped"),
    }
    Ok(())
}