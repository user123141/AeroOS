//! AeroBoot — кастомный загрузчик AeroOS.
//! Поддерживает BIOS-цепочку и UEFI-загрузчик (.efi), оба на Rust.
//!
//! ВАЖНО: AeroBootHeader помечен `#[repr(C, packed)]` — это значит,
//! что НЕЛЬЗЯ брать ссылку на его поля (создаёт UB, компилятор ловит
//! как E0793). Все чтения — через копию в локальную переменную.

extern crate std;

use anyhow::{Context, Result};
use std::path::Path;

pub const AERO_BOOT_MAGIC: u32 = 0x4145524F; // "AERO"
pub const AERO_BOOT_VERSION: u16 = 0x0111; // 1.1.1

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
                magic: AERO_BOOT_MAGIC,
                version: AERO_BOOT_VERSION,
                entry_point: 0x100000,
                kernel_size: 0,
                initramfs_size: 0,
                checksum: 0,
                fb_width: 640,
                fb_height: 480,
            },
            kernel_data: Vec::new(),
            initramfs_data: Vec::new(),
        }
    }

    pub fn load_kernel(&mut self, path: &Path) -> Result<()> {
        self.kernel_data =
            std::fs::read(path).with_context(|| format!("read kernel {}", path.display()))?;
        self.header.kernel_size = self.kernel_data.len() as u32;
        Ok(())
    }

    pub fn load_initramfs(&mut self, path: &Path) -> Result<()> {
        self.initramfs_data =
            std::fs::read(path).with_context(|| format!("read initramfs {}", path.display()))?;
        self.header.initramfs_size = self.initramfs_data.len() as u32;
        Ok(())
    }

    pub fn compute_checksum(&mut self) {
        let mut s: u32 = 0;
        for &b in &self.kernel_data {
            s = s.wrapping_add(b as u32);
        }
        for &b in &self.initramfs_data {
            s = s.wrapping_add(b as u32);
        }
        self.header.checksum = s;
    }

    pub fn verify(&self) -> bool {
        // Копируем поля в локальные переменные (packed struct — нельзя брать refs)
        let magic = self.header.magic;
        let version = self.header.version;
        magic == AERO_BOOT_MAGIC && version == AERO_BOOT_VERSION
    }

    /// Раскладывает бинарники в память VM.
    pub fn prepare_memory_layout(&self, guest_mem: *mut u8) -> Result<()> {
        unsafe {
            // `&self.header as *const _` — это указатель, разрешено
            let hdr_ptr = &self.header as *const AeroBootHeader as *const u8;
            std::ptr::copy_nonoverlapping(
                hdr_ptr,
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

    /// Генерирует UEFI `.efi` образ.
    pub fn build_uefi_image(&self, out_path: &Path) -> Result<()> {
        let mut blob = Vec::with_capacity(self.kernel_data.len() + self.initramfs_data.len() + 64);
        unsafe {
            let hdr_ptr = &self.header as *const AeroBootHeader as *const u8;
            let hdr_slice =
                std::slice::from_raw_parts(hdr_ptr, std::mem::size_of::<AeroBootHeader>());
            blob.extend_from_slice(hdr_slice);
        }
        blob.extend_from_slice(&self.kernel_data);
        blob.extend_from_slice(&self.initramfs_data);
        std::fs::write(out_path, &blob)
            .with_context(|| format!("write efi {}", out_path.display()))?;
        tracing::info!(
            "UEFI image built: {} ({} bytes)",
            out_path.display(),
            blob.len()
        );
        Ok(())
    }

    /// Загрузить kernel + initramfs из DataFolder за один вызов.
    pub fn load_from_data_folder(&mut self, kernel_path: &Path, initramfs_path: &Path) -> Result<()> {
        if kernel_path.exists() {
            self.load_kernel(kernel_path)?;
            tracing::info!("AeroBoot: kernel loaded ({} bytes)", self.kernel_data.len());
        }
        if initramfs_path.exists() {
            self.load_initramfs(initramfs_path)?;
            tracing::info!(
                "AeroBoot: initramfs loaded ({} bytes)",
                self.initramfs_data.len()
            );
        }
        self.compute_checksum();

        // Копируем в локальную переменную — иначе E0793
        let checksum = self.header.checksum;

        if self.verify() {
            tracing::info!("AeroBoot: header OK, checksum={:#010X}", checksum);
        } else {
            tracing::warn!("AeroBoot: header verification failed");
        }
        Ok(())
    }
}