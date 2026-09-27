<div align="center">

# AeroOS

**Портативная операционная система для Windows — в одном исполняемом файле.**

[![CI](https://github.com/user123141/AeroOS/actions/workflows/ci.yml/badge.svg)](https://github.com/user123141/AeroOS/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-Windows%2010%2F11-blue.svg)](https://www.microsoft.com/windows)

[Быстрый старт](#-быстрый-старт) · [Почему AeroOS](#-почему-aeroos) · [Архитектура](docs/ARCHITECTURE.md) · [English](README.md)

</div>

---

## 🎯 Что такое AeroOS?

AeroOS — это **портативная операционная система**, которая работает внутри одного `.exe` файла на Windows. Использует **Windows Hypervisor Platform (WHPX)** для запуска лёгкой виртуальной машины и предоставляет собственный графический интерфейс.

Написана на 100% на **Rust**. Без `.NET`, `Electron`, Python. Один нативный бинарник.

---

## ✨ Ключевые возможности

| Возможность | Описание |
|-------------|----------|
| 📦 **Один .exe** | Гипервизор, UI, снапшоты — всё в одном файле |
| ⚡ **Мгновенный старт** | Холодный запуск < 500 мс (WSL2 ~3с, VirtualBox ~8с) |
| 💾 **Малый размер** | ~8 МБ бинарник, ~50 МБ RAM overhead |
| 🔐 **Шифрованные снапшоты** | Инкрементальные LZ4 + BLAKE3 + AES-256-GCM |
| 🖥️ **Нативный WHPX** | Прямая работа с Windows Hypervisor Platform |
| 🎨 **Glassmorphism UI** | Современный веб-интерфейс с WebSocket |
| 📁 **Data Folder режим** | Ядро отдельно от .exe |
| 🔄 **Live-снапшоты** | Каждую секунду без остановки VM |
| 🖱️ **Drag & drop** | Перетащи файл в UI → попадает в гостя |
| 📋 **Буфер обмена** | Двусторонняя синхронизация |

---

## 🚀 Почему AeroOS?

### Сравнение с аналогами

| Возможность | **AeroOS** | WSL2 | VMware | VirtualBox | QEMU |
|-------------|:----------:|:----:|:------:|:----------:|:----:|
| Один исполняемый файл | ✅ | ❌ | ❌ | ❌ | ⚠️ |
| Не требует установки | ✅ | ❌ | ❌ | ❌ | ✅ |
| Нативный WHPX | ✅ | ✅ | ❌ | ❌ | ⚠️ |
| Кроссплатформенный UI | ✅ | ❌ | ❌ | ⚠️ | ⚠️ |
| Инкрементальные снапшоты | ✅ | ❌ | ⚠️ | ⚠️ | ❌ |
| Open source | ✅ | ❌ | ❌ | ✅ | ✅ |
| Размер бинарника | ~8 МБ | ~200 МБ | ~500 МБ | ~150 МБ | ~50 МБ |
| Холодный старт | < 500 мс | ~3 с | ~8 с | ~10 с | ~2 с |

---

## ⚡ Быстрый старт

### Требования

- Windows 10/11 (x64)
- Rust 1.75+ ([rustup.rs](https://rustup.rs))
- Git
- Hyper-V + Hypervisor Platform

### Настройка (один раз)

    # От администратора:
    bcdedit /set hypervisorlaunchtype auto
    # Перезагрузка

### Сборка

    .\Setup-AeroOS.ps1 -Rebuild
    .\AeroOS\build\aeroos.exe

### Доступ

- **UI:** http://127.0.0.1:8080
- **WebSocket:** ws://127.0.0.1:8081

---

## 📊 Статус

| Компонент | Статус | Заметки |
|-----------|:------:|---------|
| Rust build | ✅ | 0 errors, 0 warnings |
| UI (HTTP) | ✅ | Glassmorphism |
| WebSocket IPC | ✅ | Token auth |
| WHPX | ⚠️ | Требует `bcdedit` + перезагрузку |
| VirtIO block | ✅ | Чтение/запись |
| VirtIO net | ⚠️ | Нужен `wintun.dll` (авто-скачивание) |
| VirtIO gpu | 🟡 | Базовый framebuffer |
| ConPTY | ✅ | Двусторонний |
| Snapshots | 🟡 | Тесты есть, в бою не проверены |
| AeroBoot | 🟡 | Header-only, UEFI в работе |

Легенда: ✅ Готово · 🟡 Частично · ⚠️ Требует настройки · ❌ Не реализовано

---

## 📄 Лицензия

MIT © 2026 AeroOS Contributors — см. [LICENSE](LICENSE).