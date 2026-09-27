//! Host-side протокол для AeroAgent.
//! Общается с гостевым демоном через virtio-console.
//!
//! Формат: JSON-строки, по одной на команду.

use anyhow::Result;
use serde::{Deserialize, Serialize};

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

pub struct AgentClient {
    // TODO: подключение к virtio-console каналу
}

impl AgentClient {
    pub fn new() -> Self {
        Self {}
    }

    /// Отправить команду гостю и получить ответ.
    pub async fn send(&self, cmd: HostCommand) -> Result<GuestResponse> {
        let json = serde_json::to_string(&cmd)?;
        tracing::info!(target: "agent", "→ {}", json);

        // TODO: реальная передача через virtio-console
        // Пока — заглушка, отвечает что готово
        let resp = match cmd {
            HostCommand::Ping => GuestResponse::Pong {
                version: "0.1.0".into(),
            },
            HostCommand::Exec { program, .. } => GuestResponse::ExecOutput {
                stdout: format!("(stub) {} invoked", program),
                stderr: String::new(),
                code: 0,
            },
            _ => GuestResponse::Ok {
                message: "(stub) ok".into(),
            },
        };
        Ok(resp)
    }
}

impl Default for AgentClient {
    fn default() -> Self {
        Self::new()
    }
}
