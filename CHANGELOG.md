# Changelog



















## [5.0.1] - 2026-09-28 - hotfix

### Fixed
- **КРИТИЧНО:** `hotkeys.rs` — `RegisterHotKey` ожидает `u32` для mods и vk,
  а передавал `isize` (MOD_CONTROL | MOD_ALT) и `i32` (VK_UP). Добавлены
  явные касты `as UINT`.
- **КРИТИЧНО:** `hotkeys.rs` — `mods` не включал `MOD_NOREPEAT`, что
  приводило к spam-событиям при удержании клавиши.
- **КРИТИЧНО:** `main.rs` — regex-replace при добавлении hotkeys мог
  сработать криво. Файл перезаписан полностью.

### Added
- `hotkeys.rs`: счётчик зарегистрированных hotkeys (0..4), warning если <4
- `hotkeys.rs`: `UnregisterHotKey` при выходе из loop
- `hotkeys.rs`: `TranslateMessage` + `DispatchMessageW` в message loop
## [5.0.0] - 2026-09-28 - "Aegis"

### Fixed
- **КРИТИЧНО:** `build.rs` — `aes_gcm::Error` не реализует `std::error::Error`
  (opaque тип для защиты от side-channel). Заменено на `.map_err(|_| "...")`.

### Added
- **XOR-obfuscation** JS assets (`build.rs::obfuscate_web`):
  - При `AEROOS_OBFUSCATE=1` шифрует `terminal.js`, `search.js`,
    `notifications.js` через XOR (0xA7) в `.obf` файлы
- **Self-contained README**: единый документ с архитектурой, API,
  таблицами, дизайн-языком, roadmap — может заменить остальные

### Changed
- README содержит всё: quick start, возможности, сравнение, архитектуру,
  API, безопасность, конфигурацию, Aerofile, CLI, дизайн, оптимизации, roadmap

### Known issues
- Brightness через WMI — только на ноутбуках
- End-to-end VM boot — требует reboot с `bcdedit`
## [4.9.0] - 2026-09-28 - "Fortress"

### Fixed
- **КРИТИЧНО:** `main.rs` — `blocking_read()` вызывал panic в tokio runtime.
  Заменено на `lic.license.is_some()` до wrapping. Полная перезапись файла.
- Правило: **никаких regex-replace многострочного кода** — только полная
  перезапись через `Aero-W`.

### Added
- **Global hotkeys** (`src/hotkeys.rs`):
  - `Ctrl+Alt+Up` -> Volume up (+5)
  - `Ctrl+Alt+Down` -> Volume down (-5)
  - `Ctrl+Alt+Right` -> Brightness up (+5)
  - `Ctrl+Alt+Left` -> Brightness down (-5)
  - Через `RegisterHotKey` + message loop в отдельном потоке
- **Crypto-UI binding (реальный)**:
  - `build.rs` — при `AEROOS_UI_KEY` env шифрует `web/js/main.js`
    через AES-256-GCM в `web_encrypted/main.js.enc`
  - `web.rs` — `/api/ui-key` эндпоинт отдаёт ключ только с лицензией
  - `web/js/crypto.js` — fetch key → decrypt → eval (fallback на plain)
  - Если лицензии нет — UI работает в open mode
- **Calendar events**:
  - Клик по дате — добавление/редактирование события
  - Индикатор события (синяя точка под числом)
  - Сохранение в localStorage
- `Cargo.toml`: build-deps `aes-gcm`, `hex` для шифрования UI

### Changed
- `index.html` — вместо `<script src="js/main.js">` используется
  `<script src="js/crypto.js">`, который сам решает что грузить

### Known issues
- Crypto-UI — защита от casual-патча (опытный реверсер всё равно достанет).
- Brightness через WMI — работает только на ноутбуках.
## [4.7.0] - 2026-09-28 - "Crystal"

### Fixed
- **КРИТИЧНО:** PowerShell parse error `window.addEventListener('load', ...)` — незакрытая скобка. Проблема в `-match`. Заменено на прямые строки
- Light theme: белый текст больше не пропадает на белых элементах
- `select option` в light theme — правильный тёмный текст на белом фоне
- `popover` в light theme — теперь тёмный текст на светлом стекле

### Added
- **OSD (On-Screen Display)** для volume/brightness - Apple-style:
  - 160x160 скруглённый блок по центру
  - Иконка, меняющаяся по уровню (muted/low/mid/high)
  - Segmented bar с белым fill
  - Появляется на 1.5s при изменении слайдера
