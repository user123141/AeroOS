use crate::hypervisor::{VirtualMachine, VmStats};
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
    Snapshot { name: String }, Restore { name: String },
    GetStatus, GetStats, Stop,
    TerminalInput { data: String },
}
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum Response {
    Ok { message: String }, Error { message: String },
    Status { running: bool, cpu_cores: u32, ram_mb: u64, gpu: String },
    Stats { snapshots: u64, dirty_pages: u64 },
    TerminalOutput { data: String },
}

pub struct IpcState {
    pub vm: Arc<Mutex<VirtualMachine>>,
    pub snapshots: Arc<Mutex<SnapshotManager>>,
    pub token: String,
}
impl IpcState {
    pub fn new(vm: Arc<Mutex<VirtualMachine>>, token: String) -> Self {
        let c = vm.blocking_lock().config().clone();
        Self { vm, snapshots: Arc::new(Mutex::new(SnapshotManager::new(&c.snapshot))), token }
    }
}

pub async fn run_websocket_server(state: IpcState) -> Result<()> {
    let l = TcpListener::bind("127.0.0.1:8081").await?;
    tracing::info!("WebSocket: ws://127.0.0.1:8081");
    let state = Arc::new(state);
    loop {
        let (s, _) = l.accept().await?;
        let st = state.clone();
        tokio::spawn(async move {
            if let Err(e) = handle(s, st).await { tracing::error!("WS: {}", e); }
        });
    }
}

async fn handle(stream: tokio::net::TcpStream, state: Arc<IpcState>) -> Result<()> {
    let ws = tokio_tungstenite::accept_hdr_async(stream,
        |req: &tokio_tungstenite::tungstenite::handshake::server::Request, resp| {
            let ok = req.uri().query().map_or(false, |q| q.split('&').any(|p| p == format!("token={}", state.token)));
            if !ok {
                return Err(tokio_tungstenite::tungstenite::handshake::server::ErrorResponse::new(Some("Unauthorized".into())));
            }
            Ok(resp)
        }).await?;
    let (mut w, mut r) = ws.split();
    while let Some(m) = r.next().await {
        if let Message::Text(t) = m? {
            let cmd: Command = match serde_json::from_str(&t) {
                Ok(c) => c,
                Err(e) => {
                    let _ = w.send(Message::Text(serde_json::to_string(&Response::Error { message: e.to_string() })?)).await;
                    continue;
                }
            };
            let resp = match cmd {
                Command::GetStatus => {
                    let v = state.vm.lock().await;
                    Response::Status { running: v.is_running(), cpu_cores: v.config().cpu.cores, ram_mb: v.config().memory.ram_mb, gpu: v.config().gpu.model.clone() }
                }
                Command::GetStats => {
                    let v = state.vm.lock().await;
                    let s: VmStats = v.stats();
                    Response::Stats { snapshots: s.snapshots, dirty_pages: s.dirty_pages }
                }
                Command::Stop => { state.vm.lock().await.stop(); Response::Ok { message: "Stopped".into() } }
                Command::TerminalInput { data } => {
                    tracing::info!(target: "terminal", "input: {}", data);
                    Response::TerminalOutput { data: format!("$ {}\r\n", data.trim()) }
                }
                _ => Response::Ok { message: "OK".into() },
            };
            w.send(Message::Text(serde_json::to_string(&resp)?)).await?;
        }
    }
    Ok(())
}