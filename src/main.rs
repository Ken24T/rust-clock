#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

//! Rust Clock — a classic analog clock desklet with platform-specific desktop integration.
//!
//! Entry point: sets up the iced application with a transparent,
//! borderless window and a ticking subscription.

mod alarm;
mod alarm_panel;
mod app_icon;
mod clock_face;
mod config;
mod context_menu;
mod hover_panel;
mod platform;
mod theme;
mod tray;

use iced::keyboard;
use iced::widget::{canvas, operation};
use iced::{window, Color, Element, Fill, Point, Size, Subscription, Task};

/// Number of early ticks during which Linux window hints are retried.
const STARTUP_HINT_ATTEMPTS: u8 = 20;
/// Retry interval for Linux startup window hints.
const STARTUP_HINT_RETRY_INTERVAL_MS: u64 = 250;
/// Smooth-second animation cadence. 67 ms is approximately 15 fps.
const SMOOTH_SECONDS_INTERVAL_MS: u64 = 67;
const POPUP_GAP: f32 = 18.0;
const POPUP_MARGIN: f32 = 12.0;
use uuid::Uuid;

use alarm::{
    play_alarm_sound, AlarmForm, AlarmFormMode, AlarmManager, AlarmRepeatMode, AlertAction,
    ScheduleWeekday, TimerRepeatMode,
};
use clock_face::{ClockFace, HoverWindowContent, OverlayHitTarget};
use config::{AppConfig, ClockSizePreset};
use context_menu::ContextMenu;
use platform::{start_system_tray, SystemTrayHandle, TrayCommand};
use theme::window_chrome;

pub fn main() -> iced::Result {
    let config = AppConfig::load();
    iced::daemon(
        move || {
            let app = ClockApp::new(config.clone());
            let (_id, open_task) = window::open(main_window_settings(&config));

            (app, open_task.map(|_| Message::ApplyStartupHints))
        },
        ClockApp::update,
        ClockApp::view,
    )
    .title(window_title)
    .subscription(ClockApp::subscription)
    .theme(clock_theme)
    .antialiasing(true)
    .run()
}

fn main_window_settings(config: &AppConfig) -> window::Settings {
    let size = config.size as f32;
    let position = window::Position::Specific(main_window_position(config));

    let mut window_settings = window::Settings {
        transparent: true,
        decorations: false,
        size: Size::new(size, size),
        position,
        level: window::Level::Normal,
        icon: app_window_icon(),
        ..Default::default()
    };

    platform::configure_main_window_settings(&mut window_settings);

    window_settings
}

/// Application theme: transparent background so the desktop shows through.
fn clock_theme(app: &ClockApp, window: window::Id) -> iced::Theme {
    if Some(window) == app.hover_window {
        let chrome = window_chrome(&app.config.resolved_theme());
        iced::Theme::custom(
            "Clock Window".to_string(),
            iced::theme::Palette {
                background: chrome.panel_background,
                text: chrome.text,
                primary: chrome.accent,
                success: chrome.success,
                danger: chrome.danger,
                warning: chrome.warning,
            },
        )
    } else if Some(window) == app.control_window {
        let chrome = window_chrome(&app.config.resolved_theme());
        iced::Theme::custom(
            "Clock Control Window".to_string(),
            iced::theme::Palette {
                background: Color::TRANSPARENT,
                text: chrome.text,
                primary: chrome.accent,
                success: chrome.success,
                danger: chrome.danger,
                warning: chrome.warning,
            },
        )
    } else {
        iced::Theme::custom(
            "Clock".to_string(),
            iced::theme::Palette {
                background: Color::TRANSPARENT,
                text: Color::WHITE,
                primary: Color::from_rgb(0.5, 0.5, 0.5),
                success: Color::from_rgb(0.0, 1.0, 0.0),
                danger: Color::from_rgb(1.0, 0.0, 0.0),
                warning: Color::from_rgb(1.0, 0.6, 0.0),
            },
        )
    }
}

