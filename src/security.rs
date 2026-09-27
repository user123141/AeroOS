use anyhow::{bail, Context, Result};
use rand::Rng;
use sha2::{Digest, Sha256};
use std::path::Path;

pub const KERNEL_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";
pub const INITRAMFS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

pub fn sha256_file(path: &Path) -> Result<String> {
    let d = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let mut h = Sha256::new();
    h.update(&d);
    Ok(hex::encode(h.finalize()))
}

pub fn verify_artifacts() -> Result<()> {
    let k = Path::new("kernel/aeroos-kernel");
    let i = Path::new("kernel/aeroos-initramfs");
    if !k.exists() || !i.exists() {
        return Ok(());
    }
    let kh = sha256_file(k)?;
    if KERNEL_HASH != "0000000000000000000000000000000000000000000000000000000000000000"
        && kh != KERNEL_HASH
    {
        bail!("Kernel hash mismatch!");
    }
    Ok(())
}

pub fn generate_session_token() -> String {
    let mut rng = rand::thread_rng();
    let b: [u8; 16] = rng.gen();
    hex::encode(b)
}

pub fn check_disk_space(dir: &Path, required_mb: u64) -> Result<()> {
    let tf = dir.join(".aeroos_disk_test");
    let sz = required_mb * 1024 * 1024;
    if sz > 1024 * 1024 * 1024 {
        return Ok(());
    }
    std::fs::write(&tf, vec![0u8; sz as usize]).context("Not enough disk space")?;
    std::fs::remove_file(&tf).ok();
    Ok(())
}
