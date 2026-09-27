# AeroOS

Портативная операционная система для Windows, упакованная в один `.exe`.

## Что это

AeroOS запускает виртуальную машину через **Windows Hypervisor Platform (WHPX)** и предоставляет веб-интерфейс в стиле glassmorphism. Проект написан полностью на Rust без .NET, Electron и тяжёлых фреймворков.

- **Язык**: Rust (100% native)
- **Гипервизор**: WHPX (Hyper-V)
- **VirtIO**: block / net (wintun) / console / gpu
- **UI**: HTML/CSS/JS, glassmorphism
- **Терминал**: ConPTY
- **Снапшоты**: инкрементальные, LZ4 + BLAKE3 + AES-256-GCM
- **Bootloader**: AeroBoot (UEFI + BIOS)

## Статус

| Компонент | Статус |
|-----------|--------|
| Rust build | OK |
| UI (HTTP) | OK |
| WebSocket IPC | OK |
| WHPX | требует настройки (см. ниже) |
| VirtIO block | OK |
| VirtIO net (wintun) | нужен wintun.dll |
| VirtIO gpu | базовый framebuffer |
| ConPTY terminal | OK |
| Snapshots | код готов, тестов нет |
| AeroBoot UEFI | header-only |

## Быстрый старт

1. Установи Rust, Git и MSVC Build Tools (если их нет).
2. Включи Hyper-V и Windows Hypervisor Platform.
3. Настрой гипервизор (один раз, от администратора):

       bcdedit /set hypervisorlaunchtype auto

4. Перезагрузи компьютер.
5. Собери и запусти:

       .\Setup-AeroOS.ps1 -Rebuild
       .\AeroOS\build\aeroos.exe

6. UI доступен по адресу: http://127.0.0.1:8080

## Требования

- Windows 10/11 x64
- Rust 1.75+
- Hyper-V + Hypervisor Platform
- Для VM: `bcdedit /set hypervisorlaunchtype auto` + перезагрузка

## Сборка

Полная сборка:

    .\Setup-AeroOS.ps1 -Rebuild

Только Rust:

    cd AeroOS
    cargo build --release

## Структура проекта

    AeroOS/
    ├── src/                # Исходный код Rust
    │   ├── main.rs         # Точка входа
    │   ├── config.rs       # Конфигурация (TOML)
    │   ├── hypervisor.rs   # WHPX-гипервизор
    │   ├── virtio.rs       # VirtIO-устройства
    │   ├── snapshot.rs     # Инкрементальные снапшоты
    │   ├── web.rs          # HTTP-сервер для UI
    │   ├── ipc.rs          # WebSocket-сервер
    │   ├── security.rs     # SHA-256, токены
    │   ├── crypto.rs       # AES-256-GCM
    │   ├── boot.rs         # AeroBoot
    │   ├── terminal.rs     # ConPTY
    │   └── tap.rs          # wintun
    ├── vendor/
    │   ├── whpx/           # WHPX биндинги
    │   └── conpty/         # ConPTY биндинги
    ├── web/                # UI
    │   ├── index.html
    │   ├── css/style.css
    │   └── js/
    ├── kernel/             # Конфиг ядра
    ├── config/             # Конфиг AeroOS
    ├── scripts/            # Скрипты сборки
    └── build/              # Результат

## Конфигурация

Основные параметры в `aeroos.config.toml`:

| Параметр | Описание | По умолчанию |
|----------|----------|--------------|
| cpu.cores | Ядер CPU | 4 |
| memory.ram_mb | RAM (МБ) | 8192 |
| gpu.model | GPU | virtio-gpu |
| snapshot.auto_interval_ms | Интервал снапшотов | 1000 |
| snapshot.encrypt | Шифрование | false |
| mode | Режим | embedded |

## Лицензия

MIT