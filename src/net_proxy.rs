//! TCP Proxy: guest VM → host socket.

use anyhow::{Context, Result};
use smoltcp::iface::{Interface, SocketHandle, SocketSet};
use smoltcp::socket::tcp;
use smoltcp::time::Instant;
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub const PROXY_PORT: u16 = 12345;

pub struct TcpProxy {
    connections: HashMap<SocketHandle, TcpStream>,
}

impl TcpProxy {
    pub fn new() -> Self {
        Self { connections: HashMap::new() }
    }

    pub async fn poll_listener(
        &mut self,
        _iface: &mut Interface,
        sockets: &mut SocketSet<'static>,
        listener_handle: SocketHandle,
        host_target: &str,
    ) -> Result<()> {
        let socket = sockets.get_mut::<tcp::Socket>(listener_handle);
        if !socket.is_active() { return Ok(()); }
        if socket.can_recv() {
            let mut buf = [0u8; 4096];
            let n = socket.recv_slice(&mut buf).unwrap_or(0);
            if n == 0 { return Ok(()); }

            if !self.connections.contains_key(&listener_handle) {
                match TcpStream::connect(host_target).await {
                    Ok(stream) => {
                        tracing::info!("TCP proxy: connected to {}", host_target);
                        self.connections.insert(listener_handle, stream);
                    }
                    Err(e) => {
                        tracing::warn!("TCP proxy: connect failed: {}", e);
                        socket.close();
                        return Ok(());
                    }
                }
            }

            if let Some(stream) = self.connections.get_mut(&listener_handle) {
                stream.write_all(&buf[..n]).await?;
            }
        }
        Ok(())
    }

    pub async fn poll_back(&mut self, sockets: &mut SocketSet<'static>) -> Result<()> {
        let handles: Vec<SocketHandle> = self.connections.keys().copied().collect();
        for h in handles {
            let stream = match self.connections.get_mut(&h) { Some(s) => s, None => continue };
            let mut buf = [0u8; 4096];
            match stream.try_read(&mut buf) {
                Ok(0) => { self.connections.remove(&h); }
                Ok(n) => {
                    let socket = sockets.get_mut::<tcp::Socket>(h);
                    if socket.can_send() { let _ = socket.send_slice(&buf[..n]); }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(_) => { self.connections.remove(&h); }
            }
        }
        Ok(())
    }
}

impl Default for TcpProxy {
    fn default() -> Self { Self::new() }
}