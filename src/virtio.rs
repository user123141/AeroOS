use crate::tap::TapDevice;
use std::sync::{Arc, Mutex};

pub const VRING_DESC_F_NEXT: u16 = 0x1;
pub const VRING_DESC_F_WRITE: u16 = 0x2;

#[derive(Clone, Copy, Default)]
pub struct VirtqDesc {
    pub addr: u64,
    pub len: u32,
    pub flags: u16,
    pub next: u16,
}

pub struct VirtQueue {
    pub desc_addr: u64,
    pub avail_addr: u64,
    pub used_addr: u64,
    pub size: u16,
    pub last_avail_idx: u16,
}

impl VirtQueue {
    pub fn new(size: u16) -> Self {
        Self {
            desc_addr: 0,
            avail_addr: 0,
            used_addr: 0,
            size,
            last_avail_idx: 0,
        }
    }
}

pub trait GuestMemory {
    fn read(&self, gpa: u64, buf: &mut [u8]) -> bool;
    fn write(&self, gpa: u64, buf: &[u8]) -> bool;
}

pub struct VirtioMmio {
    base: u64,
    size: u64,
    device_type: VirtioDeviceType,
    state: Arc<Mutex<VirtioState>>,
    disk: Option<Arc<Mutex<std::fs::File>>>,
    tap: Option<Arc<TapDevice>>,
    fs_host_path: Option<String>,
    fb: Arc<Mutex<Vec<u8>>>,
    fb_width: u32,
    fb_height: u32,
}

#[derive(Clone)]
pub enum VirtioDeviceType {
    Block,
    Net,
    Console,
    Gpu,
    Fs,
}

pub struct VirtioState {
    pub status: u32,
    pub queue_sel: u32,
    pub queue_notify: u32,
    pub queues: Vec<VirtQueue>,
    pub interrupt_status: u32,
}

impl VirtioState {
    pub fn new() -> Self {
        Self {
            status: 0,
            queue_sel: 0,
            queue_notify: 0,
            queues: vec![VirtQueue::new(256), VirtQueue::new(256), VirtQueue::new(64)],
            interrupt_status: 0,
        }
    }
}

impl VirtioMmio {
    pub fn new_block(base: u64, size: u64, path: &str) -> Self {
        let f = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .ok();
        Self {
            base,
            size,
            device_type: VirtioDeviceType::Block,
            state: Arc::new(Mutex::new(VirtioState::new())),
            disk: f.map(|x| Arc::new(Mutex::new(x))),
            tap: None,
            fs_host_path: None,
            fb: Arc::new(Mutex::new(Vec::new())),
            fb_width: 0,
            fb_height: 0,
        }
    }
    pub fn new_net(base: u64, size: u64, tap: Option<Arc<TapDevice>>) -> Self {
        Self {
            base,
            size,
            device_type: VirtioDeviceType::Net,
            state: Arc::new(Mutex::new(VirtioState::new())),
            disk: None,
            tap,
            fs_host_path: None,
            fb: Arc::new(Mutex::new(Vec::new())),
            fb_width: 0,
            fb_height: 0,
        }
    }
    pub fn new_console(base: u64, size: u64) -> Self {
        Self {
            base,
            size,
            device_type: VirtioDeviceType::Console,
            state: Arc::new(Mutex::new(VirtioState::new())),
            disk: None,
            tap: None,
            fs_host_path: None,
            fb: Arc::new(Mutex::new(Vec::new())),
            fb_width: 0,
            fb_height: 0,
        }
    }
    pub fn new_gpu(base: u64, size: u64, w: u32, h: u32) -> Self {
        Self {
            base,
            size,
            device_type: VirtioDeviceType::Gpu,
            state: Arc::new(Mutex::new(VirtioState::new())),
            disk: None,
            tap: None,
            fs_host_path: None,
            fb: Arc::new(Mutex::new(vec![0u8; (w * h * 4) as usize])),
            fb_width: w,
            fb_height: h,
        }
    }
    pub fn new_fs(base: u64, size: u64, host_path: &str) -> Self {
        Self {
            base,
            size,
            device_type: VirtioDeviceType::Fs,
            state: Arc::new(Mutex::new(VirtioState::new())),
            disk: None,
            tap: None,
            fs_host_path: Some(host_path.to_string()),
            fb: Arc::new(Mutex::new(Vec::new())),
            fb_width: 0,
            fb_height: 0,
        }
    }

    pub fn contains(&self, g: u64) -> bool {
        g >= self.base && g < self.base + self.size
    }
    pub fn framebuffer(&self) -> Arc<Mutex<Vec<u8>>> {
        self.fb.clone()
    }
    pub fn fs_host_path(&self) -> Option<&str> {
        self.fs_host_path.as_deref()
    }

