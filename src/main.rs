//! Rust Clock — a classic analog clock desklet for Linux.
//!
//! Entry point: sets up the iced application with a transparent,
//! borderless window and a ticking subscription.

mod alarm;
mod alarm_panel;
mod clock_face;
mod config;
mod context_menu;
mod platform;
mod theme;
mod tray;

use iced::keyboard;
use iced::widget::canvas;
use iced::{window, Color, Element, Fill, Point, Size, Subscription, Task};

/// Number of early ticks during which Linux window hints are retried.
const STARTUP_HINT_ATTEMPTS: u8 = 20;
/// Retry interval for Linux startup window hints.
const STARTUP_HINT_RETRY_INTERVAL_MS: u64 = 250;
use uuid::Uuid;

use alarm::{play_alarm_sound, AlarmForm, AlarmFormMode, AlarmManager, AlertAction};
use clock_face::ClockFace;
use config::AppConfig;
use context_menu::ContextMenu;
use tray::{start_system_tray, SystemTrayHandle, TrayCommand};

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
    let position = config
        .position
        .map(|(x, y)| window::Position::Specific(Point::new(x as f32, y as f32)))
        .unwrap_or_default();

    let mut window_settings = window::Settings {
        transparent: true,
        decorations: false,
        size: Size::new(size, size),
        position,
        level: window::Level::AlwaysOnBottom,
        ..Default::default()
    };

    platform::configure_main_window_settings(&mut window_settings);

    window_settings
}

