//! Global hotkeys for AeroOS.
//!
//! Uses Windows RegisterHotKey API:
//!   Ctrl+Alt+Up    -> Volume up
//!   Ctrl+Alt+Down  -> Volume down
//!   Ctrl+Alt+Right -> Brightness up
//!   Ctrl+Alt+Left  -> Brightness down
//!
//! On hotkey, push event to channel; main thread consumes and calls
//! syscontrol::set_volume / set_brightness.

use anyhow::Result;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;

#[derive(Debug, Clone, Copy)]
pub enum HotkeyEvent {
    VolumeUp,
    VolumeDown,
    BrightnessUp,
    BrightnessDown,
}

static HOTKEY_TX: Mutex<Option<Sender<HotkeyEvent>>> = Mutex::new(None);

/// Register global hotkeys. Returns receiver for consuming events.
pub fn register() -> Result<Receiver<HotkeyEvent>> {
    let (tx, rx) = channel::<HotkeyEvent>();
    *HOTKEY_TX.lock().unwrap() = Some(tx);

    #[cfg(windows)]
    {
        use std::thread;
        thread::spawn(|| {
            if let Err(e) = hotkey_loop() {
                tracing::warn!("hotkey loop: {}", e);
            }
        });
    }

    Ok(rx)
}

#[cfg(windows)]
fn hotkey_loop() -> Result<()> {
    use winapi::shared::minwindef::{TRUE, UINT, WPARAM, LPARAM};
    use winapi::um::winuser::{
        RegisterHotKey, GetMessageW, TranslateMessage, DispatchMessageW, UnregisterHotKey, MSG,
        MOD_CONTROL, MOD_ALT, MOD_NOREPEAT, VK_UP, VK_DOWN, VK_LEFT, VK_RIGHT,
    };

    // IDs for our hotkeys
    const ID_VOL_UP:    i32 = 1;
    const ID_VOL_DOWN:  i32 = 2;
    const ID_BR_DOWN:   i32 = 3;
    const ID_BR_UP:     i32 = 4;

    // Cast to UINT (winapi expects u32)
    let mods: UINT = (MOD_CONTROL | MOD_ALT | MOD_NOREPEAT) as UINT;
    let vk_up:    UINT = VK_UP    as UINT;
    let vk_down:  UINT = VK_DOWN  as UINT;
    let vk_left:  UINT = VK_LEFT  as UINT;
    let vk_right: UINT = VK_RIGHT as UINT;

    unsafe {
        let hwnd = std::ptr::null_mut();

        let mut registered = 0;

        if RegisterHotKey(hwnd, ID_VOL_UP, mods, vk_up) == TRUE {
            registered += 1;
        } else {
            tracing::warn!("Ctrl+Alt+Up registration failed");
        }
        if RegisterHotKey(hwnd, ID_VOL_DOWN, mods, vk_down) == TRUE {
            registered += 1;
        } else {
            tracing::warn!("Ctrl+Alt+Down registration failed");
        }
        if RegisterHotKey(hwnd, ID_BR_DOWN, mods, vk_left) == TRUE {
            registered += 1;
        } else {
            tracing::warn!("Ctrl+Alt+Left registration failed");
        }
        if RegisterHotKey(hwnd, ID_BR_UP, mods, vk_right) == TRUE {
            registered += 1;
        } else {
            tracing::warn!("Ctrl+Alt+Right registration failed");
        }

        if registered == 0 {
            anyhow::bail!("no hotkeys registered");
        }
        tracing::info!("Hotkeys registered: {}/4 (Ctrl+Alt+Arrows)", registered);

        // Message loop
        let mut msg: MSG = std::mem::zeroed();
        loop {
            let r = GetMessageW(&mut msg, hwnd, 0, 0);
            if r <= 0 {
                break;
            }

            let id = msg.wParam as i32;
            let ev = match id {
                ID_VOL_UP   => Some(HotkeyEvent::VolumeUp),
                ID_VOL_DOWN => Some(HotkeyEvent::VolumeDown),
                ID_BR_DOWN  => Some(HotkeyEvent::BrightnessDown),
                ID_BR_UP    => Some(HotkeyEvent::BrightnessUp),
                _ => None,
            };

            if let Some(ev) = ev {
                if let Some(tx) = HOTKEY_TX.lock().unwrap().as_ref() {
                    let _ = tx.send(ev);
                }
            }

            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Cleanup
        UnregisterHotKey(hwnd, ID_VOL_UP);
        UnregisterHotKey(hwnd, ID_VOL_DOWN);
        UnregisterHotKey(hwnd, ID_BR_DOWN);
        UnregisterHotKey(hwnd, ID_BR_UP);
    }

    Ok(())
}

#[cfg(not(windows))]
pub fn register() -> Result<Receiver<HotkeyEvent>> {
    anyhow::bail!("Hotkeys only supported on Windows")
}