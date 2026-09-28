<div align="center">

# AeroOS

**Портативная операционная система — в одном `.exe` файле.**

[![CI](https://github.com/user123141/AeroOS/actions/workflows/ci.yml/badge.svg)](https://github.com/user123141/AeroOS/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-Windows%2010%2F11-blue.svg)](https://www.microsoft.com/windows)
[![Version](https://img.shields.io/badge/version-5.0.0-blueviolet.svg)](#)

[Быстрый старт](#-быстрый-старт) · [Возможности](#-возможности) · [Архитектура](#-архитектура) · [API](#-api) · [Roadmap](#-roadmap)

</div>

---

## О проекте

AeroOS — это **портативная ОС**, полностью работающая внутри одного `.exe`
на Windows. Использует нативный **Windows Hypervisor Platform (WHPX)** для
запуска лёгкой виртуальной машины и предоставляет собственный графический
интерфейс в стиле macOS.

Написана **на 100% на Rust**. Без `.NET`, `Electron`, `Python`.
Один нативный бинарник, размер ~8 МБ.

---

## Быстрый старт

### Требования

- Windows 10/11 (x64)
- Rust 1.75+ ([rustup.rs](https://rustup.rs))
- Git
- Hyper-V + Hypervisor Platform

### Однократная настройка

От имени администратора:

    bcdedit /set hypervisorlaunchtype auto
    Restart-Computer

### Сборка и запуск

    cd C:\path\to\AeroOS
    .\scripts\build-all.ps1
    cd build
    .\aeroos.exe

### Доступ

- **UI**: http://127.0.0.1:8080
- **WebSocket**: ws://127.0.0.1:8081
- **CLI**: `aeroctl --help`

### Горячие клавиши

| Клавиши | Действие |
|---------|----------|
| Ctrl+K | Spotlight Search |
| Ctrl+Alt+Up | Volume up |
| Ctrl+Alt+Down | Volume down |
| Ctrl+Alt+Right | Brightness up |
| Ctrl+Alt+Left | Brightness down |
| Esc | Закрыть поиск / календарь |

---

## Возможности

### Интерфейс

| Фича | Статус |
|------|:------:|
| Topbar (macOS-style) | готово |
| Spotlight Search (Ctrl+K) | готово |
| Glassmorphism | готово |
| Draggable windows | готово |
| Dark / Light / Aero темы | готово |
| Toast-уведомления (iOS-style) | готово |
| OSD при volume/brightness | готово |
| Custom scrollbars | готово |
| i18n (RU / EN) | готово |
| Real brightness / volume | готово (ноутбуки) |
| Calendar popover с событиями | готово |
| Global hotkeys | готово |

### Система

| Фича | Статус |
|------|:------:|
| WHPX hypervisor | требует `bcdedit` + reboot |
| VirtIO (block/net/gpu/fs) | код готов |
| AeroBoot (BIOS + UEFI) | header-only |
| Incremental snapshots | готово |
| AES-256-GCM шифрование | готово |
| Live migration (.aeromig) | готово |
| AeroRegistry | готово |
| OCI pull -> initramfs | pull готов |
| GPU passthrough (Hyper-V GPU-P) | детект готов |
| Multi-VM manager | готово |
| License (Ed25519 + HWID) | готово |
| Crypto-UI binding | готово |
| Global hotkeys | готово |

---

## Сравнение с аналогами

| Возможность | **AeroOS** | WSL2 | VMware | VirtualBox | QEMU |
|-------------|:----------:|:----:|:------:|:----------:|:----:|
| Один исполняемый файл | да | нет | нет | нет | частично |
| Не требует установки | да | нет | нет | нет | да |
| Нативный WHPX | да | да | нет | нет | частично |
| Свой UI | да | нет | нет | частично | частично |
| Инкрементальные снапшоты | да | нет | частично | частично | нет |
| Шифрование снапшотов | да | нет | нет | нет | нет |
| Crypto-UI binding | да | нет | нет | нет | нет |
| Open source | да | нет | нет | да | да |
| Размер бинарника | ~8 МБ | ~200 МБ | ~500 МБ | ~150 МБ | ~50 МБ |
| Холодный старт | < 500 мс | ~3 с | ~8 с | ~10 с | ~2 с |

---

## Архитектура

### Схема слоёв

    +-------------------------------------------------------------+
    |                     UI (HTML/CSS/JS)                        |
    |  Topbar · Spotlight · OSD · Calendar · Notifications        |
    +--------------------------+----------------------------------+
                               | HTTP + WebSocket (localhost)
    +--------------------------v----------------------------------+
    |                  Rust Runtime (tokio)                       |
    |  web.rs · ipc.rs · main.rs · syscontrol.rs · hotkeys.rs     |
    +-------------------------------------------------------------+
    |                     VM (WHPX)                               |
    |  hypervisor.rs · boot_vm.rs · virtio.rs · snapshot.rs       |
    +--------------------------+----------------------------------+
                               | FFI (LoadLibraryA)
    +--------------------------v----------------------------------+
    |           Windows Hypervisor Platform (WHPX)                |
    |           WinHvPlatform.dll + wintun.dll + winmm.dll         |
    +-------------------------------------------------------------+

### Модули (30 штук)

| Модуль | Ответственность |
|--------|-----------------|
| `main.rs` | Entry, tokio init |
| `config.rs` | TOML config |
| `hypervisor.rs` | WHPX wrapper |
| `boot_vm.rs` | bzImage loader (Linux boot protocol) |
| `boot.rs` | AeroBoot (custom loader) |
| `virtio.rs` | VirtIO MMIO (block/net/console/gpu/fs) |
| `virtio_fs.rs` | 9P async |
| `virtio_9p.rs` | 9P sync |
| `snapshot.rs` | Incremental snapshots (LZ4+BLAKE3) |
| `crypto.rs` | AES-256-GCM |
| `security.rs` | SHA-256, session tokens |
| `license.rs` | Ed25519 + HWID + nonce |
| `hwid.rs` | Hardware fingerprint |
| `ipc.rs` | WebSocket |
| `web.rs` | HTTP server |
| `syscontrol.rs` | Volume / brightness (Windows API) |
| `hotkeys.rs` | Global hotkeys (RegisterHotKey) |
| `net_smoltcp.rs` | User-mode TCP/IP |
| `net_proxy.rs` | TCP proxy guest -> host |
| `tap.rs` | wintun TAP |
| `vnc.rs` | Frame streaming (JPEG) |
| `migration.rs` | Live migration (.aeromig) |
| `oci.rs` | OCI runtime |
| `registry.rs` | AeroRegistry client |
| `multi_vm.rs` | Multi-VM manager |
| `sandbox.rs` | AppContainer (winapi) |
| `gpu_passthrough.rs` | Hyper-V GPU-P |
| `aerofile.rs` | Aerofile parser |
| `agent.rs` | AeroAgent protocol |
| `terminal.rs` | ConPTY |
| `data_folder.rs` | Data folder mode |

### Boot sequence

1. `[AeroOS] v5.0.0 starting`
2. `Sandbox token acquired`
3. `Tier: Community, max_vms=1, max_ram=2048 MB`
4. `HWID: 4959f73c...`
5. `Config: 4 CPU, 8192 MB RAM`
6. `AeroBoot: kernel loaded (10863616 bytes)`
7. `AeroBoot: initramfs loaded (8870371 bytes)`
8. `AeroBoot: header OK, checksum=0x99B5C7C7`
9. `WHPX: dirty_pages=true`
10. **ERROR: Windows Hypervisor NOT running** (требуется reboot)
11. `UI: http://127.0.0.1:8080`
12. `VM skipped (hypervisor off). UI is alive.`

### Memory layout (guest)

    0x0000_0000 - 0x000F_FFFF  Low memory / BIOS stub
    0x0000_7000                boot_params (zero page)
    0x0002_0000                cmdline
    0x0009_0000                temporary stack
    0x0010_0000 - kernel_end   Kernel code
    kernel_end  - +initramfs   Initramfs

### Ключевые константы

| Константа | Значение | Назначение |
|-----------|----------|------------|
| `KERNEL_START` | 0x100000 | Точка загрузки ядра (1 MB) |
| `SETUP_START` | 0x10000 | Setup code (64 KB) |
| `INITRAMFS_ADDR` | 0x4000000 | Initramfs (64 MB) |
| `BOOT_PARAMS_ADDR` | 0x7000 | Zero page |
| `CMDLINE_ADDR` | 0x20000 | Командная строка |
| `STACK_ADDR` | 0x90000 | Временный стек |
| `BOOT_MAGIC` | 0x53726448 | "HdrS" (bzImage) |
| `AERO_BOOT_MAGIC` | 0x4145524F | "AERO" (AeroBoot) |

---

## API

### IPC Commands (WebSocket, port 8081)

JSON с тегом `"type"`. Query string: `?token=<session>`.

| Command | Payload | Response |
|---------|---------|----------|
| `GetStatus` | — | `Status { running, cpu_cores, ram_mb, gpu }` |
| `GetStats` | — | `Stats { snapshots, dirty_pages }` |
| `ListVms` | — | `VmsList { vms: [VmStatsInfo] }` |
| `GetSerial` | — | `Serial { data }` |
| `ClearSerial` | — | `Ok` |
| `Snapshot` | `{ name }` | `Ok` |
| `Restore` | `{ name }` | `Ok` |
| `ListSnapshots` | — | `Snapshots { list }` |
| `DeleteSnapshot` | `{ name }` | `Ok` |
| `SetVolume` | `{ level: 0..100 }` | `Ok / Error` |
| `SetBrightness` | `{ level: 0..100 }` | `Ok / Error` |
| `GetSystemState` | — | `SystemState { volume, brightness, dnd }` |
| `Search` | `{ query }` | `SearchResults { items: [SearchItem] }` |
| `Stop` | — | `Ok` |
| `TerminalInput` | `{ data }` | `TerminalOutput { data }` |

### HTTP endpoints

| Path | Method | Response |
|------|--------|----------|
| `/` | GET | index.html |
| `/api/token` | GET | `{ token }` |
| `/api/ui-key` | GET | `{ key }` (только с лицензией) |

### Rust API (основное)

**hypervisor.rs** — управление VM:

    pub fn new(config: &AeroConfig, df: &DataFolder) -> Result<VirtualMachine>
    pub async fn run(&mut self) -> Result<()>
    pub fn stop(&self)
    pub fn get_state(&self) -> Result<Vec<u8>>
    pub fn restore_state(&mut self, data: &[u8]) -> Result<()>
    pub fn get_dirty_pages(&self) -> Vec<u64>

**snapshot.rs** — снапшоты:

    pub fn new(config: &SnapshotConfig) -> SnapshotManager
    pub fn create_snapshot(&mut self, data: &[u8], name: &str) -> Result<PathBuf>
    pub fn restore_snapshot(&self, name: &str) -> Result<Vec<u8>>
    pub fn list_snapshots(&self) -> Vec<String>

**syscontrol.rs** — управление системой:

    pub fn set_volume(level: u32) -> Result<()>
    pub fn get_volume() -> Result<u32>
    pub fn set_brightness(level: u32) -> Result<()>
    pub fn get_brightness() -> Result<u32>

**license.rs** — лицензия:

    pub fn load(path: &Path) -> Result<LicenseState>
    pub fn verify(&self) -> Result<()>
    pub fn derive_ui_key(license: &License) -> [u8; 32]

**hotkeys.rs** — горячие клавиши:

    pub fn register() -> Result<Receiver<HotkeyEvent>>

---

## Безопасность

| Угроза | Митигация |
|--------|-----------|
| Guest escape | WHPX hardware isolation |
| Snapshot tampering | BLAKE3 hash verification |
| License forgery | Ed25519 signature |
| HWID bypass | SHA-256 fingerprint |
| WebSocket hijack | Session token + localhost bind |
| XSS | CSP headers, no eval |
| Path traversal (9P) | canonicalize + root check |
| UI tampering | Crypto-UI binding (AES-256-GCM) |
| Code reverse engineering | XOR-obfuscation (JS assets) |

### Crypto-UI binding

Критические JS-файлы шифруются AES-256-GCM. Ключ производится из
подписи Ed25519-лицензии (`derive_ui_key`). Без валидной лицензии UI
работает в open mode.

Включить:

    $env:AEROOS_UI_KEY = "<64 hex chars>"
    $env:AEROOS_OBFUSCATE = "1"
    cargo build --release

---

## Конфигурация

Файл `config/aeroos.config.toml`:

| Секция | Параметр | Default | Описание |
|--------|----------|---------|----------|
| `cpu` | `cores` | 4 | Ядра CPU (1..64) |
| `cpu` | `model` | "host" | Модель CPU |
| `memory` | `ram_mb` | 8192 | RAM в МБ |
| `gpu` | `model` | "virtio-gpu" | Тип GPU |
| `gpu` | `accel_3d` | true | 3D acceleration |
| `disk` | `interface` | "virtio" | Интерфейс диска |
| `snapshot` | `auto_interval_ms` | 1000 | Интервал стриминга |
| `snapshot` | `max_size_mb` | 10240 | Лимит размера |
| `snapshot` | `encrypt` | false | Шифрование |
| `snapshot` | `ram_only` | false | Эфемерный режим |
| `mode` | — | "embedded" | embedded / data_folder |

---

## Aerofile

Декларативный конфиг VM (аналог Dockerfile):

    [vm]
    name = "dev"
    image = "alpine-3.20"
    cores = 4
    ram = 4096

    [network]
    mode = "user"

    [provision]
    run = ["apk add --no-cache curl"]

    [[share]]
    host = "C:\\Projects"
    guest = "/mnt/projects"

Запуск:

    aeroctl init myvm
    aeroctl up --file Aerofile

---

## CLI (aeroctl)

| Команда | Действие |
|---------|----------|
| `aeroctl init <name>` | Создать Aerofile |
| `aeroctl up --file <file>` | Запустить VM |
| `aeroctl down` | Остановить |
| `aeroctl status` | Статус |
| `aeroctl hwid` | Показать HWID |
| `aeroctl migrate save/load` | Миграция |
| `aeroctl oci pull/list/to-initramfs` | OCI образы |

---

## Дизайн-язык Aero

### Принципы

1. **Weightless** — элементы парят в пространстве.
2. **Translucent** — каждый слой показывает нижний.
3. **Alive** — плавное движение, никогда не статично.
4. **Precise** — каждый пиксель с причиной.

### Цвета

| Token | Dark | Light | Aero |
|-------|------|-------|------|
| `--accent` | #4a9eff | #4a9eff | #00d4ff |
| `--bg-1` | #0a0a12 | #eef2f7 | #051220 |
| `--bg-2` | #12121e | #e4e9f0 | #0a1e30 |
| `--text` | #ffffff | #1a1a2e | #e6f5ff |

### Motion

- **Easing**: `cubic-bezier(.34, 1.56, .64, 1)` — overshoot
- **Durations**: 150ms micro, 250ms transitions, 400ms theme
- **Reduced motion**: `prefers-reduced-motion` отключает blur

---

## Оптимизации

| Параметр | Значение | Зачем |
|----------|----------|-------|
| `opt-level` | 2 | скорость без агрессивности |
| `lto` | "thin" | компромисс RAM / скорость |
| `codegen-units` | 16 | параллельная кодогенерация |
| `panic` | "abort" | меньший размер |
| `strip` | "symbols" | меньше бинарник |
| `jobs` | 4 | защита от LNK1102 OOM |

---

## Roadmap

### v5.0.0 — Aegis (текущая)

- XOR-obfuscation JS assets
- Crypto-UI binding (AES-256-GCM)
- Global hotkeys (RegisterHotKey)
- Calendar events
- Исправление `aes_gcm::Error` в build.rs

### v5.1.0 — Continuum

- Real AeroBoot UEFI (.efi target)
- End-to-end VM boot (Alpine kernel)
- 9P read/write end-to-end
- OCI pull -> initramfs

### v6.0.0 — Singularity

- Linux/macOS host (KVM, HVF)
- WebRTC VP8/H.264
- Plugin system
- Time-travel diff viewer
- Encrypted disk images

### Backlog

- Sound subsystem для гостя
- Multi-monitor GPU
- USB passthrough
- Guest agent file sync

---

## Требования

- Windows 10/11 x64
- Rust 1.75+
- Hyper-V + Hypervisor Platform
- `bcdedit /set hypervisorlaunchtype auto` + reboot

---

## Документация

Дополнительные документы в `docs/`:

| Документ | Описание |
|----------|----------|
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Внутреннее устройство (расширенная версия) |
| [docs/API.md](docs/API.md) | Полный Rust API всех модулей |
| [docs/AERO.md](docs/AERO.md) | Дизайн-язык Aero |
| [docs/AEROFILE.md](docs/AEROFILE.md) | Формат Aerofile |
| [docs/ROADMAP.md](docs/ROADMAP.md) | Детальный план |
| [docs/BENCHMARKS.md](docs/BENCHMARKS.md) | Замеры производительности |
| [docs/SESSION.md](docs/SESSION.md) | Рабочий контекст |
| [CHANGELOG.md](CHANGELOG.md) | История версий |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Как помочь |
| [SECURITY.md](SECURITY.md) | Безопасность |

---

## Лицензия

MIT (c) 2026 AeroOS Contributors