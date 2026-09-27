use crate::config::AeroConfig;
use crate::data_folder::DataFolder;
use crate::virtio::{GuestMemory, VirtioMmio};
use anyhow::{Context, Result};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use whpx::ffi::*;
use whpx::{HypervisorCapabilities, Partition, VirtualProcessor};

pub struct VirtualMachine {
    config: AeroConfig,
    partition: Partition,
    vcpu: Option<VirtualProcessor>,
    memory: *mut u8,
    memory_size: usize,
    virtio_devices: Vec<VirtioMmio>,
    running: Arc<AtomicBool>,
    dirty_bitmap: Arc<std::sync::Mutex<Vec<u8>>>,
    snapshot_count: Arc<AtomicU64>,
}
unsafe impl Send for VirtualMachine {}
unsafe impl Sync for VirtualMachine {}

struct RawMem {
    ptr: *mut u8,
    size: usize,
}
unsafe impl Send for RawMem {}
unsafe impl Sync for RawMem {}
impl GuestMemory for RawMem {
    fn read(&self, gpa: u64, buf: &mut [u8]) -> bool {
        let off = gpa as usize;
        if off + buf.len() > self.size {
            return false;
        }
        unsafe {
            std::ptr::copy_nonoverlapping(self.ptr.add(off), buf.as_mut_ptr(), buf.len());
        }
        true
    }
    fn write(&self, gpa: u64, buf: &[u8]) -> bool {
        let off = gpa as usize;
        if off + buf.len() > self.size {
            return false;
        }
        unsafe {
            std::ptr::copy_nonoverlapping(buf.as_ptr(), self.ptr.add(off), buf.len());
        }
        true
    }
}

impl VirtualMachine {
    pub fn new(config: &AeroConfig, df: &DataFolder) -> Result<Self> {
        whpx::init_bindings().map_err(|e| anyhow::anyhow!("WHPX init: {}", e))?;
        let caps = HypervisorCapabilities::get().map_err(|e| anyhow::anyhow!("{}", e))?;
        tracing::info!("WHPX: dirty_pages={}", caps.dirty_page_tracking);

        let mut partition = Partition::create().map_err(|e| anyhow::anyhow!("{}", e))?;
        partition
            .set_vcpu_count(config.cpu.cores)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        partition
            .set_property(WHvPartitionPropertyCode::ProcessorCount, config.cpu.cores)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let ram_size = (config.memory.ram_mb * 1024 * 1024) as usize;

        // RAM-only mode: диск в памяти, всё стирается при выходе
        let ram_only = config.snapshot.ram_only;
        if ram_only {
            tracing::info!("RAM-only mode enabled (ephemeral VM)");
        }
        let dirty_bitmap = if caps.dirty_page_tracking {
            let _ = partition.set_property(WHvPartitionPropertyCode::DirtyPageTracking, 1);
            Arc::new(std::sync::Mutex::new(vec![0u8; (ram_size / 4096 + 7) / 8]))
        } else {
            Arc::new(std::sync::Mutex::new(Vec::new()))
        };

        let guest_mem =
            unsafe { whpx::allocate_guest_memory(ram_size).map_err(|e| anyhow::anyhow!("{}", e))? };
        let region = MemoryRegion {
            start: 0,
            size: ram_size as u64,
            host_ptr: guest_mem as *mut _,
        };
        partition
            .map_gpa_range(&region, WHvMapGpaRangeFlags::ReadWriteExecute)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let kd = std::fs::read(df.kernel_path())
            .with_context(|| format!("read {}", df.kernel_path().display()))?;
        let id = std::fs::read(df.initramfs_path())
            .with_context(|| format!("read {}", df.initramfs_path().display()))?;
        unsafe {
            std::ptr::copy_nonoverlapping(kd.as_ptr(), guest_mem, kd.len());
            let off = (kd.len() + 0xFFF) & !0xFFF;
            std::ptr::copy_nonoverlapping(id.as_ptr(), guest_mem.add(off), id.len());
        }
        tracing::info!("Kernel + initramfs loaded");

        let virtio_devices = vec![
            VirtioMmio::new_block(0x10000, 0x1000, "disk.img"),
            VirtioMmio::new_net(0x11000, 0x1000, None),
            VirtioMmio::new_console(0x12000, 0x1000),
            VirtioMmio::new_gpu(0x13000, 0x1000, 640, 480),
        ];

        Ok(Self {
            config: config.clone(),
            partition,
            vcpu: None,
            memory: guest_mem,
            memory_size: ram_size,
            virtio_devices,
            running: Arc::new(AtomicBool::new(false)),
            dirty_bitmap,
            snapshot_count: Arc::new(AtomicU64::new(0)),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut vcpu =
            VirtualProcessor::new(&self.partition, 0).map_err(|e| anyhow::anyhow!("{}", e))?;
        let _ = vcpu.set_registers(&[
            (WHvX64RegisterRip, 0x100000),
            (WHvX64RegisterRsp, 0x80000),
            (WHvX64RegisterRflags, 0x2),
        ]);

        let raw = RawMem {
            ptr: self.memory,
            size: self.memory_size,
        };
        self.running.store(true, Ordering::SeqCst);
        self.vcpu = Some(vcpu);

        while self.running.load(Ordering::Relaxed) {
            let exit = match self.vcpu.as_mut().unwrap().run() {
                Ok(e) => e,
                Err(e) => {
                    tracing::warn!("run(): {}", e);
                    break;
                }
            };
            match WHvRunVpExitReason::from_u32(exit.ExitReason) {
                WHvRunVpExitReason::MemoryAccess => {
                    let gpa = exit.MemoryAccess.Gpa;
                    for dev in &self.virtio_devices {
                        if dev.contains(gpa) {
                            let _ = dev.process_queue(&raw);
                            break;
                        }
                    }
                }
                WHvRunVpExitReason::X64Halt => {
                    tracing::info!("Guest halted");
                    self.running.store(false, Ordering::SeqCst);
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn get_dirty_pages(&self) -> Vec<u64> {
        let mut p = Vec::new();
        if let Ok(bm) = self.dirty_bitmap.lock() {
            for (i, b) in bm.iter().enumerate() {
                for bit in 0..8 {
                    if b & (1 << bit) != 0 {
                        p.push((i * 8 + bit) as u64);
                    }
                }
            }
        }
        p
    }

    pub fn clear_dirty_bitmap(&self) {
        if let Ok(mut bm) = self.dirty_bitmap.lock() {
            for b in bm.iter_mut() {
                *b = 0;
            }
        }
    }

    pub fn get_memory(&self) -> (*mut u8, usize) {
        (self.memory, self.memory_size)
    }
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
    pub fn config(&self) -> &AeroConfig {
        &self.config
    }
    pub fn stats(&self) -> VmStats {
        VmStats {
            snapshots: self.snapshot_count.load(Ordering::Relaxed),
            dirty_pages: self.get_dirty_pages().len() as u64,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct VmStats {
    pub snapshots: u64,
    pub dirty_pages: u64,
}
