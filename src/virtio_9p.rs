//! VirtIO-9P — shared folders между хостом и гостем.
//!
//! Реализует подмножество 9P2000.L:
//!   - Tversion  → Rversion
//!   - Tattach   → Rattach
//!   - Twalk     → Rwalk
//!   - Topen     → Ropen
//!   - Tread     → Rread
//!   - Twrite    → Rwrite
//!   - Tclunk    → Rclunk
//!   - Tstat     → Rstat
//!
//! Используется для монтирования папок Windows внутрь гостя
//! (например, C:\Projects → /mnt/host в Linux).

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub const P9_TVERSION: u8 = 100;
pub const P9_RVERSION: u8 = 101;
pub const P9_TATTACH:  u8 = 104;
pub const P9_RATTACH:  u8 = 105;
pub const P9_TWALK:    u8 = 110;
pub const P9_RWALK:    u8 = 111;
pub const P9_TOPEN:    u8 = 112;
pub const P9_ROPEN:    u8 = 113;
pub const P9_TREAD:    u8 = 116;
pub const P9_RREAD:    u8 = 117;
pub const P9_TWRITE:   u8 = 118;
pub const P9_RWRITE:   u8 = 119;
pub const P9_TCLUNK:   u8 = 120;
pub const P9_RCLUNK:   u8 = 121;
pub const P9_TSTAT:    u8 = 124;
pub const P9_RSTAT:    u8 = 125;
pub const P9_RLERROR:  u8 = 107;

pub const P9_NOTAG: u16 = 0xFFFF;
pub const P9_NOFID: u32 = 0xFFFFFFFF;

/// Состояние 9P-сервера для одной экспортируемой папки.
pub struct P9Server {
    host_root: PathBuf,
    fids: HashMap<u32, FidEntry>,
    next_fid: u32,
}

struct FidEntry {
    path: PathBuf,
    is_dir: bool,
    open: bool,
    offset: u64,
}

impl P9Server {
    pub fn new(host_root: &Path) -> Result<Self> {
        let canonical = host_root.canonicalize()
            .with_context(|| format!("canonicalize {}", host_root.display()))?;
        if !canonical.is_dir() {
            anyhow::bail!("{} is not a directory", canonical.display());
        }
        tracing::info!("9P server root: {}", canonical.display());
        Ok(Self {
            host_root: canonical,
            fids: HashMap::new(),
            next_fid: 1,
        })
    }

    /// Проверка, что путь не выходит за пределы host_root (защита от ../).
    fn safe_join(&self, rel: &str) -> Result<PathBuf> {
        let joined = self.host_root.join(rel);
        let canon = joined.canonicalize()
            .unwrap_or(joined.clone());
        if !canon.starts_with(&self.host_root) {
            anyhow::bail!("Path escapes host root: {}", rel);
        }
        Ok(canon)
    }

    /// Обработать входящий 9P-пакет.
    pub fn handle(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < 7 {
            anyhow::bail!("9P packet too short");
        }
        let msg_type = data[4];
        let tag = u16::from_le_bytes([data[5], data[6]]);

        match msg_type {
            P9_TVERSION => self.handle_version(data, tag),
            P9_TATTACH  => self.handle_attach(data, tag),
            P9_TWALK    => self.handle_walk(data, tag),
            P9_TOPEN    => self.handle_open(data, tag),
            P9_TREAD    => self.handle_read(data, tag),
            P9_TWRITE   => self.handle_write(data, tag),
            P9_TCLUNK   => self.handle_clunk(data, tag),
            P9_TSTAT    => self.handle_stat(data, tag),
            _ => {
                tracing::debug!("9P: unknown msg_type {}", msg_type);
                Ok(self.error(tag, 524)) // EOPNOTSUPP
            }
        }
    }

    fn reply(&self, msg_type: u8, tag: u16, payload: &[u8]) -> Vec<u8> {
        let size = 7 + payload.len() as u32;
        let mut out = Vec::with_capacity(size as usize);
        out.extend_from_slice(&size.to_le_bytes());
        out.push(msg_type);
        out.extend_from_slice(&tag.to_le_bytes());
        out.extend_from_slice(payload);
        out
    }

