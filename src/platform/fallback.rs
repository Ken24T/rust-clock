use iced::{window, Task};

use crate::tray::{SystemTrayHandle, TrayCommand};

use super::{PlatformCapabilities, WorkArea};

pub fn capabilities() -> PlatformCapabilities {
    PlatformCapabilities {
        system_tray: false,
        notifications: false,
        desktop_window_hints: false,
        sticky_workspace: false,
        skip_taskbar: false,
    }
}

pub fn work_area_for_point(_x: f32, _y: f32) -> Option<WorkArea> {
    None
}

pub fn send_notification(summary: &str, body: &str) {
    let _ = (summary, body);
    eprintln!("Notifications are not implemented on this platform");
}

pub fn start_system_tray() -> Option<(SystemTrayHandle, std::sync::mpsc::Receiver<TrayCommand>)> {
    None
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