- **Calendar popover** при клике на часы в topbar
  - Пн..Вс заголовки
  - Текущий день выделен красным
  - Навигация по месяцам (в разработке)
- **Crypto-UI binding (начало)**:
  - `web.rs`: эндпоинт `/api/ui-key` — отдаёт ключ из `derive_ui_key(license)` только при валидной лицензии
  - `main.rs`: `UiKey = Arc<RwLock<Option<[u8;32]>>>` — устанавливается при старте
  - Если лицензии нет — `/api/ui-key` отдаёт 403 (фронт будет пропускать крипто-JS)
- **Notification v3** — Crystal:
  - Скругление 18px, glass-эффект 40px blur
  - SVG close-button (кружок)
  - Заголовки по типу (Информация/Готово/Внимание/Ошибка)
  - Gradient border сверху (subtle highlight)

### Changed
- OSD вызывается из `topbar.js` при `slider.oninput`
- `Notifications`: close-кнопка стала круглой SVG

### Known issues
- Crypto-UI binding: только каркас (key отдаётся, но `main.js` пока не зашифрован)
- Calendar: нет навигации по месяцам (только текущий)
- OSD не показывается при нажатии системных клавиш Windows (нужен глобальный хук — v4.8.0)
## [4.6.0] - 2026-09-28 - "Polish"

### Fixed
- **КРИТИЧНО:** `patch` файл: `AeroW_api = @'...'@` не работало — теперь используем `Aero-W`
- **КРИТИЧНО:** Часы из окна dashboard убраны (дублировались с topbar)
- Версия в UI: v4.3.0 → v4.6.0
- Search: добавлен локальный fallback (если WS не отвечает за 300ms)

### Added
- **macOS-polish.css**: реальная macOS-эстетика
  - Topbar: `app-region: drag` (перетаскивание окна), `saturate(180%)` blur
  - Custom dark scrollbars (macOS-like)
  - Sliders: белый thumb с shadow, hover/active анимация
  - Toasts: Big Sur style (`rgba(28,28,30,0.82)` + blur 40px)
  - Dock: z-index 9999 (не уходит под окна)
  - Window close: scale(0.92) + fade

### Changed
- Topbar height: 32px → 28px (компактнее)
- Topbar icons: 14px → 16px
- Notif badge: 6px → 7px + box-shadow

### Docs
- `docs/API.md`: полная перезапись (все модули)
- `docs/ROADMAP.md`: обновлён до v4.6.0
## [4.5.0] - 2026-09-28 - "Spotlight"

### Fixed
- **КРИТИЧНО:** `ipc.rs` — regex-replace сломал `match` структуру (arm без запятой). Полная перезапись.
- **КРИТИЧНО:** `web/index.html` — PowerShell не может склеить строку через `+` в `-replace`. Заменено на `.Replace()`.
- `snapshot::SnapshotManager.config` теперь `pub` (для DeleteSnapshot)

### Added
- **Spotlight Search** (Ctrl+K): поиск по приложениям, действиям, снапшотам
- **IPC команды**: `Search`, `DeleteSnapshot`, `GetSystemState`
- **Search hint**: клавиши навигации (↑↓, Enter, Esc)
- **Cargo.toml**: `overflow-checks = false` (явно)
- **Документация**: обновлены `docs/API.md`, `docs/ROADMAP.md`

### Changed
- Окна при открытии снова центрируются (drag-смещение сбрасывается)
- `main.js` теперь пробрасывает `SearchResults` через CustomEvent
## [4.4.0] - 2026-09-28 - "Cohesion"

### Added
- **Top menu bar (macOS style)**: app name, clock with day, notification bell, DND toggle, sound, brightness
- **Real system control** via `src/syscontrol.rs`:
  - Volume: `winmm.dll::waveOutSetVolume` (system default device)
  - Brightness: WMI `WmiSetBrightness` (laptops with internal displays)
- **Popovers** for sound and brightness with sliders
- **IPC commands**: `SetVolume`, `SetBrightness`, `GetSystemState`
- **DND mode** (Do Not Disturb): suppresses non-error toasts, persisted in localStorage
- **Notification queue**: max 3 visible, oldest auto-dismissed
- **Dark-styled selects** everywhere (no more white dropdowns)
- **Better sounds**: multi-note chords + attack/decay envelope (no raw oscillator beeps)
- **SVG icons** in notifications
- Toast click-to-close (X button)
- Window close animation (`scale(0.9) + fade`)

