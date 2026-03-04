//! Alarm and timer system — data model, persistence, and fire/check logic.
//!
//! Supports two kinds of alert:
//! - **Timer**: fires after a duration from when it was created ("from now").
//! - **Alarm**: fires at a specific date/time.
//!
//! Each alarm can trigger audio playback, a desktop notification, or both.

mod manager;
mod sound;

pub use manager::AlarmManager;
pub use sound::play_alarm_sound;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// -- Alarm form state (held by the application) ---------------------------

/// Whether the form is creating a countdown timer or a fixed-time alarm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlarmFormMode {
    #[default]
    Timer,
    Alarm,
}

/// Editable form state for creating or editing an alarm.
#[derive(Debug, Clone, Default)]
pub struct AlarmForm {
    /// Display label (e.g. "Tea timer").
    pub label: String,
    /// Optional notification message.
    pub message: String,
    /// Duration in minutes (Timer mode).
    pub timer_minutes: String,
    /// Target time as "HH:MM" (Alarm mode).
    pub alarm_time: String,
    /// Target date as "YYYY-MM-DD" (Alarm mode, blank = today).
    pub alarm_date: String,
    /// Timer or Alarm.
    pub mode: AlarmFormMode,
    /// When editing, the ID of the existing alarm.
    pub editing: Option<Uuid>,
}

impl AlarmForm {
    /// Reset all fields to defaults.
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Populate the form from an existing alarm for editing.
    pub fn populate_from(&mut self, alarm: &Alarm) {
        self.label = alarm.label.clone();
        self.message = alarm.message.clone().unwrap_or_default();
        self.editing = Some(alarm.id);
        match &alarm.kind {
            AlarmKind::Timer { duration_secs, .. } => {
                self.mode = AlarmFormMode::Timer;
                self.timer_minutes = format!("{}", duration_secs / 60);
            }
            AlarmKind::AtTime { target } => {
                self.mode = AlarmFormMode::Alarm;
                self.alarm_time = target.format("%H:%M").to_string();
                self.alarm_date = target.format("%Y-%m-%d").to_string();
            }
        }
    }
}

/// Custom serde module for `DateTime<Local>` as UNIX timestamps.
/// `chrono::serde::ts_seconds` only works with `DateTime<Utc>`, so we convert
/// to/from UTC when (de)serialising.
mod ts_seconds_local {
    use chrono::{DateTime, Local, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(dt: &DateTime<Local>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(dt.timestamp())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ts = i64::deserialize(deserializer)?;
        let utc = Utc
            .timestamp_opt(ts, 0)
            .single()
            .ok_or_else(|| serde::de::Error::custom("invalid timestamp"))?;
        Ok(utc.with_timezone(&Local))
    }
}

// -- Alert action ----------------------------------------------------------

/// How the user should be alerted when an alarm fires.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AlertAction {
    /// Play an audio file.
    Sound,
    /// Send a desktop notification.
    Notification,
    /// Both sound and notification.
    #[default]
    Both,
}

// -- Alarm kind ------------------------------------------------------------

/// What triggers the alarm.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum AlarmKind {
    /// Fires at a specific date and time.
    AtTime {
        /// The target date/time (with local timezone).
        #[serde(with = "ts_seconds_local")]
        target: DateTime<Local>,
    },
    /// Fires after a duration from creation (stored as target time so it
    /// survives restarts).
    Timer {
        /// Original duration in seconds (for display).
        duration_secs: u64,
        /// The computed target time.
        #[serde(with = "ts_seconds_local")]
        target: DateTime<Local>,
    },
}

impl AlarmKind {
    /// Create a timer that fires `duration_secs` from now.
    pub fn from_now(duration_secs: u64) -> Self {
        let target = Local::now() + chrono::Duration::seconds(duration_secs as i64);
        Self::Timer {
            duration_secs,
            target,
        }
    }

    /// Create an alarm at a specific date/time.
    pub fn at_time(target: DateTime<Local>) -> Self {
        Self::AtTime { target }
    }

    /// The target time, regardless of kind.
    pub fn target(&self) -> DateTime<Local> {
        match self {
            Self::AtTime { target } | Self::Timer { target, .. } => *target,
        }
    }
}

// -- Alarm struct ----------------------------------------------------------

/// A single alarm or timer entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alarm {
    /// Unique identifier.
    pub id: Uuid,
    /// User-visible label (e.g. "Tea timer", "Meeting").
    pub label: String,
    /// What triggers this alarm.
    pub kind: AlarmKind,
    /// How to alert the user.
    #[serde(default)]
    pub alert: AlertAction,
    /// Optional message shown in the notification when the alarm fires.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Whether this alarm is active.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Has this alarm already fired? (Prevents re-firing.)
    #[serde(default)]
    pub fired: bool,
}

fn default_true() -> bool {
    true
}

impl Alarm {
    /// Create a new alarm with a generated ID.
    pub fn new(label: impl Into<String>, kind: AlarmKind, alert: AlertAction) -> Self {
        Self {
            id: Uuid::new_v4(),
            label: label.into(),
            kind,
            alert,
            message: None,
            enabled: true,
            fired: false,
        }
    }

    /// Set an optional notification message.
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Returns `true` if this alarm should fire right now.
    pub fn should_fire(&self) -> bool {
        if !self.enabled || self.fired {
            return false;
        }
        Local::now() >= self.kind.target()
    }

    /// Human-readable remaining time (e.g. "2m 30s", "in 1h 5m", or "passed").
    pub fn remaining_display(&self) -> String {
        let now = Local::now();
        let target = self.kind.target();
        if self.fired {
            return "done".to_string();
        }
        if now >= target {
            return "now!".to_string();
        }
        let diff = (target - now).num_seconds();
        if diff >= 3600 {
            format!("{}h {}m", diff / 3600, (diff % 3600) / 60)
        } else if diff >= 60 {
            format!("{}m {}s", diff / 60, diff % 60)
        } else {
            format!("{diff}s")
        }
    }

    /// Short description of what kind of alarm this is.
    pub fn kind_label(&self) -> &str {
        match &self.kind {
            AlarmKind::AtTime { .. } => "alarm",
            AlarmKind::Timer { .. } => "timer",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_from_now_creates_future_target() {
        let alarm = Alarm::new("Test", AlarmKind::from_now(300), AlertAction::Both);
        let diff = (alarm.kind.target() - Local::now()).num_seconds();
        // Should be roughly 300 seconds in the future (allow 2s tolerance).
        assert!(diff >= 298 && diff <= 302, "diff was {diff}");
    }

    #[test]
    fn alarm_should_not_fire_if_disabled() {
        let mut alarm = Alarm::new("Test", AlarmKind::from_now(0), AlertAction::Both);
        alarm.enabled = false;
        assert!(!alarm.should_fire());
    }

    #[test]
    fn alarm_should_not_fire_twice() {
        let mut alarm = Alarm::new("Test", AlarmKind::from_now(0), AlertAction::Both);
        // Simulate: it just passed the target.
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(alarm.should_fire());
        alarm.fired = true;
        assert!(!alarm.should_fire());
    }

    #[test]
    fn alarm_round_trips_through_toml() {
        let alarm = Alarm::new("Tea", AlarmKind::from_now(180), AlertAction::Sound);
        let serialised = toml::to_string_pretty(&alarm).expect("serialise");
        let deser: Alarm = toml::from_str(&serialised).expect("deserialise");
        assert_eq!(deser.label, "Tea");
        assert_eq!(deser.alert, AlertAction::Sound);
    }
}