fn window_title(app: &ClockApp, window: window::Id) -> String {
    if Some(window) == app.hover_window {
        "Rust Clock Reminder".to_string()
    } else if Some(window) == app.control_window {
        match app.control_window_content {
            Some(ControlWindowContent::Menu) => "Rust Clock Settings".to_string(),
            Some(ControlWindowContent::AlarmPanel) => "Rust Clock Alarms & Timers".to_string(),
            None => "Rust Clock".to_string(),
        }
    } else {
        "Rust Clock".to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ControlWindowContent {
    Menu,
    AlarmPanel,
}

/// Top-level application state.
struct ClockApp {
    clock_face: ClockFace,
    config: AppConfig,
    capabilities: platform::PlatformCapabilities,
    alarm_manager: AlarmManager,
    alarm_form: AlarmForm,
    startup_hint_attempts: u8,
    control_window: Option<window::Id>,
    control_window_content: Option<ControlWindowContent>,
    hover_window: Option<window::Id>,
    hover_target: Option<OverlayHitTarget>,
    hover_window_content: Option<HoverWindowContent>,
    tray_handle: Option<SystemTrayHandle>,
    tray_receiver: Option<std::sync::mpsc::Receiver<TrayCommand>>,
}

/// Messages produced by the application.
#[derive(Debug, Clone)]
enum Message {
    /// Fired periodically to update the clock hands.
    Tick,
    /// Retry Linux startup window hints.
    ApplyStartupHints,
    /// A control window finished opening.
    ControlWindowOpened(window::Id),
    /// A detached hover window finished opening.
    HoverWindowOpened(window::Id),
    /// No state change is needed for this event.
    NoOp,
    /// Poll pending tray actions.
    PollTrayCommands,
    /// Left-click: initiate OS-level window drag.
    StartDrag,
    /// Hover detail changed and may need a detached reminder window update.
    HoverWindowChanged(Option<OverlayHitTarget>),
    /// Window moved to a new position — save it.
    WindowMoved(window::Id, Point),
    /// User requested that a window close.
    WindowCloseRequested(window::Id),
    /// A window has finished closing.
    WindowClosed(window::Id),
    /// Toggle the right-click context menu.
    ToggleContextMenu,
    /// Dismiss the context menu (click-away or Escape).
    DismissMenu,
    /// Move focus to the next control in the active control window.
    FocusNextControl,
    /// Move focus to the previous control in the active control window.
    FocusPreviousControl,
    /// Switch to a named theme preset.
    SetTheme(String),
    /// Switch to a named clock size preset.
    SetSizePreset(ClockSizePreset),
    /// Adjust size relative to the preset base size.
    AdjustSize(i8),
    /// Adjust global clock opacity.
    AdjustOpacity(i8),
    /// Toggle the date display.
    ToggleDate,
    /// Toggle the smooth second hand.
    ToggleSmoothSeconds,
    /// Toggle the second hand visibility.
    ToggleSeconds,
    /// Show the alarms & timers panel.
    ShowAlarmPanel,
    /// Dismiss the alarm panel.
    DismissAlarmPanel,
    /// Add a quick timer (duration in seconds).
    AddQuickTimer(u64),
    /// Remove an alarm by ID.
    RemoveAlarm(Uuid),
    /// Edit an existing alarm — populate the form.
    EditAlarm(Uuid),
    /// Pause a running reminder by ID.
    PauseAlarm(Uuid),
    /// Pause all eligible running reminders.
    PauseAllRunning,
    /// Resume a paused reminder by ID.
    ResumeAlarm(Uuid),
    /// Resume all paused reminders.
    ResumeAllPaused,
    /// Clear all fired alarms.
    ClearFiredAlarms,
    /// Form: label text changed.
    AlarmFormLabelChanged(String),
    /// Form: message text changed.
    AlarmFormMessageChanged(String),
    /// Form: timer minutes text changed.
    AlarmFormMinutesChanged(String),
    /// Form: repeating timer cadence text changed.
    AlarmFormCadenceMinutesChanged(String),
    /// Form: alarm time (HH:MM) changed.
    AlarmFormTimeChanged(String),
    /// Form: alarm date (YYYY-MM-DD) changed.
    AlarmFormDateChanged(String),
    /// Form: switch timer between once and repeating interval.
    AlarmFormSetTimerRepeat(TimerRepeatMode),
    /// Form: switch alarm between one-shot and recurring schedule.
    AlarmFormSetAlarmRepeat(AlarmRepeatMode),
    /// Form: pick the weekday for weekly recurring alarms.
    AlarmFormSetWeeklyWeekday(ScheduleWeekday),
    /// Form: toggle a weekday for custom recurring alarms.
    AlarmFormToggleSelectedWeekday(ScheduleWeekday),
    /// Form: switch between Timer and Alarm mode.
    AlarmFormSetMode(AlarmFormMode),
    /// Form: submit (create or update).
    AlarmFormSubmit,
    /// Form: cancel editing.
    AlarmFormCancel,
    /// Quit the application.
    Quit,
}

impl ClockApp {
    fn new(config: AppConfig) -> Self {
        let theme = config.resolved_theme();
        let capabilities = platform::capabilities();
        let alarm_manager = AlarmManager::load();
        let (tray_handle, tray_receiver) = if capabilities.system_tray {
            match start_system_tray() {
                Some((tray_handle, tray_receiver)) => (Some(tray_handle), Some(tray_receiver)),
                None => (None, None),
            }
        } else {
            (None, None)
        };

        let mut app = Self {
            clock_face: ClockFace::new(
                theme,
                config.smooth_seconds,
                config.show_date,
                config.show_seconds,
            ),
            config,
            capabilities,
            alarm_manager,
            alarm_form: AlarmForm::default(),
            startup_hint_attempts: 0,
            control_window: None,
            control_window_content: None,
            hover_window: None,
            hover_target: None,
            hover_window_content: None,
            tray_handle,
            tray_receiver,
        };
        app.sync_clock_face_active_items();
        app
    }

    /// Apply the current config to the live clock face.
    fn apply_theme(&mut self) {
        let theme = self.config.resolved_clock_theme();
        self.clock_face = ClockFace::new(
            theme,
            self.config.smooth_seconds,
            self.config.show_date,
            self.config.show_seconds,
        );
        self.sync_clock_face_active_items();
    }

    fn apply_size_change(&mut self) -> Task<Message> {
        let size = self.config.size;
        let clamped_position = clamp_clock_position(
            self.config
                .position
                .map(|(x, y)| Point::new(x as f32, y as f32))
                .unwrap_or(Point::ORIGIN),
            size as f32,
        );

        self.config.position = Some((
            clamped_position.x.round() as i32,
            clamped_position.y.round() as i32,
        ));
        self.save_config();

        let mut tasks = vec![window::oldest().and_then(move |id| {
            Task::batch([
                window::move_to(id, clamped_position),
                window::resize(id, Size::new(size as f32, size as f32)),
            ])
        })];

        if let (Some(id), Some(content)) = (self.control_window, self.control_window_content) {
            tasks.push(window::move_to(
                id,
                control_window_position(content, &self.config),
            ));
        }

        if self.hover_target.is_some() {
            tasks.push(self.update_hover_window(self.hover_target));
        } else if let Some(id) = self.hover_window {
            let popup_size = self
                .hover_window_content
                .as_ref()
                .map(hover_window_size)
                .unwrap_or(Size::new(260.0, 140.0));
            tasks.push(window::move_to(
                id,
                hover_window_position(&self.config, popup_size),
            ));
        }

        Task::batch(tasks)
    }

    fn sync_clock_face_active_items(&mut self) {
        self.clock_face
            .set_active_items(self.alarm_manager.face_active_items());
    }

    /// Persist config to disk, logging any errors.
    fn save_config(&self) {
        if let Err(e) = self.config.save() {
            eprintln!("Failed to save config: {e}");
        }
    }

    fn open_control_window(&mut self, content: ControlWindowContent) -> Task<Message> {
        if self.control_window_content == Some(content) {
            if let Some(id) = self.control_window {
                return window::gain_focus(id);
            }
        }

        let mut tasks = Vec::new();

        if let Some(id) = self.control_window.take() {
            tasks.push(window::close(id));
        }

        let (id, open_task) = window::open(control_window_settings(content, &self.config));

        self.control_window = Some(id);
        self.control_window_content = Some(content);
        tasks.push(open_task.map(Message::ControlWindowOpened));

        Task::batch(tasks)
    }

    fn close_control_window(&mut self) -> Task<Message> {
        if let Some(id) = self.control_window {
            window::close(id)
        } else {
            Task::none()
        }
    }

    fn update_hover_window(&mut self, target: Option<OverlayHitTarget>) -> Task<Message> {
        self.hover_target = target;

        let Some(content) = self
            .clock_face
            .hover_window_content(clock_radius(self.config.size), target)
        else {
            return self.close_hover_window();
        };

        let size = hover_window_size(&content);
        let position = hover_window_position(&self.config, size);

        if let Some(id) = self.hover_window {
            self.hover_window_content = Some(content);
            Task::batch([window::move_to(id, position), window::resize(id, size)])
        } else {
            self.hover_window_content = Some(content);
            let (id, open_task) = window::open(hover_window_settings(
                self.hover_window_content
                    .as_ref()
                    .expect("hover content should exist before opening window"),
                &self.config,
            ));
            self.hover_window = Some(id);
            open_task.map(Message::HoverWindowOpened)
        }
    }

    fn close_hover_window(&mut self) -> Task<Message> {
        if let Some(id) = self.hover_window {
            window::close(id)
        } else {
            Task::none()
        }
    }

    /// Parse the alarm form and create or update an alarm.
    fn submit_alarm_form(&mut self) {
        use alarm::{Alarm, AlarmKind, AlertAction, RecurrenceRule};
        use chrono::{Duration, Local, LocalResult, NaiveDate, NaiveTime};

        let form = &self.alarm_form;
        let label = if form.label.trim().is_empty() {
            match form.mode {
                AlarmFormMode::Timer => "Timer".to_string(),
                AlarmFormMode::Alarm => "Alarm".to_string(),
            }
        } else {
            form.label.trim().to_string()
        };

        let message = if form.message.trim().is_empty() {
            None
        } else {
            Some(form.message.trim().to_string())
        };

        let kind = match form.mode {
            AlarmFormMode::Timer => match form.timer_repeat {
                TimerRepeatMode::Once => {
                    let minutes: u64 = match form.timer_minutes.trim().parse() {
                        Ok(m) if m > 0 => m,
                        _ => {
                            eprintln!("Invalid timer minutes: {}", form.timer_minutes);
                            return;
                        }
                    };
                    let duration_secs = match minutes.checked_mul(60) {
                        Some(secs) => secs,
                        None => {
                            eprintln!("Timer minutes are too large: {}", form.timer_minutes);
                            return;
                        }
                    };
                    AlarmKind::from_now(duration_secs)
                }
                TimerRepeatMode::Repeating => {
                    let cadence_minutes: u64 = match form.timer_cadence_minutes.trim().parse() {
                        Ok(m) if m > 0 => m,
                        _ => {
                            eprintln!(
                                "Invalid timer cadence minutes: {}",
                                form.timer_cadence_minutes
                            );
                            return;
                        }
                    };
                    let interval_secs = match cadence_minutes.checked_mul(60) {
                        Some(secs) => secs,
                        None => {
                            eprintln!(
                                "Timer cadence minutes are too large: {}",
                                form.timer_cadence_minutes
                            );
                            return;
                        }
                    };
                    AlarmKind::RepeatingInterval {
                        interval_secs,
                        next_target: Local::now() + Duration::seconds(interval_secs as i64),
                    }
                }
            },
            AlarmFormMode::Alarm => {
                let time = match NaiveTime::parse_from_str(form.alarm_time.trim(), "%H:%M") {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("Invalid alarm time '{}': {e}", form.alarm_time);
                        return;
                    }
                };
                match form.alarm_repeat {
                    AlarmRepeatMode::Once => {
                        let date = if form.alarm_date.trim().is_empty() {
                            Local::now().date_naive()
                        } else {
                            match NaiveDate::parse_from_str(form.alarm_date.trim(), "%Y-%m-%d") {
                                Ok(d) => d,
                                Err(e) => {
                                    eprintln!("Invalid alarm date '{}': {e}", form.alarm_date);
                                    return;
                                }
                            }
                        };
                        let naive_dt = date.and_time(time);
                        let target = match naive_dt.and_local_timezone(Local) {
                            LocalResult::Single(target) => target,
                            LocalResult::Ambiguous(early, late) => {
                                eprintln!(
                                    "Ambiguous local alarm time {} (DST transition), using earlier instant over {}",
                                    early, late
                                );
                                early
                            }
                            LocalResult::None => {
                                eprintln!(
                                    "Invalid local alarm time '{}' on '{}' (DST transition gap)",
                                    form.alarm_time, form.alarm_date
                                );
                                return;
                            }
                        };
                        AlarmKind::at_time(target)
                    }
                    AlarmRepeatMode::Daily => {
                        let schedule = RecurrenceRule::Daily { time };
                        let next_target = match schedule.next_after(Local::now()) {
                            Some(target) => target,
                            None => {
                                eprintln!("Could not resolve next daily alarm occurrence");
                                return;
                            }
                        };
                        AlarmKind::RepeatingSchedule {
                            schedule,
                            next_target,
                        }
                    }
                    AlarmRepeatMode::Weekdays => {
                        let schedule = RecurrenceRule::Weekdays { time };
                        let next_target = match schedule.next_after(Local::now()) {
                            Some(target) => target,
                            None => {
                                eprintln!("Could not resolve next weekday alarm occurrence");
                                return;
                            }
                        };
                        AlarmKind::RepeatingSchedule {
                            schedule,
                            next_target,
                        }
                    }
                    AlarmRepeatMode::Weekly => {
                        let schedule = RecurrenceRule::Weekly {
                            weekday: form.weekly_weekday,
                            time,
                        };
                        let next_target = match schedule.next_after(Local::now()) {
                            Some(target) => target,
                            None => {
                                eprintln!("Could not resolve next weekly alarm occurrence");
                                return;
                            }
                        };
                        AlarmKind::RepeatingSchedule {
                            schedule,
                            next_target,
                        }
                    }
                    AlarmRepeatMode::SelectedWeekdays => {
                        if form.selected_weekdays.is_empty() {
                            eprintln!("Select at least one weekday for a custom repeating alarm");
                            return;
                        }
                        let schedule = RecurrenceRule::SelectedWeekdays {
                            weekdays: form.selected_weekdays.clone(),
                            time,
                        };
                        let next_target = match schedule.next_after(Local::now()) {
                            Some(target) => target,
                            None => {
                                eprintln!(
                                    "Could not resolve next selected-weekday alarm occurrence"
                                );
                                return;
                            }
                        };
                        AlarmKind::RepeatingSchedule {
                            schedule,
                            next_target,
                        }
                    }
                }
            }
        };

        if let Some(edit_id) = form.editing {
            // Update existing alarm.
            let mut alarm = Alarm::new(label, kind, AlertAction::Both);
            alarm.id = edit_id;
            if let Some(msg) = message {
                alarm = alarm.with_message(msg);
            }
            self.alarm_manager.update(alarm);
        } else {
            // Create new alarm.
            let mut alarm = Alarm::new(label, kind, AlertAction::Both);
            if let Some(msg) = message {
                alarm = alarm.with_message(msg);
            }
            self.alarm_manager.add(alarm);
        }

        self.alarm_form.clear();
        self.sync_clock_face_active_items();
    }

    fn show_alarm_panel_from_tray(&mut self) -> Task<Message> {
        self.open_control_window(ControlWindowContent::AlarmPanel)
    }

    fn poll_tray_commands(&mut self) -> Task<Message> {
        let mut pending_commands = Vec::new();
        let mut tasks = Vec::new();
        let mut should_quit = false;

        if let Some(receiver) = &self.tray_receiver {
            loop {
                match receiver.try_recv() {
                    Ok(command) => pending_commands.push(command),
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        self.tray_receiver = None;
                        break;
                    }
                }
            }
        }

        for command in pending_commands {
            match command {
                TrayCommand::FocusClock => tasks.push(focus_clock_window()),
                TrayCommand::ShowAlarmPanel => {
                    tasks.push(self.show_alarm_panel_from_tray());
                }
                TrayCommand::AddQuickTimer(secs) => {
                    let label = format_timer_label(secs);
                    self.alarm_manager.add_timer(label, secs);
                    self.sync_clock_face_active_items();
                }
                TrayCommand::Quit => {
                    should_quit = true;
                    break;
                }
            }
        }

        if should_quit {
            Task::done(Message::Quit)
        } else if tasks.is_empty() {
            Task::none()
        } else {
            Task::batch(tasks)
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ApplyStartupHints => {
                if self.startup_hint_attempts >= STARTUP_HINT_ATTEMPTS {
                    Task::none()
                } else {
                    self.startup_hint_attempts += 1;
                    window::oldest().and_then(apply_startup_window_hints)
                }
            }
            Message::ControlWindowOpened(id) => {
                Task::batch([apply_control_window_hints(id), window::gain_focus(id)])
            }
            Message::HoverWindowOpened(id) => apply_control_window_hints(id),
            Message::NoOp => Task::none(),
            Message::Tick => {
                self.clock_face.update_time();
                // Check alarms on each tick.
                let fired = self.alarm_manager.check_and_fire();
                self.sync_clock_face_active_items();
                for alarm in fired {
                    fire_alarm(&alarm);
                }

                if self.hover_target.is_some() {
                    self.update_hover_window(self.hover_target)
                } else {
                    Task::none()
                }
            }
            Message::PollTrayCommands => self.poll_tray_commands(),
            Message::StartDrag => {
                let drag = window::oldest().and_then(window::drag);
                let close_control = self.close_control_window();
                let close_hover = self.close_hover_window();
                Task::batch([close_control, close_hover, drag])
            }
            Message::HoverWindowChanged(content) => self.update_hover_window(content),
            Message::WindowMoved(id, point) => {
                if Some(id) == self.control_window || Some(id) == self.hover_window {
                    Task::none()
                } else {
                    self.config.position = Some((point.x as i32, point.y as i32));
                    self.save_config();

                    if let Some(hover_id) = self.hover_window {
                        let popup_size = self
                            .hover_window_content
                            .as_ref()
                            .map(hover_window_size)
                            .unwrap_or(Size::new(260.0, 140.0));
                        window::move_to(hover_id, hover_window_position(&self.config, popup_size))
                    } else {
                        Task::none()
                    }
                }
            }
            Message::WindowCloseRequested(id) => {
                if Some(id) == self.control_window || Some(id) == self.hover_window {
                    window::close(id)
                } else {
                    Task::done(Message::Quit)
                }
            }
            Message::WindowClosed(id) => {
                if Some(id) == self.control_window {
                    self.control_window = None;
                    self.control_window_content = None;
                    Task::none()
                } else if Some(id) == self.hover_window {
                    self.hover_window = None;
                    self.hover_target = None;
                    self.hover_window_content = None;
                    Task::none()
                } else {
                    Task::none()
                }
            }
            Message::ToggleContextMenu => {
                let close_hover = self.close_hover_window();
                if self.control_window_content == Some(ControlWindowContent::Menu) {
                    Task::batch([close_hover, self.close_control_window()])
                } else {
                    Task::batch([
                        close_hover,
                        self.open_control_window(ControlWindowContent::Menu),
                    ])
                }
            }
            Message::DismissMenu => {
                let close_control = self.close_control_window();
                let close_hover = self.close_hover_window();
                Task::batch([close_control, close_hover])
            }
            Message::FocusNextControl => {
                if self.control_window.is_some() {
                    operation::focus_next()
                } else {
                    Task::none()
                }
            }
            Message::FocusPreviousControl => {
                if self.control_window.is_some() {
                    operation::focus_previous()
                } else {
                    Task::none()
                }
            }
            Message::SetTheme(name) => {
                self.config.theme = name;
                self.config.theme_config = None;
                self.apply_theme();
                self.save_config();
                Task::none()
            }
            Message::SetSizePreset(preset) => {
                self.config.set_size_preset(preset);
                self.save_config();
                self.apply_size_change()
            }
            Message::AdjustSize(delta) => {
                if self.config.adjust_size_adjust_percent(delta) {
                    self.save_config();
                    self.apply_size_change()
                } else {
                    Task::none()
                }
            }
            Message::AdjustOpacity(delta) => {
                if self.config.adjust_opacity_percent(delta) {
                    self.apply_theme();
                    self.save_config();
                }

                Task::none()
            }
            Message::ToggleDate => {
                self.config.show_date = !self.config.show_date;
                self.apply_theme();
                self.save_config();
                self.close_hover_window()
            }
            Message::ToggleSmoothSeconds => {
                self.config.smooth_seconds = !self.config.smooth_seconds;
                self.apply_theme();
                self.save_config();
                self.close_hover_window()
            }
            Message::ToggleSeconds => {
                self.config.show_seconds = !self.config.show_seconds;
                self.apply_theme();
                self.save_config();
                self.close_hover_window()
            }
            Message::ShowAlarmPanel => {
                let close_hover = self.close_hover_window();
                Task::batch([
                    close_hover,
                    self.open_control_window(ControlWindowContent::AlarmPanel),
                ])
            }
            Message::DismissAlarmPanel => {
                let close_control = self.close_control_window();
                let close_hover = self.close_hover_window();
                Task::batch([close_control, close_hover])
            }
            Message::AddQuickTimer(secs) => {
                let label = format_timer_label(secs);
                self.alarm_manager.add_timer(label, secs);
                self.sync_clock_face_active_items();
                self.close_hover_window()
            }
            Message::RemoveAlarm(id) => {
                self.alarm_manager.remove(id);
                self.sync_clock_face_active_items();
                self.close_hover_window()
            }
            Message::ClearFiredAlarms => {
                self.alarm_manager.clear_fired();
                self.sync_clock_face_active_items();
                self.close_hover_window()
            }
            Message::EditAlarm(id) => {
                if let Some(alarm) = self.alarm_manager.get(id) {
                    self.alarm_form.populate_from(alarm);
                    return self.open_control_window(ControlWindowContent::AlarmPanel);
                }
                Task::none()
            }
            Message::PauseAlarm(id) => {
                if self.alarm_manager.pause(id) {
                    self.sync_clock_face_active_items();
                    return self.close_hover_window();
                }

                Task::none()
            }
            Message::PauseAllRunning => {
                if self.alarm_manager.pause_all_running() > 0 {
                    self.sync_clock_face_active_items();
                    return self.close_hover_window();
                }

                Task::none()
            }
            Message::ResumeAlarm(id) => {
                if self.alarm_manager.resume(id) {
                    self.sync_clock_face_active_items();
                    return self.close_hover_window();
                }

                Task::none()
            }
            Message::ResumeAllPaused => {
                if self.alarm_manager.resume_all_paused() > 0 {
                    self.sync_clock_face_active_items();
                    return self.close_hover_window();
                }

                Task::none()
            }
            Message::AlarmFormLabelChanged(value) => {
                self.alarm_form.label = value;
                Task::none()
            }
            Message::AlarmFormMessageChanged(value) => {
                self.alarm_form.message = value;
                Task::none()
            }
            Message::AlarmFormMinutesChanged(value) => {
                self.alarm_form.timer_minutes = value;
                Task::none()
            }
            Message::AlarmFormCadenceMinutesChanged(value) => {
                self.alarm_form.timer_cadence_minutes = value;
                Task::none()
            }
            Message::AlarmFormTimeChanged(value) => {
                self.alarm_form.alarm_time = value;
                Task::none()
            }
            Message::AlarmFormDateChanged(value) => {
                self.alarm_form.alarm_date = value;
                Task::none()
            }
            Message::AlarmFormSetTimerRepeat(mode) => {
                self.alarm_form.timer_repeat = mode;
                self.alarm_form.sync_timer_fields_for_repeat_mode();
                Task::none()
            }
            Message::AlarmFormSetAlarmRepeat(mode) => {
                self.alarm_form.alarm_repeat = mode;
                Task::none()
            }
            Message::AlarmFormSetWeeklyWeekday(weekday) => {
                self.alarm_form.weekly_weekday = weekday;
                Task::none()
            }
            Message::AlarmFormToggleSelectedWeekday(weekday) => {
                self.alarm_form.toggle_selected_weekday(weekday);
                Task::none()
            }
            Message::AlarmFormSetMode(mode) => {
                self.alarm_form.mode = mode;
                Task::none()
            }
            Message::AlarmFormSubmit => {
                self.submit_alarm_form();
                self.close_hover_window()
            }
            Message::AlarmFormCancel => {
                self.alarm_form.clear();
                self.close_hover_window()
            }
            Message::Quit => {
                self.save_config();
                self.alarm_manager.save_for_shutdown();
                if let Some(tray_handle) = self.tray_handle.take() {
                    tray_handle.shutdown();
                }
                self.control_window = None;
                self.hover_window = None;
                self.control_window_content = None;
                self.hover_target = None;
                self.hover_window_content = None;

                iced::exit()
            }
        }
    }

    fn view(&self, window: window::Id) -> Element<'_, Message> {
        let chrome = window_chrome(&self.config.resolved_theme());

        if Some(window) == self.hover_window {
            if let Some(content) = &self.hover_window_content {
                hover_panel::hover_panel(content, chrome)
            } else {
                iced::widget::text("").into()
            }
        } else if Some(window) == self.control_window {
            match self.control_window_content {
                Some(ControlWindowContent::AlarmPanel) => {
                    alarm_panel::alarm_panel(&self.alarm_manager, &self.alarm_form, chrome)
                }
                Some(ControlWindowContent::Menu) => {
                    ContextMenu::widget(&self.config, &self.alarm_manager, chrome)
                }
                None => canvas(&self.clock_face).width(Fill).height(Fill).into(),
            }
        } else {
            canvas(&self.clock_face).width(Fill).height(Fill).into()
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let tick_interval = if self.config.smooth_seconds {
            std::time::Duration::from_millis(SMOOTH_SECONDS_INTERVAL_MS) // ~15 fps
        } else {
            std::time::Duration::from_secs(1)
        };
        let tick = iced::time::every(tick_interval).map(|_| Message::Tick);
        let startup_hint_retries = if self.capabilities.desktop_window_hints
            && self.startup_hint_attempts < STARTUP_HINT_ATTEMPTS
        {
            iced::time::every(std::time::Duration::from_millis(
                STARTUP_HINT_RETRY_INTERVAL_MS,
            ))
            .map(|_| Message::ApplyStartupHints)
        } else {
            Subscription::none()
        };
        let tray_events = if self.capabilities.system_tray && self.tray_receiver.is_some() {
            iced::time::every(std::time::Duration::from_millis(150))
                .map(|_| Message::PollTrayCommands)
        } else {
            Subscription::none()
        };

        // Listen for window move events to save position after dragging.
        let window_events = window::events().map(|(id, event)| match event {
            window::Event::Moved(point) => Message::WindowMoved(id, point),
            window::Event::CloseRequested => Message::WindowCloseRequested(id),
            window::Event::Closed => Message::WindowClosed(id),
            _ => Message::NoOp,
        });

        // Listen for Escape to dismiss overlays, Tab to traverse controls, and Ctrl+Q to quit.
        let keyboard_events = keyboard::listen().map(|event| match event {
            keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            } => Message::DismissMenu,
            keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Tab),
                modifiers,
                ..
            } => {
                if modifiers.shift() {
                    Message::FocusPreviousControl
                } else {
                    Message::FocusNextControl
                }
            }
            keyboard::Event::KeyPressed {
                key,
                modifiers,
                physical_key,
                repeat,
                ..
            } if !repeat && modifiers.command() && key.to_latin(physical_key) == Some('q') => {
                Message::Quit
            }
            _ => Message::NoOp,
        });

        Subscription::batch([
            tick,
            startup_hint_retries,
            tray_events,
            window_events,
            keyboard_events,
        ])
    }
}

