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

/// Send a platform-native notification when available.
pub fn send_notification(summary: &str, body: &str) {
    implementation::send_notification(summary, body);
}
