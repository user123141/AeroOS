use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use clap::Parser;
use ed25519_dalek::{Signer, SigningKey};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Tier { Community, Pro, Enterprise }

#[derive(Debug, Serialize, Deserialize)]
struct License {
    holder: String, email: String, tier: Tier,
    issued_at: i64, expires_at: i64,
    #[serde(default)] features: Vec<String>,
    #[serde(default)] hwid: String,
    #[serde(default)] nonce: String,
    signature: String,
}

impl License {
    fn signing_payload(&self) -> String {
        format!("aeroos-v2|{}|{}|{:?}|{}|{}|{}|{}|{}",
            self.holder, self.email, self.tier,
            self.issued_at, self.expires_at,
            self.features.join(","), self.hwid, self.nonce)
    }
}

#[derive(Parser)]
#[command(name = "gen-license", version)]
struct Cli {
    #[arg(long)] gen_keys: bool,
    #[arg(long)] sign: bool,
    #[arg(long)] show_hwid: bool,
    #[arg(long)] private_key: Option<String>,
    #[arg(long, default_value = "Unnamed")] holder: String,
    #[arg(long, default_value = "")] email: String,
    #[arg(long, default_value = "community")] tier: String,
    #[arg(long, default_value = "365")] days: i64,
    #[arg(long, default_value = "")] features: String,
    #[arg(long, default_value = "")] hwid: String,
    #[arg(long, default_value = "license.key")] out: PathBuf,
}

fn compute_hwid() -> Result<String> {
    let mut data = String::new();
    if let Ok(out) = Command::new("cmd").args(["/C", "vol C:"]).output() {
        data.push_str(&String::from_utf8_lossy(&out.stdout));
    }
    if let Ok(out) = Command::new("wmic").args(["cpu", "get", "ProcessorId"]).output() {
        data.push_str(&String::from_utf8_lossy(&out.stdout));
    }
    if let Ok(out) = Command::new("wmic").args(["baseboard", "get", "SerialNumber"]).output() {
        data.push_str(&String::from_utf8_lossy(&out.stdout));
    }
    let cleaned: String = data.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-').collect();
    if cleaned.len() < 8 { anyhow::bail!("Not enough HWID data"); }
    let hash = Sha256::digest(cleaned.as_bytes());
    Ok(hex::encode(&hash[..16]))
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.gen_keys {
        let sk = SigningKey::generate(&mut OsRng);
        let vk = sk.verifying_key();
        println!("Private: {}", B64.encode(sk.to_bytes()));
        println!("Public:  {}", B64.encode(vk.to_bytes()));
        return Ok(());
    }

    if cli.show_hwid {
        println!("{}", compute_hwid()?);
        return Ok(());
    }

    if cli.sign {
        let pk_b64 = cli.private_key
            .or_else(|| std::env::var("AERO_PRIVATE_KEY").ok())
            .context("Missing --private-key")?;
        let pk_bytes = B64.decode(&pk_b64)?;
        let pk_arr: [u8; 32] = pk_bytes.as_slice().try_into().context("key size")?;
        let sk = SigningKey::from_bytes(&pk_arr);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64;
        let expires = if cli.days > 0 { now + cli.days * 86400 } else { 0 };

        let tier = match cli.tier.to_lowercase().as_str() {
            "pro" => Tier::Pro,
            "enterprise" => Tier::Enterprise,
            _ => Tier::Community,
        };
        let features: Vec<String> = cli.features.split(',')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
            .collect();

        let mut nonce_bytes = [0u8; 16];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = hex::encode(nonce_bytes);

        let mut lic = License {
            holder: cli.holder.clone(),
            email: cli.email.clone(),
            tier,
            issued_at: now,
            expires_at: expires,
            features,
            hwid: cli.hwid.clone(),
            nonce,
            signature: String::new(),
        };
        let sig = sk.sign(lic.signing_payload().as_bytes());
        lic.signature = B64.encode(sig.to_bytes());

        let json = serde_json::to_string_pretty(&lic)?;
        std::fs::write(&cli.out, &json)?;
        println!("Written: {}", cli.out.display());
        return Ok(());
    }

    println!("Commands:");
    println!("  --gen-keys         Generate Ed25519 key pair");
    println!("  --show-hwid        Show this machine's HWID");
    println!("  --sign             Sign a license");
    Ok(())
}