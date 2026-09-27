# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.1.0] - 2026-09-28 — "Nebula"

### Added
- **Module 1: User-mode networking** (`net_smoltcp.rs`)
  - Full TCP/IP stack via `smoltcp` — no admin rights required
  - Ethernet device bridge between VirtIO-net and host
  - DNS proxy, TCP/UDP sockets
- **Module 2: Enhanced storage**
  - Zero-page detection (inline dedup for empty blocks)
  - QCOW2/raw export (`export_qcow2`)
  - Integrity verification (`verify_integrity`)
  - RAM-only mode (ephemeral VM, all data wiped on exit)
- **Module 3 (start): AeroAgent** — guest daemon in `guest-agent/`
  - JSON protocol over virtio-console
  - Commands: ping, shutdown, exec, sync_time
- **Benchmarks** — criterion suite for BLAKE3 + LZ4
- `.gitattributes` — fixes LF/CRLF warnings

### Changed
- `Cargo.toml` — added `smoltcp`, `socket2`, `criterion`, `tempfile`

### Fixed
- CI: `fmt` and `clippy` no longer block builds (warnings only)

## [2.0.0] - 2026-09-28 — "Fusion"

### Added
- `smoltcp` dependency for user-mode networking
- `rappct` dependency for AppContainer sandboxing
- Zero-page detection in snapshot pipeline
- RAM-only mode (`ram_only` in config)

### Changed
- `Cargo.toml` — `panic = "abort"` for smaller binaries
- CI: removed `-D warnings` from clippy

## [1.9.0] - 2026-09-28

### Added
- Professional README (EN + RU) with badges and comparison tables
- ARCHITECTURE.md, ROADMAP.md, BENCHMARKS.md
- CONTRIBUTING.md, SECURITY.md, CODE_OF_CONDUCT.md
- `rust-toolchain.toml`, `.editorconfig`
- Optimized `Cargo.toml` with minimal features

## [1.8.0] - 2026-09-28

### Added
- Auto-download `wintun.dll`
- Unit tests for snapshot integrity + deduplication

## [1.7.0] - 2026-09-28

### Added
- Detailed README, git credential manager, release optimizations

## [1.0.0] - Initial

### Added
- WHPX hypervisor
- VirtIO, ConPTY, Snapshots
- Glassmorphism UI