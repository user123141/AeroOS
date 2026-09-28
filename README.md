<div align="center">

# AeroOS

**Portable virtualization OS for Windows — in a single executable.**

[![CI](https://github.com/user123141/AeroOS/actions/workflows/ci.yml/badge.svg)](https://github.com/user123141/AeroOS/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-Windows%2010%2F11-blue.svg)](https://www.microsoft.com/windows)

[Quick Start](#-quick-start) &middot; [Architecture](docs/ARCHITECTURE.md) &middot; [API](docs/API.md) &middot; [Roadmap](docs/ROADMAP.md) &middot; [Русский](README.ru.md)

</div>

---

## What is AeroOS?

AeroOS is a **portable operating system** that runs entirely inside a single `.exe` on Windows. It uses the native **Windows Hypervisor Platform (WHPX)** to spin up a lightweight VM, and presents its own graphical interface in a browser-like window.

100% Rust. No `.NET`, no `Electron`, no Python. One native binary.

## Current version: **v4.3.0 "Integrity"**

## Features

| Feature | Status |
|---------|:------:|
| Single `.exe` | OK |
| WHPX hypervisor | requires `bcdedit` + reboot |
| Draggable windows (native macOS-style) | OK |
| i18n (RU / EN) | OK |
| Theme (Dark / Light / Aero) | OK |
| Brightness & volume control | OK |
| Toast notifications + sound | OK |
| Dashboard clock | OK |
| Multi-VM UI | OK |
| Incremental encrypted snapshots (LZ4 + BLAKE3 + AES-256-GCM) | OK |
| Live migration (`.aeromig`) | OK |
| AeroRegistry (pull micro-images) | OK |
| OCI pull + initramfs conversion | OK |
| GPU passthrough (Hyper-V GPU-P) | OK |
| License: Ed25519 + HWID + nonce | OK |
| AeroBoot (BIOS + UEFI image builder) | partial |

## Quick Start

Admin PowerShell (one-time):

    bcdedit /set hypervisorlaunchtype auto
    Restart-Computer

Then:

    cd AeroOS\build
    .\aeroos.exe

UI: http://127.0.0.1:8080

## Requirements

- Windows 10/11 (x64)
- Rust 1.75+
- Hyper-V + Hypervisor Platform

## Documentation

| Document | Description |
|----------|-------------|
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | Internals, threads, memory layout |
| [API.md](docs/API.md) | Public API of every module |
| [ROADMAP.md](docs/ROADMAP.md) | Planned features |
| [BENCHMARKS.md](docs/BENCHMARKS.md) | Performance data |
| [AERO.md](docs/AERO.md) | Aero Design Language |
| [AEROFILE.md](docs/AEROFILE.md) | Declarative VM config |
| [SESSION.md](docs/SESSION.md) | Dev session context |
| [CHANGELOG.md](CHANGELOG.md) | Version history |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Contribution guide |
| [SECURITY.md](SECURITY.md) | Reporting vulnerabilities |
| [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) | Community rules |

## License

MIT © 2026 AeroOS Contributors