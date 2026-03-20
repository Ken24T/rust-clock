//! Alarm and timer system — data model, persistence, and fire/check logic.
//!
//! Supports both one-shot and recurring reminders:
//! - **Timer**: fires after a duration from when it was created.
//! - **Alarm**: fires at a specific date/time.
//! - **Repeating timer**: fires on a fixed elapsed interval.
//! - **Repeating alarm**: fires on a local calendar schedule.
//!
//! Each alarm can trigger audio playback, a desktop notification, or both.

mod manager;
mod sound;

pub use manager::AlarmManager;
pub use sound::play_alarm_sound;

use chrono::{DateTime, Datelike, Duration, Local, LocalResult, NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Compact kind marker for items projected onto the clock face.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceActiveItemKind {
    Alarm,
    Timer,
}

/// A face-oriented summary of an active alarm or timer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FaceActiveItem {
    pub id: Uuid,
    pub label: String,
    pub description: Option<String>,
    pub kind: FaceActiveItemKind,
    pub target: DateTime<Local>,
    pub remaining_text: String,
}

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
            AlarmKind::Timer { duration_secs, .. }
            | AlarmKind::RepeatingInterval {
                interval_secs: duration_secs,
                ..
            } => {
                self.mode = AlarmFormMode::Timer;
                self.timer_minutes = format!("{}", duration_secs / 60);
            }
            AlarmKind::AtTime { target } => {
                self.mode = AlarmFormMode::Alarm;
                self.alarm_time = target.format("%H:%M").to_string();
                self.alarm_date = target.format("%Y-%m-%d").to_string();
            }
            AlarmKind::RepeatingSchedule { next_target, .. } => {
                self.mode = AlarmFormMode::Alarm;
                self.alarm_time = next_target.format("%H:%M").to_string();
                self.alarm_date.clear();
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

/// Serialise a local wall-clock time as `HH:MM`.
mod naive_time_hm {
    use chrono::NaiveTime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(time: &NaiveTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&time.format("%H:%M").to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        NaiveTime::parse_from_str(&raw, "%H:%M")
            .map_err(|e| serde::de::Error::custom(format!("invalid time '{raw}': {e}")))
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

// -- Recurrence ------------------------------------------------------------

/// Supported day-of-week values for recurring schedules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleWeekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl ScheduleWeekday {
    fn to_chrono(self) -> chrono::Weekday {
        match self {
            Self::Monday => chrono::Weekday::Mon,
            Self::Tuesday => chrono::Weekday::Tue,
            Self::Wednesday => chrono::Weekday::Wed,
            Self::Thursday => chrono::Weekday::Thu,
            Self::Friday => chrono::Weekday::Fri,
            Self::Saturday => chrono::Weekday::Sat,
            Self::Sunday => chrono::Weekday::Sun,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Monday => "Monday",
            Self::Tuesday => "Tuesday",
            Self::Wednesday => "Wednesday",
            Self::Thursday => "Thursday",
            Self::Friday => "Friday",
            Self::Saturday => "Saturday",
            Self::Sunday => "Sunday",
        }
    }
}

/// Local calendar recurrence rules for repeating alarms/events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum RecurrenceRule {
    Daily {
        #[serde(with = "naive_time_hm")]
        time: NaiveTime,
    },
    Weekdays {
        #[serde(with = "naive_time_hm")]
        time: NaiveTime,
    },
    Weekly {
        weekday: ScheduleWeekday,
        #[serde(with = "naive_time_hm")]
        time: NaiveTime,
    },
    SelectedWeekdays {
        weekdays: Vec<ScheduleWeekday>,
        #[serde(with = "naive_time_hm")]
        time: NaiveTime,
    },
}

impl RecurrenceRule {
    pub fn next_after(&self, after: DateTime<Local>) -> Option<DateTime<Local>> {
        let time = match self {
            Self::Daily { time }
            | Self::Weekdays { time }
            | Self::Weekly { time, .. }
            | Self::SelectedWeekdays { time, .. } => *time,
        };

        for offset in 0..=400 {
            let date = after.date_naive() + Duration::days(offset);
            if !self.matches_date(date) {
                continue;
            }

            if let Some(next) = resolve_local_datetime_after(date, time, after) {
                return Some(next);
            }
        }

        None
    }

    pub fn summary(&self) -> String {
        match self {
            Self::Daily { time } => format!("Daily at {}", time.format("%H:%M")),
            Self::Weekdays { time } => format!("Weekdays at {}", time.format("%H:%M")),
            Self::Weekly { weekday, time } => {
                format!("Every {} at {}", weekday.label(), time.format("%H:%M"))
            }
            Self::SelectedWeekdays { weekdays, time } => {
                let labels = weekdays
                    .iter()
                    .map(|weekday| weekday.label())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} at {}", labels, time.format("%H:%M"))
            }
        }
    }

    fn matches_date(&self, date: NaiveDate) -> bool {
        match self {
            Self::Daily { .. } => true,
            Self::Weekdays { .. } => matches!(
                date.weekday(),
                chrono::Weekday::Mon
                    | chrono::Weekday::Tue
                    | chrono::Weekday::Wed
                    | chrono::Weekday::Thu
                    | chrono::Weekday::Fri
            ),
            Self::Weekly { weekday, .. } => date.weekday() == weekday.to_chrono(),
            Self::SelectedWeekdays { weekdays, .. } => weekdays
                .iter()
                .any(|weekday| date.weekday() == weekday.to_chrono()),
        }
    }
}

