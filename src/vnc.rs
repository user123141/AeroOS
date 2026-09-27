//! VNC/Frame streaming: framebuffer → JPEG → WebSocket → Canvas.
//! Простая и надёжная реализация без WebRTC.

use anyhow::Result;
use std::sync::{Arc, Mutex};

pub struct FrameStream {
    framebuffer: Arc<Mutex<Vec<u8>>>,
    width: u32,
    height: u32,
    quality: u8,
}

impl FrameStream {
    pub fn new(fb: Arc<Mutex<Vec<u8>>>, w: u32, h: u32) -> Self {
        Self { framebuffer: fb, width: w, height: h, quality: 75 }
    }

    pub fn capture_jpeg(&self) -> Result<Vec<u8>> {
        let fb = self.framebuffer.lock().unwrap();
        if fb.is_empty() || self.width == 0 || self.height == 0 {
            anyhow::bail!("Empty framebuffer");
        }
        let expected = (self.width * self.height * 4) as usize;
        if fb.len() < expected {
            anyhow::bail!("Framebuffer too small: {} < {}", fb.len(), expected);
        }

        // RGBA → RGB
        let mut rgb = Vec::with_capacity((self.width * self.height * 3) as usize);
        for chunk in fb[..expected].chunks_exact(4) {
            rgb.push(chunk[0]);
            rgb.push(chunk[1]);
            rgb.push(chunk[2]);
        }

        let img = image::RgbImage::from_raw(self.width, self.height, rgb)
            .ok_or_else(|| anyhow::anyhow!("RgbImage build failed"))?;

        let mut jpeg = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut jpeg);
        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, self.quality);
        img.write_with_encoder(encoder)?;
        Ok(jpeg)
    }

    pub async fn run(self, tx: tokio::sync::broadcast::Sender<Vec<u8>>) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
        loop {
            interval.tick().await;
            if let Ok(jpeg) = self.capture_jpeg() {
                let _ = tx.send(jpeg);
            }
        }
    }

    pub fn dimensions(&self) -> (u32, u32) { (self.width, self.height) }
}