//! VNC/WebRTC stream: framebuffer → JPEG → WebSocket → Canvas.
//!
//! Простая реализация: каждые N мс берём framebuffer из virtio-gpu,
//! кодируем в JPEG, отправляем по WebSocket клиенту.
//!
//! Полная WebRTC (VP8/H.264) — в v3.3.

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
        Self {
            framebuffer: fb,
            width: w,
            height: h,
            quality: 75,
        }
    }

    /// Снять текущий кадр и закодировать в JPEG.
    pub fn capture_jpeg(&self) -> Result<Vec<u8>> {
        let fb = self.framebuffer.lock().unwrap();
        if fb.is_empty() || self.width == 0 || self.height == 0 {
            anyhow::bail!("Empty framebuffer");
        }

        // framebuffer — RGBA (w*h*4)
        let expected = (self.width * self.height * 4) as usize;
        if fb.len() < expected {
            anyhow::bail!("Framebuffer too small: {} < {}", fb.len(), expected);
        }

        // Конвертируем RGBA → RGB (image crate)
        let mut rgb = Vec::with_capacity((self.width * self.height * 3) as usize);
        for chunk in fb[..expected].chunks_exact(4) {
            rgb.push(chunk[0]);
            rgb.push(chunk[1]);
            rgb.push(chunk[2]);
        }

        let img = image::RgbImage::from_raw(self.width, self.height, rgb)
            .ok_or_else(|| anyhow::anyhow!("Failed to build RgbImage"))?;

        let mut jpeg = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut jpeg);
        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, self.quality);
        img.write_with_encoder(encoder)?;
        Ok(jpeg)
    }

    /// Запустить фоновый стрим: каждые 100 мс публикует кадр в broadcast-канал.
    pub async fn run(self, tx: tokio::sync::broadcast::Sender<Vec<u8>>) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
        loop {
            interval.tick().await;
            match self.capture_jpeg() {
                Ok(jpeg) => {
                    let _ = tx.send(jpeg);
                }
                Err(_) => {}
            }
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}
