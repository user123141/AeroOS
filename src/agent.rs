//! Host-side протокол для AeroAgent.
//! Общается с гостевым демоном через virtio-console.
//!
//! Формат: JSON-строки, по одной на команду.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum HostCommand {
    Ping,
    Shutdown,
    Exec { program: String, args: Vec<String> },
    SyncTime { unix_ts: i64 },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GuestResponse {
    Pong {
        version: String,
    },
    Ok {
        message: String,
    },
    Error {
        message: String,
    },
    ExecOutput {
        stdout: String,
        stderr: String,
        code: i32,
    },
}

/// Канал для общения с virtio-console.
#[derive(Clone)]
pub struct AgentChannel {
    /// Пакеты от хоста к гостю (virtio-console TX).
    pub to_guest: Arc<Mutex<VecDeque<Vec<u8>>>>,
    /// Пакеты от гостя к хосту (virtio-console RX).
    pub from_guest: Arc<Mutex<VecDeque<Vec<u8>>>>,
}

impl AgentChannel {
    pub fn new() -> Self {
        Self {
            to_guest: Arc::new(Mutex::new(VecDeque::new())),
            from_guest: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn send_to_guest(&self, data: Vec<u8>) {
        self.to_guest.lock().unwrap().push_back(data);
    }

    pub fn recv_from_guest(&self) -> Option<Vec<u8>> {
        self.from_guest.lock().unwrap().pop_front()
    }

    pub fn push_from_guest(&self, data: Vec<u8>) {
        self.from_guest.lock().unwrap().push_back(data);
    }

    pub fn take_to_guest(&self) -> Option<Vec<u8>> {
        self.to_guest.lock().unwrap().pop_front()
    }
}

impl Default for AgentChannel {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AgentClient {
    channel: AgentChannel,
}

impl AgentClient {
    pub fn new(channel: AgentChannel) -> Self {
        Self { channel }
    }

    pub fn channel(&self) -> AgentChannel {
        self.channel.clone()
    }

    /// Отправить команду гостю и получить ответ.
    pub async fn send(&self, cmd: HostCommand) -> Result<GuestResponse> {
        let json = serde_json::to_string(&cmd)? + "\n";
        tracing::info!(target: "agent", "→ {}", json.trim());
        self.channel.send_to_guest(json.into_bytes());

        // Ждём ответ (в реальности — с timeout и retry)
        let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(5);
        loop {
            if tokio::time::Instant::now() > deadline {
                anyhow::bail!("Agent timeout");
            }
            if let Some(data) = self.channel.recv_from_guest() {
                let text = String::from_utf8_lossy(&data);
                let resp: GuestResponse = serde_json::from_str(text.trim())?;
                return Ok(resp);
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    }
}
