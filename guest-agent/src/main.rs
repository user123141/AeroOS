//! AeroAgent — гостевой демон для AeroOS.
//!
//! Компилируется под Linux, работает внутри виртуалки.
//! Общается с хостом через virtio-console (/dev/hvc0).
//!
//! Команды (JSON, по одной строке):
//!   {"cmd": "ping"}
//!   {"cmd": "shutdown"}
//!   {"cmd": "exec", "program": "apt", "args": ["update"]}
//!   {"cmd": "sync_time"}

use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::process::Command;

#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
enum Command {
    Ping,
    Shutdown,
    Exec { program: String, args: Vec<String> },
    SyncTime { unix_ts: i64 },
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Response {
    Pong { version: String },
    Ok { message: String },
    Error { message: String },
    ExecOutput { stdout: String, stderr: String, code: i32 },
}

fn main() -> std::io::Result<()> {
    let console_path = std::env::var("AERO_CONSOLE").unwrap_or_else(|_| "/dev/hvc0".to_string());
    let file = match std::fs::OpenOptions::new().read(true).write(true).open(&console_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Cannot open {}: {}", console_path, e);
            return Ok(());
        }
    };

    let mut writer = file.try_clone()?;
    let reader = BufReader::new(file);

    writeln!(writer, "{{\"type\":\"ok\",\"message\":\"aeroagent ready\"}}")?;
    writer.flush()?;

    for line in reader.lines() {
        let line = match line { Ok(l) => l, Err(_) => break };
        if line.is_empty() { continue; }
        let cmd: Command = match serde_json::from_str(&line) {
            Ok(c) => c,
            Err(e) => {
                let resp = Response::Error { message: format!("parse error: {}", e) };
                writeln!(writer, "{}", serde_json::to_string(&resp).unwrap())?;
                writer.flush()?;
                continue;
            }
        };

        let resp = handle(cmd);
        writeln!(writer, "{}", serde_json::to_string(&resp).unwrap())?;
        writer.flush()?;
    }

    Ok(())
}

fn handle(cmd: Command) -> Response {
    match cmd {
        Command::Ping => Response::Pong { version: env!("CARGO_PKG_VERSION").into() },
        Command::Shutdown => {
            tracing::info!("shutdown requested");
            let _ = Command::new("shutdown").args(["-h", "now"]).spawn();
            Response::Ok { message: "shutting down".into() }
        }
        Command::Exec { program, args } => {
            match Command::new(&program).args(&args).output() {
                Ok(o) => Response::ExecOutput {
                    stdout: String::from_utf8_lossy(&o.stdout).into(),
                    stderr: String::from_utf8_lossy(&o.stderr).into(),
                    code: o.status.code().unwrap_or(-1),
                },
                Err(e) => Response::Error { message: format!("exec failed: {}", e) },
            }
        }
        Command::SyncTime { unix_ts } => {
            let _ = Command::new("date").arg("-s").arg(format!("@{}", unix_ts)).spawn();
            Response::Ok { message: "time synced".into() }
        }
    }
}