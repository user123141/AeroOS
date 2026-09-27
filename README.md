# AeroOS

Портативная операционная система для Windows, упакованная в один `.exe`.

## Стек

- **Язык**: Rust (100% native)
- **Гипервизор**: Windows Hypervisor Platform (WHPX)
- **VirtIO**: block / net (wintun) / console / gpu
- **UI**: HTML/CSS/JS, glassmorphism
- **Терминал**: ConPTY
- **Снапшоты**: LZ4 + BLAKE3 + AES-256-GCM
- **Bootloader**: AeroBoot (UEFI + BIOS)

## Статус

| Компонент | Статус |
|-----------|--------|
| Rust build | OK |
| UI (HTTP) | OK |
| WebSocket IPC | OK |
| WHPX | требует настройки (см. ниже) |
| VirtIO block | OK |
| VirtIO net | нужен wintun.dll |
| VirtIO gpu | базовый framebuffer |
| ConPTY | OK |
| Snapshots | код готов, тестов нет |
| AeroBoot UEFI | header-only |

## Требования

- Windows 10/11 x64
- Rust 1.75+
- Hyper-V + Hypervisor Platform

## Включение VM

Один раз от администратора:

    bcdedit /set hypervisorlaunchtype auto

Затем перезагрузить. Без этого UI всё равно работает, VM пропускается.

## Сборка

    .\Setup-AeroOS.ps1

## Лицензия

MIT