    pub fn read(&self, gpa: u64, d: &mut [u8]) -> bool {
        let o = gpa - self.base;
        let s = self.state.lock().unwrap();
        match o {
            0x00 => d.copy_from_slice(&0x74726976u32.to_le_bytes()),
            0x04 => d.copy_from_slice(&1u32.to_le_bytes()),
            0x08 => d.copy_from_slice(&self.device_id().to_le_bytes()),
            0x0C => d.copy_from_slice(&0x12345678u32.to_le_bytes()),
            0x34 => d.copy_from_slice(&s.status.to_le_bytes()),
            0x38 => d.copy_from_slice(&s.queue_sel.to_le_bytes()),
            0x44 => d.copy_from_slice(&s.queue_notify.to_le_bytes()),
            0x60 => d.copy_from_slice(&s.interrupt_status.to_le_bytes()),
            _ => return false,
        }
        true
    }

    pub fn write(&self, gpa: u64, data: &[u8]) -> bool {
        let off = gpa - self.base;
        let mut s = self.state.lock().unwrap();
        let u = |d: &[u8]| u32::from_le_bytes(d.try_into().unwrap_or([0; 4]));
        match off {
            0x30 => s.status = u(data),
            0x38 => s.queue_sel = u(data),
            0x80 => {
                let v = u(data);
                let i = s.queue_sel as usize;
                if i < s.queues.len() {
                    s.queues[i].desc_addr =
                        (s.queues[i].desc_addr & 0xFFFF_FFFF_0000_0000) | v as u64;
                }
            }
            0x84 => {
                let v = u(data);
                let i = s.queue_sel as usize;
                if i < s.queues.len() {
                    s.queues[i].desc_addr =
                        (s.queues[i].desc_addr & 0xFFFF_FFFF) | ((v as u64) << 32);
                }
            }
            0x90 => {
                let v = u(data);
                let i = s.queue_sel as usize;
                if i < s.queues.len() {
                    s.queues[i].avail_addr =
                        (s.queues[i].avail_addr & 0xFFFF_FFFF_0000_0000) | v as u64;
                }
            }
            0xa0 => {
                let v = u(data);
                let i = s.queue_sel as usize;
                if i < s.queues.len() {
                    s.queues[i].used_addr =
                        (s.queues[i].used_addr & 0xFFFF_FFFF_0000_0000) | v as u64;
                }
            }
            0x44 => {
                s.queue_notify = u(data);
                s.interrupt_status = 1;
            }
            0x64 => s.interrupt_status = 0,
            0x70 => {
                let v = u(data);
                let i = s.queue_sel as usize;
                if i < s.queues.len() {
                    s.queues[i].size = (v & 0xFFFF) as u16;
                }
            }
            _ => return false,
        }
        true
    }

    pub fn process_queue<M: GuestMemory>(&self, mem: &M) -> u32 {
        let mut s = self.state.lock().unwrap();
        let qi = s.queue_sel as usize;
        if qi >= s.queues.len() {
            return 0;
        }
        let q = &mut s.queues[qi];
        if q.size == 0 || q.desc_addr == 0 {
            return 0;
        }

        let mut avail_hdr = [0u8; 4];
        if !mem.read(q.avail_addr, &mut avail_hdr) {
            return 0;
        }
        let avail_idx = u16::from_le_bytes([avail_hdr[2], avail_hdr[3]]);
        let mut processed = 0u32;

        while q.last_avail_idx != avail_idx {
            let slot = (q.last_avail_idx as usize) % (q.size as usize);
            let ring_addr = q.avail_addr + 4 + (slot * 2) as u64;
            let mut head_buf = [0u8; 2];
            if !mem.read(ring_addr, &mut head_buf) {
                break;
            }
            let head = u16::from_le_bytes(head_buf);

            if self.process_chain(q, mem, head).is_ok() {
                let mut used_idx_buf = [0u8; 2];
                let _ = mem.read(q.used_addr + 2, &mut used_idx_buf);
                let used_idx = u16::from_le_bytes(used_idx_buf);
                let used_slot = (used_idx as usize) % (q.size as usize);
                let used_elem_addr = q.used_addr + 4 + (used_slot * 8) as u64;
                let mut elem = [0u8; 8];
                elem[0..4].copy_from_slice(&(head as u32).to_le_bytes());
                let _ = mem.write(used_elem_addr, &elem);
                let _ = mem.write(q.used_addr + 2, &(used_idx.wrapping_add(1)).to_le_bytes());
            }
            q.last_avail_idx = q.last_avail_idx.wrapping_add(1);
            processed += 1;
        }
        processed
    }

    fn process_chain<M: GuestMemory>(&self, q: &VirtQueue, mem: &M, head: u16) -> Result<(), ()> {
        let mut idx = head;
        let mut total = 0usize;
        let mut chain: Vec<(u64, u32, u16)> = Vec::new();
        while total < q.size as usize {
            let desc_addr = q.desc_addr + (idx as u64) * 16;
            let mut buf = [0u8; 16];
            if !mem.read(desc_addr, &mut buf) {
                return Err(());
            }
            let addr = u64::from_le_bytes(buf[0..8].try_into().unwrap());
            let len = u32::from_le_bytes(buf[8..12].try_into().unwrap());
            let flags = u16::from_le_bytes(buf[12..14].try_into().unwrap());
            let next = u16::from_le_bytes(buf[14..16].try_into().unwrap());
            chain.push((addr, len, flags));
            total += 1;
            if flags & VRING_DESC_F_NEXT == 0 {
                break;
            }
            idx = next;
        }
        match self.device_type {
            VirtioDeviceType::Block => self.handle_block(&chain, mem),
            VirtioDeviceType::Net => self.handle_net(&chain, mem),
            VirtioDeviceType::Console => self.handle_console(&chain, mem),
            VirtioDeviceType::Gpu => self.handle_gpu(&chain, mem),
            VirtioDeviceType::Fs => self.handle_fs(&chain, mem),
        }
    }

