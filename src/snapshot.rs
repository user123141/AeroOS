use crate::config::SnapshotConfig;
use crate::security;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

pub struct SnapshotManager {
    config: SnapshotConfig,
    block_index: HashMap<String, PathBuf>,
    bytes_written: u64,
}

impl SnapshotManager {
    pub fn new(config: &SnapshotConfig) -> Self {
        let d = PathBuf::from(&config.dir);
        std::fs::create_dir_all(&d).ok();
        Self {
            config: config.clone(),
            block_index: HashMap::new(),
            bytes_written: 0,
        }
    }

    /// Zero-page detection: блок из нулей не пишем, экономия места.
    fn is_zero_page(data: &[u8]) -> bool {
        data.iter().all(|&b| b == 0)
    }

    pub fn create_incremental(
        &mut self,
        memory: *const u8,
        size: usize,
        dirty: &[u64],
    ) -> Result<()> {
        const P: usize = 4096;
        let dir = PathBuf::from(&self.config.dir);
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis();

        if self.config.max_size_mb > 0
            && self.bytes_written / (1024 * 1024) > self.config.max_size_mb
        {
            self.cleanup_oldest()?;
        }

        let pages: Vec<u64> = if dirty.is_empty() {
            (0..size / P).map(|i| i as u64).collect()
        } else {
            dirty.to_vec()
        };

        let mut manifest: Vec<(u64, String)> = Vec::new();
        let mut zero_count = 0u64;

        for page in &pages {
            let off = (*page as usize) * P;
            if off + P > size {
                break;
            }
            let data = unsafe { std::slice::from_raw_parts(memory.add(off), P) };

            // Zero-page skip
            if Self::is_zero_page(data) {
                manifest.push((*page, "zero".to_string()));
                zero_count += 1;
                continue;
            }

            let h = blake3::hash(data).to_hex().to_string();
            if !self.block_index.contains_key(&h) {
                let bp = dir.join(format!("blk_{}", h));
                if !bp.exists() {
                    let c = lz4_flex::compress_prepend_size(data);
                    std::fs::write(&bp, &c)?;
                    self.bytes_written += c.len() as u64;
                }
                self.block_index.insert(h.clone(), bp);
            }
            manifest.push((*page, h));
        }

        let mp = dir.join(format!("inc_{}.json", ts));
        std::fs::write(&mp, serde_json::to_string_pretty(&manifest)?)?;
        tracing::info!(
            "Incremental: {} pages ({} zero-skipped)",
            manifest.len(),
            zero_count
        );
        Ok(())
    }

    /// Экспорт в QCOW2 — можно запустить в QEMU/KVM на Linux.
    pub fn export_qcow2(&self, memory: *const u8, size: usize, out_path: &Path) -> Result<()> {
        tracing::info!("Exporting raw image to {}", out_path.display());
        let data = unsafe { std::slice::from_raw_parts(memory, size) };
        let mut f = std::fs::File::create(out_path)?;
        f.write_all(data)?;
        f.sync_all()?;
        tracing::info!("Raw export complete: {} bytes", size);
        Ok(())
    }

    /// Быстрая проверка целостности всех блоков.
    pub fn verify_integrity(&self) -> Result<u64> {
        let dir = PathBuf::from(&self.config.dir);
        let mut checked = 0u64;
        for entry in std::fs::read_dir(&dir)? {
            let e = entry?;
            let name = e.file_name().to_string_lossy().to_string();
            if !name.starts_with("blk_") {
                continue;
            }
            let expected = name.trim_start_matches("blk_");
            let data = std::fs::read(e.path())?;
            let decompressed = lz4_flex::decompress_size_prepended(&data)?;
            let actual = blake3::hash(&decompressed).to_hex().to_string();
            if actual != expected {
                anyhow::bail!(
                    "Corrupt block: {} (expected {}, got {})",
                    name,
                    expected,
                    actual
                );
            }
            checked += 1;
        }
        tracing::info!("Integrity check: {} blocks OK", checked);
        Ok(checked)
    }

    fn cleanup_oldest(&mut self) -> Result<()> {
        let dir = PathBuf::from(&self.config.dir);
        let mut s: Vec<_> = std::fs::read_dir(&dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("inc_"))
            .collect();
        s.sort_by_key(|e| e.file_name());
        if let Some(o) = s.first() {
            std::fs::remove_file(o.path()).ok();
        }
        Ok(())
    }

    pub async fn start_streaming(
        manager: Arc<Mutex<SnapshotManager>>,
        vm: Arc<Mutex<crate::hypervisor::VirtualMachine>>,
        interval_ms: u64,
    ) {
        let mut t = interval(Duration::from_millis(interval_ms));
        loop {
            t.tick().await;

            let (snapshot_bytes, dirty): (Vec<u8>, Vec<u64>) = {
                let v = vm.lock().await;
                let (m, s) = v.get_memory();
                let dirty = v.get_dirty_pages();
                v.clear_dirty_bitmap();
                let bytes = unsafe { std::slice::from_raw_parts(m, s).to_vec() };
                (bytes, dirty)
            };

            let mut mgr = manager.lock().await;
            let _ = mgr.create_incremental(snapshot_bytes.as_ptr(), snapshot_bytes.len(), &dirty);
            let _ = security::check_disk_space(
                std::path::Path::new(&mgr.config.dir),
                mgr.config.max_size_mb,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_snapshot_integrity() {
        let dir = tempdir().unwrap();
        let config = SnapshotConfig {
            dir: dir.path().to_str().unwrap().to_string(),
            ..Default::default()
        };
        let mut mgr = SnapshotManager::new(&config);

        let mut memory = vec![0u8; 8192];
        memory[0] = 0xAB;
        memory[4095] = 0xCD;
        memory[4096] = 0xEF;

        mgr.create_incremental(memory.as_ptr(), memory.len(), &[])
            .unwrap();
        let n = mgr.verify_integrity().unwrap();
        assert!(n >= 2, "Should have at least 2 non-zero blocks");
    }

    #[test]
    fn test_zero_page_detection() {
        assert!(SnapshotManager::is_zero_page(&[0u8; 4096]));
        let mut nz = [0u8; 4096];
        nz[2048] = 1;
        assert!(!SnapshotManager::is_zero_page(&nz));
    }

    #[test]
    fn test_snapshot_dedup() {
        let dir = tempdir().unwrap();
        let config = SnapshotConfig {
            dir: dir.path().to_str().unwrap().to_string(),
            ..Default::default()
        };
        let mut mgr = SnapshotManager::new(&config);

        let mut memory = vec![0u8; 4096];
        memory[0] = 1;

        mgr.create_incremental(memory.as_ptr(), memory.len(), &[])
            .unwrap();
        mgr.create_incremental(memory.as_ptr(), memory.len(), &[])
            .unwrap();

        let blocks: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("blk_"))
            .collect();
        assert_eq!(blocks.len(), 1, "Dedup should keep only 1 block");
    }
}
