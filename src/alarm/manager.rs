//! Alarm manager — holds the list of alarms, checks for firing, and persists.
//!
//! The manager is stored alongside `AppConfig` and checked on every tick.

use std::fs;
use std::path::PathBuf;

use chrono::Local;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Alarm, AlarmKind, AlertAction, FaceActiveItem};

mod optional_ts_seconds_local {
    use chrono::{DateTime, Local, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &Option<DateTime<Local>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(dt) => serializer.serialize_some(&dt.timestamp()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Local>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ts = Option::<i64>::deserialize(deserializer)?;

        ts.map(|value| {
            Utc.timestamp_opt(value, 0)
                .single()
                .map(|utc| utc.with_timezone(&Local))
                .ok_or_else(|| serde::de::Error::custom("invalid timestamp"))
        })
        .transpose()
    }
}

/// Wrapper for TOML serialisation of alarm list.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct AlarmFile {
    #[serde(default)]
    alarm: Vec<Alarm>,
    #[serde(default, with = "optional_ts_seconds_local")]
    suspended_at: Option<chrono::DateTime<Local>>,
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
                Ok(file) => {
                    let mut manager = Self { alarms: file.alarm };

                    let resumed_at = Local::now();
                    let mut changed = false;

                    for alarm in &mut manager.alarms {
                        if alarm.restore_from_restart_snapshot(resumed_at) {
                            changed = true;
                            continue;
                        }

                        if let Some(suspended_at) = file.suspended_at {
                            changed |= alarm.resume_after_restart(suspended_at, resumed_at);
                        }
                    }

                    if changed {
                        manager.save();
                    }

                    manager
                }
                Err(e) => {
                    eprintln!("Failed to parse alarms at {}: {e}", path.display());
                    Self::backup_corrupted_file(&path);
                    Self::default()
                }
            },
            Err(e) => {
                eprintln!("Failed to read alarms at {}: {e}", path.display());
                Self::default()
            }
        }
    }

    /// Attempt to preserve a corrupted alarms file for manual recovery.
    fn backup_corrupted_file(path: &PathBuf) {
        let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let backup = path.with_file_name(format!("alarms.corrupt-{stamp}.toml"));
        match fs::copy(path, &backup) {
            Ok(_) => eprintln!("Backed up corrupted alarms file to {}", backup.display()),
            Err(e) => eprintln!(
                "Failed to backup corrupted alarms file {}: {e}",
                path.display()
            ),
        }
    }

    /// Save the current alarm list to disk.
    pub fn save(&self) {
        self.save_with_suspended_at(None);
    }

    pub fn save_for_shutdown(&self) {
        self.save_with_suspended_at(Some(Local::now()));
    }

    fn save_with_suspended_at(&self, suspended_at: Option<chrono::DateTime<Local>>) {
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
        let snapshot_at = Local::now();
        let file = AlarmFile {
            alarm: self
                .alarms
                .iter()
                .map(|alarm| alarm.persisted_for_restart(snapshot_at))
                .collect(),
            suspended_at,
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

    /// Number of active (enabled, not completed) alarms.
    pub fn active_count(&self) -> usize {
        self.alarms.iter().filter(|alarm| alarm.is_live()).count()
    }

    /// Number of paused reminders.
    pub fn paused_count(&self) -> usize {
        self.alarms
            .iter()
            .filter(|alarm| alarm.enabled && alarm.is_paused())
            .count()
    }

    /// Number of reminders that are currently eligible to be paused.
    pub fn pausable_count(&self) -> usize {
        let now = Local::now();

        self.alarms
            .iter()
            .filter(|alarm| alarm.can_pause(now))
            .count()
    }

    pub fn has_live_restartable_reminders(&self) -> bool {
        self.alarms.iter().any(|alarm| {
            alarm.enabled
                && !alarm.is_paused()
                && !alarm.is_completed()
                && matches!(
                    alarm.kind,
                    AlarmKind::Timer { .. } | AlarmKind::RepeatingInterval { .. }
                )
        })
    }

    /// Active alarms/timers projected for compact clock-face display.
    pub fn face_active_items(&self) -> Vec<FaceActiveItem> {
        let mut items: Vec<_> = self
            .alarms
            .iter()
            .filter_map(Alarm::face_active_item)
            .collect();
        items.sort_by_key(|item| item.target.timestamp());
        items
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

    /// Find an alarm by ID.
    pub fn get(&self, id: Uuid) -> Option<&Alarm> {
        self.alarms.iter().find(|a| a.id == id)
    }

    /// Replace an existing alarm (same ID) with updated data.
    pub fn update(&mut self, updated: Alarm) {
        if let Some(existing) = self.alarms.iter_mut().find(|a| a.id == updated.id) {
            *existing = updated;
            self.save();
        }
    }

    /// Pause an existing reminder by ID.
    pub fn pause(&mut self, id: Uuid) -> bool {
        let changed = self
            .alarms
            .iter_mut()
            .find(|alarm| alarm.id == id)
            .is_some_and(|alarm| alarm.pause(Local::now()));

        if changed {
            self.save();
        }

        changed
    }

    /// Pause all currently running reminders and return the number paused.
    pub fn pause_all_running(&mut self) -> usize {
        let now = Local::now();
        let mut paused = 0;

        for alarm in &mut self.alarms {
            if alarm.pause(now) {
                paused += 1;
            }
        }

        if paused > 0 {
            self.save();
        }

        paused
    }

    /// Resume a previously paused reminder by ID.
    pub fn resume(&mut self, id: Uuid) -> bool {
        let changed = self
            .alarms
            .iter_mut()
            .find(|alarm| alarm.id == id)
            .is_some_and(|alarm| alarm.resume_from_pause(Local::now()));

        if changed {
            self.save();
        }

        changed
    }

    /// Resume all paused reminders and return the number resumed.
    pub fn resume_all_paused(&mut self) -> usize {
        let now = Local::now();
        let mut resumed = 0;

        for alarm in &mut self.alarms {
            if alarm.resume_from_pause(now) {
                resumed += 1;
            }
        }

        if resumed > 0 {
            self.save();
        }

        resumed
    }

    /// Remove all one-shot alarms that have already fired.
    pub fn clear_fired(&mut self) {
        self.alarms
            .retain(|alarm| alarm.kind.is_recurring() || !alarm.fired);
        self.save();
    }

    // -- Tick check --------------------------------------------------------

    /// Check all alarms and return a list of those that should fire right now.
    /// One-shot alarms are marked as fired. Recurring alarms are advanced to
    /// their next future target and only fire once per check.
    pub fn check_and_fire(&mut self) -> Vec<Alarm> {
        let now = Local::now();
        let mut fired = Vec::new();
        let mut changed = false;

        for alarm in &mut self.alarms {
            if !alarm.should_fire_at(now) {
                continue;
            }

            let emitted = alarm.clone();
            if alarm.kind.is_recurring() {
                if !alarm.advance_after_fire(now) {
                    alarm.fired = true;
                }
            } else {
                alarm.fired = true;
            }

            fired.push(emitted);
            changed = true;
        }

        if changed {
            self.save();
        }

        fired
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn add_and_remove_alarm() {
        let mut mgr = AlarmManager::default();
        let alarm = Alarm::new("Test", AlarmKind::from_now(60), AlertAction::Notification);
        let id = alarm.id;
        mgr.alarms.push(alarm);
        assert_eq!(mgr.active_count(), 1);
        mgr.alarms.retain(|a| a.id != id);
        assert_eq!(mgr.active_count(), 0);
    }

    #[test]
    fn check_and_fire_marks_one_shot_as_fired() {
        let mut mgr = AlarmManager::default();
        let alarm = Alarm::new("Now", AlarmKind::from_now(0), AlertAction::Both);
        mgr.alarms.push(alarm);
        std::thread::sleep(std::time::Duration::from_millis(10));
        let fired = mgr.check_and_fire();
        assert_eq!(fired.len(), 1);
        assert_eq!(fired[0].label, "Now");
        let fired2 = mgr.check_and_fire();
        assert!(fired2.is_empty());
    }

    #[test]
    fn check_and_fire_advances_recurring_timer() {
        let mut mgr = AlarmManager::default();
        let alarm = Alarm::new(
            "Hourly",
            AlarmKind::RepeatingInterval {
                interval_secs: 3600,
                next_target: Local::now() - Duration::seconds(1),
            },
            AlertAction::Both,
        );
        mgr.alarms.push(alarm);

        let fired = mgr.check_and_fire();
        assert_eq!(fired.len(), 1);
        assert_eq!(mgr.active_count(), 1);
        assert!(!mgr.alarms[0].fired);
        assert!(mgr.alarms[0].kind.target() > Local::now());
    }

    #[test]
    fn face_active_items_are_sorted_and_filtered() {
        let mut mgr = AlarmManager::default();

        let mut disabled = Alarm::new("Disabled", AlarmKind::from_now(60), AlertAction::Both);
        disabled.enabled = false;

        let mut fired = Alarm::new("Fired", AlarmKind::from_now(120), AlertAction::Both);
        fired.fired = true;

        let later = Alarm::new("Later", AlarmKind::from_now(300), AlertAction::Both);
        let sooner = Alarm::new("Sooner", AlarmKind::from_now(30), AlertAction::Both);
        let recurring = Alarm::new(
            "Hourly",
            AlarmKind::RepeatingInterval {
                interval_secs: 3600,
                next_target: Local::now() + Duration::seconds(15),
            },
            AlertAction::Both,
        );

        mgr.alarms.push(later);
        mgr.alarms.push(disabled);
        mgr.alarms.push(fired);
        mgr.alarms.push(sooner);
        mgr.alarms.push(recurring);

        let items = mgr.face_active_items();
        let labels: Vec<_> = items.iter().map(|item| item.label.as_str()).collect();

        assert_eq!(labels, vec!["Hourly", "Sooner", "Later"]);
    }

    #[test]
    fn paused_alarm_is_excluded_from_active_count_and_face_items() {
        let mut mgr = AlarmManager::default();
        let mut alarm = Alarm::new("Tea", AlarmKind::from_now(600), AlertAction::Both);
        assert!(alarm.pause(Local::now()));
        mgr.alarms.push(alarm);

        assert_eq!(mgr.active_count(), 0);
        assert_eq!(mgr.paused_count(), 1);
        assert!(mgr.face_active_items().is_empty());
    }

    #[test]
    fn resume_all_paused_resumes_every_paused_reminder() {
        let now = Local::now();
        let mut mgr = AlarmManager::default();

        let mut timer = Alarm::new(
            "Tea",
            AlarmKind::Timer {
                duration_secs: 600,
                target: now + Duration::minutes(8),
            },
            AlertAction::Both,
        );
        let mut alarm = Alarm::new(
            "Meeting",
            AlarmKind::AtTime {
                target: now + Duration::minutes(20),
            },
            AlertAction::Both,
        );

        assert!(timer.pause(now));
        assert!(alarm.pause(now));
        mgr.alarms.push(timer);
        mgr.alarms.push(alarm);

        assert_eq!(mgr.paused_count(), 2);
        assert_eq!(mgr.resume_all_paused(), 2);
        assert_eq!(mgr.paused_count(), 0);
        assert_eq!(mgr.active_count(), 2);
    }

    #[test]
    fn pause_all_running_skips_done_one_shot_reminders() {
        let now = Local::now();
        let mut mgr = AlarmManager::default();

        let running = Alarm::new(
            "Tea",
            AlarmKind::Timer {
                duration_secs: 600,
                target: now + Duration::minutes(8),
            },
            AlertAction::Both,
        );
        let mut done = Alarm::new(
            "Done",
            AlarmKind::Timer {
                duration_secs: 60,
                target: now - Duration::seconds(5),
            },
            AlertAction::Both,
        );
        done.fired = true;

        mgr.alarms.push(running);
        mgr.alarms.push(done);

        assert_eq!(mgr.pausable_count(), 1);
        assert_eq!(mgr.pause_all_running(), 1);
        assert_eq!(mgr.paused_count(), 1);
        assert!(mgr.alarms[1].fired);
        assert!(!mgr.alarms[1].is_paused());
    }
}
