# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [2.2.0] - 2026-09-28 — "Horizon"

### Added
- **aeroctl** — full CLI tool (`status`, `stop`, `snapshot create/list/restore`, `exec`)
- **AeroAgent host protocol** (`src/agent.rs`) — JSON commands to guest
- **Time Travel UI** — snapshot timeline with slider
- **Real-time charts** — CPU/memory canvas (no external libs)
- **Themes** — Dark / Light / Aero (signature glass style)
- **Custom accent color + blur intensity** settings
- **`docs/AERO.md`** — Aero Design Language (brandbook)

### Changed
- `web/index.html` — full dashboard redesign
- `web/css/style.css` — new design system with CSS variables
- `Cargo.toml` — added `clap`, `colored` for CLI

### Fixed
- Time Travel slider correctly restores any previous snapshot

## [2.1.0] - 2026-09-28 — "Nebula"

### Added
- Module 1: User-mode networking (`net_smoltcp.rs`)
- Module 2: Zero-page detection, QCOW2 export, RAM-only mode
- AeroAgent guest daemon (basic)
- Criterion benchmarks
- `.gitattributes`

## [2.0.0] - 2026-09-28 — "Fusion"

### Added
- `smoltcp` and `rappct` dependencies
- Zero-page detection
- RAM-only mode (`ram_only` config flag)

## [1.9.0] - 2026-09-28

### Added
- Professional README (EN + RU) with badges
- ARCHITECTURE, ROADMAP, BENCHMARKS docs
- CONTRIBUTING, SECURITY, CODE_OF_CONDUCT
- `rust-toolchain.toml`, `.editorconfig`
- Optimized `Cargo.toml`

## [1.8.0] - 2026-09-28

### Added
- wintun.dll auto-download
- Snapshot tests

## [1.0.0] - Initial

### Added
- WHPX hypervisor, VirtIO, ConPTY, Snapshots, UI