    fn error(&self, tag: u16, errcode: u32) -> Vec<u8> {
        let mut p = Vec::new();
        p.extend_from_slice(&errcode.to_le_bytes());
        self.reply(P9_RLERROR, tag, &p)
    }

    fn handle_version(&self, _data: &[u8], tag: u16) -> Result<Vec<u8>> {
        // Rversion: msize(4) + "9P2000.L"(8)
        let mut p = Vec::new();
        p.extend_from_slice(&65536u32.to_le_bytes());
        p.extend_from_slice(b"9P2000.L");
        Ok(self.reply(P9_RVERSION, tag, &p))
    }

    fn handle_attach(&mut self, data: &[u8], tag: u16) -> Result<Vec<u8>> {
        if data.len() < 7 + 4 + 4 + 12 {
            return Ok(self.error(tag, 22));
        }
        let fid = u32::from_le_bytes(data[7..11].try_into().unwrap());
        self.fids.insert(fid, FidEntry {
            path: self.host_root.clone(),
            is_dir: true,
            open: false,
            offset: 0,
        });
        // Rattach: qid (13 bytes)
        let qid = self.make_qid(&self.host_root.clone());
        Ok(self.reply(P9_RATTACH, tag, &qid))
    }

    fn handle_walk(&mut self, data: &[u8], tag: u16) -> Result<Vec<u8>> {
        if data.len() < 7 + 4 + 4 + 2 {
            return Ok(self.error(tag, 22));
        }
        let fid = u32::from_le_bytes(data[7..11].try_into().unwrap());
        let newfid = u32::from_le_bytes(data[11..15].try_into().unwrap());
        let nwname = u16::from_le_bytes(data[15..17].try_into().unwrap()) as usize;

        let entry = match self.fids.get(&fid) {
            Some(e) => e,
            None => return Ok(self.error(tag, 22)),
        };
        let mut current = entry.path.clone();

        let mut pos = 17usize;
        for _ in 0..nwname {
            if pos + 2 > data.len() { return Ok(self.error(tag, 22)); }
            let name_len = u16::from_le_bytes(data[pos..pos+2].try_into().unwrap()) as usize;
            pos += 2;
            if pos + name_len > data.len() { return Ok(self.error(tag, 22)); }
            let name = std::str::from_utf8(&data[pos..pos+name_len]).unwrap_or("");
            pos += name_len;

            if name == ".." { return Ok(self.error(tag, 13)); }
            current = match self.safe_join(&format!("{}/{}", current.strip_prefix(&self.host_root).unwrap_or(Path::new("")).display(), name)) {
                Ok(p) => p,
                Err(_) => return Ok(self.error(tag, 13)),
            };
        }

        let is_dir = current.is_dir();
        self.fids.insert(newfid, FidEntry {
            path: current.clone(),
            is_dir,
            open: false,
            offset: 0,
        });

        // Rwalk: nwqid(2) + nwqid × qid(13)
        let mut p = Vec::new();
        p.extend_from_slice(&(nwname as u16).to_le_bytes());
        for _ in 0..nwname { p.extend_from_slice(&self.make_qid(&current)); }
        Ok(self.reply(P9_RWALK, tag, &p))
    }

    fn handle_open(&mut self, data: &[u8], tag: u16) -> Result<Vec<u8>> {
        if data.len() < 7 + 4 + 1 {
            return Ok(self.error(tag, 22));
        }
        let fid = u32::from_le_bytes(data[7..11].try_into().unwrap());
        if let Some(e) = self.fids.get_mut(&fid) {
            e.open = true;
            let qid = self.make_qid(&e.path.clone());
            let mut p = Vec::new();
            p.extend_from_slice(&qid);
            p.extend_from_slice(&0u32.to_le_bytes()); // iounit
            return Ok(self.reply(P9_ROPEN, tag, &p));
        }
        Ok(self.error(tag, 2))
    }

