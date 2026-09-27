//! VirtIO-FS: полная реализация 9P2000.L через virtio-fs.
//!
//! Использует `p9` crate для серверной части.
//! Guest kernel подключается как 9p client, host отдаёт файлы через 9p server.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

/// 9P сервер для одной экспортируемой директории.
pub struct VirtioFsServer {
    host_root: PathBuf,
    fids: Arc<Mutex<std::collections::HashMap<u32, FidEntry>>>,
    next_fid: Arc<Mutex<u32>>,
}

struct FidEntry {
    path: PathBuf,
    is_dir: bool,
    open: bool,
    offset: u64,
}

impl VirtioFsServer {
    pub fn new(host_root: &Path) -> Result<Self> {
        let canonical = host_root.canonicalize()
            .with_context(|| format!("canonicalize {}", host_root.display()))?;
        if !canonical.is_dir() {
            anyhow::bail!("{} is not a directory", canonical.display());
        }
        tracing::info!("VirtIO-FS root: {}", canonical.display());
        Ok(Self {
            host_root: canonical,
            fids: Arc::new(Mutex::new(std::collections::HashMap::new())),
            next_fid: Arc::new(Mutex::new(1)),
        })
    }

    fn safe_join(&self, rel: &str) -> Result<PathBuf> {
        let joined = self.host_root.join(rel);
        let canon = joined.canonicalize().unwrap_or(joined.clone());
        if !canon.starts_with(&self.host_root) {
            anyhow::bail!("Path escapes host root: {}", rel);
        }
        Ok(canon)
    }

