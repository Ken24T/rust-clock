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

/// Whether a timer runs once or repeats at a fixed interval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimerRepeatMode {
    #[default]
    Once,
    Repeating,
}

/// Whether an alarm is one-shot or follows a recurring schedule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlarmRepeatMode {
    #[default]
    Once,
    Daily,
    Weekdays,
    Weekly,
    SelectedWeekdays,
}

/// Editable form state for creating or editing an alarm.
#[derive(Debug, Clone, Default)]
pub struct AlarmForm {
    /// Display label (e.g. "Tea timer").
    pub label: String,
    /// Optional notification message.
    pub message: String,
    /// Duration in minutes for one-shot timers.
    pub timer_minutes: String,
    /// Interval cadence in minutes for repeating timers.
    pub timer_cadence_minutes: String,
    /// Target time as "HH:MM" (Alarm mode).
    pub alarm_time: String,
    /// Target date as "YYYY-MM-DD" (Alarm mode, blank = today).
    pub alarm_date: String,
    /// Timer or Alarm.
    pub mode: AlarmFormMode,
    /// Timer repeat behaviour.
    pub timer_repeat: TimerRepeatMode,
    /// Alarm repeat behaviour.
    pub alarm_repeat: AlarmRepeatMode,
    /// Weekday used for weekly schedules.
    pub weekly_weekday: ScheduleWeekday,
    /// Selected weekdays for custom weekly schedules.
    pub selected_weekdays: Vec<ScheduleWeekday>,
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
        self.clear();
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
                self.timer_repeat = if matches!(alarm.kind, AlarmKind::RepeatingInterval { .. }) {
                    self.timer_cadence_minutes = format!("{}", duration_secs / 60);
                    TimerRepeatMode::Repeating
                } else {
                    self.timer_minutes = format!("{}", duration_secs / 60);
                    TimerRepeatMode::Once
                };
            }
            AlarmKind::AtTime { target } => {
                self.mode = AlarmFormMode::Alarm;
                self.alarm_time = target.format("%H:%M").to_string();
                self.alarm_date = target.format("%Y-%m-%d").to_string();
                self.alarm_repeat = AlarmRepeatMode::Once;
            }
            AlarmKind::RepeatingSchedule {
                schedule,
                next_target,
            } => {
                self.mode = AlarmFormMode::Alarm;
                self.alarm_time = next_target.format("%H:%M").to_string();
                self.alarm_date.clear();
                match schedule {
                    RecurrenceRule::Daily { .. } => {
                        self.alarm_repeat = AlarmRepeatMode::Daily;
                    }
                    RecurrenceRule::Weekdays { .. } => {
                        self.alarm_repeat = AlarmRepeatMode::Weekdays;
                    }
                    RecurrenceRule::Weekly { weekday, .. } => {
                        self.alarm_repeat = AlarmRepeatMode::Weekly;
                        self.weekly_weekday = *weekday;
                    }
                    RecurrenceRule::SelectedWeekdays { weekdays, .. } => {
                        self.alarm_repeat = AlarmRepeatMode::SelectedWeekdays;
                        self.selected_weekdays = weekdays.clone();
                        if let Some(first) = weekdays.first().copied() {
                            self.weekly_weekday = first;
                        }
                    }
                }
                self.normalise_selected_weekdays();
            }
        }
    }

    pub fn toggle_selected_weekday(&mut self, weekday: ScheduleWeekday) {
        if let Some(index) = self
            .selected_weekdays
            .iter()
            .position(|day| *day == weekday)
        {
            self.selected_weekdays.remove(index);
        } else {
            self.selected_weekdays.push(weekday);
        }
        self.normalise_selected_weekdays();
    }

    fn normalise_selected_weekdays(&mut self) {
        self.selected_weekdays
            .sort_by_key(|weekday| weekday.sort_order());
        self.selected_weekdays.dedup();
    }

    pub fn sync_timer_fields_for_repeat_mode(&mut self) {
        match self.timer_repeat {
            TimerRepeatMode::Once if self.timer_minutes.trim().is_empty() => {
                self.timer_minutes = self.timer_cadence_minutes.clone();
            }
            TimerRepeatMode::Repeating if self.timer_cadence_minutes.trim().is_empty() => {
                self.timer_cadence_minutes = self.timer_minutes.clone();
            }
            TimerRepeatMode::Once | TimerRepeatMode::Repeating => {}
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

/// Temporary pause state for a reminder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum PausedState {
    /// Countdown-style reminders store the remaining time when paused.
    Countdown { remaining_secs: i64 },
    /// Clock-time reminders simply suppress firing until resumed.
    Suppressed,
}

/// Persisted restart snapshot for live countdown reminders.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum RestartSnapshot {
    Countdown { remaining_secs: i64 },
}

