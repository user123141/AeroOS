//! aeroctl — CLI для управления AeroOS.

use clap::{Parser, Subcommand};
use colored::*;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio_tungstenite::tungstenite::Message;

const DEFAULT_HTTP: &str = "http://127.0.0.1:8080";
const DEFAULT_WS: &str = "ws://127.0.0.1:8081";

#[derive(Parser)]
#[command(
    name = "aeroctl",
    version,
    about = "AeroOS control tool",
    long_about = "Управление AeroOS из командной строки: статус, снапшоты, команды в гостя."
)]
struct Cli {
    /// HTTP endpoint AeroOS
    #[arg(long, default_value = DEFAULT_HTTP)]
    http: String,

    /// WebSocket endpoint AeroOS
    #[arg(long, default_value = DEFAULT_WS)]
    ws: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Показать статус VM
    Status,

    /// Остановить VM
    Stop,

    /// Управление снапшотами
    #[command(subcommand)]
    Snapshot(SnapshotCmd),

    /// Выполнить команду внутри гостя (через AeroAgent)
    Exec {
        /// Программа
        program: String,
        /// Аргументы
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
enum SnapshotCmd {
    /// Создать снапшот
    Create { name: String },
    /// Восстановить снапшот
    Restore { name: String },
    /// Список снапшотов
    List,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum Command {
    GetStatus,
    GetStats,
    Stop,
    Snapshot { name: String },
    Restore { name: String },
    ListSnapshots,
    TerminalInput { data: String },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[allow(dead_code)]
enum Response {
    Ok {
        message: String,
    },
    Error {
        message: String,
    },
    Status {
        running: bool,
        cpu_cores: u32,
        ram_mb: u64,
        gpu: String,
    },
    Stats {
        snapshots: u64,
        dirty_pages: u64,
    },
    Snapshots {
        list: Vec<String>,
    },
    TerminalOutput {
        data: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Получаем токен
    let token_url = format!("{}/api/token", cli.http);
    let token = match reqwest_get(&token_url).await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{} {}", "ERROR:".red().bold(), e);
            eprintln!("{}", "Убедись, что aeroos.exe запущен.".yellow());
            std::process::exit(1);
        }
    };

    let ws_url = format!("{}/?token={}", cli.ws, token);

    match cli.command {
        Commands::Status => {
            let r = send_ws(&ws_url, Command::GetStatus).await?;
            print_response(&r);
        }
        Commands::Stop => {
            let r = send_ws(&ws_url, Command::Stop).await?;
            print_response(&r);
        }
        Commands::Snapshot(SnapshotCmd::Create { name }) => {
            let r = send_ws(&ws_url, Command::Snapshot { name: name.clone() }).await?;
            print_response(&r);
        }
        Commands::Snapshot(SnapshotCmd::Restore { name }) => {
            let r = send_ws(&ws_url, Command::Restore { name: name.clone() }).await?;
            print_response(&r);
        }
        Commands::Snapshot(SnapshotCmd::List) => {
            // Список — не реализован в ipc.rs, шлём как особый случай
            let r = send_ws_raw(&ws_url, r#"{"type":"ListSnapshots"}"#).await?;
            print_response(&r);
        }
        Commands::Exec { program, args } => {
            let cmd_str = if args.is_empty() {
                program.clone()
            } else {
                format!("{} {}", program, args.join(" "))
            };
            // Через TerminalInput — временный обход
            let r = send_ws(&ws_url, Command::TerminalInput { data: cmd_str }).await?;
            print_response(&r);
        }
    }

    Ok(())
}

async fn reqwest_get(url: &str) -> anyhow::Result<String> {
    // Простой HTTP GET через tokio
    let url = url.strip_prefix("http://").unwrap_or(url);
    let mut stream = tokio::net::TcpStream::connect(url).await?;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let req = format!(
        "GET /api/token HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        url
    );
    stream.write_all(req.as_bytes()).await?;
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).await?;
    let text = String::from_utf8_lossy(&buf);
    let body = text.split("\r\n\r\n").nth(1).unwrap_or("");
    let v: serde_json::Value = serde_json::from_str(body)?;
    Ok(v["token"].as_str().unwrap_or("").to_string())
}

async fn send_ws(url: &str, cmd: Command) -> anyhow::Result<Response> {
    let json = serde_json::to_string(&cmd)?;
    send_ws_raw(url, &json).await
}

async fn send_ws_raw(url: &str, json: &str) -> anyhow::Result<Response> {
    let (mut ws, _) = tokio_tungstenite::connect_async(url).await?;
    ws.send(Message::Text(json.to_string())).await?;

    let timeout = tokio::time::timeout(Duration::from_secs(5), ws.next()).await?;
    if let Some(msg) = timeout {
        let msg = msg?;
        if let Message::Text(text) = msg {
            let resp: Response = serde_json::from_str(&text)?;
            return Ok(resp);
        }
    }
    anyhow::bail!("No response from AeroOS")
}

fn print_response(r: &Response) {
    match r {
        Response::Ok { message } => println!("{} {}", "OK:".green().bold(), message),
        Response::Error { message } => println!("{} {}", "ERROR:".red().bold(), message),
        Response::Status {
            running,
            cpu_cores,
            ram_mb,
            gpu,
        } => {
            println!("{}", "AeroOS Status".bold().underline());
            println!(
                "  State:     {}",
                if *running {
                    "Running".green()
                } else {
                    "Stopped".red()
                }
            );
            println!("  CPU cores: {}", cpu_cores);
            println!("  RAM:       {} MB", ram_mb);
            println!("  GPU:       {}", gpu);
        }
        Response::Stats {
            snapshots,
            dirty_pages,
        } => {
            println!("{}", "Statistics".bold().underline());
            println!("  Snapshots:    {}", snapshots);
            println!("  Dirty pages:  {}", dirty_pages);
        }
        Response::Snapshots { list } => {
            println!("{}", "Snapshots".bold().underline());
            for s in list {
                println!("  - {}", s);
            }
            if list.is_empty() {
                println!("  {}", "(none)".dimmed());
            }
        }
        Response::TerminalOutput { data } => {
            print!("{}", data);
        }
    }
}
