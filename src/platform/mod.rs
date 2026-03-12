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

/// Send a platform-native notification when available.
pub fn send_notification(summary: &str, body: &str) {
    implementation::send_notification(summary, body);
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