// -- Recurrence ------------------------------------------------------------

/// Supported day-of-week values for recurring schedules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleWeekday {
    #[default]
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl ScheduleWeekday {
    pub const ALL: [ScheduleWeekday; 7] = [
        ScheduleWeekday::Monday,
        ScheduleWeekday::Tuesday,
        ScheduleWeekday::Wednesday,
        ScheduleWeekday::Thursday,
        ScheduleWeekday::Friday,
        ScheduleWeekday::Saturday,
        ScheduleWeekday::Sunday,
    ];

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

    pub fn label(self) -> &'static str {
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

    pub fn short_label(self) -> &'static str {
        match self {
            Self::Monday => "Mon",
            Self::Tuesday => "Tue",
            Self::Wednesday => "Wed",
            Self::Thursday => "Thu",
            Self::Friday => "Fri",
            Self::Saturday => "Sat",
            Self::Sunday => "Sun",
        }
    }

    fn sort_order(self) -> u8 {
        match self {
            Self::Monday => 0,
            Self::Tuesday => 1,
            Self::Wednesday => 2,
            Self::Thursday => 3,
            Self::Friday => 4,
            Self::Saturday => 5,
            Self::Sunday => 6,
        }
    }
}

const MAX_RECURRENCE_LOOKAHEAD_DAYS: i64 = 400;

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

        for offset in 0..=MAX_RECURRENCE_LOOKAHEAD_DAYS {
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

    fn detail_text(&self) -> Option<String> {
        match self {
            Self::RepeatingInterval { interval_secs, .. } => {
                Some(format!("Every {}", format_duration(*interval_secs)))
            }
            Self::RepeatingSchedule { schedule, .. } => Some(schedule.summary()),
            Self::AtTime { target } => Some(format!("At {}", target.format("%Y-%m-%d %H:%M"))),
            Self::Timer { duration_secs, .. } => {
                Some(format!("Once after {}", format_duration(*duration_secs)))
            }
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
    /// Whether the reminder is temporarily paused by the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paused: Option<PausedState>,
    /// Persisted live countdown state used to restore timers after restart.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    restart_snapshot: Option<RestartSnapshot>,
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
            paused: None,
            restart_snapshot: None,
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
        if !self.enabled || self.is_paused() {
            return false;
        }

        if self.is_completed() {
            return false;
        }

        now >= self.kind.target()
    }

    pub fn advance_after_fire(&mut self, now: DateTime<Local>) -> bool {
        self.kind.advance_after(now)
    }

    pub fn is_paused(&self) -> bool {
        self.paused.is_some()
    }

    pub fn is_completed(&self) -> bool {
        self.fired && !self.kind.is_recurring()
    }

    pub fn is_live(&self) -> bool {
        self.enabled && !self.is_paused() && !self.is_completed()
    }

    pub fn can_pause(&self, now: DateTime<Local>) -> bool {
        if !self.enabled || self.is_paused() || self.is_completed() {
            return false;
        }

        match &self.kind {
            AlarmKind::Timer { target, .. } => *target > now,
            AlarmKind::RepeatingInterval { .. } => true,
            AlarmKind::AtTime { target } => *target > now,
            AlarmKind::RepeatingSchedule { .. } => true,
        }
    }

    pub fn can_resume(&self) -> bool {
        self.is_paused()
    }

    pub fn pause(&mut self, now: DateTime<Local>) -> bool {
        if !self.can_pause(now) {
            return false;
        }

        let paused = match &self.kind {
            AlarmKind::Timer { target, .. } => {
                let remaining = (*target - now).num_seconds();
                if remaining <= 0 {
                    return false;
                }

                PausedState::Countdown {
                    remaining_secs: remaining,
                }
            }
            AlarmKind::RepeatingInterval { next_target, .. } => {
                let remaining = (*next_target - now).num_seconds();
                let remaining = remaining.max(1);

                PausedState::Countdown {
                    remaining_secs: remaining,
                }
            }
            AlarmKind::AtTime { target } => {
                if *target <= now {
                    return false;
                }

                PausedState::Suppressed
            }
            AlarmKind::RepeatingSchedule { .. } => PausedState::Suppressed,
        };

        self.paused = Some(paused);
        true
    }

    pub fn resume_from_pause(&mut self, resumed_at: DateTime<Local>) -> bool {
        let Some(paused) = self.paused.take() else {
            return false;
        };

        match (&mut self.kind, paused) {
            (AlarmKind::Timer { target, .. }, PausedState::Countdown { remaining_secs }) => {
                if self.fired {
                    return false;
                }

                if remaining_secs <= 0 {
                    self.fired = true;
                } else {
                    *target = resumed_at + Duration::seconds(remaining_secs);
                }
            }
            (
                AlarmKind::RepeatingInterval { next_target, .. },
                PausedState::Countdown { remaining_secs },
            ) => {
                let next_in = remaining_secs.max(1);
                *next_target = resumed_at + Duration::seconds(next_in);
            }
            (AlarmKind::AtTime { target }, PausedState::Suppressed) => {
                if self.fired {
                    return false;
                }

                if *target <= resumed_at {
                    self.fired = true;
                }
            }
            (
                AlarmKind::RepeatingSchedule {
                    schedule,
                    next_target,
                },
                PausedState::Suppressed,
            ) => {
                if *next_target <= resumed_at {
                    if let Some(next) = schedule.next_after(resumed_at) {
                        *next_target = next;
                    } else {
                        self.fired = true;
                    }
                }
            }
            (AlarmKind::AtTime { target }, PausedState::Countdown { remaining_secs }) => {
                *target = resumed_at + Duration::seconds(remaining_secs.max(1));
            }
            (
                AlarmKind::RepeatingSchedule { next_target, .. },
                PausedState::Countdown { remaining_secs },
            ) => {
                *next_target = resumed_at + Duration::seconds(remaining_secs.max(1));
            }
            (AlarmKind::Timer { target, .. }, PausedState::Suppressed) => {
                if *target <= resumed_at {
                    self.fired = true;
                }
            }
            (AlarmKind::RepeatingInterval { .. }, PausedState::Suppressed) => {}
        }

        true
    }

    pub fn resume_after_restart(
        &mut self,
        paused_at: DateTime<Local>,
        resumed_at: DateTime<Local>,
    ) -> bool {
        if resumed_at <= paused_at || !self.enabled || self.is_paused() {
            return false;
        }

        match &mut self.kind {
            AlarmKind::Timer { target, .. } => {
                if self.fired {
                    return false;
                }

                let remaining = (*target - paused_at).num_seconds();

                if remaining <= 0 {
                    self.fired = true;
                } else {
                    *target = resumed_at + Duration::seconds(remaining);
                }

                true
            }
            AlarmKind::RepeatingInterval {
                interval_secs,
                next_target,
            } => {
                let remaining = (*next_target - paused_at).num_seconds();
                let next_in = if remaining > 0 {
                    remaining
                } else {
                    *interval_secs as i64
                };

                *next_target = resumed_at + Duration::seconds(next_in);
                true
            }
            AlarmKind::AtTime { target } => {
                if self.fired || *target > resumed_at {
                    return false;
                }

                self.fired = true;
                true
            }
            AlarmKind::RepeatingSchedule {
                schedule,
                next_target,
            } => {
                if *next_target > resumed_at {
                    return false;
                }

                if let Some(next) = schedule.next_after(resumed_at) {
                    *next_target = next;
                } else {
                    self.fired = true;
                }

                true
            }
        }
    }

    fn persisted_for_restart(&self, snapshot_at: DateTime<Local>) -> Self {
        let mut persisted = self.clone();
        persisted.restart_snapshot = self.restart_snapshot(snapshot_at);
        persisted
    }

    fn restart_snapshot(&self, snapshot_at: DateTime<Local>) -> Option<RestartSnapshot> {
        if !self.enabled || self.is_paused() || self.is_completed() {
            return None;
        }

        match &self.kind {
            AlarmKind::Timer { target, .. } => Some(RestartSnapshot::Countdown {
                remaining_secs: (*target - snapshot_at).num_seconds().max(1),
            }),
            AlarmKind::RepeatingInterval { next_target, .. } => Some(RestartSnapshot::Countdown {
                remaining_secs: (*next_target - snapshot_at).num_seconds().max(1),
            }),
            AlarmKind::AtTime { .. } | AlarmKind::RepeatingSchedule { .. } => None,
        }
    }

    fn restore_from_restart_snapshot(&mut self, resumed_at: DateTime<Local>) -> bool {
        let Some(snapshot) = self.restart_snapshot.take() else {
            return false;
        };

        if !self.enabled || self.is_paused() {
            return false;
        }

        match (&mut self.kind, snapshot) {
            (AlarmKind::Timer { target, .. }, RestartSnapshot::Countdown { remaining_secs }) => {
                if self.fired {
                    return false;
                }

                *target = resumed_at + Duration::seconds(remaining_secs.max(1));
                true
            }
            (
                AlarmKind::RepeatingInterval { next_target, .. },
                RestartSnapshot::Countdown { remaining_secs },
            ) => {
                *next_target = resumed_at + Duration::seconds(remaining_secs.max(1));
                true
            }
            _ => false,
        }
    }

    /// Human-readable remaining time (e.g. "2m 30s", "in 1h 5m", or "passed").
    pub fn remaining_display(&self) -> String {
        if let Some(paused) = &self.paused {
            return paused.status_text_for(self);
        }

        let now = Local::now();
        let target = self.kind.target();
        if self.is_completed() {
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

    pub fn detail_text(&self) -> Option<String> {
        self.kind.detail_text()
    }

    /// Project this alarm into a compact face-visible summary when active.
    pub fn face_active_item(&self) -> Option<FaceActiveItem> {
        if !self.is_live() {
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

impl PausedState {
    pub fn status_text_for(&self, alarm: &Alarm) -> String {
        match self {
            Self::Countdown { remaining_secs } => {
                format!(
                    "Paused with {} left",
                    format_remaining_secs(*remaining_secs)
                )
            }
            Self::Suppressed => match &alarm.kind {
                AlarmKind::AtTime { target } => {
                    format!("Paused before {}", target.format("%H:%M"))
                }
                AlarmKind::RepeatingSchedule { .. } => "Paused schedule".to_string(),
                AlarmKind::Timer { .. } | AlarmKind::RepeatingInterval { .. } => {
                    "Paused".to_string()
                }
            },
        }
    }
}

fn format_remaining_secs(remaining_secs: i64) -> String {
    let diff = remaining_secs.max(0);

    if diff >= 3600 {
        format!("{}h {}m", diff / 3600, (diff % 3600) / 60)
    } else if diff >= 60 {
        format!("{}m {}s", diff / 60, diff % 60)
    } else {
        format!("{diff}s")
    }
}

fn format_duration(secs: u64) -> String {
    if secs >= 3600 {
        let hours = secs / 3600;
        let minutes = (secs % 3600) / 60;
        if minutes > 0 {
            format!("{hours}h {minutes}m")
        } else {
            format!("{hours}h")
        }
    } else if secs >= 60 {
        format!("{}m", secs / 60)
    } else {
        format!("{secs}s")
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
    fn expired_one_shot_timer_cannot_be_paused() {
        let now = Local::now();
        let mut alarm = Alarm::new(
            "Done",
            AlarmKind::Timer {
                duration_secs: 60,
                target: now - Duration::seconds(5),
            },
            AlertAction::Both,
        );
        alarm.fired = true;

        assert!(!alarm.can_pause(now));
        assert!(!alarm.pause(now));
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
    fn one_shot_timer_pause_and_resume_preserves_remaining_time() {
        let paused_at = Local::now();
        let resumed_at = paused_at + Duration::minutes(12);
        let mut alarm = Alarm::new(
            "Tea",
            AlarmKind::Timer {
                duration_secs: 600,
                target: paused_at + Duration::minutes(8),
            },
            AlertAction::Both,
        );

        assert!(alarm.pause(paused_at));
        assert!(alarm.is_paused());
        assert_eq!(alarm.remaining_display(), "Paused with 8m 0s left");
        assert!(alarm.resume_from_pause(resumed_at));
        assert!(!alarm.is_paused());

        let remaining = (alarm.kind.target() - resumed_at).num_seconds();
        assert!(remaining >= 479 && remaining <= 481);
    }

    #[test]
    fn repeating_interval_pause_and_resume_preserves_next_fire_offset() {
        let paused_at = Local::now();
        let resumed_at = paused_at + Duration::minutes(30);
        let mut alarm = Alarm::new(
            "Stretch",
            AlarmKind::RepeatingInterval {
                interval_secs: 900,
                next_target: paused_at + Duration::minutes(4),
            },
            AlertAction::Both,
        );

        assert!(alarm.pause(paused_at));
        assert!(alarm.resume_from_pause(resumed_at));

        let remaining = (alarm.kind.target() - resumed_at).num_seconds();
        assert!(remaining >= 239 && remaining <= 241);
    }

    #[test]
    fn one_shot_alarm_resuming_after_target_marks_it_done() {
        let paused_at = Local::now();
        let resumed_at = paused_at + Duration::hours(2);
        let mut alarm = Alarm::new(
            "Meeting",
            AlarmKind::AtTime {
                target: paused_at + Duration::minutes(20),
            },
            AlertAction::Both,
        );

        assert!(alarm.pause(paused_at));
        assert!(alarm.resume_from_pause(resumed_at));
        assert!(alarm.fired);
        assert!(!alarm.should_fire_at(resumed_at));
    }

    #[test]
    fn paused_reminder_is_hidden_from_face_projection() {
        let now = Local::now();
        let mut alarm = Alarm::new(
            "Tea",
            AlarmKind::Timer {
                duration_secs: 600,
                target: now + Duration::minutes(10),
            },
            AlertAction::Both,
        );

        assert!(alarm.face_active_item().is_some());
        assert!(alarm.pause(now));
        assert!(alarm.face_active_item().is_none());
    }

    #[test]
    fn one_shot_timer_resumes_from_remaining_time_after_restart() {
        let paused_at = Local::now();
        let resumed_at = paused_at + Duration::minutes(30);
        let mut alarm = Alarm::new(
            "Tea",
            AlarmKind::Timer {
                duration_secs: 600,
                target: paused_at + Duration::minutes(8),
            },
            AlertAction::Both,
        );

        assert!(alarm.resume_after_restart(paused_at, resumed_at));

        let remaining = (alarm.kind.target() - resumed_at).num_seconds();
        assert!(
            remaining >= 479 && remaining <= 481,
            "remaining was {remaining}"
        );
        assert!(!alarm.fired);
    }

    #[test]
    fn one_shot_timer_resumes_from_persisted_restart_snapshot() {
        let snapshot_at = Local::now();
        let resumed_at = snapshot_at + Duration::hours(2);
        let alarm = Alarm::new(
            "Tea",
            AlarmKind::Timer {
                duration_secs: 600,
                target: snapshot_at + Duration::minutes(8),
            },
            AlertAction::Both,
        );
        let mut persisted = alarm.persisted_for_restart(snapshot_at);

        assert!(persisted.restore_from_restart_snapshot(resumed_at));

        let remaining = (persisted.kind.target() - resumed_at).num_seconds();
        assert!(
            remaining >= 479 && remaining <= 481,
            "remaining was {remaining}"
        );
        assert!(!persisted.fired);
    }

    #[test]
    fn repeating_interval_resumes_from_remaining_time_after_restart() {
        let paused_at = Local::now();
        let resumed_at = paused_at + Duration::minutes(20);
        let mut alarm = Alarm::new(
            "Stretch",
            AlarmKind::RepeatingInterval {
                interval_secs: 900,
                next_target: paused_at + Duration::minutes(4),
            },
            AlertAction::Both,
        );

        assert!(alarm.resume_after_restart(paused_at, resumed_at));

        let remaining = (alarm.kind.target() - resumed_at).num_seconds();
        assert!(
            remaining >= 239 && remaining <= 241,
            "remaining was {remaining}"
        );
        assert!(!alarm.fired);
    }

    #[test]
    fn repeating_interval_resumes_from_persisted_restart_snapshot() {
        let snapshot_at = Local::now();
        let resumed_at = snapshot_at + Duration::hours(3);
        let alarm = Alarm::new(
            "Stretch",
            AlarmKind::RepeatingInterval {
                interval_secs: 900,
                next_target: snapshot_at + Duration::minutes(4),
            },
            AlertAction::Both,
        );
        let mut persisted = alarm.persisted_for_restart(snapshot_at);

        assert!(persisted.restore_from_restart_snapshot(resumed_at));

        let remaining = (persisted.kind.target() - resumed_at).num_seconds();
        assert!(
            remaining >= 239 && remaining <= 241,
            "remaining was {remaining}"
        );
        assert!(!persisted.fired);
    }

    #[test]
    fn one_shot_alarm_missed_while_closed_does_not_fire_late() {
        let paused_at = Local::now();
        let resumed_at = paused_at + Duration::hours(2);
        let mut alarm = Alarm::new(
            "Meeting",
            AlarmKind::AtTime {
                target: paused_at + Duration::minutes(30),
            },
            AlertAction::Both,
        );

        assert!(alarm.resume_after_restart(paused_at, resumed_at));
        assert!(alarm.fired);
        assert!(!alarm.should_fire_at(resumed_at));
    }

    #[test]
    fn repeating_schedule_skips_missed_occurrence_while_closed() {
        let paused_at = Local::now();
        let resumed_at = paused_at + Duration::days(2);
        let time = (paused_at - Duration::minutes(1)).time();
        let schedule = RecurrenceRule::Daily { time };
        let mut alarm = Alarm::new(
            "Daily",
            AlarmKind::RepeatingSchedule {
                next_target: paused_at + Duration::hours(12),
                schedule,
            },
            AlertAction::Both,
        );

        assert!(alarm.resume_after_restart(paused_at, resumed_at));
        assert!(alarm.kind.target() > resumed_at);
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
    fn selected_weekdays_schedule_without_days_has_no_occurrence() {
        let rule = RecurrenceRule::SelectedWeekdays {
            weekdays: Vec::new(),
            time: NaiveTime::from_hms_opt(9, 0, 0).expect("valid time"),
        };

        assert!(rule.next_after(Local::now()).is_none());
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
