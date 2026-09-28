//! System Control - real Windows APIs for volume and brightness.
//!
//! Volume: winmm.dll::waveOutSetVolume (WAVE_MAPPER)
//! Brightness: PowerShell WMI (WmiMonitorBrightnessMethods) - works on laptops
//!
//! Note: waveOutSetVolume works on the DEFAULT audio device.
//! For per-app volume, we would need Core Audio (IMMDeviceEnumerator) - future work.

use anyhow::Result;

#[cfg(windows)]
mod win {
    use std::os::raw::c_uint;
    use std::ptr;

    pub type HWAVEOUT = *mut std::ffi::c_void;

    pub const WAVE_MAPPER: c_uint = 0xFFFFFFFF;

    #[link(name = "winmm")]
    extern "system" {
        pub fn waveOutSetVolume(hwo: HWAVEOUT, dwVolume: u32) -> u32;
        pub fn waveOutGetVolume(hwo: HWAVEOUT, pdwVolume: *mut u32) -> u32;
    }
}

/// Set system volume. `level` is 0..100.
#[cfg(windows)]
pub fn set_volume(level: u32) -> Result<()> {
    let level = level.min(100);
    // Windows volume is 16-bit per channel, left in low word, right in high word
    let per_channel: u32 = ((level as f32 / 100.0) * 0xFFFF as f32) as u32;
    let packed: u32 = per_channel | (per_channel << 16);

    unsafe {
        let h = win::WAVE_MAPPER as win::HWAVEOUT;
        let rc = win::waveOutSetVolume(h, packed);
        if rc != 0 {
            anyhow::bail!("waveOutSetVolume failed: {}", rc);
        }
    }
    tracing::info!("System volume set to {}%", level);
    Ok(())
}

/// Get system volume. Returns 0..100.
#[cfg(windows)]
pub fn get_volume() -> Result<u32> {
    unsafe {
        let h = win::WAVE_MAPPER as win::HWAVEOUT;
        let mut packed: u32 = 0;
        let rc = win::waveOutGetVolume(h, &mut packed);
        if rc != 0 {
            anyhow::bail!("waveOutGetVolume failed: {}", rc);
        }
        let left = packed & 0xFFFF;
        let level = ((left as f32 / 0xFFFF as f32) * 100.0) as u32;
        Ok(level)
    }
}

/// Set monitor brightness via PowerShell WMI.
/// Works on laptops with internal displays. Returns Err on desktops.
#[cfg(windows)]
pub fn set_brightness(level: u32) -> Result<()> {
    let level = level.clamp(0, 100);
    let script = format!(
        "$b = Get-WmiObject -Namespace root/WMI -Class WmiMonitorBrightnessMethods -ErrorAction Stop; \
         if ($b) {{ $b.WmiSetBrightness(1, {}) | Out-Null }} else {{ exit 1 }}",
        level
    );

    let out = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()?;

    if !out.status.success() {
        anyhow::bail!(
            "Brightness control not available: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    tracing::info!("System brightness set to {}%", level);
    Ok(())
}

#[cfg(windows)]
pub fn get_brightness() -> Result<u32> {
    let script =
        "(Get-WmiObject -Namespace root/WMI -Class WmiMonitorBrightness -ErrorAction Stop).CurrentBrightness";

    let out = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()?;

    if !out.status.success() {
        anyhow::bail!("Brightness read failed");
    }

    let s = String::from_utf8_lossy(&out.stdout);
    let level: u32 = s.trim().parse().unwrap_or(100);
    Ok(level)
}

// ------- non-Windows stubs -------
#[cfg(not(windows))]
pub fn set_volume(_level: u32) -> Result<()> {
    anyhow::bail!("syscontrol: not supported on this platform")
}
#[cfg(not(windows))]
pub fn get_volume() -> Result<u32> { Ok(50) }
#[cfg(not(windows))]
pub fn set_brightness(_level: u32) -> Result<()> {
    anyhow::bail!("syscontrol: not supported")
}
#[cfg(not(windows))]
pub fn get_brightness() -> Result<u32> { Ok(100) }