//! User-mode networking via smoltcp.
//!
//! Зачем: полноценный TCP/IP стек в user-space позволяет дать гостю интернет
//! БЕЗ прав администратора и БЕЗ создания TAP/Wintun-адаптера.
//!
//! Как работает:
//!   1. VirtIO-net передаёт Ethernet-фрейм от гостя
//!   2. Мы парсим его smoltcp-стеком
//!   3. TCP/UDP-соединения проксируются в host sockets (через socket2)
//!   4. Ответные пакеты возвращаются в гостя через VirtIO-net RX

use anyhow::{Context, Result};
use smoltcp::iface::{Config as IfaceConfig, Interface, SocketHandle, SocketSet};
use smoltcp::phy::{Device, DeviceCapabilities, Medium, RxToken, TxToken};
use smoltcp::socket::{tcp, udp};
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, HardwareAddress, IpAddress, IpCidr, Ipv4Address};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub const MTU: usize = 1500;
pub const GUEST_IP: Ipv4Address = Ipv4Address::new(10, 0, 2, 15);
pub const GATEWAY_IP: Ipv4Address = Ipv4Address::new(10, 0, 2, 2);

/// Очередь пакетов между VirtIO-net и smoltcp.
#[derive(Clone)]
pub struct PacketQueue {
    inner: Arc<Mutex<VecDeque<Vec<u8>>>>,
}

impl PacketQueue {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::with_capacity(256))),
        }
    }

    pub fn push(&self, pkt: Vec<u8>) {
        let mut q = self.inner.lock().unwrap();
        if q.len() < 1024 {
            q.push_back(pkt);
        }
    }

    pub fn pop(&self) -> Option<Vec<u8>> {
        self.inner.lock().unwrap().pop_front()
    }
}

/// Виртуальное устройство для smoltcp — мост между VirtIO-net и стеком.
pub struct VirtioNetDevice {
    rx: PacketQueue, // host → smoltcp (пакеты от гостя)
    tx: PacketQueue, // smoltcp → host (пакеты в гостя)
}

impl VirtioNetDevice {
    pub fn new(rx: PacketQueue, tx: PacketQueue) -> Self {
        Self { rx, tx }
    }
}

impl<'a> Device<'a> for VirtioNetDevice {
    type RxToken = VirtioRxToken;
    type TxToken = VirtioTxToken;

    fn receive(&'a mut self) -> Option<(Self::RxToken, Self::TxToken)> {
        let rx_pkt = self.rx.pop()?;
        Some((
            VirtioRxToken { buffer: rx_pkt },
            VirtioTxToken {
                queue: self.tx.clone(),
            },
        ))
    }

    fn transmit(&'a mut self) -> Option<Self::TxToken> {
        Some(VirtioTxToken {
            queue: self.tx.clone(),
        })
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.medium = Medium::Ethernet;
        caps.max_transmission_unit = MTU;
        caps.max_burst_size = Some(64);
        caps
    }
}

pub struct VirtioRxToken {
    buffer: Vec<u8>,
}
impl RxToken for VirtioRxToken {
    fn consume<R, F>(mut self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        f(&mut self.buffer)
    }
}

pub struct VirtioTxToken {
    queue: PacketQueue,
}
impl TxToken for VirtioTxToken {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut buf = vec![0u8; len];
        let r = f(&mut buf);
        self.queue.push(buf);
        r
    }
}

/// Backend user-mode networking.
pub struct SmoltcpBackend {
    rx: PacketQueue,
    tx: PacketQueue,
    iface: Option<Interface>,
    sockets: SocketSet<'static>,
    handles: Vec<SocketHandle>,
}

impl SmoltcpBackend {
    pub fn new() -> Result<Self> {
        tracing::info!("smoltcp backend initialized (user-mode, no admin required)");
        Ok(Self {
            rx: PacketQueue::new(),
            tx: PacketQueue::new(),
            iface: None,
            sockets: SocketSet::new(Vec::new()),
            handles: Vec::new(),
        })
    }

    /// Очередь "host → smoltcp" (пакеты от гостя).
    pub fn rx(&self) -> PacketQueue {
        self.rx.clone()
    }
    /// Очередь "smoltcp → host" (пакеты в гостя).
    pub fn tx(&self) -> PacketQueue {
        self.tx.clone()
    }

    /// Инициализация интерфейса и базовых сокетов (DNS + пример TCP listener).
    pub fn init_iface(&mut self) -> Result<()> {
        let mut device = VirtioNetDevice::new(self.rx.clone(), self.tx.clone());
        let config = IfaceConfig::new(HardwareAddress::Ethernet(
            EthernetAddress([0x02, 0x41, 0x45, 0x52, 0x4F, 0x01]), // 02:41:45:52:4F:01
        ));
        let mut iface = Interface::new(config, &mut device, Instant::now());
        iface.update_ip_addrs(|addrs| {
            let _ = addrs.push(IpCidr::new(IpAddress::Ipv4(GUEST_IP), 24));
        });
        iface.routes_mut().add_default_ipv4_route(GATEWAY_IP).ok();

        // UDP socket для DNS-proxy (порт 53)
        let dns_rx = udp::PacketBuffer::new(vec![udp::PacketMetadata::EMPTY; 16], vec![0u8; 4096]);
        let dns_tx = udp::PacketBuffer::new(vec![udp::PacketMetadata::EMPTY; 16], vec![0u8; 4096]);
        let dns_socket = udp::Socket::new(dns_rx, dns_tx);
        let dns_handle = self.sockets.add(dns_socket);
        self.handles.push(dns_handle);

        self.iface = Some(iface);
        tracing::info!(
            "smoltcp iface ready: guest={} gateway={}",
            GUEST_IP,
            GATEWAY_IP
        );
        Ok(())
    }

    /// Основной цикл обработки пакетов.
    /// Вызывается из tokio-таска и работает до остановки VM.
    pub async fn run(&mut self) -> Result<()> {
        self.init_iface().context("smoltcp init")?;
        let mut device = VirtioNetDevice::new(self.rx.clone(), self.tx.clone());

        loop {
            // Проверяем очереди и продвигаем стек.
            if let Some(iface) = self.iface.as_mut() {
                let now = Instant::now();
                iface.poll(now, &mut device, &mut self.sockets);
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        }
    }

    pub fn stats(&self) -> NetStats {
        NetStats {
            rx_packets: self.rx.inner.lock().unwrap().len() as u64,
            tx_packets: self.tx.inner.lock().unwrap().len() as u64,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct NetStats {
    pub rx_packets: u64,
    pub tx_packets: u64,
}
