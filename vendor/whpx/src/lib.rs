#![allow(non_snake_case, static_mut_refs, dead_code)]

extern crate std;
use std::ffi::c_void;
use std::mem::transmute;
use std::ptr;
use winapi::um::libloaderapi::{GetProcAddress, LoadLibraryA};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::memoryapi::VirtualAlloc;

pub mod ffi {
    use std::ffi::c_void;
    pub type HResult = i32;
    pub type WHvPartitionHandle = *mut c_void;
    pub type WHvVirtualProcessorHandle = *mut c_void;

    pub const WHV_MAP_GPA_RANGE_READ: u32    = 0x1;
    pub const WHV_MAP_GPA_RANGE_WRITE: u32   = 0x2;
    pub const WHV_MAP_GPA_RANGE_EXECUTE: u32 = 0x4;

    pub const WHV_PARTITION_PROPERTY_PROCESSOR_COUNT: u32 = 0x00000001;
    pub const WHV_PARTITION_PROPERTY_DIRTY_PAGE_TRACKING: u32 = 0x0000000F;

    pub const WHV_RUN_VP_EXIT_REASON_MEMORY_ACCESS: u32      = 0x00000000;
    pub const WHV_RUN_VP_EXIT_REASON_X64_IO_PORT_ACCESS: u32 = 0x00000001;
    pub const WHV_RUN_VP_EXIT_REASON_X64_HALT: u32           = 0x00000006;

    // x64 registers
    pub const WHV_X64_REGISTER_RAX: u32    = 0x00000000;
    pub const WHV_X64_REGISTER_RBX: u32    = 0x00000003;
    pub const WHV_X64_REGISTER_RCX: u32    = 0x00000001;
    pub const WHV_X64_REGISTER_RDX: u32    = 0x00000002;
    pub const WHV_X64_REGISTER_RSI: u32    = 0x00000004;
    pub const WHV_X64_REGISTER_RDI: u32    = 0x00000005;
    pub const WHV_X64_REGISTER_RSP: u32    = 0x00000006;
    pub const WHV_X64_REGISTER_RBP: u32    = 0x00000007;
    pub const WHV_X64_REGISTER_R8:  u32    = 0x00000008;
    pub const WHV_X64_REGISTER_R9:  u32    = 0x00000009;
    pub const WHV_X64_REGISTER_R10: u32    = 0x0000000A;
    pub const WHV_X64_REGISTER_R11: u32    = 0x0000000B;
    pub const WHV_X64_REGISTER_R12: u32    = 0x0000000C;
    pub const WHV_X64_REGISTER_R13: u32    = 0x0000000D;
    pub const WHV_X64_REGISTER_R14: u32    = 0x0000000E;
    pub const WHV_X64_REGISTER_R15: u32    = 0x0000000F;
    pub const WHV_X64_REGISTER_RIP: u32    = 0x00000010;
    pub const WHV_X64_REGISTER_RFLAGS: u32 = 0x00000011;
    pub const WHV_X64_REGISTER_CR0: u32    = 0x0000001C;
    pub const WHV_X64_REGISTER_CR3: u32    = 0x0000001E;
    pub const WHV_X64_REGISTER_CR4: u32    = 0x00000020;
    pub const WHV_X64_REGISTER_EFER: u32   = 0x00000036;

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct MemoryRegion {
        pub start: u64,
        pub size: u64,
        pub host_ptr: *mut c_void,
    }
    unsafe impl Send for MemoryRegion {}
    unsafe impl Sync for MemoryRegion {}

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct WHvMemoryAccessExitContext {
        pub Gpa: u64,
        pub Gva: u64,
        pub Flags: u32,
        pub _pad: u32,
    }
    impl Default for WHvMemoryAccessExitContext {
        fn default() -> Self { Self { Gpa: 0, Gva: 0, Flags: 0, _pad: 0 } }
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct WHvIoPortAccessExitContext {
        pub Port: u16,
        pub AccessSize: u8,
        pub _pad: [u8; 5],
    }
    impl Default for WHvIoPortAccessExitContext {
        fn default() -> Self { Self { Port: 0, AccessSize: 0, _pad: [0; 5] } }
    }