fn resolve_local_datetime_after(
    date: NaiveDate,
    time: NaiveTime,
    after: DateTime<Local>,
) -> Option<DateTime<Local>> {
    let naive = date.and_time(time);
    match naive.and_local_timezone(Local) {
        LocalResult::Single(candidate) => (candidate > after).then_some(candidate),
        LocalResult::Ambiguous(early, late) => {
            if early > after {
                Some(early)
            } else if late > after {
                Some(late)
            } else {
                None
            }
        }
        LocalResult::None => None,
    }
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
    /// Repeats at a fixed elapsed interval.
    RepeatingInterval {
        /// Interval between firings in seconds.
        interval_secs: u64,
        /// The next scheduled fire time.
        #[serde(with = "ts_seconds_local")]
        next_target: DateTime<Local>,
    },
    /// Repeats according to a local calendar rule.
    RepeatingSchedule {
        schedule: RecurrenceRule,
        /// The next scheduled fire time.
        #[serde(with = "ts_seconds_local")]
        next_target: DateTime<Local>,
    },
}

impl AlarmKind {
    /// Create a timer that fires `duration_secs` from now.
    pub fn from_now(duration_secs: u64) -> Self {
        let target = Local::now() + Duration::seconds(duration_secs as i64);
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
            Self::RepeatingInterval { next_target, .. }
            | Self::RepeatingSchedule { next_target, .. } => *next_target,
        }
    }

    pub fn is_recurring(&self) -> bool {
        matches!(
            self,
            Self::RepeatingInterval { .. } | Self::RepeatingSchedule { .. }
        )
    }

    fn advance_after(&mut self, now: DateTime<Local>) -> bool {
        match self {
            Self::RepeatingInterval {
                interval_secs,
                next_target,
            } => {
                let step = Duration::seconds(*interval_secs as i64);
                while *next_target <= now {
                    *next_target += step;
                }
                true
            }
            Self::RepeatingSchedule {
                schedule,
                next_target,
            } => {
                if let Some(next) = schedule.next_after(now) {
                    *next_target = next;
                    true
                } else {
                    false
                }
            }
            Self::AtTime { .. } | Self::Timer { .. } => false,
        }
    }

    fn kind_name(&self) -> &'static str {
        match self {
            Self::AtTime { .. } | Self::RepeatingSchedule { .. } => "alarm",
            Self::Timer { .. } | Self::RepeatingInterval { .. } => "timer",
        }
    }

    fn default_label(&self) -> &'static str {
        match self {
            Self::AtTime { .. } | Self::RepeatingSchedule { .. } => "Alarm",
            Self::Timer { .. } | Self::RepeatingInterval { .. } => "Timer",
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
    /// Has this one-shot alarm already fired? (Recurring alarms advance instead.)
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

    pub fn should_fire_at(&self, now: DateTime<Local>) -> bool {
        if !self.enabled {
            return false;
        }

        if !self.kind.is_recurring() && self.fired {
            return false;
        }

        now >= self.kind.target()
    }

    pub fn advance_after_fire(&mut self, now: DateTime<Local>) -> bool {
        self.kind.advance_after(now)
    }

    /// Human-readable remaining time (e.g. "2m 30s", "in 1h 5m", or "passed").
    pub fn remaining_display(&self) -> String {
        let now = Local::now();
        let target = self.kind.target();
        if self.fired && !self.kind.is_recurring() {
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
        self.kind.kind_name()
    }

    /// Project this alarm into a compact face-visible summary when active.
    pub fn face_active_item(&self) -> Option<FaceActiveItem> {
        if !self.enabled || (self.fired && !self.kind.is_recurring()) {
            return None;
        }

        let label = if self.label.trim().is_empty() {
            self.kind.default_label().to_string()
        } else {
            self.label.trim().to_string()
        };

        Some(FaceActiveItem {
            id: self.id,
            label,
            description: self
                .message
                .as_ref()
                .map(|message| message.trim())
                .filter(|message| !message.is_empty())
                .map(ToOwned::to_owned),
            kind: match self.kind {
                AlarmKind::AtTime { .. } | AlarmKind::RepeatingSchedule { .. } => {
                    FaceActiveItemKind::Alarm
                }
                AlarmKind::Timer { .. } | AlarmKind::RepeatingInterval { .. } => {
                    FaceActiveItemKind::Timer
                }
            },
            target: self.kind.target(),
            remaining_text: self.remaining_display(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    fn first_local_datetime_on(
        weekday: chrono::Weekday,
        hour: u32,
        minute: u32,
    ) -> DateTime<Local> {
        let time = NaiveTime::from_hms_opt(hour, minute, 0).expect("valid time");
        for offset in 0..=30 {
            let date = Local::now().date_naive() + Duration::days(offset);
            if date.weekday() != weekday {
                continue;
            }

            let naive = date.and_time(time);
            match naive.and_local_timezone(Local) {
                LocalResult::Single(value) => return value,
                LocalResult::Ambiguous(early, _) => return early,
                LocalResult::None => continue,
            }
        }

        panic!("failed to find a valid local datetime for test");
    }

    #[test]
    fn timer_from_now_creates_future_target() {
        let alarm = Alarm::new("Test", AlarmKind::from_now(300), AlertAction::Both);
        let diff = (alarm.kind.target() - Local::now()).num_seconds();
        assert!(diff >= 298 && diff <= 302, "diff was {diff}");
    }

    #[test]
    fn alarm_should_not_fire_if_disabled() {
        let mut alarm = Alarm::new("Test", AlarmKind::from_now(0), AlertAction::Both);
        alarm.enabled = false;
        assert!(!alarm.should_fire_at(Local::now()));
    }

    #[test]
    fn one_shot_alarm_should_not_fire_twice() {
        let mut alarm = Alarm::new("Test", AlarmKind::from_now(0), AlertAction::Both);
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(alarm.should_fire_at(Local::now()));
        alarm.fired = true;
        assert!(!alarm.should_fire_at(Local::now()));
    }

    #[test]
    fn repeating_interval_advances_to_future_after_fire() {
        let mut alarm = Alarm::new(
            "Hourly",
            AlarmKind::RepeatingInterval {
                interval_secs: 3600,
                next_target: Local::now() - Duration::seconds(5),
            },
            AlertAction::Both,
        );
        let now = Local::now();

        assert!(alarm.should_fire_at(now));
        assert!(alarm.advance_after_fire(now));
        assert!(alarm.kind.target() > now);
        assert!(!alarm.fired);
    }

    #[test]
    fn weekdays_schedule_skips_weekend() {
        let saturday = first_local_datetime_on(chrono::Weekday::Sat, 12, 0);
        let rule = RecurrenceRule::Weekdays {
            time: NaiveTime::from_hms_opt(9, 0, 0).expect("valid time"),
        };

        let next = rule.next_after(saturday).expect("next weekday alarm");
        assert_eq!(next.weekday(), chrono::Weekday::Mon);
        assert_eq!(next.time().hour(), 9);
        assert_eq!(next.time().minute(), 0);
    }

    #[test]
    fn repeating_alarm_round_trips_through_toml() {
        let schedule = RecurrenceRule::Weekly {
            weekday: ScheduleWeekday::Friday,
            time: NaiveTime::from_hms_opt(8, 30, 0).expect("valid time"),
        };
        let next_target = schedule
            .next_after(Local::now())
            .expect("construct recurring schedule");
        let kind = AlarmKind::RepeatingSchedule {
            schedule: schedule.clone(),
            next_target,
        };
        let alarm = Alarm::new("Stand-up", kind, AlertAction::Notification);

        let serialised = toml::to_string_pretty(&alarm).expect("serialise");
        let deser: Alarm = toml::from_str(&serialised).expect("deserialise");
        assert_eq!(deser.label, "Stand-up");
        assert_eq!(deser.kind.kind_name(), "alarm");
        assert_eq!(schedule.summary(), "Every Friday at 08:30");
    }
}
