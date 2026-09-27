//! TCP Proxy: guest VM → host socket.
//!
//! Идея: гость открывает TCP-соединение к `10.0.2.2:port`,
//! наш smoltcp-стек перехватывает и проксирует через tokio TcpStream
//! к реальному хосту (`127.0.0.1:port` или внешнему адресу).
//!
//! Реализация использует двойной канал между smoltcp socket и tokio.

use anyhow::{Context, Result};
use smoltcp::iface::{Interface, SocketHandle, SocketSet};
use smoltcp::socket::tcp;
use smoltcp::time::Instant;
use smoltcp::wire::{IpAddress, Ipv4Address};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Порт гостя, на который приходят соединения (проксирует на host).
pub const PROXY_PORT: u16 = 12345;

/// Специальный IP, через который гость обращается к хосту.
pub const HOST_IP_IN_GUEST: Ipv4Address = Ipv4Address::new(10, 0, 2, 2);

pub struct TcpProxy {
    /// smoltcp handle → tokio stream
    connections: HashMap<SocketHandle, TcpStream>,
}

impl TcpProxy {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }

    /// Обрабатывает новые соединения, пришедшие на smoltcp listener.
    pub async fn poll_listener(
        &mut self,
        iface: &mut Interface,
        sockets: &mut SocketSet<'static>,
        listener_handle: SocketHandle,
        host_target: &str,
    ) -> Result<()> {
        let socket = sockets.get_mut::<tcp::Socket>(listener_handle);
        if !socket.is_active() {
            return Ok(());
        }
        if !socket.is_open() {
            // Принимаем входящее
            if socket.can_recv() {
                socket.close();
            }
            return Ok(());
        }

        // Есть данные от гостя — проксируем
        while socket.can_recv() {
            let mut buf = [0u8; 4096];
            let n = socket.recv_slice(&mut buf).unwrap_or(0);
            if n == 0 {
                break;
            }

            // Подключаемся к host (если ещё нет)
            if !self.connections.contains_key(&listener_handle) {
                match TcpStream::connect(host_target).await {
                    Ok(stream) => {
                        tracing::info!("TCP proxy: connected to {}", host_target);
                        self.connections.insert(listener_handle, stream);
                    }
                    Err(e) => {
                        tracing::warn!("TCP proxy: connect to {} failed: {}", host_target, e);
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

    /// Обрабатывает данные, пришедшие с host-сокета, и пишет их в guest.
    pub async fn poll_back(&mut self, sockets: &mut SocketSet<'static>) -> Result<()> {
        let handles: Vec<SocketHandle> = self.connections.keys().copied().collect();
        for h in handles {
            let stream = match self.connections.get_mut(&h) {
                Some(s) => s,
                None => continue,
            };
            let mut buf = [0u8; 4096];
            match stream.try_read(&mut buf) {
                Ok(0) => {
                    tracing::info!("TCP proxy: host closed");
                    self.connections.remove(&h);
                }
                Ok(n) => {
                    let socket = sockets.get_mut::<tcp::Socket>(h);
                    if socket.can_send() {
                        let _ = socket.send_slice(&buf[..n]);
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) => {
                    tracing::warn!("TCP proxy: read error: {}", e);
                    self.connections.remove(&h);
                }
            }
        }
        Ok(())
    }
}

impl Default for TcpProxy {
    fn default() -> Self {
        Self::new()
    }
}
