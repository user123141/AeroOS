#![allow(dead_code)]

use crate::config::AeroConfig;
use crate::data_folder::DataFolder;
use crate::virtio::{GuestMemory, VirtioMmio};
use anyhow::{Context, Result};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use whpx::ffi::*;
use whpx::{HypervisorCapabilities, Partition, VirtualProcessor};
use whpx::MapFlags;

/// Serial output buffer (COM1). Ядро пишет сюда, UI читает.
pub type SerialBuffer = Arc<std::sync::Mutex<Vec<u8>>>;

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
    serial: SerialBuffer,
}
unsafe impl Send for VirtualMachine {}
unsafe impl Sync for VirtualMachine {}

struct RawMem { ptr: *mut u8, size: usize }
unsafe impl Send for RawMem {}
unsafe impl Sync for RawMem {}
impl GuestMemory for RawMem {
    fn read(&self, gpa: u64, buf: &mut [u8]) -> bool {
        let off = gpa as usize;
        if off + buf.len() > self.size { return false; }
        unsafe { std::ptr::copy_nonoverlapping(self.ptr.add(off), buf.as_mut_ptr(), buf.len()); }
        true
    }
    fn write(&self, gpa: u64, buf: &[u8]) -> bool {
        let off = gpa as usize;
        if off + buf.len() > self.size { return false; }
        unsafe { std::ptr::copy_nonoverlapping(buf.as_ptr(), self.ptr.add(off), buf.len()); }
        true
    }
}