// -- Helper functions ------------------------------------------------------

fn focus_clock_window() -> Task<Message> {
    window::oldest()
        .and_then(|id| Task::batch([window::minimize(id, false), window::gain_focus(id)]))
}

fn main_window_position(config: &AppConfig) -> Point {
    let anchor = config
        .position
        .map(|(x, y)| Point::new(x as f32, y as f32))
        .unwrap_or(Point::ORIGIN);

    clamp_clock_position(anchor, config.size as f32)
}

fn control_window_size(content: ControlWindowContent) -> Size {
    match content {
        ControlWindowContent::Menu => Size::new(300.0, 420.0),
        ControlWindowContent::AlarmPanel => Size::new(300.0, 580.0),
    }
}

fn control_window_settings(content: ControlWindowContent, config: &AppConfig) -> window::Settings {
    let size = control_window_size(content);

    let mut settings = window::Settings {
        transparent: true,
        decorations: false,
        resizable: false,
        minimizable: false,
        size,
        position: window::Position::Specific(control_window_position(content, config)),
        level: window::Level::Normal,
        icon: app_window_icon(),
        ..Default::default()
    };

    platform::configure_control_window_settings(&mut settings);

    settings
}

fn hover_window_settings(content: &HoverWindowContent, config: &AppConfig) -> window::Settings {
    let size = hover_window_size(content);
    let mut settings = window::Settings {
        transparent: false,
        decorations: false,
        resizable: false,
        minimizable: false,
        size,
        position: window::Position::Specific(hover_window_position(config, size)),
        level: window::Level::AlwaysOnTop,
        icon: app_window_icon(),
        ..Default::default()
    };

    platform::configure_control_window_settings(&mut settings);

    settings
}

