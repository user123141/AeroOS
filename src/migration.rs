//! Live migration: снапшот VM → передача → восстановление.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationHeader {
    pub version: u32,
    pub vm_name: String,
    pub ram_mb: u64,
    pub cpu_cores: u32,
    pub snapshot_ts: i64,
    pub sha256: String,
}

pub const MIGRATION_VERSION: u32 = 1;

pub struct Migrator;

impl Migrator {
    pub fn save_to_file(memory: &[u8], header: MigrationHeader, out_path: &Path) -> Result<()> {
        let mut file = std::fs::File::create(out_path)?;
        let header_json = serde_json::to_vec(&header)?;
        let header_len = header_json.len() as u32;
        file.write_all(&header_len.to_le_bytes())?;
        file.write_all(&header_json)?;

        const CHUNK: usize = 4 * 1024 * 1024;
        for chunk in memory.chunks(CHUNK) {
            let compressed = lz4_flex::compress_prepend_size(chunk);
            let clen = compressed.len() as u32;
            file.write_all(&clen.to_le_bytes())?;
            file.write_all(&compressed)?;
        }
        file.sync_all()?;
        tracing::info!("Migration saved: {}", out_path.display());
        Ok(())
    }

    pub fn load_from_file(path: &Path) -> Result<(MigrationHeader, Vec<u8>)> {
        let mut file = std::fs::File::open(path)?;
        let mut len_buf = [0u8; 4];
        file.read_exact(&mut len_buf)?;
        let header_len = u32::from_le_bytes(len_buf) as usize;
        let mut header_buf = vec![0u8; header_len];
        file.read_exact(&mut header_buf)?;
        let header: MigrationHeader = serde_json::from_slice(&header_buf)?;

        let mut memory = Vec::with_capacity((header.ram_mb * 1024 * 1024) as usize);
        loop {
            let mut clen_buf = [0u8; 4];
            if file.read_exact(&mut clen_buf).is_err() { break; }
            let clen = u32::from_le_bytes(clen_buf) as usize;
            let mut compressed = vec![0u8; clen];
            file.read_exact(&mut compressed)?;
            let decompressed = lz4_flex::decompress_size_prepended(&compressed)?;
            memory.extend_from_slice(&decompressed);
        }
        tracing::info!("Migration loaded: {}", path.display());
        Ok((header, memory))
    }

    pub fn checksum(memory: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(memory);
        hex::encode(h.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.aeromig");
        let memory = vec![0xABu8; 1024 * 1024];
        let header = MigrationHeader {
            version: MIGRATION_VERSION,
            vm_name: "test".into(),
            ram_mb: 1,
            cpu_cores: 1,
            snapshot_ts: 0,
            sha256: Migrator::checksum(&memory),
        };
        Migrator::save_to_file(&memory, header, &path).unwrap();
        let (h, loaded) = Migrator::load_from_file(&path).unwrap();
        assert_eq!(h.vm_name, "test");
        assert_eq!(loaded.len(), memory.len());
    }
}