# Changelog

## [3.2.0] - 2026-09-28 — "Momentum"

### Fixed
- **КРИТИЧНО:** `SnapshotConfig: Default` — добавлен `impl Default` в `config.rs`
- `hypervisor.rs` — убран unnecessary `unsafe` блок

### Added
- **TCP Proxy** (`src/net_proxy.rs`) — реальный guest → host socket bridge
- **VNC/Frame streaming** (`src/vnc.rs`) — framebuffer → JPEG → WebSocket
- **Live migration** (`src/migration.rs`) — `.aeromig` формат + тесты
- **OCI runtime** (`src/oci.rs`) — разбор манифестов, тесты
- **GPU passthrough** (`src/gpu_passthrough.rs`) — детект GPU через WMI
- `image` crate — JPEG encoding
- CLI: `aeroctl migrate save/load`, `aeroctl oci pull/list`

### Changed
- `Cargo.toml` — версия 3.2.0, добавлен `image`
- `main.rs` — подключены все новые модули
- Объединены v3.1 "Continuum" и v3.2 "Horizon" в одну итерацию

## [3.0.0] - 2026-09-28 — "Singularity"
### Fixed
- smoltcp 0.11 Device trait (лaйфтаймы + Instant)
- config.rs ram_only, .cargo/config.toml (OOM)

## [2.9.0] - Vertex
## [2.8.0] - Aurora
## [2.7.0] - Forge
## [2.0.0] - Fusion
## [1.0.0] - Initial