fn hover_window_size(content: &HoverWindowContent) -> Size {
    let longest_line = std::iter::once(content.title.chars().count())
        .chain(content.detail_lines.iter().map(|line| line.chars().count()))
        .max()
        .unwrap_or(18) as f32;
    let width = (longest_line * 7.2 + 44.0).clamp(220.0, 360.0);
    let height = (52.0 + content.detail_lines.len() as f32 * 22.0).clamp(84.0, 260.0);

    Size::new(width, height)
}

fn hover_window_position(config: &AppConfig, popup_size: Size) -> Point {
    popup_position(config, popup_size, config.size as f32 * 0.24)
}

fn control_window_position(content: ControlWindowContent, config: &AppConfig) -> Point {
    popup_position(config, control_window_size(content), 0.0)
}

fn popup_position(config: &AppConfig, popup_size: Size, y_offset: f32) -> Point {
    let clock_size = config.size as f32;
    let anchor = config
        .position
        .map(|(x, y)| Point::new(x as f32, y as f32))
        .unwrap_or(Point::ORIGIN);
    let monitor_point = Point::new(anchor.x + clock_size * 0.5, anchor.y + clock_size * 0.5);

    let desired_right_x = anchor.x + clock_size + POPUP_GAP;
    let desired_left_x = anchor.x - popup_size.width - POPUP_GAP;
    let desired_y = anchor.y + y_offset;

    if let Some(work_area) = platform::work_area_for_point(monitor_point.x, monitor_point.y) {
        let min_x = work_area.x + POPUP_MARGIN;
        let max_x = work_area.x + work_area.width - popup_size.width - POPUP_MARGIN;
        let min_y = work_area.y + POPUP_MARGIN;
        let max_y = work_area.y + work_area.height - popup_size.height - POPUP_MARGIN;

        let right_fits = desired_right_x <= max_x;
        let left_fits = desired_left_x >= min_x;

        let x = if right_fits {
            desired_right_x
        } else if left_fits {
            desired_left_x
        } else {
            desired_right_x.clamp(min_x, max_x.max(min_x))
        };
        let y = desired_y.clamp(min_y, max_y.max(min_y));

        Point::new(x, y)
    } else {
        let x = if desired_left_x >= POPUP_MARGIN {
            desired_left_x.max(POPUP_MARGIN)
        } else {
            desired_right_x.max(POPUP_MARGIN)
        };
        let y = desired_y.max(POPUP_MARGIN);

        Point::new(x, y)
    }
}

