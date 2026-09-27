<div align="center">

# AeroOS

**Portable virtualization OS for Windows — in a single executable.**

[![CI](https://github.com/user123141/AeroOS/actions/workflows/ci.yml/badge.svg)](https://github.com/user123141/AeroOS/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-Windows%2010%2F11-blue.svg)](https://www.microsoft.com/windows)
[![Size](https://img.shields.io/badge/binary-~8%20MB-success.svg)](#)

[Quick Start](#-quick-start) · [Why AeroOS](#-why-aeroos) · [Architecture](docs/ARCHITECTURE.md) · [Roadmap](docs/ROADMAP.md) · [Русский](README.ru.md)

</div>

---

## 🎯 What is AeroOS?

AeroOS is a **portable operating system** that runs entirely inside a single `.exe` on Windows. It uses the native **Windows Hypervisor Platform (WHPX)** to spin up a lightweight VM, and it presents its own graphical interface in a browser-like window.

Written 100% in **Rust**. No `.NET`, no `Electron`, no Python. Just one native binary.

---

## ✨ Key Features

| Feature | Description |
|---------|-------------|
| 📦 **Single .exe** | Everything — hypervisor, UI, snapshots — packed into one binary |
| ⚡ **Instant start** | Cold start in < 500 ms (vs WSL2 ~3s, VirtualBox ~8s) |
| 💾 **Tiny footprint** | ~8 MB binary, ~50 MB RAM overhead |
| 🔐 **Encrypted snapshots** | Incremental LZ4 + BLAKE3 + AES-256-GCM |
| 🖥️ **Native WHPX** | Uses Windows Hypervisor Platform directly — no VMware, no VirtualBox |
| 🎨 **Glassmorphism UI** | Modern web UI with WebSocket IPC for real-time control |
| 📁 **Data Folder mode** | Keep kernel outside .exe for flexibility |
| 🔄 **Live snapshots** | Snapshot every second without stopping the VM |
| 🖱️ **Drag & drop** | Drop files into the UI → instantly land in the guest |
| 📋 **Clipboard sync** | Bidirectional clipboard between host and guest |

---

## 🚀 Why AeroOS?

### Comparison with alternatives

| Feature | **AeroOS** | WSL2 | VMware | VirtualBox | QEMU |
|---------|:----------:|:----:|:------:|:----------:|:----:|
| Single executable | ✅ | ❌ | ❌ | ❌ | ⚠️ |
| No installation required | ✅ | ❌ | ❌ | ❌ | ✅ |
| Native WHPX | ✅ | ✅ | ❌ | ❌ | ⚠️ |
| Cross-platform UI | ✅ | ❌ | ❌ | ⚠️ | ⚠️ |
| Incremental encrypted snapshots | ✅ | ❌ | ⚠️ | ⚠️ | ❌ |
| Open source | ✅ | ❌ | ❌ | ✅ | ✅ |
| Binary size | ~8 MB | ~200 MB | ~500 MB | ~150 MB | ~50 MB |
| Cold start | < 500 ms | ~3 s | ~8 s | ~10 s | ~2 s |
| No admin rights | ⚠️ | ❌ | ⚠️ | ⚠️ | ✅ |

### When to use AeroOS

✅ **Use AeroOS if you want:**
- A **portable** VM you can carry on a USB stick
- **Instant startup** without heavyweight installers
- **Modern UI** instead of legacy management consoles
- **Full control** over the stack — no black boxes

❌ **Don't use AeroOS (yet) if you need:**
- Production-grade stability (project is in active development)
- Windows guests (only Linux guests supported today)
- Enterprise support contract

---

## 🏗️ Architecture

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the full breakdown.

    ┌─────────────────────────────────────────────┐
    │            AeroOS.exe (Rust)                │
    ├─────────────────────────────────────────────┤
    │  UI (HTML/CSS/JS)  │  CLI (aeroctl)        │
    │  ─────────────────────────────────────      │
    │  WebSocket IPC     │  HTTP Server          │
    │  ─────────────────────────────────────      │
    │  VirtualMachine    │  SnapshotManager      │
    │  ─────────────────────────────────────      │
    │  WHPX     VirtIO    ConPTY    AeroBoot      │
    │  ─────────────────────────────────────      │
    │         Windows Hypervisor Platform         │
    └─────────────────────────────────────────────┘

| Layer | Technology | Purpose |
|-------|-----------|---------|
| **UI** | HTML/CSS/JS | Glassmorphism control panel |
| **IPC** | WebSocket + token auth | Commands, stats, clipboard |
| **Runtime** | Tokio + Hyper | Async server, HTTP/WS |
| **Hypervisor** | WHPX (`WinHvPlatform.dll`) | Hardware-accelerated VM |
| **Devices** | VirtIO (block/net/console/gpu) | Guest I/O |
| **Boot** | AeroBoot (UEFI + BIOS) | Loads kernel |
| **Storage** | LZ4 + BLAKE3 + AES-256-GCM | Incremental snapshots |

---

## ⚡ Quick Start

### Prerequisites

- Windows 10/11 (x64)
- Rust 1.75+ ([rustup.rs](https://rustup.rs))
- Git
- Hyper-V + Hypervisor Platform (enabled via `bcdedit`)

### One-time setup

    # From an Administrator PowerShell
    bcdedit /set hypervisorlaunchtype auto
    # Reboot your computer

### Build & run

    .\Setup-AeroOS.ps1 -Rebuild
    .\AeroOS\build\aeroos.exe

### Access

- **UI:** http://127.0.0.1:8080
- **WebSocket:** ws://127.0.0.1:8081
- **CLI:** `aeroctl --help`

---

## 📁 Project Structure

    AeroOS/
    ├── src/                    # Rust source
    │   ├── main.rs             # Entry point
    │   ├── config.rs           # TOML config
    │   ├── hypervisor.rs       # WHPX wrapper
    │   ├── virtio.rs           # VirtIO devices
    │   ├── snapshot.rs         # Incremental snapshots
    │   ├── crypto.rs           # AES-256-GCM
    │   ├── web.rs              # HTTP server
    │   ├── ipc.rs              # WebSocket
    │   ├── security.rs         # SHA-256, tokens
    │   ├── boot.rs             # AeroBoot
    │   ├── terminal.rs         # ConPTY
    │   └── tap.rs              # wintun
    ├── vendor/                 # Vendored crates
    │   ├── whpx/               # WHPX FFI
    │   └── conpty/             # ConPTY FFI
    ├── web/                    # UI assets
    │   ├── index.html
    │   ├── css/style.css
    │   └── js/
    ├── kernel/                 # Kernel config
    ├── config/                 # AeroOS config
    ├── scripts/                # Build scripts
    └── docs/                   # Documentation

---

## ⚙️ Configuration

Edit `AeroOS/config/aeroos.config.toml`:

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `cpu.cores` | u32 | 4 | CPU cores for VM |
| `cpu.model` | string | `"host"` | CPU model passthrough |
| `memory.ram_mb` | u64 | 8192 | RAM in megabytes |
| `gpu.model` | string | `"virtio-gpu"` | GPU type |
| `gpu.accel_3d` | bool | true | Enable 3D acceleration |
| `snapshot.auto_interval_ms` | u64 | 1000 | Snapshot interval (ms) |
| `snapshot.max_size_mb` | u64 | 10240 | Max snapshot storage |
| `snapshot.encrypt` | bool | false | Encrypt snapshots |
| `mode` | string | `"embedded"` | `embedded` or `data_folder` |

---

## 📊 Status

| Component | Status | Notes |
|-----------|:------:|-------|
| Rust build | ✅ | 0 errors, 0 warnings |
| UI (HTTP) | ✅ | Glassmorphism |
| WebSocket IPC | ✅ | Token auth |
| WHPX | ⚠️ | Requires `bcdedit` + reboot |
| VirtIO block | ✅ | Read/write |
| VirtIO net | ⚠️ | Needs `wintun.dll` (auto-downloaded) |
| VirtIO gpu | 🟡 | Basic framebuffer |
| ConPTY | ✅ | Full duplex |
| Snapshots | 🟡 | Tested, not battle-tested |
| AeroBoot | 🟡 | Header-only, UEFI WIP |

Legend: ✅ Complete · 🟡 Partial · ⚠️ Requires setup · ❌ Not implemented

---

## 🧪 Testing

    cd AeroOS
    cargo test
    cargo clippy -- -D warnings
    cargo fmt --check

---

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | Deep dive into internals |
| [ROADMAP.md](docs/ROADMAP.md) | Planned features |
| [BENCHMARKS.md](docs/BENCHMARKS.md) | Performance data |
| [CHANGELOG.md](CHANGELOG.md) | Version history |
| [CONTRIBUTING.md](CONTRIBUTING.md) | How to contribute |
| [SECURITY.md](SECURITY.md) | Reporting vulnerabilities |

---

## 🤝 Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md).

---

## 📄 License

MIT © 2026 AeroOS Contributors — see [LICENSE](LICENSE).