    #[repr(C)]
    pub struct WHvRunVpExitContext {
        pub ExitReason: u32,
        pub _padding: [u8; 4],
        pub MemoryAccess: WHvMemoryAccessExitContext,
        pub IoPortAccess: WHvIoPortAccessExitContext,
        pub _pad: [u8; 256],
    }
    impl Default for WHvRunVpExitContext {
        fn default() -> Self {
            Self {
                ExitReason: 0,
                _padding: [0; 4],
                MemoryAccess: Default::default(),
                IoPortAccess: Default::default(),
                _pad: [0u8; 256],
            }
        }
    }

    /// 16-byte register value (union in the C API).
    #[repr(C, align(16))]
    #[derive(Copy, Clone)]
    pub struct WHvRegisterValue {
        pub low: u64,
        pub high: u64,
    }
    impl WHvRegisterValue {
        pub fn from_u64(v: u64) -> Self { Self { low: v, high: 0 } }
        pub fn zero() -> Self { Self { low: 0, high: 0 } }
    }

    #[derive(Clone, Copy)]
    pub enum WHvPartitionPropertyCode {
        ProcessorCount,
        DirtyPageTracking,
    }
    impl WHvPartitionPropertyCode {
        pub fn as_u32(self) -> u32 {
            match self {
                Self::ProcessorCount    => WHV_PARTITION_PROPERTY_PROCESSOR_COUNT,
                Self::DirtyPageTracking => WHV_PARTITION_PROPERTY_DIRTY_PAGE_TRACKING,
            }
        }
    }

    #[derive(Clone, Copy)]
    pub enum WHvMapGpaRangeFlags { ReadWriteExecute }
    impl WHvMapGpaRangeFlags {
        pub fn as_u32(self) -> u32 {
            WHV_MAP_GPA_RANGE_READ | WHV_MAP_GPA_RANGE_WRITE | WHV_MAP_GPA_RANGE_EXECUTE
        }
    }

