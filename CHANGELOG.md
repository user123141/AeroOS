# Changelog

## [3.0.0] - 2026-09-28 — "Singularity"

### Fixed
- **КРИТИЧНО:** `src/net_smoltcp.rs` — API `smoltcp 0.11`:
  - `Device` без лайфтайма, `type RxToken<'a>`, `type TxToken<'a>`
  - `receive(&mut self, Instant)`, `transmit(&mut self, Instant)`
- **КРИТИЧНО:** `src/config.rs` — добавлено поле `ram_only` в initializer
- `src/bin/aeroctl.rs` — убран unused import `Context`

### Added
- `.cargo/config.toml` — `jobs = 4` (защита от LNK1102 / OOM)
- Профессиональный `.gitignore` с 8 категориями
- README v3.0.0

### Changed
- `Cargo.toml` — версия 3.0.0, стабильный профиль release
- Удалены `registry.rs`, `gpu_stream.rs` (вернутся в v3.1 с полной реализацией)

## [2.9.0] - Vertex
## [2.8.0] - Aurora
## [2.7.0] - Forge
## [2.0.0] - Fusion
## [1.0.0] - Initial