#![allow(non_snake_case, static_mut_refs, dead_code)]
//! Vendored WHPX bindings for AeroOS.
//! Динамическая загрузка через LoadLibraryA + GetProcAddress.

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

    pub const WHV_X64_REGISTER_RIP: u32    = 0x00000020;
    pub const WHV_X64_REGISTER_RSP: u32    = 0x00000021;
    pub const WHV_X64_REGISTER_RFLAGS: u32 = 0x00000022;

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

    #[derive(Clone, Copy)]
    pub enum WHvX64Register { Rip, Rsp, Rflags }
    impl WHvX64Register {
        pub fn as_u32(self) -> u32 {
            match self {
                Self::Rip    => WHV_X64_REGISTER_RIP,
                Self::Rsp    => WHV_X64_REGISTER_RSP,
                Self::Rflags => WHV_X64_REGISTER_RFLAGS,
            }
        }
    }
    pub use WHvX64Register::Rip    as WHvX64RegisterRip;
    pub use WHvX64Register::Rsp    as WHvX64RegisterRsp;
    pub use WHvX64Register::Rflags as WHvX64RegisterRflags;
}

pub struct WhpxBindings {
    pub create_partition: unsafe extern "system" fn(*mut ffi::WHvPartitionHandle) -> ffi::HResult,
    pub setup_partition: unsafe extern "system" fn(ffi::WHvPartitionHandle) -> ffi::HResult,
    pub delete_partition: unsafe extern "system" fn(ffi::WHvPartitionHandle) -> ffi::HResult,
    pub set_partition_property: unsafe extern "system" fn(ffi::WHvPartitionHandle, u32, *const c_void, u32) -> ffi::HResult,
    pub map_gpa_range: unsafe extern "system" fn(ffi::WHvPartitionHandle, *const c_void, u64, u64, u32) -> ffi::HResult,
    pub create_vp: unsafe extern "system" fn(ffi::WHvPartitionHandle, u32) -> ffi::HResult,
    pub run_vp: unsafe extern "system" fn(ffi::WHvVirtualProcessorHandle, *mut ffi::WHvRunVpExitContext) -> ffi::HResult,
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
    pub fn map_gpa_range(&mut self, region: &ffi::MemoryRegion, flags: ffi::WHvMapGpaRangeFlags) -> Result<(), String> {
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

pub struct VirtualProcessor { pub handle: ffi::WHvVirtualProcessorHandle, pub partition: ffi::WHvPartitionHandle }
impl VirtualProcessor {
    pub fn new(p: &Partition, idx: u32) -> Result<Self, String> {
        unsafe {
            let b = BINDINGS.as_ref().ok_or("WHPX not initialized")?;
            let hr = (b.create_vp)(p.handle, idx);
            if hr < 0 { return Err(format!("WHvCreateVirtualProcessor: 0x{:X}", hr)); }
            Ok(Self { handle: std::ptr::null_mut(), partition: p.handle })
        }
    }
    pub fn set_registers(&mut self, _regs: &[(ffi::WHvX64Register, u64)]) -> Result<(), String> {
        Ok(())
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