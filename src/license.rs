//! License system — Ed25519 подписи + HWID привязка + offline-валидация.
//! Enterprise offline: всё криптографически, без сервера.

use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Публичный ключ AeroOS. Заменить на свой при выпуске.
pub const AEROOS_PUBLIC_KEY: &str = "MCowBQYDK2VwAyEAGb9ECWmEzf6FQbrBZ9w7lshQhqowtrbLDFw4rXAxZuE=";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier { Community, Pro, Enterprise }

impl Tier {
    pub fn max_vms(&self) -> u32 {
        match self { Self::Community => 1, Self::Pro => 8, Self::Enterprise => 64 }
    }
    pub fn max_ram_mb(&self) -> u64 {
        match self { Self::Community => 2048, Self::Pro => 32768, Self::Enterprise => 262144 }
    }
    pub fn can_use(&self, feature: &str) -> bool {
        match (self, feature) {
            (Self::Community, "encryption") | (Self::Community, "gpu") |
            (Self::Community, "migration")  | (Self::Community, "oci") |
            (Self::Community, "multi_vm")   | (Self::Community, "api") => false,
            _ => true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub holder: String,
    pub email: String,
    pub tier: Tier,
    pub issued_at: i64,
    pub expires_at: i64,
    #[serde(default)] pub features: Vec<String>,
    /// HWID (hex). Пусто = универсальная.
    #[serde(default)] pub hwid: String,
    /// Nonce — защита от replay.
    #[serde(default)] pub nonce: String,
    pub signature: String,
}

impl License {
    /// Канонический payload для подписи.
    pub fn signing_payload(&self) -> String {
        format!("aeroos-v2|{}|{}|{:?}|{}|{}|{}|{}|{}",
            self.holder, self.email, self.tier,
            self.issued_at, self.expires_at,
            self.features.join(","),
            self.hwid, self.nonce)
    }

    /// Полная offline-валидация: подпись + срок + HWID.
    pub fn verify(&self) -> Result<()> {
        // 1. Подпись Ed25519
        let pk = B64.decode(AEROOS_PUBLIC_KEY).context("decode public key")?;
        let pk_arr: [u8; 32] = pk.as_slice().try_into().context("public key size")?;
        let vk = VerifyingKey::from_bytes(&pk_arr).context("parse public key")?;

        let sig_bytes = B64.decode(&self.signature).context("decode signature")?;
        let sig = Signature::from_slice(&sig_bytes).context("parse signature")?;

        vk.verify(self.signing_payload().as_bytes(), &sig)
            .context("signature verification failed")?;

        // 2. Срок действия
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64;
        if self.issued_at > now + 3600 {
            bail!("License issued in the future");
        }
        if self.expires_at > 0 && now > self.expires_at {
            let days = (now - self.expires_at) / 86400;
            bail!("License expired {} days ago", days);
        }

        // 3. HWID
        if !self.hwid.is_empty() {
            crate::hwid::check_hwid(&self.hwid)
                .context("HWID mismatch")?;
        }

        Ok(())
    }
}

pub struct LicenseState {
    pub license: Option<License>,
    pub tier: Tier,
}

impl LicenseState {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            tracing::info!("No license — Community mode");
            return Ok(Self { license: None, tier: Tier::Community });
        }
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("read {}", path.display()))?;
        let lic: License = serde_json::from_str(&text)
            .context("parse license JSON")?;
        lic.verify()?;
        tracing::info!(
            "License OK: holder={}, tier={:?}, hwid={}, expires={}",
            lic.holder, lic.tier,
            if lic.hwid.is_empty() { "universal" } else { &lic.hwid },
            if lic.expires_at == 0 { "never".into() } else { lic.expires_at.to_string() }
        );
        Ok(Self { tier: lic.tier, license: Some(lic) })
    }

    pub fn check(&self, feature: &str) -> Result<()> {
        if self.tier.can_use(feature) {
            Ok(())
        } else {
            bail!("Feature '{}' not available in {:?} tier", feature, self.tier)
        }
    }

    pub fn max_vms(&self) -> u32 { self.tier.max_vms() }
    pub fn max_ram_mb(&self) -> u64 { self.tier.max_ram_mb() }
}
/// Derive a 32-byte UI key from the license signature.
/// Used for decrypting UI assets (crypto-UI binding).
pub fn derive_ui_key(license: &License) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(b"aeroos-ui-v1");
    h.update(license.signature.as_bytes());
    h.update(license.nonce.as_bytes());
    h.update(license.holder.as_bytes());
    let d = h.finalize();
    let mut k = [0u8; 32];
    k.copy_from_slice(&d);
    k
}