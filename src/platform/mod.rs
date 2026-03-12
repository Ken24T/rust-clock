#[cfg(not(any(target_os = "linux", target_os = "windows")))]
mod fallback;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
use fallback as implementation;
#[cfg(target_os = "linux")]
use linux as implementation;
#[cfg(target_os = "windows")]
use windows as implementation;

use iced::{window, Task};

pub use crate::tray::{SystemTrayHandle, TrayCommand};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformCapabilities {
    pub system_tray: bool,
    pub notifications: bool,
    pub desktop_window_hints: bool,
    pub sticky_workspace: bool,
    pub skip_taskbar: bool,
}

pub fn capabilities() -> PlatformCapabilities {
    implementation::capabilities()
}

/// Send a platform-native notification when available.
pub fn send_notification(summary: &str, body: &str) {
    implementation::send_notification(summary, body);
}

pub fn start_system_tray() -> Option<(SystemTrayHandle, std::sync::mpsc::Receiver<TrayCommand>)> {
    implementation::start_system_tray()
}

pub fn configure_main_window_settings(settings: &mut window::Settings) {
    implementation::configure_main_window_settings(settings);
}

pub fn configure_control_window_settings(settings: &mut window::Settings) {
    implementation::configure_control_window_settings(settings);
}

pub fn apply_startup_window_hints(id: window::Id) -> Task<crate::Message> {
    implementation::apply_startup_window_hints(id)
}

pub fn apply_control_window_hints(id: window::Id) -> Task<crate::Message> {
    implementation::apply_control_window_hints(id)
}