/// Application theme: transparent background so the desktop shows through.
fn clock_theme(_app: &ClockApp, _window: window::Id) -> iced::Theme {
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

fn window_title(app: &ClockApp, window: window::Id) -> String {
    if Some(window) == app.control_window {
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
    alarm_manager: AlarmManager,
    alarm_form: AlarmForm,
    startup_hint_attempts: u8,
    control_window: Option<window::Id>,
    control_window_content: Option<ControlWindowContent>,
    tray_handle: Option<SystemTrayHandle>,
    tray_receiver: Option<std::sync::mpsc::Receiver<TrayCommand>>,
}

/// Messages produced by the application.
#[derive(Debug, Clone)]
pub enum Message {
    /// Fired periodically to update the clock hands.
    Tick,
    /// Retry Linux startup window hints.
    ApplyStartupHints,
    /// A control window finished opening.
    ControlWindowOpened(window::Id),
    /// No state change is needed for this event.
    NoOp,
    /// Poll pending tray actions.
    PollTrayCommands,
    /// Left-click: initiate OS-level window drag.
    StartDrag,
    /// Window moved to a new position — save it.
    WindowMoved(window::Id, Point),
    /// User requested that a window close.
    WindowCloseRequested(window::Id),
    /// Toggle the right-click context menu.
    ToggleContextMenu,
    /// Dismiss the context menu (click-away or Escape).
    DismissMenu,
    /// Switch to a named theme preset.
    SetTheme(String),
    /// Resize the clock window.
    SetSize(u32),
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
    /// Clear all fired alarms.
    ClearFiredAlarms,
    /// Form: label text changed.
    AlarmFormLabelChanged(String),
    /// Form: message text changed.
    AlarmFormMessageChanged(String),
    /// Form: timer minutes text changed.
    AlarmFormMinutesChanged(String),
    /// Form: alarm time (HH:MM) changed.
    AlarmFormTimeChanged(String),
    /// Form: alarm date (YYYY-MM-DD) changed.
    AlarmFormDateChanged(String),
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
        let alarm_manager = AlarmManager::load();
        let (tray_handle, tray_receiver) = match start_system_tray() {
            Some((tray_handle, tray_receiver)) => (Some(tray_handle), Some(tray_receiver)),
            None => (None, None),
        };

        Self {
            clock_face: ClockFace::new(
                theme,
                config.smooth_seconds,
                config.show_date,
                config.show_seconds,
            ),
            config,
            alarm_manager,
            alarm_form: AlarmForm::default(),
            startup_hint_attempts: 0,
            control_window: None,
            control_window_content: None,
            tray_handle,
            tray_receiver,
        }
    }

    /// Apply the current config to the live clock face.
    fn apply_theme(&mut self) {
        let theme = self.config.resolved_theme();
        self.clock_face = ClockFace::new(
            theme,
            self.config.smooth_seconds,
            self.config.show_date,
            self.config.show_seconds,
        );
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
        self.control_window_content = None;

        if let Some(id) = self.control_window.take() {
            window::close(id)
        } else {
            Task::none()
        }
    }

    /// Parse the alarm form and create or update an alarm.
    fn submit_alarm_form(&mut self) {
        use alarm::{Alarm, AlarmKind, AlertAction};
        use chrono::{Local, LocalResult, NaiveDate, NaiveTime};

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
            AlarmFormMode::Timer => {
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
            AlarmFormMode::Alarm => {
                let time = match NaiveTime::parse_from_str(form.alarm_time.trim(), "%H:%M") {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("Invalid alarm time '{}': {e}", form.alarm_time);
                        return;
                    }
                };
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
            Message::NoOp => Task::none(),
            Message::Tick => {
                self.clock_face.update_time();
                // Check alarms on each tick.
                let fired = self.alarm_manager.check_and_fire();
                for alarm in fired {
                    fire_alarm(&alarm);
                }
                Task::none()
            }
            Message::PollTrayCommands => self.poll_tray_commands(),
            Message::StartDrag => {
                let drag = window::oldest().and_then(window::drag);
                Task::batch([self.close_control_window(), drag])
            }
            Message::WindowMoved(id, point) => {
                if Some(id) == self.control_window {
                    Task::none()
                } else {
                    self.config.position = Some((point.x as i32, point.y as i32));
                    self.save_config();
                    Task::none()
                }
            }
            Message::WindowCloseRequested(id) => {
                if Some(id) == self.control_window {
                    self.control_window = None;
                    self.control_window_content = None;
                    Task::none()
                } else {
                    Task::done(Message::Quit)
                }
            }
            Message::ToggleContextMenu => {
                if self.control_window_content == Some(ControlWindowContent::Menu) {
                    self.close_control_window()
                } else {
                    self.open_control_window(ControlWindowContent::Menu)
                }
            }
            Message::DismissMenu => self.close_control_window(),
            Message::SetTheme(name) => {
                self.config.theme = name;
                self.config.theme_config = None;
                self.apply_theme();
                self.save_config();
                self.close_control_window()
            }
            Message::SetSize(size) => {
                self.config.size = size;
                self.save_config();
                Task::batch([
                    self.close_control_window(),
                    window::oldest().and_then(move |id| {
                        window::resize(id, Size::new(size as f32, size as f32))
                    }),
                ])
            }
            Message::ToggleDate => {
                self.config.show_date = !self.config.show_date;
                self.apply_theme();
                self.save_config();
                Task::none()
            }
            Message::ToggleSmoothSeconds => {
                self.config.smooth_seconds = !self.config.smooth_seconds;
                self.apply_theme();
                self.save_config();
                Task::none()
            }
            Message::ToggleSeconds => {
                self.config.show_seconds = !self.config.show_seconds;
                self.apply_theme();
                self.save_config();
                Task::none()
            }
            Message::ShowAlarmPanel => self.open_control_window(ControlWindowContent::AlarmPanel),
            Message::DismissAlarmPanel => self.close_control_window(),
            Message::AddQuickTimer(secs) => {
                let label = format_timer_label(secs);
                self.alarm_manager.add_timer(label, secs);
                Task::none()
            }
            Message::RemoveAlarm(id) => {
                self.alarm_manager.remove(id);
                Task::none()
            }
            Message::ClearFiredAlarms => {
                self.alarm_manager.clear_fired();
                Task::none()
            }
            Message::EditAlarm(id) => {
                if let Some(alarm) = self.alarm_manager.get(id) {
                    self.alarm_form.populate_from(alarm);
                    return self.open_control_window(ControlWindowContent::AlarmPanel);
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
            Message::AlarmFormTimeChanged(value) => {
                self.alarm_form.alarm_time = value;
                Task::none()
            }
            Message::AlarmFormDateChanged(value) => {
                self.alarm_form.alarm_date = value;
                Task::none()
            }
            Message::AlarmFormSetMode(mode) => {
                self.alarm_form.mode = mode;
                Task::none()
            }
            Message::AlarmFormSubmit => {
                self.submit_alarm_form();
                Task::none()
            }
            Message::AlarmFormCancel => {
                self.alarm_form.clear();
                Task::none()
            }
            Message::Quit => {
                self.save_config();
                if let Some(tray_handle) = self.tray_handle.take() {
                    tray_handle.shutdown();
                }
                let mut tasks = Vec::new();

                if let Some(id) = self.control_window.take() {
                    tasks.push(window::close(id));
                }

                self.control_window_content = None;
                tasks.push(window::oldest().and_then(window::close));

                Task::batch(tasks)
            }
        }
    }

    fn view(&self, window: window::Id) -> Element<'_, Message> {
        if Some(window) == self.control_window {
            match self.control_window_content {
                Some(ControlWindowContent::AlarmPanel) => {
                    alarm_panel::alarm_panel(&self.alarm_manager, &self.alarm_form)
                }
                Some(ControlWindowContent::Menu) => {
                    ContextMenu::widget(&self.config, &self.alarm_manager)
                }
                None => canvas(&self.clock_face).width(Fill).height(Fill).into(),
            }
        } else {
            canvas(&self.clock_face).width(Fill).height(Fill).into()
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let tick_interval = if self.config.smooth_seconds {
            std::time::Duration::from_millis(16) // ~60 fps
        } else {
            std::time::Duration::from_secs(1)
        };
        let tick = iced::time::every(tick_interval).map(|_| Message::Tick);
        let startup_hint_retries = if self.startup_hint_attempts < STARTUP_HINT_ATTEMPTS {
            iced::time::every(std::time::Duration::from_millis(
                STARTUP_HINT_RETRY_INTERVAL_MS,
            ))
            .map(|_| Message::ApplyStartupHints)
        } else {
            Subscription::none()
        };
        let tray_events = iced::time::every(std::time::Duration::from_millis(150))
            .map(|_| Message::PollTrayCommands);

        // Listen for window move events to save position after dragging.
        let window_events = window::events().map(|(id, event)| match event {
            window::Event::Moved(point) => Message::WindowMoved(id, point),
            window::Event::CloseRequested => Message::WindowCloseRequested(id),
            _ => Message::NoOp,
        });

        // Listen for Escape to dismiss overlays and Ctrl+Q to quit.
        let keyboard_events = keyboard::listen().map(|event| match event {
            keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            } => Message::DismissMenu,
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

fn control_window_settings(content: ControlWindowContent, config: &AppConfig) -> window::Settings {
    let size = match content {
        ControlWindowContent::Menu => Size::new(280.0, 360.0),
        ControlWindowContent::AlarmPanel => Size::new(300.0, 520.0),
    };

    let position = config
        .position
        .map(|(x, y)| {
            window::Position::Specific(Point::new(x as f32 + config.size as f32 + 24.0, y as f32))
        })
        .unwrap_or_default();

    let mut settings = window::Settings {
        transparent: true,
        decorations: false,
        resizable: false,
        minimizable: false,
        size,
        position,
        level: window::Level::AlwaysOnTop,
        ..Default::default()
    };

    platform::configure_control_window_settings(&mut settings);

    settings
}

fn apply_startup_window_hints(id: window::Id) -> Task<Message> {
    platform::apply_startup_window_hints(id)
}

fn apply_control_window_hints(id: window::Id) -> Task<Message> {
    platform::apply_control_window_hints(id)
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
            alarm::AlarmKind::AtTime { target } => {
                format!("Alarm at {}", target.format("%H:%M"))
            }
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
