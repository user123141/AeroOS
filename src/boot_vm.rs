//! Boot VM — загрузка bzImage (Linux 64-bit boot protocol).
//!
//! Reference: https://www.kernel.org/doc/html/latest/x86/boot.html
//!
//! Ключевые смещения в setup header:
//!   0x1F1: setup_sects (u8)
//!   0x1FE: boot_flag (u16) = 0xAA55
//!   0x202: magic "HdrS" (u32) = 0x53726448
//!   0x206: protocol version (u16)
//!   0x211: loadflags (u8)
//!   0x214: code32_start (u32)
//!   0x218: ramdisk_image (u32)
//!   0x21C: ramdisk_size (u32)
//!   0x228: cmd_line_ptr (u32)

use anyhow::{Context, Result};
use std::path::Path;
use whpx::Register;

pub const KERNEL_START: u64     = 0x100000;   // 1 MB
pub const SETUP_START: u64      = 0x10000;    // 64 KB
pub const INITRAMFS_ADDR: u64   = 0x4000000;  // 64 MB
pub const BOOT_PARAMS_ADDR: u64 = 0x7000;     // zero page
pub const CMDLINE_ADDR: u64     = 0x20000;    // cmdline
pub const STACK_ADDR: u64       = 0x90000;

pub const BOOT_MAGIC: u32 = 0x5372_6448;      // "HdrS"

pub struct BootResult {
    pub entry_point: u64,
    pub initramfs_size: u64,
}