### Changed
- `system.js` now uses IPC for real control (CSS overlay only as visual fallback)
- CSS: added `.topbar`, `.popover`, dark `select option` styling

### Known issues
- Brightness control fails on desktop PCs (no WMI brightness methods) - falls back silently
- Volume is per-device (default output), not per-application
## [4.3.2] - 2026-09-28 - hotfix

### Fixed
- **КРИТИЧНО:** `boot.rs` — E0793 "reference to field of packed struct is unaligned". Теперь все чтения полей `AeroBootHeader` идут через локальные копии.
- `load_from_data_folder` — `checksum` копируется в local перед `tracing::info!`
- `verify()` — читает `magic`/`version` через локальные копии

### Added
- Toast click-to-close
- Clock: дата показывается при наведении (`title`)
- `license::derive_ui_key()` — каркас для v4.4.0 (крипто-привязка UI)
## [4.3.1] - 2026-09-28 - hotfix

### Fixed
- **КРИТИЧНО:** `boot::AeroBoot::new().into()` — убран бессмысленный `.into()` (E0277)
- **КРИТИЧНО:** `Option::<AeroBoot>::Some(...)` — упрощён вызов
- Скрипт: функции `W`/`Rm` переименованы в `Aero-W`/`Aero-Rm` — больше не конфликтуют с алиасами PowerShell

### Added
- `AeroBoot::load_from_data_folder()` — удобный helper
## [4.3.0] - 2026-09-28 - "Integrity"

### Fixed
- **КРИТИЧНО:** окна больше не дёргаются при drag/minimize (конфликт `transform` + `transition`)
- Кнопка minimize теперь плавно скрывает окно, restore через док
- Кнопка maximize корректно растягивает на весь экран
- `vm-tabs.js` подключён к реальному API (`ListVms`)

### Added
- **Toast notifications** (`web/js/notifications.js`) — success / warning / error / info
- **Sound engine** (Web Audio API) — нет аудиофайлов, всё синтезируется
- **Clock** в заголовке dashboard
- **Brightness control** (CSS overlay) — слайдер в настройках
- **Volume control** — слайдер в настройках
- **AeroBoot** preload — kernel + initramfs подготавливаются через `boot.rs`

### Changed
- `docs/README.md`, `docs/STRUCTURE.md` — удалены (дубликаты / генерируются)
- Документация собрана в одном `README.md` в корне
- Стили тостов вынесены в `web/css/notifications.css`

### Planned (v4.4.0)
- Криптографическая привязка UI к лицензии
- End-to-end boot Alpine kernel
- 9P full read/write через virtio-fs
## [4.2.0] - 2026-09-28 — "Interface"

### Added
- **Draggable windows** — за header мышью, с границами экрана
- **Window focus** — z-index при клике, окна поднимаются
- **i18n** (RU/EN) — `web/js/i18n.js`, переключатель в настройках, сохранение в localStorage
- **SVG иконки** в доке вместо пустых квадратов
- **Кнопки minimize/maximize** работают (анимации)
- **docs/README.md** — единая точка входа

### Changed
- `web/index.html` — data-i18n атрибуты для мультиязычности
- `web/css/style.css` — убран лишний `::before` glow, улучшены transitions
- `web/js/main.js` — переписан с drag + focus + i18n
- `web/js/theme.js` — интеграция с языками

### Fixed
- Окна больше не "висят" — можно двигать, фокусировать, скрывать
- Кнопка minimize сворачивает в док-подобное состояние
- Кнопка maximize растягивает на всё окно
## [4.1.8] - 2026-09-28

### Fixed
- **КРИТИЧНО:** `src/main.rs` — при `VM skipped` главный процесс завершался мгновенно
  (`tokio::select!` видел завершённую ветку и выходил). Теперь `std::future::pending()` держит процесс живым.
- Все сообщения tracing переведены на английский (устранены кракозябры в консоли)
- Логика fallback: если WHPX недоступен, UI продолжает работать

### Changed
- `tracing::info!("[AeroOS] v4.1.8 starting")` — теперь версия в строке старта соответствует Cargo.toml

### Verified
- `.exe` собирается ✅
- `.exe` запускается, HTTP сервер поднимается ✅
- UI доступен на http://127.0.0.1:8080 ✅
- Fallback без hypervisor работает ✅
## [4.1.6] - 2026-09-28

### Fixed
- **КРИТИЧНО:** `src/boot_vm.rs` — magic "HdrS" читался как u16, а нужен u32 (0x53726448)
- **КРИТИЧНО:** bump версии не работал — regex не находил 4.1.4 в Cargo.toml
- Добавлена проверка `boot_flag == 0xAA55`
- Проверка границ при копировании setup header

