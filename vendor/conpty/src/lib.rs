#![allow(dead_code, non_snake_case)]
//! ConPTY bindings для AeroOS.
//! Полная реализация: CreatePipe + CreatePseudoConsole + Read/WriteFile.

extern crate std;

use std::io;
use std::ptr;

use winapi::shared::minwindef::DWORD;
use winapi::um::consoleapi::{ClosePseudoConsole, CreatePseudoConsole, ResizePseudoConsole};
use winapi::um::fileapi::{ReadFile, WriteFile};
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::namedpipeapi::CreatePipe;
use winapi::um::wincon::COORD;
use winapi::um::winnt::HANDLE;

/// HPCON = HANDLE (псевдоконсоль).
pub type HPCON = HANDLE;

pub struct PseudoConsole {
    pub hpc: HPCON,
    pub input_write:  HANDLE,
    pub output_read:  HANDLE,
    input_read:  HANDLE,
    output_write: HANDLE,
    cols: i16,
    rows: i16,
}

unsafe impl Send for PseudoConsole {}
unsafe impl Sync for PseudoConsole {}

impl PseudoConsole {
    pub fn new(cols: i16, rows: i16) -> Result<Self, String> {
        unsafe {
            let mut input_read:   HANDLE = ptr::null_mut();
            let mut input_write:  HANDLE = ptr::null_mut();
            let mut output_read:  HANDLE = ptr::null_mut();
            let mut output_write: HANDLE = ptr::null_mut();

            if CreatePipe(&mut input_read, &mut input_write, ptr::null_mut(), 0) == 0 {
                return Err(format!("CreatePipe(input) failed: {}", io::Error::last_os_error()));
            }
            if CreatePipe(&mut output_read, &mut output_write, ptr::null_mut(), 0) == 0 {
                CloseHandle(input_read); CloseHandle(input_write);
                return Err(format!("CreatePipe(output) failed: {}", io::Error::last_os_error()));
            }

            let size = COORD { X: cols, Y: rows };
            let mut hpc: HPCON = ptr::null_mut();
            let hr = CreatePseudoConsole(size, input_read, output_write, 0, &mut hpc);
            if hr < 0 {
                CloseHandle(input_read);  CloseHandle(input_write);
                CloseHandle(output_read); CloseHandle(output_write);
                return Err(format!("CreatePseudoConsole failed: HRESULT=0x{:X}", hr));
            }

            Ok(Self {
                hpc,
                input_write,
                output_read,
                input_read,
                output_write,
                cols,
                rows,
            })
        }
    }

    pub fn resize(&self, cols: i16, rows: i16) -> Result<(), String> {
        let size = COORD { X: cols, Y: rows };
        let hr = unsafe { ResizePseudoConsole(self.hpc, size) };
        if hr < 0 {
            Err(format!("ResizePseudoConsole failed: 0x{:X}", hr))
        } else {
            Ok(())
        }
    }

    pub fn write_input(&self, data: &[u8]) -> Result<(), String> {
        let mut written: DWORD = 0;
        let ok = unsafe {
            WriteFile(
                self.input_write,
                data.as_ptr() as *const _,
                data.len() as DWORD,
                &mut written,
                ptr::null_mut(),
            )
        };
        if ok == 0 {
            Err(format!("WriteFile(input) failed: {}", io::Error::last_os_error()))
        } else {
            Ok(())
        }
    }

    pub fn read_output(&self, buf_size: usize) -> Result<Vec<u8>, String> {
        let mut buf = vec![0u8; buf_size];
        let mut read: DWORD = 0;
        let ok = unsafe {
            ReadFile(
                self.output_read,
                buf.as_mut_ptr() as *mut _,
                buf_size as DWORD,
                &mut read,
                ptr::null_mut(),
            )
        };
        if ok == 0 {
            Err(format!("ReadFile(output) failed: {}", io::Error::last_os_error()))
        } else {
            buf.truncate(read as usize);
            Ok(buf)
        }
    }

    pub fn hpc(&self) -> HPCON { self.hpc }
}

impl Drop for PseudoConsole {
    fn drop(&mut self) {
        unsafe {
            if !self.hpc.is_null() {
                ClosePseudoConsole(self.hpc);
            }
            for h in [self.input_read, self.input_write, self.output_read, self.output_write] {
                if h != INVALID_HANDLE_VALUE && !h.is_null() {
                    CloseHandle(h);
                }
            }
        }
    }
}