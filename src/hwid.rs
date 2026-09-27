//! Hardware ID (HWID) — привязка лицензии к железу.
//!
//! Собирает стабильные идентификаторы:
//!   - Volume serial number системного диска
//!   - CPU vendor + brand
//!   - MAC-адрес первого физического адаптера
//!
//! Хэширует всё через BLAKE3 → 32-байтовый hex.
//!
//! Используется в license.rs для привязки: лицензия работает только на
//! железе, где сгенерирован HWID.

use anyhow::Result;
use sha2::{Digest, Sha256};

#[cfg(windows)]
pub fn compute_hwid() -> Result<String> {
    use std::process::Command;

    let mut data = String::new();

    // Volume serial number системного диска (C:)
    if let Ok(out) = Command::new("cmd").args(["/C", "vol C:"]).output() {
        data.push_str(&String::from_utf8_lossy(&out.stdout));
    }

    // CPU
    if let Ok(out) = Command::new("wmic")
        .args(["cpu", "get", "ProcessorId"])
        .output()
    {
        data.push_str(&String::from_utf8_lossy(&out.stdout));
    }

    // Motherboard
    if let Ok(out) = Command::new("wmic")
        .args(["baseboard", "get", "SerialNumber"])
        .output()
    {
        data.push_str(&String::from_utf8_lossy(&out.stdout));
    }

    // Убираем шум
    let cleaned: String = data
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect();

    if cleaned.len() < 8 {
        anyhow::bail!("Not enough HWID data");
    }

    let hash = Sha256::digest(cleaned.as_bytes());
    Ok(hex::encode(&hash[..16]))
}

#[cfg(not(windows))]
pub fn compute_hwid() -> Result<String> {
    // Fallback для Linux/macOS
    use std::fs;
    let mut data = String::new();
    if let Ok(m) = fs::read_to_string("/etc/machine-id") {
        data.push_str(&m);
    }
    if let Ok(h) = fs::read_to_string("/proc/sys/kernel/hostname") {
        data.push_str(&h);
    }
    let hash = Sha256::digest(data.as_bytes());
    Ok(hex::encode(&hash[..16]))
}

/// Проверка, совпадает ли HWID с ожидаемым (из лицензии).
pub fn check_hwid(expected: &str) -> Result<()> {
    let actual = compute_hwid()?;
    if actual != expected {
        anyhow::bail!("HWID mismatch: expected {}, got {}", expected, actual);
    }
    Ok(())
}
