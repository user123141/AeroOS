# AeroOS v4.2.0 "Interface"

Портативная операционная система для Windows, упакованная в один `.exe`.

## Статус

| Компонент | Статус |
|-----------|--------|
| Rust build | OK |
| `.exe` запуск | OK |
| UI + WebSocket | OK |
| Sandbox (winapi) | OK |
| License (Ed25519) | OK |
| HWID | OK |
| Registry | OK |
| GPU detection | OK |
| **WHPX partition** | **нужна перезагрузка** |
| Boot Alpine kernel | ждёт WHPX |
| Multi-VM UI | OK |
| i18n (RU/EN) | OK |
| Draggable windows | OK |

## Архитектура

    UI (HTML/CSS/JS + i18n)
      ↓ HTTP + WebSocket
    Rust Runtime (tokio)
      ↓
    ┌── Hypervisor (WHPX) ──── Boot VM ─── VirtIO ──┐
    │  Snapshot (LZ4+BLAKE3)  Crypto (AES-GCM)     │
    │  License (Ed25519)      HWID                  │
    │  Network (smoltcp)      Migration             │
    │  Registry (reqwest)     OCI pull              │
    └───────────────────────────────────────────────┘

## Файлы документации

| Файл | Описание |
|------|----------|
| `docs/README.md` | Этот файл |
| `docs/API.md` | Полный список публичных API всех модулей |
| `docs/ARCHITECTURE.md` | Архитектура и потоки данных |
| `docs/AERO.md` | Дизайн-язык Aero (брендбук) |
| `docs/AEROFILE.md` | Формат Aerofile (декларативный конфиг VM) |
| `docs/ROADMAP.md` | План развития |
| `docs/BENCHMARKS.md` | Замеры производительности |
| `docs/SESSION.md` | Рабочая память (последняя итерация) |

## Быстрый старт

    # От админа (один раз):
    bcdedit /set hypervisorlaunchtype auto
    Restart-Computer

    # После перезагрузки:
    cd AeroOS\build
    .\aeroos.exe

UI: http://127.0.0.1:8080

## Требования

- Windows 10/11 x64
- Rust 1.75+
- Hyper-V + Hypervisor Platform

## Лицензия

MIT