fn clamp_clock_position(anchor: Point, size: f32) -> Point {
    let monitor_point = Point::new(anchor.x + size * 0.5, anchor.y + size * 0.5);

    if let Some(work_area) = platform::work_area_for_point(monitor_point.x, monitor_point.y) {
        let min_x = work_area.x + POPUP_MARGIN;
        let max_x = work_area.x + work_area.width - size - POPUP_MARGIN;
        let min_y = work_area.y + POPUP_MARGIN;
        let max_y = work_area.y + work_area.height - size - POPUP_MARGIN;

        Point::new(
            anchor.x.clamp(min_x, max_x.max(min_x)),
            anchor.y.clamp(min_y, max_y.max(min_y)),
        )
    } else {
        Point::new(anchor.x.max(POPUP_MARGIN), anchor.y.max(POPUP_MARGIN))
    }
}

fn clock_radius(size: u32) -> f32 {
    size as f32 / 2.0 * 0.95
}

fn apply_startup_window_hints(id: window::Id) -> Task<Message> {
    platform::apply_startup_window_hints(id)
}

fn apply_control_window_hints(id: window::Id) -> Task<Message> {
    platform::apply_control_window_hints(id)
}

fn app_window_icon() -> Option<window::Icon> {
    window::icon::from_rgba(
        app_icon::clock_face_icon_rgba(app_icon::CLOCK_FACE_ICON_SIZE),
        app_icon::CLOCK_FACE_ICON_SIZE,
        app_icon::CLOCK_FACE_ICON_SIZE,
    )
    .ok()
}