impl VirtualMachine {
    pub fn new(config: &AeroConfig, df: &DataFolder) -> Result<Self> {
        whpx::init_bindings().map_err(|e| anyhow::anyhow!("WHPX init: {}", e))?;
        let caps = HypervisorCapabilities::get().map_err(|e| anyhow::anyhow!("{}", e))?;
        tracing::info!("WHPX: dirty_pages={}", caps.dirty_page_tracking);

        let mut partition = Partition::create().map_err(|e| anyhow::anyhow!("{}", e))?;
        partition.set_vcpu_count(config.cpu.cores).map_err(|e| anyhow::anyhow!("{}", e))?;
        partition.set_property(WHvPartitionPropertyCode::ProcessorCount, config.cpu.cores)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let ram_size = (config.memory.ram_mb * 1024 * 1024) as usize;
        let dirty_bitmap = if caps.dirty_page_tracking {
            let _ = partition.set_property(WHvPartitionPropertyCode::DirtyPageTracking, 1);
            Arc::new(std::sync::Mutex::new(vec![0u8; (ram_size / 4096 + 7) / 8]))
        } else {
            Arc::new(std::sync::Mutex::new(Vec::new()))
        };

        let guest_mem = whpx::allocate_guest_memory(ram_size).map_err(|e| anyhow::anyhow!("{}", e))?;
        let region = MemoryRegion { start: 0, size: ram_size as u64, host_ptr: guest_mem as *mut _ };
        partition.map_gpa_range(&region, MapFlags::ReadWriteExecute)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let kernel = df.kernel_path();
        let initramfs = df.initramfs_path();
        let boot = crate::boot_vm::load_kernel(
            guest_mem,
            ram_size,
            &kernel,
            if initramfs.exists() { Some(&initramfs) } else { None },
            "console=ttyS0,115200 earlyprintk=serial,ttyS0,115200 quiet",
        )?;
        tracing::info!("Kernel loaded: entry={:#x}", boot.entry_point);

        let virtio_devices = vec![
            VirtioMmio::new_block(0x10000, 0x1000, "disk.img"),
            VirtioMmio::new_net(0x11000, 0x1000, None),
            VirtioMmio::new_console(0x12000, 0x1000),
            VirtioMmio::new_gpu(0x13000, 0x1000, 640, 480),
        ];

        Ok(Self {
            config: config.clone(), partition, vcpu: None,
            memory: guest_mem, memory_size: ram_size, virtio_devices,
            running: Arc::new(AtomicBool::new(false)),
            dirty_bitmap, snapshot_count: Arc::new(AtomicU64::new(0)),
            serial: Arc::new(std::sync::Mutex::new(Vec::with_capacity(64 * 1024))),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut vcpu = VirtualProcessor::new(&self.partition, 0)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        crate::boot_vm::setup_vcpu_registers(&mut vcpu, crate::boot_vm::KERNEL_START)?;

        let raw = RawMem { ptr: self.memory, size: self.memory_size };
        self.running.store(true, Ordering::SeqCst);
        self.vcpu = Some(vcpu);

        while self.running.load(Ordering::Relaxed) {
            let exit = match self.vcpu.as_mut().unwrap().run() {
                Ok(e) => e,
                Err(e) => { tracing::warn!("run(): {}", e); break; }
            };
            match WHvRunVpExitReason::from_u32(exit.ExitReason) {
                WHvRunVpExitReason::MemoryAccess => {
                    let gpa = exit.MemoryAccess.Gpa;
                    for dev in &self.virtio_devices {
                        if dev.contains(gpa) { let _ = dev.process_queue(&raw); break; }
                    }
                }
                WHvRunVpExitReason::X64IoPortAccess => {
                    // Serial port (COM1 = 0x3F8) — читаем вывод ядра
                    let port = exit.IoPortAccess.Port;
                    if port == 0x3F8 {
                        // Ядро пишет в COM1 — сохраняем в serial буфер
                        // (в реальности нужно читать значение из регистра)
                        if let Ok(mut buf) = self.serial.lock() {
                            // placeholder: mark IO activity
                            if buf.len() < 60_000 {
                                buf.push(b'.');
                            }
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

    pub fn stop(&self) { self.running.store(false, Ordering::SeqCst); }

    /// Копия всей гостевой памяти (для снапшота).
    pub fn get_state(&self) -> Result<Vec<u8>> {
        let mut out = vec![0u8; self.memory_size];
        unsafe {
            std::ptr::copy_nonoverlapping(self.memory, out.as_mut_ptr(), self.memory_size);
        }
        Ok(out)
    }

    /// Восстановить гостевую память из снапшота.
    pub fn restore_state(&mut self, data: &[u8]) -> Result<()> {
        let n = data.len().min(self.memory_size);
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), self.memory, n);
        }
        tracing::info!("Restored {} bytes to guest memory", n);
        Ok(())
    }

    pub fn get_dirty_pages(&self) -> Vec<u64> {
        let mut p = Vec::new();
        if let Ok(bm) = self.dirty_bitmap.lock() {
            for (i, b) in bm.iter().enumerate() {
                for bit in 0..8 { if b & (1 << bit) != 0 { p.push((i * 8 + bit) as u64); } }
            }
        }
        p
    }

    pub fn clear_dirty_bitmap(&self) {
        if let Ok(mut bm) = self.dirty_bitmap.lock() { for b in bm.iter_mut() { *b = 0; } }
    }

    pub fn serial(&self) -> SerialBuffer { self.serial.clone() }

    pub fn read_serial(&self) -> Vec<u8> {
        self.serial.lock().map(|b| b.clone()).unwrap_or_default()
    }

    pub fn clear_serial(&self) {
        if let Ok(mut b) = self.serial.lock() { b.clear(); }
    }

    pub fn get_memory(&self) -> (*mut u8, usize) { (self.memory, self.memory_size) }
    pub fn is_running(&self) -> bool { self.running.load(Ordering::Relaxed) }
    pub fn config(&self) -> &AeroConfig { &self.config }
    pub fn stats(&self) -> VmStats {
        VmStats {
            snapshots: self.snapshot_count.load(Ordering::Relaxed),
            dirty_pages: self.get_dirty_pages().len() as u64,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct VmStats { pub snapshots: u64, pub dirty_pages: u64 }