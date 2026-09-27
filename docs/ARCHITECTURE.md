# AeroOS Architecture

## Layers

    UI (HTML/CSS/JS)
      ↓ HTTP + WebSocket
    Rust Runtime (tokio)
      ↓
    Subsystems:
      - hypervisor.rs   → WHPX (WinHvPlatform.dll)
      - virtio.rs       → VirtIO devices (block/net/console/gpu/fs)
      - net_smoltcp.rs  → User-mode TCP/IP (guest networking)
      - snapshot.rs     → Incremental LZ4+BLAKE3+AES-256-GCM
      - license.rs      → Ed25519 signature + HWID binding
      - multi_vm.rs     → Multi-VM manager
      - registry.rs     → AeroRegistry client
      - sandbox.rs      → AppContainer isolation (winapi)
      - agent.rs        → Guest agent protocol
      - aerofile.rs     → Declarative VM config

## Modules (src/)

| Module | Purpose |
|--------|---------|
| `main.rs` | Entry, task orchestration |
| `config.rs` | TOML config loading |
| `hypervisor.rs` | WHPX wrapper |
| `virtio.rs` | VirtIO 1.0 MMIO |
| `virtio_9p.rs` | 9P file sharing protocol |
| `net_smoltcp.rs` | User-mode networking + TCP proxy |
| `snapshot.rs` | Incremental snapshots |
| `crypto.rs` | AES-256-GCM |
| `security.rs` | SHA-256, session tokens |
| `license.rs` | Ed25519 licenses |
| `hwid.rs` | Hardware fingerprint |
| `aerofile.rs` | Declarative VM config |
| `multi_vm.rs` | Multiple VMs |
| `registry.rs` | Image registry client |
| `sandbox.rs` | Windows AppContainer |
| `agent.rs` | Guest protocol |
| `boot.rs` | AeroBoot loader |
| `terminal.rs` | ConPTY |
| `tap.rs` | wintun TAP |
| `web.rs` | HTTP server |
| `ipc.rs` | WebSocket |
| `data_folder.rs` | Data-folder mode |

## Threading

- Main thread → tokio multi-thread runtime
- Each VM has own `Arc<Mutex<VirtualMachine>>`
- WebSocket per-connection tokio task
- Snapshot streaming: 1 Hz tokio interval

## Memory layout (guest)

    0x0000_0000 ─ 0x000F_FFFF   Low memory / BIOS stub
    0x0010_0000 ─ kernel_end    Kernel image
    kernel_end  ─ +initramfs    Initramfs
    +8000                       AeroBoot header

## Security

- AppContainer sandbox (winapi, securitybaseapi)
- Ed25519 license signatures
- HWID binding
- Session token per session
- SHA-256 artifact verification
- CSP headers in UI