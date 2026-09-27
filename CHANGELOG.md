# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.9.0] - 2026-09-28

### Added
- **Professional documentation**: README (EN + RU), ARCHITECTURE, ROADMAP, BENCHMARKS
- **Community files**: CONTRIBUTING, SECURITY, CODE_OF_CONDUCT
- **Badges** in README (CI, license, Rust, platform)
- **Comparison tables** vs WSL2, VMware, VirtualBox, QEMU
- `rust-toolchain.toml`, `.editorconfig`
- Optimized `Cargo.toml` with minimal feature sets

### Changed
- **Performance profile**: `opt-level = 3` (was `"z"`), LTO fat, no incremental
- **Tokio features**: minimal (was `full`)
- **Blake3**: enabled `rayon` + `mmap` for parallelism

### Fixed
- Removed all dead_code warnings in vendored crates

## [1.8.0] - 2026-09-28

### Added
- Auto-download `wintun.dll` in setup script
- Unit tests for snapshot integrity + deduplication

## [1.7.0] - 2026-09-28

### Added
- Detailed README with structure and quick start
- Git Credential Manager configuration
- Release profile optimizations (LTO, codegen-units=1)

### Fixed
- Warnings in vendor/whpx and vendor/conpty
- Duplicate documentation

## [1.6.0] - 2026-09-27

### Fixed
- `aeroos.exe` no longer crashes with code 1 on `0xC0351000`
- Helpful WHPX error messages with `bcdedit` instructions

### Added
- VM is optional
- LICENSE, CHANGELOG, GitHub Actions CI

## [1.0.0] - Initial

### Added
- WHPX hypervisor
- VirtIO devices
- ConPTY terminal
- Incremental snapshots
- Glassmorphism UI