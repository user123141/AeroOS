//! User-mode networking via smoltcp 0.11.
//!
//! API smoltcp 0.11:
//!   - `Device::receive(&mut self, Instant)`
//!   - `type RxToken<'a>`, `type TxToken<'a>`
//!   - `Device` без лайфтайма, лайфтайм только у ассоциированных типов

use anyhow::{Context, Result};
use smoltcp::iface::{Config as IfaceConfig, Interface, SocketHandle, SocketSet};
use smoltcp::phy::{self, Device, DeviceCapabilities, Medium, RxToken, TxToken};
use smoltcp::socket::{tcp, udp};
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, HardwareAddress, IpAddress, IpCidr, Ipv4Address};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub const MTU: usize = 1500;
pub const GUEST_IP: Ipv4Address = Ipv4Address::new(10, 0, 2, 15);
pub const GATEWAY_IP: Ipv4Address = Ipv4Address::new(10, 0, 2, 2);

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
    pub fn len(&self) -> usize {
        self.inner.lock().unwrap().len()
    }
}

/// Виртуальное устройство для smoltcp.
pub struct VirtioNetDevice {
    rx: PacketQueue,
    tx: PacketQueue,
}

impl VirtioNetDevice {
    pub fn new(rx: PacketQueue, tx: PacketQueue) -> Self {
        Self { rx, tx }
    }
}

impl Device for VirtioNetDevice {
    type RxToken<'a> = VirtioRxToken;
    type TxToken<'a> = VirtioTxToken;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let rx_pkt = self.rx.pop()?;
        Some((
            VirtioRxToken { buffer: rx_pkt },
            VirtioTxToken {
                queue: self.tx.clone(),
            },
        ))
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
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

pub struct SmoltcpBackend {
    rx: PacketQueue,
    tx: PacketQueue,
    iface: Option<Interface>,
    sockets: SocketSet<'static>,
    handles: Vec<SocketHandle>,
}

impl SmoltcpBackend {
    pub fn new() -> Result<Self> {
        tracing::info!("smoltcp backend initialized (user-mode)");
        Ok(Self {
            rx: PacketQueue::new(),
            tx: PacketQueue::new(),
            iface: None,
            sockets: SocketSet::new(Vec::new()),
            handles: Vec::new(),
        })
    }
    pub fn rx(&self) -> PacketQueue {
        self.rx.clone()
    }
    pub fn tx(&self) -> PacketQueue {
        self.tx.clone()
    }

    pub fn init_iface(&mut self) -> Result<()> {
        let mut device = VirtioNetDevice::new(self.rx.clone(), self.tx.clone());
        let config = IfaceConfig::new(HardwareAddress::Ethernet(EthernetAddress([
            0x02, 0x41, 0x45, 0x52, 0x4F, 0x01,
        ])));
        let mut iface = Interface::new(config, &mut device, Instant::now());
        iface.update_ip_addrs(|addrs| {
            let _ = addrs.push(IpCidr::new(IpAddress::Ipv4(GUEST_IP), 24));
        });
        iface.routes_mut().add_default_ipv4_route(GATEWAY_IP).ok();

        let dns_rx = udp::PacketBuffer::new(vec![udp::PacketMetadata::EMPTY; 16], vec![0u8; 4096]);
        let dns_tx = udp::PacketBuffer::new(vec![udp::PacketMetadata::EMPTY; 16], vec![0u8; 4096]);
        let dns_socket = udp::Socket::new(dns_rx, dns_tx);
        let dns_handle = self.sockets.add(dns_socket);
        self.handles.push(dns_handle);

        let tcp_rx = tcp::SocketBuffer::new(vec![0u8; 4096]);
        let tcp_tx = tcp::SocketBuffer::new(vec![0u8; 4096]);
        let tcp_socket = tcp::Socket::new(tcp_rx, tcp_tx);
        let tcp_handle = self.sockets.add(tcp_socket);
        self.handles.push(tcp_handle);

        self.iface = Some(iface);
        tracing::info!(
            "smoltcp iface ready: guest={} gateway={}",
            GUEST_IP,
            GATEWAY_IP
        );
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        self.init_iface().context("smoltcp init")?;
        let mut device = VirtioNetDevice::new(self.rx.clone(), self.tx.clone());
        loop {
            if let Some(iface) = self.iface.as_mut() {
                iface.poll(Instant::now(), &mut device, &mut self.sockets);
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        }
    }

    pub fn stats(&self) -> NetStats {
        NetStats {
            rx_packets: self.rx.len() as u64,
            tx_packets: self.tx.len() as u64,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct NetStats {
    pub rx_packets: u64,
    pub tx_packets: u64,
}
