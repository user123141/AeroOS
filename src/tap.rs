extern crate std;
use std::ffi::c_void;
use std::ptr;
use anyhow::Result;

pub struct TapDevice { session: *mut c_void, recv: unsafe extern "system" fn(*mut c_void, *mut u32) -> *mut u8, release: unsafe extern "system" fn(*mut c_void, *const u8), alloc: unsafe extern "system" fn(*mut c_void, u32) -> *mut u8, send: unsafe extern "system" fn(*mut c_void, *const u8) }
unsafe impl Send for TapDevice {}
unsafe impl Sync for TapDevice {}

impl TapDevice {
    pub fn open(name: &str) -> Result<Self> {
        #[cfg(target_os = "windows")]
        unsafe {
            use std::os::windows::ffi::OsStrExt;
            use std::ffi::OsStr;
            let dll = winapi::um::libloaderapi::LoadLibraryA(b"wintun.dll\0".as_ptr() as *const i8);
            if dll.is_null() { anyhow::bail!("wintun.dll not found: https://www.wintun.net/"); }
            let get = |n: &[u8]| winapi::um::libloaderapi::GetProcAddress(dll, n.as_ptr() as *const i8) as *const c_void;
            let create: unsafe extern "system" fn(*const u16, *const u16, *const c_void) -> *mut c_void = std::mem::transmute(get(b"WintunCreateAdapter\0"));
            let start: unsafe extern "system" fn(*mut c_void, u32) -> *mut c_void = std::mem::transmute(get(b"WintunStartSession\0"));
            let recv: unsafe extern "system" fn(*mut c_void, *mut u32) -> *mut u8 = std::mem::transmute(get(b"WintunReceivePacket\0"));
            let release: unsafe extern "system" fn(*mut c_void, *const u8) = std::mem::transmute(get(b"WintunReleaseReceivePacket\0"));
            let alloc: unsafe extern "system" fn(*mut c_void, u32) -> *mut u8 = std::mem::transmute(get(b"WintunAllocateSendPacket\0"));
            let send: unsafe extern "system" fn(*mut c_void, *const u8) = std::mem::transmute(get(b"WintunSendPacket\0"));
            let name_w: Vec<u16> = OsStr::new(name).encode_wide().chain(Some(0)).collect();
            let tun_w: Vec<u16> = OsStr::new("AeroOS").encode_wide().chain(Some(0)).collect();
            let adapter = create(name_w.as_ptr(), tun_w.as_ptr(), ptr::null());
            if adapter.is_null() { anyhow::bail!("WintunCreateAdapter failed"); }
            let session = start(adapter, 0x400000);
            if session.is_null() { anyhow::bail!("WintunStartSession failed"); }
            return Ok(Self { session, recv, release, alloc, send });
        }
        #[cfg(not(target_os = "windows"))]
        anyhow::bail!("TAP only on Windows")
    }
    pub fn try_recv(&self) -> Option<Vec<u8>> {
        unsafe {
            let mut sz: u32 = 0;
            let p = (self.recv)(self.session, &mut sz);
            if p.is_null() { return None; }
            let d = std::slice::from_raw_parts(p, sz as usize).to_vec();
            (self.release)(self.session, p);
            Some(d)
        }
    }
    pub fn send(&self, data: &[u8]) -> Result<()> {
        unsafe {
            let b = (self.alloc)(self.session, data.len() as u32);
            if b.is_null() { anyhow::bail!("alloc failed"); }
            std::ptr::copy_nonoverlapping(data.as_ptr(), b, data.len());
            (self.send)(self.session, b);
            Ok(())
        }
    }
}