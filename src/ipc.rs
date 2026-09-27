//! WebSocket IPC между UI и VM.
//! Токен-аутентификация, JSON-команды, serial output stream.

use crate::hypervisor::VirtualMachine;
use crate::snapshot::SnapshotManager;
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Command {
    Snapshot { name: String },
    Restore { name: String },
    ListSnapshots,
    GetStatus,
    GetStats,
    ListVms,
    GetSerial,
    ClearSerial,
    Stop,
    TerminalInput { data: String },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum Response {
    Ok { message: String },
    Error { message: String },
    Status { running: bool, cpu_cores: u32, ram_mb: u64, gpu: String },
    Stats { snapshots: u64, dirty_pages: u64 },
    Snapshots { list: Vec<String> },
    VmsList { vms: Vec<VmStatsInfo> },
    Serial { data: String },
    TerminalOutput { data: String },
}

#[derive(Debug, Clone, Serialize)]
pub struct VmStatsInfo {
    pub id: String,
    pub name: String,
    pub running: bool,
    pub snapshots: u64,
    pub dirty_pages: u64,
}

pub struct IpcState {
    pub vm: Arc<Mutex<VirtualMachine>>,
    pub snapshots: Arc<Mutex<SnapshotManager>>,
    pub token: String,
}

impl IpcState {
    pub fn new(vm: Arc<Mutex<VirtualMachine>>, token: String) -> Self {
        let config = vm.blocking_lock().config().clone();
        Self {
            vm,
            snapshots: Arc::new(Mutex::new(SnapshotManager::new(&config.snapshot))),
            token,
        }
    }
}

pub async fn run_websocket_server(state: IpcState) -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8081").await?;
    tracing::info!("WebSocket: ws://127.0.0.1:8081");
    let state = Arc::new(state);
    loop {
        let (stream, _) = listener.accept().await?;
        let state_clone = state.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, state_clone).await {
                tracing::error!("WS: {}", e);
            }
        });
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    state: Arc<IpcState>,
) -> Result<()> {
    let token = state.token.clone();
    let ws = tokio_tungstenite::accept_hdr_async(
        stream,
        move |req: &tokio_tungstenite::tungstenite::handshake::server::Request, resp| {
            let ok = req.uri().query().map_or(false, |q| {
                q.split('&').any(|p| p == format!("token={}", token))
            });
            if !ok {
                return Err(tokio_tungstenite::tungstenite::handshake::server::ErrorResponse::new(
                    Some("Unauthorized".into()),
                ));
            }
            Ok(resp)
        },
    ).await?;

    let (mut write, mut read) = ws.split();

    while let Some(msg) = read.next().await {
        let msg = msg?;
        if let Message::Text(text) = msg {
            let cmd: Command = match serde_json::from_str(&text) {
                Ok(c) => c,
                Err(e) => {
                    let err = Response::Error { message: format!("Parse error: {}", e) };
                    write.send(Message::Text(serde_json::to_string(&err)?)).await?;
                    continue;
                }
            };

            let resp = process_command(cmd, &state).await;
            write.send(Message::Text(serde_json::to_string(&resp)?)).await?;
        }
    }

    Ok(())
}

async fn process_command(cmd: Command, state: &IpcState) -> Response {
    match cmd {
        Command::GetStatus => {
            let vm = state.vm.lock().await;
            Response::Status {
                running: vm.is_running(),
                cpu_cores: vm.config().cpu.cores,
                ram_mb: vm.config().memory.ram_mb,
                gpu: vm.config().gpu.model.clone(),
            }
        }
        Command::GetStats => {
            let vm = state.vm.lock().await;
            let s = vm.stats();
            Response::Stats { snapshots: s.snapshots, dirty_pages: s.dirty_pages }
        }
        Command::ListVms => {
            let vm = state.vm.lock().await;
            let s = vm.stats();
            Response::VmsList {
                vms: vec![VmStatsInfo {
                    id: "default".into(),
                    name: "AeroOS Default".into(),
                    running: vm.is_running(),
                    snapshots: s.snapshots,
                    dirty_pages: s.dirty_pages,
                }],
            }
        }
        Command::GetSerial => {
            let vm = state.vm.lock().await;
            let data = vm.read_serial();
            Response::Serial { data: String::from_utf8_lossy(&data).into() }
        }
        Command::ClearSerial => {
            let vm = state.vm.lock().await;
            vm.clear_serial();
            Response::Ok { message: "Serial cleared".into() }
        }
        Command::Stop => {
            state.vm.lock().await.stop();
            Response::Ok { message: "Stopped".into() }
        }
        Command::Snapshot { name } => {
            let vm = state.vm.lock().await;
            match vm.get_state() {
                Ok(data) => {
                    drop(vm);
                    let mut mgr = state.snapshots.lock().await;
                    match mgr.create_snapshot(&data, &name) {
                        Ok(_) => Response::Ok { message: format!("Snapshot '{}' created", name) },
                        Err(e) => Response::Error { message: e.to_string() },
                    }
                }
                Err(e) => Response::Error { message: e.to_string() },
            }
        }
        Command::Restore { name } => {
            let mgr = state.snapshots.lock().await;
            match mgr.restore_snapshot(&name) {
                Ok(data) => {
                    drop(mgr);
                    let mut vm = state.vm.lock().await;
                    match vm.restore_state(&data) {
                        Ok(_) => Response::Ok { message: format!("Snapshot '{}' restored", name) },
                        Err(e) => Response::Error { message: e.to_string() },
                    }
                }
                Err(e) => Response::Error { message: e.to_string() },
            }
        }
        Command::ListSnapshots => {
            let mgr = state.snapshots.lock().await;
            Response::Snapshots { list: mgr.list_snapshots() }
        }
        Command::TerminalInput { data } => {
            tracing::info!(target: "terminal", "input: {}", data);
            Response::TerminalOutput { data: format!("$ {}\r\n", data.trim()) }
        }
    }
}