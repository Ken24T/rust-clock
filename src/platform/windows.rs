use iced::{window, Task};
use winrt_notification::{Duration, Toast};

use crate::tray::{self, SystemTrayHandle, TrayCommand};

use super::PlatformCapabilities;

pub fn capabilities() -> PlatformCapabilities {
    PlatformCapabilities {
        system_tray: true,
        notifications: true,
        desktop_window_hints: false,
        sticky_workspace: false,
        skip_taskbar: false,
    }
}

pub fn send_notification(summary: &str, body: &str) {
    if let Err(error) = Toast::new(&windows_toast_app_id())
        .title(summary)
        .text1(body)
        .duration(Duration::Short)
        .sound(None)
        .show()
    {
        eprintln!("Failed to send Windows toast notification: {error}");
    }
}

pub fn start_system_tray() -> Option<(SystemTrayHandle, std::sync::mpsc::Receiver<TrayCommand>)> {
    tray::start_system_tray()
}

pub fn configure_main_window_settings(settings: &mut window::Settings) {
    settings.level = window::Level::Normal;
}

pub fn configure_control_window_settings(settings: &mut window::Settings) {
    settings.level = window::Level::AlwaysOnTop;
}

pub fn apply_startup_window_hints(id: window::Id) -> Task<crate::Message> {
    let _ = id;
    Task::none()
}

pub fn apply_control_window_hints(id: window::Id) -> Task<crate::Message> {
    let _ = id;
    Task::none()
}

fn windows_toast_app_id() -> String {
    select_windows_toast_app_id(std::env::var("RUST_CLOCK_WINDOWS_AUMID").ok())
}

fn select_windows_toast_app_id(value: Option<String>) -> String {
    value
        .filter(|candidate| !candidate.trim().is_empty())
        .unwrap_or_else(|| Toast::POWERSHELL_APP_ID.to_string())
}

#[cfg(test)]
mod tests {
    use super::{select_windows_toast_app_id, Toast};

    #[test]
    fn uses_explicit_aumid_when_present() {
        assert_eq!(
            select_windows_toast_app_id(Some("RustClock.Test".to_string())),
            "RustClock.Test"
        );
    }

    #[test]
    fn falls_back_for_empty_aumid() {
        assert_eq!(
            select_windows_toast_app_id(Some("   ".to_string())),
            Toast::POWERSHELL_APP_ID
        );
    }

    #[test]
    fn falls_back_when_aumid_missing() {
        assert_eq!(select_windows_toast_app_id(None), Toast::POWERSHELL_APP_ID);
    }
}
