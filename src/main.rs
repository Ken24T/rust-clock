//! Rust Clock — a classic analog clock desklet for Linux.
//!
//! Entry point: sets up the iced application with a transparent,
//! borderless window and a ticking subscription.

mod clock_face;
mod config;
mod theme;

use iced::widget::canvas;
use iced::{window, Color, Element, Fill, Size, Subscription, Task};

use clock_face::ClockFace;
use config::AppConfig;
use theme::ClockTheme;

pub fn main() -> iced::Result {
    let config = AppConfig::load();
    let size = config.size as f32;

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
    _config: AppConfig,
}

/// Messages produced by the application.
#[derive(Debug, Clone)]
pub enum Message {
    /// Fired every second to update the clock hands.
    Tick,
}

impl ClockApp {
    fn new(config: AppConfig) -> Self {
        Self {
            clock_face: ClockFace::new(ClockTheme::classic()),
            _config: config,
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.clock_face.update_time();
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        canvas(&self.clock_face).width(Fill).height(Fill).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::Tick)
    }
}
