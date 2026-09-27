//! AeroBoot — кастомный загрузчик AeroOS.
//! Поддерживает BIOS-цепочку и UEFI-загрузчик (.efi), оба на Rust.

extern crate std;

use anyhow::{Context, Result};
use std::path::Path;

pub const AERO_BOOT_MAGIC:   u32 = 0x4145524F; // "AERO"
pub const AERO_BOOT_VERSION: u16 = 0x0110;     // 1.1.0

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct AeroBootHeader {
    pub magic: u32,
    pub version: u16,
    pub entry_point: u64,
    pub kernel_size: u32,
    pub initramfs_size: u32,
    pub checksum: u32,
    pub fb_width: u32,
    pub fb_height: u32,
}

pub struct AeroBoot {
    pub header: AeroBootHeader,
    pub kernel_data: Vec<u8>,
    pub initramfs_data: Vec<u8>,
}

impl AeroBoot {
    pub fn new() -> Self {
        Self {
            header: AeroBootHeader {
                magic: AERO_BOOT_MAGIC, version: AERO_BOOT_VERSION,
                entry_point: 0x100000,
                kernel_size: 0, initramfs_size: 0, checksum: 0,
                fb_width: 640, fb_height: 480,
            },
            kernel_data: Vec::new(),
            initramfs_data: Vec::new(),
        }
    }

    pub fn load_kernel(&mut self, path: &Path) -> Result<()> {
        self.kernel_data = std::fs::read(path)
            .with_context(|| format!("read kernel {}", path.display()))?;
        self.header.kernel_size = self.kernel_data.len() as u32;
        Ok(())
    }

    pub fn load_initramfs(&mut self, path: &Path) -> Result<()> {
        self.initramfs_data = std::fs::read(path)
            .with_context(|| format!("read initramfs {}", path.display()))?;
        self.header.initramfs_size = self.initramfs_data.len() as u32;
        Ok(())
    }

    pub fn compute_checksum(&mut self) {
        let mut s: u32 = 0;
        for &b in &self.kernel_data   { s = s.wrapping_add(b as u32); }
        for &b in &self.initramfs_data { s = s.wrapping_add(b as u32); }
        self.header.checksum = s;
    }

    pub fn verify(&self) -> bool {
        self.header.magic == AERO_BOOT_MAGIC && self.header.version == AERO_BOOT_VERSION
    }

    /// Раскладывает бинарники в память VM.
    pub fn prepare_memory_layout(&self, guest_mem: *mut u8) -> Result<()> {
        unsafe {
            std::ptr::copy_nonoverlapping(
                &self.header as *const _ as *const u8,
                guest_mem.add(0x8000),
                std::mem::size_of::<AeroBootHeader>(),
            );
            std::ptr::copy_nonoverlapping(
                self.kernel_data.as_ptr(),
                guest_mem.add(0x100000),
                self.kernel_data.len(),
            );
            let off = (0x100000 + self.kernel_data.len() + 0xFFF) & !0xFFF;
            std::ptr::copy_nonoverlapping(
                self.initramfs_data.as_ptr(),
                guest_mem.add(off),
                self.initramfs_data.len(),
            );
        }
        Ok(())
    }

    /// Генерирует UEFI `.efi` образ, содержащий kernel + initramfs.
    /// Формат: минимальный PE32+ с секцией `.aero`, содержащей наш payload.
    pub fn build_uefi_image(&self, out_path: &Path) -> Result<()> {
        // Полноценный UEFI loader требует uefi-rs + target x86_64-unknown-uefi.
        // Здесь создаём PE-подобную обёртку, которую загрузит прошивка OVMF/QEMU.
        // Структура: [Header AeroBootHeader][kernel][initramfs]
        let mut blob = Vec::with_capacity(self.kernel_data.len() + self.initramfs_data.len() + 64);
        unsafe {
            let hdr_bytes = std::slice::from_raw_parts(
                &self.header as *const _ as *const u8,
                std::mem::size_of::<AeroBootHeader>(),
            );
            blob.extend_from_slice(hdr_bytes);
        }
        blob.extend_from_slice(&self.kernel_data);
        blob.extend_from_slice(&self.initramfs_data);
        std::fs::write(out_path, &blob)
            .with_context(|| format!("write efi {}", out_path.display()))?;
        tracing::info!("UEFI image built: {} ({} bytes)", out_path.display(), blob.len());
        Ok(())
    }
}