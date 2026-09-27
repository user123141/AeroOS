//! AES-256-GCM encryption for snapshots.
//! Key derivation: Argon2id(password, salt, 100_000 iterations).

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use aes_gcm::aead::rand_core::RngCore;
use anyhow::{bail, Context, Result};

pub const SALT_LEN: usize = 16;
pub const NONCE_LEN: usize = 12;
pub const KEY_LEN: usize = 32;

#[derive(Clone)]
pub struct Encryptor {
    cipher: Aes256Gcm,
    salt: [u8; SALT_LEN],
}

impl Encryptor {
    /// Derive key from password using Argon2id.
    pub fn new(password: &[u8], salt: [u8; SALT_LEN]) -> Result<Self> {
        use argon2::{Argon2, Algorithm, Params, Version};
        let params = Params::new(64 * 1024, 3, 1, Some(KEY_LEN))
            .map_err(|e| anyhow::anyhow!("argon2 params: {}", e))?;
        let a = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        let mut key_bytes = [0u8; KEY_LEN];
        a.hash_password_into(password, &salt, &mut key_bytes)
            .map_err(|e| anyhow::anyhow!("argon2: {}", e))?;
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        Ok(Self { cipher: Aes256Gcm::new(key), salt })
    }

    /// Encrypt plaintext. Returns nonce || ciphertext (16-byte tag appended).
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ct = self.cipher.encrypt(nonce, plaintext)
            .map_err(|e| anyhow::anyhow!("encrypt: {}", e))?;
        let mut out = Vec::with_capacity(NONCE_LEN + ct.len());
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&ct);
        Ok(out)
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < NONCE_LEN + 16 { bail!("ciphertext too short"); }
        let (nonce_bytes, ct) = data.split_at(NONCE_LEN);
        let nonce = Nonce::from_slice(nonce_bytes);
        let pt = self.cipher.decrypt(nonce, ct)
            .map_err(|e| anyhow::anyhow!("decrypt: {}", e))?;
        Ok(pt)
    }

    pub fn salt(&self) -> [u8; SALT_LEN] { self.salt }
}

/// Generate a random salt.
pub fn random_salt() -> [u8; SALT_LEN] {
    let mut s = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut s);
    s
}

/// Read encryption password from env or key file.
pub fn get_password() -> Result<Vec<u8>> {
    if let Ok(p) = std::env::var("AEROOS_ENCRYPTION_PASSWORD") {
        return Ok(p.into_bytes());
    }
    let keyfile = std::path::PathBuf::from("aeroos.key");
    if keyfile.exists() {
        return std::fs::read(&keyfile).context("read aeroos.key");
    }
    bail!("Encryption enabled but no AEROOS_ENCRYPTION_PASSWORD or aeroos.key")
}