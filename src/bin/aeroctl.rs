//! aeroctl — CLI для AeroOS.
//!
//! Команды:
//!   aeroctl init <name>        Создать Aerofile
//!   aeroctl up                 Запустить VM по Aerofile
//!   aeroctl down               Остановить
//!   aeroctl status             Статус
//!   aeroctl snapshot create/list/restore
//!   aeroctl exec <cmd>         Выполнить в госте

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "aeroctl", version, about = "AeroOS control tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Создать Aerofile
    Init {
        name: String,
        #[arg(long, default_value = "Aerofile")]
        out: PathBuf,
    },
    /// Показать HWID текущей машины
    Hwid,
    /// Запустить VM по Aerofile
    Up {
        #[arg(long, default_value = "Aerofile")]
        file: PathBuf,
    },
    /// Остановить VM
    Down,
    /// Показать статус (заглушка, зависит от запущенного aeroos.exe)
    Status,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { name, out } => {
            let af = aeroos_aerofile_template(&name);
            std::fs::write(&out, af)?;
            println!("{} Created {}", "OK".green().bold(), out.display());
        }
        Commands::Hwid => {
            let out = std::process::Command::new("cmd")
                .args(["/C", "vol C:"])
                .output();
            println!("Run: cargo run --bin gen-license -- --show-hwid");
            println!("(fallback: {:?})", out.is_ok());
        }
        Commands::Up { file } => {
            println!("{} Loading Aerofile: {}", ">>".cyan(), file.display());
            if !file.exists() {
                anyhow::bail!("Aerofile not found: {}", file.display());
            }
            let text = std::fs::read_to_string(&file)?;
            let v: toml::Value = toml::from_str(&text)?;
            let name = v
                .get("vm")
                .and_then(|v| v.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("unnamed");
            println!("  VM name: {}", name);
            println!("  {}", "Launch aeroos.exe to bring this up".yellow());
        }
        Commands::Down => {
            println!("{} Send stop to running aeroos.exe", ">>".cyan());
        }
        Commands::Status => {
            println!("{} Check http://127.0.0.1:8080", ">>".cyan());
        }
    }
    Ok(())
}

fn aeroos_aerofile_template(name: &str) -> String {
    format!(
        r#"[vm]
name = "{}"
image = "alpine-3.20"
cores = 2
ram = 2048
disk_mb = 1024

[network]
mode = "user"

[provision]
run = [
  "apk add --no-cache curl",
]

# [[share]]
# host = "C:\\Projects"
# guest = "/mnt/projects"
"#,
        name
    )
}
