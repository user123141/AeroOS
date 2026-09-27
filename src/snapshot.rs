use crate::config::SnapshotConfig;
use crate::security;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
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

    /// Вызывается синхронно, указатель живёт только внутри функции.
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
        for page in &pages {
            let off = (*page as usize) * P;
            if off + P > size {
                break;
            }
            let data = unsafe { std::slice::from_raw_parts(memory.add(off), P) };
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
        tracing::info!("Incremental: {} pages", manifest.len());
        Ok(())
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

    /// Фоновый стриминг: копирует память в Vec<u8> ДО await, чтобы указатель не жил между await.
    pub async fn start_streaming(
        manager: Arc<Mutex<SnapshotManager>>,
        vm: Arc<Mutex<crate::hypervisor::VirtualMachine>>,
        interval_ms: u64,
    ) {
        let mut t = interval(Duration::from_millis(interval_ms));
        loop {
            t.tick().await;

            // Сначала собираем всё в синхронном блоке, чтобы *mut u8 не пересекал await.
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

        let mut memory = vec![0u8; 4096];
        memory[0] = 0xAB;
        memory[4095] = 0xCD;

        mgr.create_incremental(memory.as_ptr(), memory.len(), &[])
            .unwrap();

        let mut restored = vec![0u8; 4096];
        mgr.restore(restored.as_mut_ptr(), restored.len()).unwrap();
        assert_eq!(restored[0], 0xAB);
        assert_eq!(restored[4095], 0xCD);
    }

    #[test]
    fn test_snapshot_dedup() {
        let dir = tempdir().unwrap();
        let config = SnapshotConfig {
            dir: dir.path().to_str().unwrap().to_string(),
            ..Default::default()
        };
        let mut mgr = SnapshotManager::new(&config);

        let memory = vec![0u8; 4096];
        mgr.create_incremental(memory.as_ptr(), memory.len(), &[])
            .unwrap();
        mgr.create_incremental(memory.as_ptr(), memory.len(), &[])
            .unwrap();

        let blocks: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("blk_"))
            .collect();
        assert_eq!(blocks.len(), 1);
    }
}
/// Проверка, является ли блок полностью нулевым.
/// Если да — не сохраняем его, а помечаем в манифесте как "zero".
fn is_zero_page(data: &[u8]) -> bool {
    data.iter().all(|&b| b == 0)
}
