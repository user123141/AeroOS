//! Multi-VM manager.
//!
//! Позволяет запускать несколько VM из одной панели:
//!   - Создание/удаление экземпляров
//!   - Старт/стоп/пауза каждой
//!   - Статистика по каждой
//!   - Общий лимит ресурсов хоста

use crate::hypervisor::VirtualMachine;
use anyhow::{bail, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type VmId = String;

pub struct VmInstance {
    pub id: VmId,
    pub name: String,
    pub vm: Arc<Mutex<VirtualMachine>>,
    pub created_at: i64,
}

pub struct MultiVmManager {
    vms: HashMap<VmId, VmInstance>,
    max_vms: u32,
}

impl MultiVmManager {
    pub fn new(max_vms: u32) -> Self {
        Self {
            vms: HashMap::new(),
            max_vms,
        }
    }

    pub fn create(&mut self, id: &str, name: &str, vm: VirtualMachine) -> Result<()> {
        if self.vms.len() as u32 >= self.max_vms {
            bail!("Maximum VM count reached ({})", self.max_vms);
        }
        if self.vms.contains_key(id) {
            bail!("VM '{}' already exists", id);
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;
        self.vms.insert(
            id.into(),
            VmInstance {
                id: id.into(),
                name: name.into(),
                vm: Arc::new(Mutex::new(vm)),
                created_at: now,
            },
        );
        tracing::info!("VM '{}' created", id);
        Ok(())
    }

    pub fn remove(&mut self, id: &str) -> Result<()> {
        match self.vms.remove(id) {
            Some(_) => {
                tracing::info!("VM '{}' removed", id);
                Ok(())
            }
            None => bail!("VM '{}' not found", id),
        }
    }

    pub fn get(&self, id: &str) -> Option<&VmInstance> {
        self.vms.get(id)
    }

    pub fn list(&self) -> Vec<VmId> {
        self.vms.keys().cloned().collect()
    }

    pub fn count(&self) -> usize {
        self.vms.len()
    }

    /// Статистика по всем VM.
    pub async fn all_stats(&self) -> HashMap<VmId, VmStats> {
        let mut out = HashMap::new();
        for (id, inst) in &self.vms {
            let vm = inst.vm.lock().await;
            let s = vm.stats();
            out.insert(
                id.clone(),
                VmStats {
                    id: id.clone(),
                    name: inst.name.clone(),
                    running: vm.is_running(),
                    snapshots: s.snapshots,
                    dirty_pages: s.dirty_pages,
                },
            );
        }
        out
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct VmStats {
    pub id: VmId,
    pub name: String,
    pub running: bool,
    pub snapshots: u64,
    pub dirty_pages: u64,
}
