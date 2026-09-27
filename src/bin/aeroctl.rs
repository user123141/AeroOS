//! aeroctl — CLI для AeroOS.

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
    Init { name: String, #[arg(long, default_value = "Aerofile")] out: PathBuf },
    Up { #[arg(long, default_value = "Aerofile")] file: PathBuf },
    Down,
    Status,
    Hwid,
    #[command(subcommand)]
    Migrate(MigrateCmd),
    #[command(subcommand)]
    Oci(OciCmd),
}

#[derive(Subcommand)]
enum MigrateCmd {
    Save { file: PathBuf },
    Load { file: PathBuf },
}

#[derive(Subcommand)]
enum OciCmd {
    Pull { image: String },
    List,
    ToInitramfs { image: String, out: PathBuf },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { name, out } => {
            let af = template(&name);
            std::fs::write(&out, af)?;
            println!("{} Created {}", "OK".green().bold(), out.display());
        }
        Commands::Up { file } => {
            println!("{} Loading Aerofile: {}", ">>".cyan(), file.display());
            if !file.exists() { anyhow::bail!("Aerofile not found"); }
            let text = std::fs::read_to_string(&file)?;
            let v: toml::Value = toml::from_str(&text)?;
            let name = v.get("vm").and_then(|v| v.get("name")).and_then(|v| v.as_str()).unwrap_or("unnamed");
            let cores = v.get("vm").and_then(|v| v.get("cores")).and_then(|v| v.as_integer()).unwrap_or(2);
            let ram = v.get("vm").and_then(|v| v.get("ram")).and_then(|v| v.as_integer()).unwrap_or(2048);
            println!("  VM:     {}", name);
            println!("  Cores:  {}", cores);
            println!("  RAM:    {} MB", ram);
            println!("  {}", "Start aeroos.exe to bring this up".yellow());
        }
        Commands::Down => println!("{} Send stop via WS", ">>".cyan()),
        Commands::Status => println!("{} Check http://127.0.0.1:8080", ">>".cyan()),
        Commands::Hwid => {
            println!("{} Run: gen-license --show-hwid", ">>".cyan());
        }
        Commands::Migrate(MigrateCmd::Save { file }) => {
            println!("{} Saving migration to {}", ">>".cyan(), file.display());
        }
        Commands::Migrate(MigrateCmd::Load { file }) => {
            println!("{} Loading migration from {}", ">>".cyan(), file.display());
        }
        Commands::Oci(OciCmd::Pull { image }) => {
            println!("{} OCI pull: {}", ">>".cyan(), image);
        }
        Commands::Oci(OciCmd::List) => {
            println!("{} Available OCI images:", ">>".cyan());
            println!("  - alpine:latest");
            println!("  - ubuntu:24.04");
            println!("  - archlinux:latest");
        }
        Commands::Oci(OciCmd::ToInitramfs { image, out }) => {
            println!("{} Converting {} to initramfs at {}", ">>".cyan(), image, out.display());
        }
    }
    Ok(())
}

fn template(name: &str) -> String {
    format!(r#"[vm]
name = "{}"
image = "alpine-3.20"
cores = 2
ram = 2048
disk_mb = 1024

[network]
mode = "user"

[provision]
run = ["apk add --no-cache curl"]

# [[share]]
# host = "C:\\Projects"
# guest = "/mnt/projects"
"#, name)
}