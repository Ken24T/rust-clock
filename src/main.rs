//! Rust Clock — a classic analog clock desklet for Linux.
//!
//! Entry point: sets up the iced application with a transparent,
//! borderless window and a ticking subscription.

mod clock_face;
mod config;
mod context_menu;
mod theme;

use iced::keyboard;
use iced::widget::{canvas, stack};
use iced::{window, Color, Element, Fill, Point, Size, Subscription, Task};

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
    show_menu: bool,
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
    /// Quit the application.
    Quit,
}

impl ClockApp {
    fn new(config: AppConfig) -> Self {
        let smooth_seconds = config.smooth_seconds;
        let show_date = config.show_date;
        let theme = config.resolved_theme();
        Self {
            clock_face: ClockFace::new(theme, smooth_seconds, show_date),
            config,
            show_menu: false,
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

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.clock_face.update_time();
                Task::none()
            }
            Message::StartDrag => {
                self.show_menu = false;
                window::oldest().and_then(window::drag)
            }
            Message::WindowMoved(point) => {
                self.config.position = Some((point.x as i32, point.y as i32));
                self.save_config();
                Task::none()
            }
            Message::ToggleContextMenu => {
                self.show_menu = !self.show_menu;
                Task::none()
            }
            Message::DismissMenu => {
                self.show_menu = false;
                Task::none()
            }
            Message::SetTheme(name) => {
                self.config.theme = name;
                self.config.theme_config = None;
                self.apply_theme();
                self.save_config();
                self.show_menu = false;
                Task::none()
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
            Message::Quit => {
                self.save_config();
                window::oldest().and_then(window::close)
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let clock = canvas(&self.clock_face).width(Fill).height(Fill);

        if self.show_menu {
            let menu = ContextMenu::widget(&self.config);
            stack![clock, menu].into()
        } else {
            clock.into()
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