    #[derive(Clone, Copy)]
    pub enum WHvRunVpExitReason { MemoryAccess, X64IoPortAccess, X64Halt, Unknown }
    impl WHvRunVpExitReason {
        pub fn from_u32(v: u32) -> Self {
            match v {
                WHV_RUN_VP_EXIT_REASON_MEMORY_ACCESS      => Self::MemoryAccess,
                WHV_RUN_VP_EXIT_REASON_X64_IO_PORT_ACCESS => Self::X64IoPortAccess,
                WHV_RUN_VP_EXIT_REASON_X64_HALT           => Self::X64Halt,
                _ => Self::Unknown,
            }
        }
    }
}

/// Регистры x64 (name + value).
#[derive(Clone, Copy)]
pub enum Register {
    Rip, Rsp, Rflags, Rsi, Rax, Rcx, Rdx, Rbx,
    Rdi, Cr0, Cr3, Cr4, Efer,
}
impl Register {
    pub fn name(self) -> u32 {
        use ffi::*;
        match self {
            Self::Rax    => WHV_X64_REGISTER_RAX,
            Self::Rbx    => WHV_X64_REGISTER_RBX,
            Self::Rcx    => WHV_X64_REGISTER_RCX,
            Self::Rdx    => WHV_X64_REGISTER_RDX,
            Self::Rsi    => WHV_X64_REGISTER_RSI,
            Self::Rdi    => WHV_X64_REGISTER_RDI,
            Self::Rsp    => WHV_X64_REGISTER_RSP,
            Self::Rip    => WHV_X64_REGISTER_RIP,
            Self::Rflags => WHV_X64_REGISTER_RFLAGS,
            Self::Cr0    => WHV_X64_REGISTER_CR0,
            Self::Cr3    => WHV_X64_REGISTER_CR3,
            Self::Cr4    => WHV_X64_REGISTER_CR4,
            Self::Efer   => WHV_X64_REGISTER_EFER,
        }
    }
}

pub struct WhpxBindings {
    pub create_partition:       unsafe extern "system" fn(*mut ffi::WHvPartitionHandle) -> ffi::HResult,
    pub setup_partition:        unsafe extern "system" fn(ffi::WHvPartitionHandle) -> ffi::HResult,
    pub delete_partition:       unsafe extern "system" fn(ffi::WHvPartitionHandle) -> ffi::HResult,
    pub set_partition_property: unsafe extern "system" fn(ffi::WHvPartitionHandle, u32, *const c_void, u32) -> ffi::HResult,
    pub map_gpa_range:          unsafe extern "system" fn(ffi::WHvPartitionHandle, *const c_void, u64, u64, u32) -> ffi::HResult,
    pub create_vp:              unsafe extern "system" fn(ffi::WHvPartitionHandle, u32) -> ffi::HResult,
    pub run_vp:                 unsafe extern "system" fn(ffi::WHvVirtualProcessorHandle, *mut ffi::WHvRunVpExitContext) -> ffi::HResult,
    // Правильная сигнатура: (handle, names_ptr, count, values_ptr)
    pub set_vp_registers: unsafe extern "system" fn(
        ffi::WHvVirtualProcessorHandle,
        *const u32,
        u32,
        *const ffi::WHvRegisterValue,
    ) -> ffi::HResult,
}

static mut BINDINGS: Option<WhpxBindings> = None;
static mut INIT: bool = false;

pub fn init_bindings() -> Result<(), String> {
    unsafe {
        if INIT { return Ok(()); }
        let dll = b"WinHvPlatform.dll\0";
        let handle = LoadLibraryA(dll.as_ptr() as *const i8);
        if handle.is_null() { return Err(format!("LoadLibraryA failed: {}", GetLastError())); }
        let get = |name: &[u8]| -> *const c_void {
            GetProcAddress(handle, name.as_ptr() as *const i8) as *const c_void
        };
        BINDINGS = Some(WhpxBindings {
            create_partition:       transmute(get(b"WHvCreatePartition\0")),
            setup_partition:        transmute(get(b"WHvSetupPartition\0")),
            delete_partition:       transmute(get(b"WHvDeletePartition\0")),
            set_partition_property: transmute(get(b"WHvSetPartitionProperty\0")),
            map_gpa_range:          transmute(get(b"WHvMapGpaRange\0")),
            create_vp:              transmute(get(b"WHvCreateVirtualProcessor\0")),
            run_vp:                 transmute(get(b"WHvRunVirtualProcessor\0")),
            set_vp_registers:       transmute(get(b"WHvSetVirtualProcessorRegisters\0")),
        });
        INIT = true;
        Ok(())
    }
}

pub fn allocate_guest_memory(size: usize) -> Result<*mut u8, String> {
    unsafe {
        const MEM_COMMIT: u32 = 0x1000;
        const MEM_RESERVE: u32 = 0x2000;
        const PAGE_READWRITE: u32 = 0x04;
        let p = VirtualAlloc(ptr::null_mut(), size, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
        if p.is_null() { Err("VirtualAlloc failed".into()) } else { Ok(p as *mut u8) }
    }
}

pub struct Partition { pub handle: ffi::WHvPartitionHandle }
impl Partition {
    pub fn create() -> Result<Self, String> {
        unsafe {
            let mut h: ffi::WHvPartitionHandle = ptr::null_mut();
            let b = BINDINGS.as_ref().ok_or("WHPX not initialized")?;
            let hr = (b.create_partition)(&mut h);
            if hr < 0 { return Err(format!("WHvCreatePartition: 0x{:X}", hr)); }
            let hr = (b.setup_partition)(h);
            if hr < 0 { return Err(format!("WHvSetupPartition: 0x{:X}", hr)); }
            Ok(Self { handle: h })
        }
    }
    pub fn set_vcpu_count(&mut self, n: u32) -> Result<(), String> {
        self.set_property(ffi::WHvPartitionPropertyCode::ProcessorCount, n)
    }
    pub fn set_property(&mut self, prop: ffi::WHvPartitionPropertyCode, value: u32) -> Result<(), String> {
        unsafe {
            let b = BINDINGS.as_ref().ok_or("WHPX not initialized")?;
            let hr = (b.set_partition_property)(
                self.handle, prop.as_u32(),
                &value as *const u32 as *const c_void, 4,
            );
            if hr < 0 { Err(format!("set_property: 0x{:X}", hr)) } else { Ok(()) }
        }
    }
    pub fn map_gpa_range(&mut self, region: &ffi::MemoryRegion, flags: MapFlags) -> Result<(), String> {
        unsafe {
            let b = BINDINGS.as_ref().ok_or("WHPX not initialized")?;
            let hr = (b.map_gpa_range)(
                self.handle,
                region as *const _ as *const c_void,
                region.start, region.size, flags.as_u32(),
            );
            if hr < 0 { Err(format!("map_gpa_range: 0x{:X}", hr)) } else { Ok(()) }
        }
    }
}
unsafe impl Send for Partition {}
unsafe impl Sync for Partition {}

pub use ffi::WHvMapGpaRangeFlags as MapFlags;

pub struct VirtualProcessor {
    pub handle: ffi::WHvVirtualProcessorHandle,
    pub partition: ffi::WHvPartitionHandle,
}
impl VirtualProcessor {
    pub fn new(p: &Partition, idx: u32) -> Result<Self, String> {
        unsafe {
            let b = BINDINGS.as_ref().ok_or("WHPX not initialized")?;
            let hr = (b.create_vp)(p.handle, idx);
            if hr < 0 { return Err(format!("WHvCreateVirtualProcessor: 0x{:X}", hr)); }
            Ok(Self { handle: std::ptr::null_mut(), partition: p.handle })
        }
    }

    /// Устанавливает регистры. Порядок параметров правильный:
    /// (handle, names_ptr, count, values_ptr). Значения — 16-byte WHvRegisterValue.
    pub fn set_registers(&mut self, regs: &[(Register, u64)]) -> Result<(), String> {
        unsafe {
            let b = BINDINGS.as_ref().ok_or("WHPX not initialized")?;
            let names: Vec<u32> = regs.iter().map(|(r, _)| r.name()).collect();
            let values: Vec<ffi::WHvRegisterValue> = regs.iter()
                .map(|(_, v)| ffi::WHvRegisterValue::from_u64(*v))
                .collect();
            let hr = (b.set_vp_registers)(
                self.handle,
                names.as_ptr(),
                names.len() as u32,
                values.as_ptr(),
            );
            if hr < 0 { Err(format!("WHvSetVirtualProcessorRegisters: 0x{:X}", hr)) } else { Ok(()) }
        }
    }

    pub fn run(&mut self) -> Result<ffi::WHvRunVpExitContext, String> {
        unsafe {
            let b = BINDINGS.as_ref().ok_or("WHPX not initialized")?;
            let mut ctx: ffi::WHvRunVpExitContext = Default::default();
            let hr = (b.run_vp)(self.handle, &mut ctx);
            if hr < 0 { return Err(format!("WHvRunVirtualProcessor: 0x{:X}", hr)); }
            Ok(ctx)
        }
    }
}
unsafe impl Send for VirtualProcessor {}
unsafe impl Sync for VirtualProcessor {}

pub struct HypervisorCapabilities { pub dirty_page_tracking: bool }
impl HypervisorCapabilities {
    pub fn get() -> Result<Self, String> { Ok(Self { dirty_page_tracking: true }) }
}