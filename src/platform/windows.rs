use iced::{window, Task};
use windows_sys::Win32::Foundation::{POINT, RECT};
use windows_sys::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use winrt_notification::{Duration, Toast};

use crate::tray::{self, SystemTrayHandle, TrayCommand};

use super::{PlatformCapabilities, WorkArea};

pub fn capabilities() -> PlatformCapabilities {
    PlatformCapabilities {
        system_tray: true,
        notifications: true,
        desktop_window_hints: false,
        sticky_workspace: false,
        skip_taskbar: true,
        tray_only_main_window: false,
    }
}

pub fn work_area_for_point(x: f32, y: f32) -> Option<WorkArea> {
    let point = POINT {
        x: x.round() as i32,
        y: y.round() as i32,
    };

    // SAFETY: The POINT is passed by value, the monitor handle is only used for the immediate
    // query, and the MONITORINFO struct is correctly initialised with its size before the call.
    let monitor = unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST) };
    if monitor.is_null() {
        None
    } else {
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            rcMonitor: RECT::default(),
            rcWork: RECT::default(),
            dwFlags: 0,
        };

        // SAFETY: `info` points to a valid MONITORINFO buffer for the duration of the call.
        let ok = unsafe { GetMonitorInfoW(monitor, &mut info as *mut MONITORINFO) };
        if ok == 0 {
            return None;
        }

        Some(WorkArea {
            x: info.rcWork.left as f32,
            y: info.rcWork.top as f32,
            width: (info.rcWork.right - info.rcWork.left) as f32,
            height: (info.rcWork.bottom - info.rcWork.top) as f32,
        })
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
    settings.platform_specific.skip_taskbar = true;
}

pub fn configure_control_window_settings(settings: &mut window::Settings) {
    settings.level = window::Level::AlwaysOnTop;
    settings.platform_specific.skip_taskbar = true;
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
