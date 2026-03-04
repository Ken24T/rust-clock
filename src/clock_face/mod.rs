//! Clock face rendering using iced's canvas widget.
//!
//! Draws a classic analog clock: circular face, 60 tick marks,
//! configurable numerals, and hour/minute/second hands in various styles.

mod drawing;

use iced::mouse;
use iced::widget::canvas::{self, Cache, Geometry};
use iced::{Point, Rectangle, Renderer, Theme};

use crate::theme::ThemeConfig;

/// Holds the clock state and rendering cache.
pub struct ClockFace {
    theme: ThemeConfig,
    now: chrono::NaiveTime,
    today: chrono::NaiveDate,
    smooth_seconds: bool,
    show_date: bool,
    cache: Cache,
}

impl ClockFace {
    /// Create a new clock face with the given theme, initialised to the current time.
    pub fn new(theme: ThemeConfig, smooth_seconds: bool, show_date: bool) -> Self {
        let now = chrono::Local::now();
        Self {
            theme,
            now: now.time(),
            today: now.date_naive(),
            smooth_seconds,
            show_date,
            cache: Cache::new(),
        }
    }

    /// Refresh the stored time and invalidate the drawing cache.
    pub fn update_time(&mut self) {
        let now = chrono::Local::now();
        self.now = now.time();
        self.today = now.date_naive();
        self.cache.clear();
    }
}

// -- Canvas Program implementation ----------------------------------------

impl<Message> canvas::Program<Message> for ClockFace {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let clock = self.cache.draw(renderer, bounds.size(), |frame| {
            let centre = Point::new(bounds.width / 2.0, bounds.height / 2.0);
            let radius = bounds.width.min(bounds.height) / 2.0 * 0.95;

            self.draw_face(frame, centre, radius);
            self.draw_hands(frame, centre, radius);
        });

        vec![clock]
    }
}