    fn handle_read(&mut self, data: &[u8], tag: u16) -> Result<Vec<u8>> {
        if data.len() < 7 + 4 + 8 + 4 {
            return Ok(self.error(tag, 22));
        }
        let fid = u32::from_le_bytes(data[7..11].try_into().unwrap());
        let offset = u64::from_le_bytes(data[11..19].try_into().unwrap());
        let count = u32::from_le_bytes(data[19..23].try_into().unwrap());

        let entry = match self.fids.get(&fid) {
            Some(e) => e,
            None => return Ok(self.error(tag, 2)),
        };
        let path = entry.path.clone();
        let is_dir = entry.is_dir;

        let payload = if is_dir {
            // Читаем директорию как список имён (упрощённо)
            match std::fs::read_dir(&path) {
                Ok(rd) => {
                    let names: Vec<String> = rd.filter_map(|e| e.ok())
                        .map(|e| e.file_name().to_string_lossy().to_string())
                        .collect();
                    names.join("\n").into_bytes()
                }
                Err(_) => return Ok(self.error(tag, 5)),
            }
        } else {
            let full = std::fs::read(&path)?;
            let start = offset as usize;
            if start >= full.len() { Vec::new() }
            else {
                let end = (start + count as usize).min(full.len());
                full[start..end].to_vec()
            }
        };

        let mut p = Vec::new();
        p.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        p.extend_from_slice(&payload);
        Ok(self.reply(P9_RREAD, tag, &p))
    }

    fn handle_write(&mut self, data: &[u8], tag: u16) -> Result<Vec<u8>> {
        if data.len() < 7 + 4 + 8 + 4 {
            return Ok(self.error(tag, 22));
        }
        let fid = u32::from_le_bytes(data[7..11].try_into().unwrap());
        let _offset = u64::from_le_bytes(data[11..19].try_into().unwrap());
        let count = u32::from_le_bytes(data[19..23].try_into().unwrap());
        let payload = &data[23..23 + count as usize];

        let entry = match self.fids.get(&fid) {
            Some(e) => e,
            None => return Ok(self.error(tag, 2)),
        };
        let path = entry.path.clone();
        if entry.is_dir { return Ok(self.error(tag, 21)); }

        std::fs::write(&path, payload)?;
        let mut p = Vec::new();
        p.extend_from_slice(&count.to_le_bytes());
        Ok(self.reply(P9_RWRITE, tag, &p))
    }

    fn handle_clunk(&mut self, data: &[u8], tag: u16) -> Result<Vec<u8>> {
        if data.len() < 7 + 4 { return Ok(self.error(tag, 22)); }
        let fid = u32::from_le_bytes(data[7..11].try_into().unwrap());
        self.fids.remove(&fid);
        Ok(self.reply(P9_RCLUNK, tag, &[]))
    }

    fn handle_stat(&self, data: &[u8], tag: u16) -> Result<Vec<u8>> {
        if data.len() < 7 + 4 { return Ok(self.error(tag, 22)); }
        let fid = u32::from_le_bytes(data[7..11].try_into().unwrap());
        let entry = match self.fids.get(&fid) {
            Some(e) => e,
            None => return Ok(self.error(tag, 2)),
        };
        let qid = self.make_qid(&entry.path);
        let mut p = Vec::new();
        p.extend_from_slice(&(qid.len() as u16 + 2).to_le_bytes()); // size
        p.extend_from_slice(&qid);
        p.extend_from_slice(&0u32.to_le_bytes()); // mode
        p.extend_from_slice(&0u32.to_le_bytes()); // atime
        p.extend_from_slice(&0u32.to_le_bytes()); // mtime
        p.extend_from_slice(&0u64.to_le_bytes()); // length
        Ok(self.reply(P9_RSTAT, tag, &p))
    }

    fn make_qid(&self, path: &Path) -> Vec<u8> {
        // qid: type(1) + version(4) + path(8)
        let md = std::fs::metadata(path).ok();
        let is_dir = md.as_ref().map(|m| m.is_dir()).unwrap_or(false);
        let size = md.as_ref().map(|m| m.len()).unwrap_or(0);
        let mut q = Vec::with_capacity(13);
        q.push(if is_dir { 0x80 } else { 0x00 });
        q.extend_from_slice(&0u32.to_le_bytes());
        q.extend_from_slice(&size.to_le_bytes());
        q
    }
}

pub type SharedP9 = Arc<Mutex<P9Server>>;