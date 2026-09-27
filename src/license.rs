use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const AEROOS_PUBLIC_KEY: &str = "MCowBQYDK2VwAyEAGb9ECWmEzf6FQbrBZ9w7lshQhqowtrbLDFw4rXAxZuE=";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Community,
    Pro,
    Enterprise,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub holder: String,
    pub email: String,
    pub tier: Tier,
    pub issued_at: i64,
    pub expires_at: i64,
    #[serde(default)]
    pub features: Vec<String>,
    /// HWID (hex) — если задан, лицензия работает только на этом железе.
    /// Пустая строка = лицензия универсальная.
    #[serde(default)]
    pub hwid: String,
    pub signature: String,
}

impl License {
    pub fn signing_payload(&self) -> String {
        format!(
            "aeroos-v1|{}|{}|{:?}|{}|{}|{}|{}",
            self.holder,
            self.email,
            self.tier,
            self.issued_at,
            self.expires_at,
            self.features.join(","),
            self.hwid
        )
    }

    pub fn verify(&self) -> Result<()> {
        let pk = B64.decode(AEROOS_PUBLIC_KEY).context("decode pk")?;
        let pk_arr: [u8; 32] = pk.as_slice().try_into().context("pk size")?;
        let vk = VerifyingKey::from_bytes(&pk_arr).context("parse pk")?;
        let sig_bytes = B64.decode(&self.signature).context("decode sig")?;
        let sig = Signature::from_slice(&sig_bytes).context("parse sig")?;
        vk.verify(self.signing_payload().as_bytes(), &sig)
            .context("signature verification failed")?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;
        if self.expires_at > 0 && now > self.expires_at {
            bail!("License expired");
        }

        // HWID check
        if !self.hwid.is_empty() {
            crate::hwid::check_hwid(&self.hwid).context("HWID check failed")?;
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
            return Ok(Self {
                license: None,
                tier: Tier::Community,
            });
        }
        let text = std::fs::read_to_string(path)?;
        let lic: License = serde_json::from_str(&text)?;
        lic.verify()?;
        tracing::info!(
            "License OK: {:?}, hwid={}",
            lic.tier,
            if lic.hwid.is_empty() {
                "universal"
            } else {
                &lic.hwid
            }
        );
        Ok(Self {
            tier: lic.tier,
            license: Some(lic),
        })
    }
}
