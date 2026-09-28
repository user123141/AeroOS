# AeroOS Development Session Context

> Рабочая память. Обновляется каждую итерацию.

## Текущая версия

- **Cargo.toml**: 4.1.8
- **Стадия**: **boot-testing** — `.exe` собирается и запускается
- **Известный блокер**: hypervisor выключен (нет перезагрузки)

## Что работает ✅

- [x] Rust build (без ошибок)
- [x] Tests: 6 из 7 (test_parse_manifest падал на media_type)
- [x] `.exe` собирается
- [x] `.exe` **запускается** и создаёт HTTP сервер
- [x] UI доступен на http://127.0.0.1:8080
- [x] License system (Community mode)
- [x] HWID: 4959f73c39dcd83df28371eca84a3895
- [x] Registry, GPU detection
- [x] Sandbox (basic)
- [x] Конфиг читается корректно (4 CPU, 8192 MB RAM)

## Что блокировано ⚠️

- [ ] **WHPX partition** — hypervisor не запущен
  - Причина: `bcdedit /set hypervisorlaunchtype auto` не выполнен
  - Решение: одна перезагрузка + команда от админа

## Что требует работы 🔧

- [ ] End-to-end boot Alpine kernel (после включения hypervisor)
- [ ] Serial output COM1 в UI
- [ ] E820 memory map для ядра
- [ ] ACPI таблицы
- [ ] 9P read/write end-to-end

## Структура src/ (актуально)

| Модуль | Строк | Ответственность |
|--------|-------|-----------------|
| main.rs | ~130 | Entry, tokio init |
| hypervisor.rs | ~200 | WHPX VM |
| boot_vm.rs | ~200 | bzImage loader |
| virtio.rs | ~400 | VirtIO MMIO |
| snapshot.rs | ~250 | Incremental snapshots |
| ipc.rs | ~230 | WebSocket |
| web.rs | ~80 | HTTP + UI embed |
| license.rs | ~150 | Ed25519 |
| hwid.rs | ~50 | Fingerprint |
| config.rs | ~120 | TOML |
| crypto.rs | ~80 | AES-GCM |
| security.rs | ~60 | SHA-256, tokens |
| net_smoltcp.rs | ~180 | User-mode TCP |
| net_proxy.rs | ~90 | TCP proxy |
| vnc.rs | ~70 | JPEG streaming |
| migration.rs | ~110 | Live migration |
| oci.rs | ~200 | OCI pull |
| registry.rs | ~90 | AeroRegistry |
| multi_vm.rs | ~110 | Multi-VM |
| gpu_passthrough.rs | ~150 | Hyper-V GPU-P |
| sandbox.rs | ~60 | AppContainer |
| aerofile.rs | ~140 | Aerofile parser |
| agent.rs | ~110 | AeroAgent |
| boot.rs | ~100 | AeroBoot header |
| data_folder.rs | ~70 | Data folder mode |
| tap.rs | ~90 | wintun |
| terminal.rs | ~40 | ConPTY |
| virtio_9p.rs | ~200 | 9P sync |
| virtio_fs.rs | ~200 | 9P async |

## Ключевые константы

    KERNEL_START     = 0x100000    (1 MB)
    SETUP_START      = 0x10000     (64 KB)
    INITRAMFS_ADDR   = 0x4000000   (64 MB)
    BOOT_PARAMS_ADDR = 0x7000      (zero page)
    CMDLINE_ADDR     = 0x20000
    STACK_ADDR       = 0x90000
    BOOT_MAGIC       = 0x53726448  ("HdrS")

## Boot sequence (что смотреть)

1. `[AeroOS] v4.3.0 starting`
2. `Sandbox token acquired`
3. `Tier: Community`
4. `HWID: ...`
5. `Config: 4 CPU, 8192 MB RAM`
6. `WHPX: dirty_pages=true`
7. **Сейчас: ERROR "Windows Hypervisor is NOT running"** ← блокер
8. `UI: http://127.0.0.1:8080`
9. `VM skipped (hypervisor off). UI is alive.`

## Чтобы сдвинуться дальше

От админа:
    bcdedit /set hypervisorlaunchtype auto
    Restart-Computer

После перезагрузки:
    .\run_test.ps1

Ожидаем:
    WHPX OK
    bzImage: setup_sects=... magic OK
    Kernel code: ... bytes at 0x100000
    Initramfs: ... bytes at 0x4000000
    vCPU: RIP=0x100000, RSI=0x7000, ...

## Известные проблемы

1. **Кракозябры в логе** — русский текст в tracing! пишется в UTF-8, консоль читает CP866
   - Решение: перевести все сообщения на английский (сделано в v4.3.0)
2. **NativeCommandError** от cargo — PowerShell 5.1 ругается на stderr
   - Решение: `cmd /c "cargo ... 2>&1"`
3. **test_parse_manifest падал** — fix в v4.3.0 (media_type в JSON)

## Оптимизации

- `lto = "thin"`, `codegen-units = 16`, `opt-level = 2`, `panic = "abort"`
- `.cargo/config.toml` → `jobs = 4` (защита от LNK1102 OOM)

## Артефакты

- `kernel/output/aeroos-kernel` — bzImage (Alpine virt)
- `kernel/output/aeroos-initramfs` — cpio.gz
- `build/aeroos.exe` — итоговый бинарник
- `build/kernel/` — копия kernel+initramfs
- `build/wintun.dll` — TAP driver
- `build/aeroos.log` — лог последнего запуска