pub fn load_kernel(
    guest_mem: *mut u8,
    guest_mem_size: usize,
    kernel_path: &Path,
    initramfs_path: Option<&Path>,
    cmdline: &str,
) -> Result<BootResult> {
    let kernel_data = std::fs::read(kernel_path)
        .with_context(|| format!("read {}", kernel_path.display()))?;

    if kernel_data.len() < 0x250 {
        anyhow::bail!("Kernel too small: {} bytes", kernel_data.len());
    }

    // 1. Parse setup header (offsets from Linux boot protocol)
    let setup_sects = kernel_data[0x1F1] as u64;

    // magic — это 4 байта u32 little-endian
    let magic = u32::from_le_bytes([
        kernel_data[0x202],
        kernel_data[0x203],
        kernel_data[0x204],
        kernel_data[0x205],
    ]);

    if magic != BOOT_MAGIC {
        anyhow::bail!(
            "Invalid bzImage magic: got 0x{:08X}, expected 0x{:08X}",
            magic, BOOT_MAGIC
        );
    }

    let boot_flag = u16::from_le_bytes([kernel_data[0x1FE], kernel_data[0x1FF]]);
    if boot_flag != 0xAA55 {
        anyhow::bail!("Invalid boot_flag: 0x{:04X}", boot_flag);
    }

    let setup_size = (setup_sects + 1) * 512;
    let kernel_offset = setup_size as usize;

    tracing::info!(
        "bzImage: setup_sects={}, setup_size={}, kernel_offset={}, magic OK",
        setup_sects, setup_size, kernel_offset
    );

    if kernel_offset >= kernel_data.len() {
        anyhow::bail!("Kernel offset {} >= file size {}", kernel_offset, kernel_data.len());
    }

    // 2. Copy setup code to SETUP_START
    let setup_code = &kernel_data[..setup_size as usize];
    unsafe {
        std::ptr::copy_nonoverlapping(
            setup_code.as_ptr(),
            guest_mem.add(SETUP_START as usize),
            setup_code.len(),
        );
    }

    // 3. Copy kernel code to KERNEL_START
    let kernel_code = &kernel_data[kernel_offset..];
    let copy_size = kernel_code.len().min(guest_mem_size - KERNEL_START as usize);
    unsafe {
        std::ptr::copy_nonoverlapping(
            kernel_code.as_ptr(),
            guest_mem.add(KERNEL_START as usize),
            copy_size,
        );
    }
    tracing::info!("Kernel code: {} bytes at {:#x}", copy_size, KERNEL_START);

    // 4. Initramfs
    let mut initramfs_size = 0u64;
    if let Some(path) = initramfs_path {
        let data = std::fs::read(path)?;
        if INITRAMFS_ADDR as usize + data.len() > guest_mem_size {
            anyhow::bail!("Initramfs too large: {} bytes", data.len());
        }
        unsafe {
            std::ptr::copy_nonoverlapping(
                data.as_ptr(),
                guest_mem.add(INITRAMFS_ADDR as usize),
                data.len(),
            );
        }
        initramfs_size = data.len() as u64;
        tracing::info!("Initramfs: {} bytes at {:#x}", initramfs_size, INITRAMFS_ADDR);
    }

    // 5. cmdline
    let mut cmdline_bytes = cmdline.as_bytes().to_vec();
    cmdline_bytes.push(0);
    unsafe {
        std::ptr::copy_nonoverlapping(
            cmdline_bytes.as_ptr(),
            guest_mem.add(CMDLINE_ADDR as usize),
            cmdline_bytes.len(),
        );
    }

    // 6. Setup zero page (boot_params)
    unsafe {
        let zp = guest_mem.add(BOOT_PARAMS_ADDR as usize);
        std::ptr::write_bytes(zp, 0, 4096);

        // Copy setup header portion (from 0x1F1 onward) to zero page
        let header_len = 0x100.min(kernel_data.len() - 0x1F1);
        std::ptr::copy_nonoverlapping(
            kernel_data[0x1F1..].as_ptr(),
            zp.add(0x1F1),
            header_len,
        );

        // boot_flag (0x1FE)
        let bf: u16 = 0xAA55;
        std::ptr::copy_nonoverlapping(bf.to_le_bytes().as_ptr(), zp.add(0x1FE), 2);

        // code32_start (0x214)
        std::ptr::copy_nonoverlapping(
            (KERNEL_START as u32).to_le_bytes().as_ptr(),
            zp.add(0x214),
            4,
        );
        // ramdisk_image (0x218)
        std::ptr::copy_nonoverlapping(
            (INITRAMFS_ADDR as u32).to_le_bytes().as_ptr(),
            zp.add(0x218),
            4,
        );
        // ramdisk_size (0x21C)
        std::ptr::copy_nonoverlapping(
            (initramfs_size as u32).to_le_bytes().as_ptr(),
            zp.add(0x21C),
            4,
        );
        // cmd_line_ptr (0x228)
        std::ptr::copy_nonoverlapping(
            (CMDLINE_ADDR as u32).to_le_bytes().as_ptr(),
            zp.add(0x228),
            4,
        );
    }

    tracing::info!("Zero page written at {:#x}", BOOT_PARAMS_ADDR);

    Ok(BootResult {
        entry_point: KERNEL_START,
        initramfs_size,
    })
}

/// Настройка регистров vCPU для Linux 64-bit boot protocol.
pub fn setup_vcpu_registers(
    vcpu: &mut whpx::VirtualProcessor,
    entry: u64,
) -> Result<()> {
    // Long mode: CR0.PE=1, CR0.PG=1, CR4.PAE=1, EFER.LME=1, EFER.SCE=1
    let cr0: u64 = 0x8001_0033;   // PE | MP | ET | NE | WP | AM | PG
    let cr4: u64 = 0x0000_0020;   // PAE
    let efer: u64 = 0x0000_0D01;  // SCE | LME | LMA | NXE

    let regs: [(Register, u64); 8] = [
        (Register::Rip,    entry),
        (Register::Rsi,    BOOT_PARAMS_ADDR),
        (Register::Rsp,    STACK_ADDR),
        (Register::Rflags, 0x2),
        (Register::Cr0,    cr0),
        (Register::Cr4,    cr4),
        (Register::Efer,   efer),
        (Register::Rax,    0),
    ];

    vcpu.set_registers(&regs).map_err(|e| anyhow::anyhow!("{}", e))?;
    tracing::info!(
        "vCPU: RIP={:#x}, RSI={:#x}, CR0={:#x}, CR4={:#x}, EFER={:#x}",
        entry, BOOT_PARAMS_ADDR, cr0, cr4, efer
    );
    Ok(())
}