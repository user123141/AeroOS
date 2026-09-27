# Roadmap

## Current: v1.9.0 (2026-09)

- [x] Single .exe
- [x] WHPX integration
- [x] VirtIO block / net / console / gpu
- [x] ConPTY terminal
- [x] Incremental encrypted snapshots
- [x] Glassmorphism UI
- [x] Clipboard sync
- [x] wintun.dll auto-download

## v2.0.0 — "Fusion" (2026-Q4)

- [ ] **Full Linux boot protocol** — launch Alpine/Debian kernels
- [ ] **UEFI AeroBoot** — replace BIOS stub with real `.efi`
- [ ] **VirtIO-FS** — shared folders host↔guest
- [ ] **Multi-VM** — manage several VMs from one UI
- [ ] **CLI `aeroctl`** — full command set (start/stop/snapshot/exec)
- [ ] **Benchmarks** — cold start, RAM overhead, IOPS

## v2.1.0 — "Nebula" (2027-Q1)

- [ ] **User-mode NAT** — networking without admin rights
- [ ] **gfxstream / VirGL** — 3D acceleration via WebGPU
- [ ] **Dynamic resolution** — auto-resize guest display
- [ ] **Themes** — light/dark, custom accent colors
- [ ] **Real-time charts** — CPU/RAM/disk/network

## v3.0.0 — "Horizon" (2027-Q2)

- [ ] **OCI compatibility** — run Docker images as microVMs
- [ ] **Linux host support** — KVM backend
- [ ] **macOS host** — HVF backend
- [ ] **ARM64** — Windows on ARM

## Backlog

- VM snapshot diff viewer
- Performance profiler (guest ↔ host)
- Plugin system for UI extensions
- Encrypted disk images
- Guest agent for file operations
- Time-travel debugging (replay from any snapshot)