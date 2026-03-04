//! Alarm manager — holds the list of alarms, checks for firing, and persists.
//!
//! The manager is stored alongside `AppConfig` and checked on every tick.

use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Alarm, AlarmKind, AlertAction};

/// Wrapper for TOML serialisation of alarm list.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct AlarmFile {
    #[serde(default)]
    alarm: Vec<Alarm>,
}

/// Manages a collection of alarms — creation, deletion, persistence, and
/// tick-based checking.
#[derive(Debug, Clone, Default)]
pub struct AlarmManager {
    alarms: Vec<Alarm>,
}

impl AlarmManager {
    // -- Persistence -------------------------------------------------------

    /// Path to the alarms file: `~/.config/rust-clock/alarms.toml`.
    fn file_path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "rust-clock").map(|dirs| dirs.config_dir().join("alarms.toml"))
    }

    /// Load alarms from disk, returning an empty manager on any error.
    pub fn load() -> Self {
        let Some(path) = Self::file_path() else {
            return Self::default();
        };
        if !path.exists() {
            return Self::default();
        }
        match fs::read_to_string(&path) {
            Ok(contents) => match toml::from_str::<AlarmFile>(&contents) {
                Ok(file) => Self { alarms: file.alarm },
                Err(e) => {
                    eprintln!("Failed to parse alarms at {}: {e}", path.display());
                    Self::default()
                }
            },
            Err(e) => {
                eprintln!("Failed to read alarms at {}: {e}", path.display());
                Self::default()
            }
        }
    }

    /// Save the current alarm list to disk.
    pub fn save(&self) {
        let Some(path) = Self::file_path() else {
            eprintln!("Could not determine config directory for alarms");
            return;
        };
        if let Some(parent) = path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("Failed to create config directory: {e}");
                return;
            }
        }
        let file = AlarmFile {
            alarm: self.alarms.clone(),
        };
        match toml::to_string_pretty(&file) {
            Ok(contents) => {
                if let Err(e) = fs::write(&path, contents) {
                    eprintln!("Failed to write alarms: {e}");
                }
            }
            Err(e) => eprintln!("Failed to serialise alarms: {e}"),
        }
    }

    // -- Queries -----------------------------------------------------------

    /// All alarms (active and fired).
    pub fn all(&self) -> &[Alarm] {
        &self.alarms
    }

    /// Number of active (unfired, enabled) alarms.
    pub fn active_count(&self) -> usize {
        self.alarms.iter().filter(|a| a.enabled && !a.fired).count()
    }

    /// Is the list empty?
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.alarms.is_empty()
    }

    // -- Mutations ---------------------------------------------------------

    /// Add a pre-built alarm.
    pub fn add(&mut self, alarm: Alarm) {
        self.alarms.push(alarm);
        self.save();
    }

    /// Create and add a quick timer (from now).
    pub fn add_timer(&mut self, label: impl Into<String>, duration_secs: u64) {
        let alarm = Alarm::new(label, AlarmKind::from_now(duration_secs), AlertAction::Both);
        self.add(alarm);
    }

    /// Remove an alarm by ID.
    pub fn remove(&mut self, id: Uuid) {
        self.alarms.retain(|a| a.id != id);
        self.save();
    }

    /// Remove all alarms that have already fired.
    pub fn clear_fired(&mut self) {
        self.alarms.retain(|a| !a.fired);
        self.save();
    }

    // -- Tick check --------------------------------------------------------

    /// Check all alarms and return a list of those that should fire right now.
    /// Marks them as fired so they won't fire again.
    pub fn check_and_fire(&mut self) -> Vec<Alarm> {
        let mut fired = Vec::new();
        for alarm in &mut self.alarms {
            if alarm.should_fire() {
                alarm.fired = true;
                fired.push(alarm.clone());
            }
        }
        if !fired.is_empty() {
            self.save();
        }
        fired
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_remove_alarm() {
        let mut mgr = AlarmManager::default();
        let alarm = Alarm::new("Test", AlarmKind::from_now(60), AlertAction::Notification);
        let id = alarm.id;
        mgr.alarms.push(alarm); // Skip save for test
        assert_eq!(mgr.active_count(), 1);
        mgr.alarms.retain(|a| a.id != id);
        assert_eq!(mgr.active_count(), 0);
    }

    #[test]
    fn check_and_fire_marks_fired() {
        let mut mgr = AlarmManager::default();
        // Create a timer that should fire immediately.
        let alarm = Alarm::new("Now", AlarmKind::from_now(0), AlertAction::Both);
        mgr.alarms.push(alarm);
        std::thread::sleep(std::time::Duration::from_millis(10));
        let fired = mgr.check_and_fire();
        assert_eq!(fired.len(), 1);
        assert_eq!(fired[0].label, "Now");
        // Should not fire again.
        let fired2 = mgr.check_and_fire();
        assert!(fired2.is_empty());
    }
}
