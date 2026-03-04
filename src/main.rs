//! Rust Clock — a classic analog clock desklet for Linux.
//!
//! Entry point: sets up the iced application with a transparent,
//! borderless window and a ticking subscription.

mod alarm;
mod alarm_panel;
mod clock_face;
mod config;
mod context_menu;
mod theme;

use iced::keyboard;
use iced::widget::{canvas, center, stack};
use iced::{window, Color, Element, Fill, Point, Size, Subscription, Task};

/// Minimum window size when an overlay (context menu or alarm panel) is visible.
const OVERLAY_MIN_SIZE: f32 = 300.0;
use uuid::Uuid;

use alarm::{play_alarm_sound, AlarmManager, AlertAction};
use clock_face::ClockFace;
use config::AppConfig;
use context_menu::ContextMenu;

pub fn main() -> iced::Result {
    let config = AppConfig::load();
    let size = config.size as f32;

    let position = config
        .position
        .map(|(x, y)| window::Position::Specific(Point::new(x as f32, y as f32)))
        .unwrap_or_default();

    iced::application(
        move || (ClockApp::new(config.clone()), Task::none()),
        ClockApp::update,
        ClockApp::view,
    )
    .title("Rust Clock")
    .subscription(ClockApp::subscription)
    .window(window::Settings {
        transparent: true,
        decorations: false,
        size: Size::new(size, size),
        position,
        ..Default::default()
    })
    .theme(clock_theme)
    .antialiasing(true)
    .run()
}