### Added
- **docs/SESSION.md** — рабочий контекст (структура, API, константы, TODO)
- Явные константы boot protocol в комментариях

### Changed
- `boot_vm.rs` — полная перезапись с правильными смещениями
## [4.1.5] - 2026-09-28

### Fixed
- **КРИТИЧНО:** `src/oci.rs` — `walkdir` не был в `Cargo.toml`
- Добавлен `walkdir = "2.5"` в зависимости

### Added
- **docs/API.md** — справочник публичного API всех модулей (ручной, но полный)
## [4.1.4] - 2026-09-28

### Fixed
- **КРИТИЧНО:** `src/snapshot.rs` — методы `create_snapshot`/`restore_snapshot`/`list_snapshots` были внутри `mod tests`, теперь в `impl SnapshotManager`
- **КРИТИЧНО:** `src/hypervisor.rs` — `use whpx::MapFlags;` (был в корне, не в `ffi`)
- `hypervisor.rs` — убран `unsafe` вокруг `exit.IoPortAccess.Port`
- `whpx/lib.rs` — `pub use ffi::WHvMapGpaRangeFlags as MapFlags;`

### Added
- Тест `test_named_snapshot_roundtrip` для именованных снапшотов
## [4.1.3] - 2026-09-28

### Fixed
- **КРИТИЧНО:** `src/ipc.rs` — вызовы методов, которых не существовало:
  - `VirtualMachine::get_state()` и `restore_state()` — **добавлены** в hypervisor.rs
  - `SnapshotManager::create_snapshot()`, `restore_snapshot()`, `list_snapshots()` — **добавлены**
- `src/virtio_fs.rs` — все async-методы теперь вызываются с `.await`
- Serial output совместим с `console=ttyS0,115200`

### Added
- **Serial output** через COM1 (port 0x3F8) — можно смотреть вывод ядра
- IPC команды `GetSerial` / `ClearSerial` — UI может читать serial лог
- Boot cmdline расширен: `earlyprintk=serial,ttyS0,115200`
- Serial buffer 64 KB

### Changed
- `console=ttyS0,115200 earlyprintk=serial,ttyS0,115200 quiet` — стандартный Linux serial для отладки
## [4.1.2] - 2026-09-28

### Fixed
- **КРИТИЧНО:** `src/ipc.rs` — полностью переписан (regex-replace из v4.1.0 сломал `#[derive(Deserialize)]`)
- `src/oci.rs` — убран unused `mut` в `CpioNewcBuilder::finish`
- `src/hypervisor.rs` — добавлен `#![allow(dead_code)]` для служебных полей

### Added
- `Command::ListSnapshots` в IPC (для UI timeline)
- `Response::VmsList` с per-VM статистикой
## [4.1.1] - 2026-09-28

### Fixed
- `whpx::Partition::map_gpa_range` — тип `MapFlags` импортировался из `ffi`, но он re-export на верхнем уровне

### Changed
- `tokio` — убраны неиспользуемые features (`signal`, `io-std`)
## [4.1.0] - 2026-09-28 — "Horizon"

### Fixed
- **КРИТИЧНО:** `WHvSetVirtualProcessorRegisters` — правильный порядок `(handle, names_ptr, count, values_ptr)`
- **КРИТИЧНО:** Регистры теперь передаются как 16-байтовые `WHvRegisterValue` (union)
- Boot: правильная загрузка setup + kernel раздельно

### Added
- **End-to-end boot** — `setup_vcpu_registers` с CR0/CR4/EFER для long mode
- **GPU-P** (`gpu_passthrough.rs`) — реальный Hyper-V GPU Partitioning через PowerShell
- **9P wiring** — `virtio.rs` вызывает `virtio_fs::VirtioFsServer`
- **Multi-VM UI** — per-VM статистика в dashboard + авто-обновление
- **Enterprise offline licensing** — Ed25519 + HWID + nonce, без сервера
- **Tier gating** — `max_vms()`, `max_ram_mb()`, `can_use()` по тиру

### Changed
- `Cargo.toml` — версия 4.1.0
- `license.rs` — payload v2 с nonce (защита от replay)
- `gen-license` — флаг `--nonce`, автоматическая генерация

## [4.0.0] - Genesis
## [3.3.0] - Continuum
## [3.0.0] - Singularity
## [1.0.0] - Initial