/// Fire an alarm: play sound and/or send a notification based on alert action.
fn fire_alarm(alarm: &alarm::Alarm) {
    match &alarm.alert {
        AlertAction::Sound => {
            play_alarm_sound(None);
        }
        AlertAction::Notification => {
            send_notification(alarm);
        }
        AlertAction::Both => {
            play_alarm_sound(None);
            send_notification(alarm);
        }
    }
}

/// Send a desktop notification for a fired alarm.
fn send_notification(alarm: &alarm::Alarm) {
    let summary = format!("⏰ {}", alarm.label);
    let body = if let Some(msg) = &alarm.message {
        msg.clone()
    } else {
        match &alarm.kind {
            alarm::AlarmKind::Timer { duration_secs, .. } => {
                format!("{} timer finished", format_timer_label(*duration_secs))
            }
            alarm::AlarmKind::RepeatingInterval { interval_secs, .. } => {
                format!(
                    "{} repeating timer fired",
                    format_timer_label(*interval_secs)
                )
            }
            alarm::AlarmKind::AtTime { target } => {
                format!("Alarm at {}", target.format("%H:%M"))
            }
            alarm::AlarmKind::RepeatingSchedule { schedule, .. } => schedule.summary(),
        }
    };
    platform::send_notification(&summary, &body);
}

/// Human-friendly label for a timer duration.
fn format_timer_label(secs: u64) -> String {
    if secs >= 3600 {
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        if m > 0 {
            format!("{h}h {m}m timer")
        } else {
            format!("{h}h timer")
        }
    } else if secs >= 60 {
        format!("{}m timer", secs / 60)
    } else {
        format!("{secs}s timer")
    }
}