/// Application theme: transparent background so the desktop shows through.
fn clock_theme(_app: &ClockApp) -> iced::Theme {
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

/// Top-level application state.
struct ClockApp {
    clock_face: ClockFace,
    config: AppConfig,
    alarm_manager: AlarmManager,
    show_menu: bool,
    show_alarm_panel: bool,
}

/// Messages produced by the application.
#[derive(Debug, Clone)]
pub enum Message {
    /// Fired periodically to update the clock hands.
    Tick,
    /// Left-click: initiate OS-level window drag.
    StartDrag,
    /// Window moved to a new position — save it.
    WindowMoved(Point),
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
    /// Show the alarms & timers panel.
    ShowAlarmPanel,
    /// Dismiss the alarm panel.
    DismissAlarmPanel,
    /// Add a quick timer (duration in seconds).
    AddQuickTimer(u64),
    /// Remove an alarm by ID.
    RemoveAlarm(Uuid),
    /// Clear all fired alarms.
    ClearFiredAlarms,
    /// Quit the application.
    Quit,
}

impl ClockApp {
    fn new(config: AppConfig) -> Self {
        let smooth_seconds = config.smooth_seconds;
        let show_date = config.show_date;
        let theme = config.resolved_theme();
        let alarm_manager = AlarmManager::load();
        Self {
            clock_face: ClockFace::new(theme, smooth_seconds, show_date),
            config,
            alarm_manager,
            show_menu: false,
            show_alarm_panel: false,
        }
    }

    /// Apply the current config to the live clock face.
    fn apply_theme(&mut self) {
        let theme = self.config.resolved_theme();
        self.clock_face = ClockFace::new(theme, self.config.smooth_seconds, self.config.show_date);
    }

    /// Persist config to disk, logging any errors.
    fn save_config(&self) {
        if let Err(e) = self.config.save() {
            eprintln!("Failed to save config: {e}");
        }
    }

    /// Expand the window if the configured size is too small for an overlay.
    fn expand_for_overlay(&self) -> Task<Message> {
        let s = self.config.size as f32;
        if s < OVERLAY_MIN_SIZE {
            window::oldest()
                .and_then(|id| window::resize(id, Size::new(OVERLAY_MIN_SIZE, OVERLAY_MIN_SIZE)))
        } else {
            Task::none()
        }
    }

    /// Restore the window to the configured clock size after an overlay closes.
    fn restore_window_size(&self) -> Task<Message> {
        let s = self.config.size as f32;
        if s < OVERLAY_MIN_SIZE {
            window::oldest().and_then(move |id| window::resize(id, Size::new(s, s)))
        } else {
            Task::none()
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.clock_face.update_time();
                // Check alarms on each tick.
                let fired = self.alarm_manager.check_and_fire();
                for alarm in fired {
                    fire_alarm(&alarm);
                }
                Task::none()
            }
            Message::StartDrag => {
                let was_overlay = self.show_menu || self.show_alarm_panel;
                self.show_menu = false;
                self.show_alarm_panel = false;
                let drag = window::oldest().and_then(window::drag);
                if was_overlay {
                    Task::batch([self.restore_window_size(), drag])
                } else {
                    drag
                }
            }
            Message::WindowMoved(point) => {
                self.config.position = Some((point.x as i32, point.y as i32));
                self.save_config();
                Task::none()
            }
            Message::ToggleContextMenu => {
                self.show_alarm_panel = false;
                self.show_menu = !self.show_menu;
                if self.show_menu {
                    self.expand_for_overlay()
                } else {
                    self.restore_window_size()
                }
            }
            Message::DismissMenu => {
                self.show_menu = false;
                self.show_alarm_panel = false;
                self.restore_window_size()
            }
            Message::SetTheme(name) => {
                self.config.theme = name;
                self.config.theme_config = None;
                self.apply_theme();
                self.save_config();
                self.show_menu = false;
                self.restore_window_size()
            }
            Message::SetSize(size) => {
                self.config.size = size;
                self.save_config();
                self.show_menu = false;
                window::oldest().and_then(move |id| {
                    let s = size as f32;
                    window::resize(id, Size::new(s, s))
                })
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
            Message::ShowAlarmPanel => {
                self.show_menu = false;
                self.show_alarm_panel = true;
                self.expand_for_overlay()
            }
            Message::DismissAlarmPanel => {
                self.show_alarm_panel = false;
                self.restore_window_size()
            }
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
            Message::Quit => {
                self.save_config();
                window::oldest().and_then(window::close)
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let overlay_visible = self.show_alarm_panel || self.show_menu;
        let clock_size = self.config.size as f32;

        // When an overlay is visible and the window has been expanded,
        // constrain the clock canvas to its configured size so it doesn't
        // scale up with the larger window.
        let clock: Element<'_, Message> = if overlay_visible && clock_size < OVERLAY_MIN_SIZE {
            let sized = canvas(&self.clock_face)
                .width(clock_size)
                .height(clock_size);
            center(sized).width(Fill).height(Fill).into()
        } else {
            canvas(&self.clock_face).width(Fill).height(Fill).into()
        };

        if self.show_alarm_panel {
            let panel = alarm_panel::alarm_panel(&self.alarm_manager);
            stack![clock, panel].into()
        } else if self.show_menu {
            let menu = ContextMenu::widget(&self.config, &self.alarm_manager);
            stack![clock, menu].into()
        } else {
            clock
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let tick_interval = if self.config.smooth_seconds {
            std::time::Duration::from_millis(16) // ~60 fps
        } else {
            std::time::Duration::from_secs(1)
        };
        let tick = iced::time::every(tick_interval).map(|_| Message::Tick);

        // Listen for window move events to save position after dragging.
        let window_events = window::events().map(|(_, event)| match event {
            window::Event::Moved(point) => Message::WindowMoved(point),
            _ => Message::Tick, // Ignore other window events
        });

        // Listen for Escape key to dismiss the context menu.
        let keyboard_events = keyboard::listen().map(|event| match event {
            keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            } => Message::DismissMenu,
            _ => Message::Tick,
        });

        Subscription::batch([tick, window_events, keyboard_events])
    }
}

// -- Helper functions ------------------------------------------------------

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
    let body = match &alarm.kind {
        alarm::AlarmKind::Timer { duration_secs, .. } => {
            format!("{} timer finished", format_timer_label(*duration_secs))
        }
        alarm::AlarmKind::AtTime { target } => {
            format!("Alarm at {}", target.format("%H:%M"))
        }
    };
    if let Err(e) = notify_rust::Notification::new()
        .summary(&summary)
        .body(&body)
        .appname("Rust Clock")
        .timeout(10_000)
        .show()
    {
        eprintln!("Failed to send notification: {e}");
    }
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