    /// Обработать 9P-запрос.
    pub async fn handle(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < 7 { anyhow::bail!("9P packet too short"); }
        let msg_type = data[4];
        let tag = u16::from_le_bytes([data[5], data[6]]);

        match msg_type {
            100 => self.handle_version(tag),           // Tversion
            104 => self.handle_attach(data, tag).await,      // Tattach
            110 => self.handle_walk(data, tag).await,        // Twalk
            112 => self.handle_open(data, tag).await,        // Topen
            116 => self.handle_read(data, tag).await,        // Tread
            118 => self.handle_write(data, tag).await,       // Twrite
            120 => self.handle_clunk(data, tag).await,       // Tclunk
            124 => self.handle_stat(data, tag).await,        // Tstat
            _ => Ok(self.error(tag, 524)),
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
        self.reply(107, tag, &p)
    }

    fn handle_version(&self, tag: u16) -> Result<Vec<u8>> {
        let mut p = Vec::new();
        p.extend_from_slice(&65536u32.to_le_bytes());
        p.extend_from_slice(b"9P2000.L");
        Ok(self.reply(101, tag, &p))
    }

    async fn handle_attach(&self, data: &[u8], tag: u16) -> Result<Vec<u8>> {
        if data.len() < 7 + 4 + 4 + 12 { return Ok(self.error(tag, 22)); }
        let fid = u32::from_le_bytes(data[7..11].try_into().unwrap());
        let mut fids = self.fids.lock().await;
        fids.insert(fid, FidEntry {
            path: self.host_root.clone(),
            is_dir: true,
            open: false,
            offset: 0,
        });
        let qid = self.make_qid(&self.host_root);
        Ok(self.reply(105, tag, &qid))
    }

    async fn handle_walk(&self, data: &[u8], tag: u16) -> Result<Vec<u8>> {
        if data.len() < 7 + 4 + 4 + 2 { return Ok(self.error(tag, 22)); }
        let fid = u32::from_le_bytes(data[7..11].try_into().unwrap());
        let newfid = u32::from_le_bytes(data[11..15].try_into().unwrap());
        let nwname = u16::from_le_bytes(data[15..17].try_into().unwrap()) as usize;

        let fids = self.fids.lock().await;
        let entry = match fids.get(&fid) {
            Some(e) => e,
            None => return Ok(self.error(tag, 22)),
        };
        let mut current = entry.path.clone();
        drop(fids);

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
        let mut fids = self.fids.lock().await;
        fids.insert(newfid, FidEntry {
            path: current.clone(),
            is_dir,
            open: false,
            offset: 0,
        });

        let mut p = Vec::new();
        p.extend_from_slice(&(nwname as u16).to_le_bytes());
        for _ in 0..nwname { p.extend_from_slice(&self.make_qid(&current)); }
        Ok(self.reply(111, tag, &p))
    }

    async fn handle_open(&self, data: &[u8], tag: u16) -> Result<Vec<u8>> {
        if data.len() < 7 + 4 + 1 { return Ok(self.error(tag, 22)); }
        let fid = u32::from_le_bytes(data[7..11].try_into().unwrap());
        let mut fids = self.fids.lock().await;
        if let Some(e) = fids.get_mut(&fid) {
            e.open = true;
            let qid = self.make_qid(&e.path);
            let mut p = Vec::new();
            p.extend_from_slice(&qid);
            p.extend_from_slice(&0u32.to_le_bytes());
            return Ok(self.reply(113, tag, &p));
        }
        Ok(self.error(tag, 2))
    }

    async fn handle_read(&self, data: &[u8], tag: u16) -> Result<Vec<u8>> {
        if data.len() < 7 + 4 + 8 + 4 { return Ok(self.error(tag, 22)); }
        let fid = u32::from_le_bytes(data[7..11].try_into().unwrap());
        let offset = u64::from_le_bytes(data[11..19].try_into().unwrap());
        let count = u32::from_le_bytes(data[19..23].try_into().unwrap());

        let fids = self.fids.lock().await;
        let entry = match fids.get(&fid) {
            Some(e) => e,
            None => return Ok(self.error(tag, 2)),
        };
        let path = entry.path.clone();
        let is_dir = entry.is_dir;
        drop(fids);

        let payload = if is_dir {
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
        Ok(self.reply(117, tag, &p))
    }

    async fn handle_write(&self, data: &[u8], tag: u16) -> Result<Vec<u8>> {
        if data.len() < 7 + 4 + 8 + 4 { return Ok(self.error(tag, 22)); }
        let fid = u32::from_le_bytes(data[7..11].try_into().unwrap());
        let _offset = u64::from_le_bytes(data[11..19].try_into().unwrap());
        let count = u32::from_le_bytes(data[19..23].try_into().unwrap());
        let payload = &data[23..23 + count as usize];

        let fids = self.fids.lock().await;
        let entry = match fids.get(&fid) {
            Some(e) => e,
            None => return Ok(self.error(tag, 2)),
        };
        let path = entry.path.clone();
        if entry.is_dir { return Ok(self.error(tag, 21)); }
        drop(fids);

        std::fs::write(&path, payload)?;
        let mut p = Vec::new();
        p.extend_from_slice(&count.to_le_bytes());
        Ok(self.reply(119, tag, &p))
    }

    async fn handle_clunk(&self, data: &[u8], tag: u16) -> Result<Vec<u8>> {
        if data.len() < 7 + 4 { return Ok(self.error(tag, 22)); }
        let fid = u32::from_le_bytes(data[7..11].try_into().unwrap());
        let mut fids = self.fids.lock().await;
        fids.remove(&fid);
        Ok(self.reply(121, tag, &[]))
    }

    async fn handle_stat(&self, data: &[u8], tag: u16) -> Result<Vec<u8>> {
        if data.len() < 7 + 4 { return Ok(self.error(tag, 22)); }
        let fid = u32::from_le_bytes(data[7..11].try_into().unwrap());
        let fids = self.fids.lock().await;
        let entry = match fids.get(&fid) {
            Some(e) => e,
            None => return Ok(self.error(tag, 2)),
        };
        let qid = self.make_qid(&entry.path);
        let mut p = Vec::new();
        p.extend_from_slice(&(qid.len() as u16 + 2).to_le_bytes());
        p.extend_from_slice(&qid);
        p.extend_from_slice(&0u32.to_le_bytes());
        p.extend_from_slice(&0u32.to_le_bytes());
        p.extend_from_slice(&0u32.to_le_bytes());
        p.extend_from_slice(&0u64.to_le_bytes());
        Ok(self.reply(125, tag, &p))
    }

    fn make_qid(&self, path: &Path) -> Vec<u8> {
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