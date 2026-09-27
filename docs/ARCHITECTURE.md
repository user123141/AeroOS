# Architecture

## Overview

AeroOS is a single Rust binary that orchestrates five subsystems:

    ┌─────────────────────────────────────────────┐
    │              UI (HTML/CSS/JS)               │
    └──────────────────────┬──────────────────────┘
                           │ HTTP + WebSocket
    ┌──────────────────────▼──────────────────────┐
    │              Rust Runtime                    │
    │  ┌────────────┐  ┌────────────┐  ┌────────┐ │
    │  │  web.rs    │  │   ipc.rs   │  │config  │ │
    │  │ HTTP server│  │ WebSocket  │  │  TOML  │ │
    │  └────────────┘  └────────────┘  └────────┘ │
    │  ┌────────────┐  ┌────────────┐  ┌────────┐ │
    │  │hypervisor  │  │  snapshot  │  │crypto  │ │
    │  │   WHPX     │  │LZ4+BLAKE3  │  │AES-GCM │ │
    │  └────────────┘  └────────────┘  └────────┘ │
    └──────────────────────┬──────────────────────┘
                           │ FFI
    ┌──────────────────────▼──────────────────────┐
    │           Windows Hypervisor Platform       │
    └─────────────────────────────────────────────┘

## Modules

### `main.rs`

Entry point. Initializes tracing, loads config, starts async tasks
for web server, IPC server, and VM. Uses `tokio::select!` to gracefully
shut down on any task completion.

**Key decisions:**
- VM is optional — if WHPX fails to init, the UI still works
- Errors are logged with actionable messages (e.g., "run bcdedit")

### `hypervisor.rs`

Wraps WHPX (`WinHvPlatform.dll`) via `vendor/whpx`. Manages:
- Partition creation (`WHvCreatePartition`)
- Guest memory allocation (`VirtualAlloc`)
- vCPU lifecycle (`WHvCreateVirtualProcessor`)
- Exit handling (MMIO, I/O port, halt)

**Flow:**

1. Load WHPX DLL → bind functions
2. Create partition → set vCPU count
3. Allocate guest RAM → map GPA ranges
4. Load kernel + initramfs into guest memory
5. Create vCPU → set registers → run loop
6. On MMIO exit → forward to VirtIO device

### `virtio.rs`

Implements VirtIO 1.0 over MMIO. Supports:

- **Block** — read/write sectors from `disk.img`
- **Net** — packet TX/RX via `wintun.dll`
- **Console** — guest → host log
- **GPU** — 2D framebuffer, `VIRTIO_GPU_CMD_*`

Each device has a `process_queue()` that walks the descriptor chain:
`avail ring → descriptor → process → used ring`.

### `snapshot.rs`

Incremental snapshots every `auto_interval_ms`:

1. Read dirty page bitmap from WHPX
2. For each dirty page:
   - Compress with LZ4
   - Hash with BLAKE3 (dedup)
   - Encrypt with AES-256-GCM (if enabled)
   - Write block to disk
3. Append block hashes to manifest

**Restore** reads manifests in order, decompresses, decrypts, and
copies bytes back into guest RAM.

### `crypto.rs`

- **Argon2id** — password → 256-bit key
- **AES-256-GCM** — authenticated encryption
- **BLAKE3** — fast content hash for dedup

### `web.rs`

HTTP server using `hyper 1.0` + `hyper-util`. Serves embedded assets
via `rust-embed`. Adds security headers:

- `X-Content-Type-Options: nosniff`
- `Referrer-Policy: no-referrer`
- CSP via `index.html`

### `ipc.rs`

WebSocket server on `127.0.0.1:8081`. Requires session token in query
string. Commands:

| Command | Response |
|---------|----------|
| `GetStatus` | `{running, cpu_cores, ram_mb, gpu}` |
| `GetStats` | `{snapshots, dirty_pages}` |
| `Snapshot{name}` | Create snapshot |
| `Restore{name}` | Restore from snapshot |
| `Stop` | Stop VM |
| `ClipboardGet` | Read host clipboard |
| `ClipboardSet` | Write host clipboard |
| `FileUpload` | Write file to shared folder |

### `security.rs`

- `verify_artifacts()` — SHA-256 check of kernel + initramfs
- `generate_session_token()` — 128-bit random hex
- `check_disk_space()` — pre-flight disk check

## Threading model

    Main thread ──▶ tokio runtime (multi-thread)
                    │
                    ├─▶ Task: web server (HTTP)
                    ├─▶ Task: IPC server (WebSocket)
                    ├─▶ Task: VM run loop
                    └─▶ Task: snapshot streaming

All tasks communicate via `Arc<Mutex<...>>`. The VM is behind a
single mutex to prevent data races on guest memory.

## Memory model

    Host RAM
      │
      ├─ VirtualAlloc(8 GB) ────► Guest RAM (GPA 0..8GB)
      │                          │
      │                          ├─ 0x0000_0000 - 0x000F_FFFF: BIOS/low
      │                          ├─ 0x0010_0000 - kernel end: Kernel
      │                          └─ kernel end.. + initramfs: Initramfs
      │
      └─ Snapshot buffer ────────► Dirty pages → LZ4 → disk

## Dependencies

See `Cargo.toml`. Total: ~30 direct deps. Key ones:

| Crate | Purpose |
|-------|---------|
| `tokio` | Async runtime |
| `hyper` | HTTP server |
| `tokio-tungstenite` | WebSocket |
| `whpx` | WHPX FFI (vendored) |
| `conpty` | ConPTY FFI (vendored) |
| `aes-gcm` | Snapshot encryption |
| `blake3` | Content hashing |
| `lz4_flex` | Compression |
| `serde` | Config serialization |

## Build pipeline

    setup.ps1
      │
      ├─▶ Check: Rust / Git / MSVC / WHPX
      ├─▶ Ensure: wintun.dll
      ├─▶ cargo generate-lockfile
      ├─▶ cargo build --release
      └─▶ Copy to build/aeroos.exe