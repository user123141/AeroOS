//! Build script for AeroOS.
//!
//! - Cosmo APE build (feature `ape`)
//! - Crypto-UI binding: encrypts web/js/main.js with AES-256-GCM
//!   when AEROOS_UI_KEY env var is set.
//! - Custom obfuscation: XOR-string pass on selected JS files.

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    #[cfg(feature = "ape")]
    cosmo_build::apeify().expect("APE build failed");

    println!("cargo:rerun-if-changed=web/");
    println!("cargo:rerun-if-changed=config/aeroos.config.toml");
    println!("cargo:rerun-if-env-changed=AEROOS_UI_KEY");
    println!("cargo:rerun-if-env-changed=AEROOS_OBFUSCATE");

    if let Ok(key_hex) = env::var("AEROOS_UI_KEY") {
        if let Err(e) = encrypt_ui(&key_hex) {
            println!("cargo:warning=Crypto-UI: {}", e);
        }
    }

    if env::var("AEROOS_OBFUSCATE").is_ok() {
        if let Err(e) = obfuscate_web() {
            println!("cargo:warning=Obfuscate: {}", e);
        }
    }
}

fn encrypt_ui(key_hex: &str) -> Result<(), Box<dyn std::error::Error>> {
    use aes_gcm::aead::{Aead, KeyInit, OsRng};
    use aes_gcm::aead::rand_core::RngCore;
    use aes_gcm::{Aes256Gcm, Key, Nonce};

    let key_bytes = hex::decode(key_hex)?;
    if key_bytes.len() != 32 {
        return Err("AEROOS_UI_KEY must be 32 bytes hex (64 chars)".into());
    }

    let src = Path::new("web/js/main.js");
    if !src.exists() {
        return Err("web/js/main.js not found".into());
    }
    let plaintext = fs::read(src)?;

    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // FIX: aes_gcm::Error doesn't implement std::error::Error
    // (deliberately opaque to avoid side-channel leaks).
    // Map it to a plain string manually.
    let ct = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|_| "AES-GCM encryption failed")?;

    let out_dir = Path::new("web_encrypted");
    fs::create_dir_all(out_dir)?;

    let mut blob = Vec::with_capacity(12 + ct.len());
    blob.extend_from_slice(&nonce_bytes);
    blob.extend_from_slice(&ct);
    fs::write(out_dir.join("main.js.enc"), blob)?;

    println!(
        "cargo:warning=Crypto-UI: main.js encrypted ({} bytes)",
        plaintext.len()
    );
    Ok(())
}

/// Simple XOR-obfuscation of selected JS files.
/// Produces *.js.obf next to originals.
fn obfuscate_web() -> Result<(), Box<dyn std::error::Error>> {
    let files = [
        "web/js/terminal.js",
        "web/js/search.js",
        "web/js/notifications.js",
    ];
    let key: u8 = 0xA7;

    for rel in files.iter() {
        let src = Path::new(rel);
        if !src.exists() {
            continue;
        }
        let data = fs::read(src)?;
        let obf: Vec<u8> = data.iter().map(|b| b ^ key).collect();
        let out = format!("{}.obf", rel);
        fs::write(&out, obf)?;
        println!("cargo:warning=Obfuscate: {} -> {}", rel, out);
    }
    Ok(())
}