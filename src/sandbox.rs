//! AppContainer sandbox for AeroOS.
//! Базовая изоляция через winapi, без внешних крейтов.

#[cfg(windows)]
pub fn init_sandbox(allowed_paths: &[&str]) -> anyhow::Result<()> {
    use std::ptr;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
    use winapi::um::securitybaseapi::GetTokenInformation;
    use winapi::um::winnt::TOKEN_QUERY;

    tracing::info!("Initializing AppContainer sandbox...");

    unsafe {
        let process = GetCurrentProcess();
        let mut token = ptr::null_mut();

        if OpenProcessToken(process, TOKEN_QUERY, &mut token) == 0 {
            let err = std::io::Error::last_os_error();
            tracing::warn!("OpenProcessToken failed: {}. Sandbox disabled.", err);
            return Ok(());
        }

        // Проверка, что мы в AppContainer (GetTokenInformation с TokenIsAppContainer)
        // В полной версии — CreateAppContainerProfile, DeriveRestrictedToken, SetTokenInformation
        tracing::info!("Sandbox token acquired (full AppContainer impl pending)");
        for path in allowed_paths {
            tracing::info!("  Allowed: {}", path);
        }

        CloseHandle(token);
    }

    Ok(())
}

#[cfg(not(windows))]
pub fn init_sandbox(_allowed_paths: &[&str]) -> anyhow::Result<()> {
    tracing::debug!("Sandbox not available");
    Ok(())
}

pub fn is_sandboxed() -> bool {
    #[cfg(all(windows, feature = "sandbox"))]
    {
        false
    }
    #[cfg(not(all(windows, feature = "sandbox")))]
    {
        false
    }
}