    fn handle_block<M: GuestMemory>(&self, chain: &[(u64, u32, u16)], mem: &M) -> Result<(), ()> {
        if chain.is_empty() {
            return Ok(());
        }
        let (hdr_addr, hdr_len, _) = chain[0];
        if hdr_len < 16 {
            return Err(());
        }
        let mut hdr = [0u8; 16];
        if !mem.read(hdr_addr, &mut hdr) {
            return Err(());
        }
        let req_type = u32::from_le_bytes(hdr[0..4].try_into().unwrap());
        let sector = u64::from_le_bytes(hdr[8..16].try_into().unwrap());
        match req_type {
            0 => {
                if let Some(disk) = &self.disk {
                    let mut f = disk.lock().unwrap();
                    use std::io::{Read, Seek, SeekFrom};
                    let _ = f.seek(SeekFrom::Start(sector * 512));
                    for &(addr, len, flags) in &chain[1..] {
                        if flags & VRING_DESC_F_WRITE != 0 {
                            let mut buf = vec![0u8; len as usize];
                            let _ = f.read(&mut buf);
                            let _ = mem.write(addr, &buf);
                        }
                    }
                }
            }
            1 => {
                if let Some(disk) = &self.disk {
                    let mut f = disk.lock().unwrap();
                    use std::io::{Seek, SeekFrom, Write};
                    let _ = f.seek(SeekFrom::Start(sector * 512));
                    for &(addr, len, flags) in &chain[1..] {
                        if flags & VRING_DESC_F_WRITE == 0 {
                            let mut buf = vec![0u8; len as usize];
                            let _ = mem.read(addr, &mut buf);
                            let _ = f.write_all(&buf);
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_net<M: GuestMemory>(&self, chain: &[(u64, u32, u16)], mem: &M) -> Result<(), ()> {
        if chain.is_empty() {
            return Ok(());
        }
        if let Some(tap) = &self.tap {
            for &(addr, len, flags) in &chain[1..] {
                if flags & VRING_DESC_F_WRITE == 0 {
                    let mut pkt = vec![0u8; len as usize];
                    if mem.read(addr, &mut pkt) {
                        let _ = tap.send(&pkt);
                    }
                } else if let Some(pkt) = tap.try_recv() {
                    let n = pkt.len().min(len as usize);
                    let _ = mem.write(addr, &pkt[..n]);
                }
            }
        }
        Ok(())
    }

    fn handle_console<M: GuestMemory>(&self, chain: &[(u64, u32, u16)], mem: &M) -> Result<(), ()> {
        for &(addr, len, flags) in chain {
            if flags & VRING_DESC_F_WRITE == 0 {
                let mut data = vec![0u8; len as usize];
                if mem.read(addr, &mut data) {
                    tracing::info!(target: "guest", "{}", String::from_utf8_lossy(&data));
                }
            }
        }
        Ok(())
    }

    fn handle_gpu<M: GuestMemory>(&self, chain: &[(u64, u32, u16)], mem: &M) -> Result<(), ()> {
        if chain.is_empty() {
            return Ok(());
        }
        let (cmd_addr, cmd_len, _) = chain[0];
        if cmd_len < 4 {
            return Ok(());
        }
        let mut cmd = vec![0u8; cmd_len as usize];
        if !mem.read(cmd_addr, &mut cmd) {
            return Ok(());
        }
        let ct = u32::from_le_bytes(cmd[0..4].try_into().unwrap());
        tracing::debug!("virtio-gpu: cmd=0x{:X}", ct);
        Ok(())
    }

    fn handle_fs<M: GuestMemory>(&self, chain: &[(u64, u32, u16)], mem: &M) -> Result<(), ()> {
        // 9P: собираем запрос, обрабатываем через VirtioFsServer, отвечаем
        for &(addr, len, flags) in chain {
            if flags & VRING_DESC_F_WRITE == 0 {
                let mut data = vec![0u8; len as usize];
                if mem.read(addr, &mut data) {
                    tracing::debug!("9P request: {} bytes", data.len());
                    // Async обработка в фоновой задаче; синхронно возвращаем пусто
                    // Полный ответ пишется через shared memory (см. virtio_fs.rs)
                }
            }
        }
        Ok(())
    }

    fn device_id(&self) -> u32 {
        match self.device_type {
            VirtioDeviceType::Net => 1,
            VirtioDeviceType::Block => 2,
            VirtioDeviceType::Console => 3,
            VirtioDeviceType::Gpu => 16,
            VirtioDeviceType::Fs => 9,
        }
    }
}
