//! Clock face rendering using iced's canvas widget.
//!
//! Draws a classic analog clock: circular face, 60 tick marks,
//! configurable numerals, and hour/minute/second hands in various styles.
//! Handles left-click (drag) and right-click (context menu) interactions.

mod drawing;
mod overlay;

use iced::mouse;
use iced::widget::canvas::{self, Action, Cache, Event, Geometry};
use iced::{Point, Rectangle, Renderer, Theme};

use crate::alarm::FaceActiveItem;
use crate::theme::ThemeConfig;
use crate::Message;

/// Holds the clock state and rendering cache.
pub struct ClockFace {
    theme: ThemeConfig,
    now: chrono::NaiveTime,
    today: chrono::NaiveDate,
    smooth_seconds: bool,
    show_date: bool,
    show_seconds: bool,
    active_items: Vec<FaceActiveItem>,
    cache: Cache,
}

impl ClockFace {
    /// Create a new clock face with the given theme, initialised to the current time.
    pub fn new(
        theme: ThemeConfig,
        smooth_seconds: bool,
        show_date: bool,
        show_seconds: bool,
    ) -> Self {
        let now = chrono::Local::now();
        Self {
            theme,
            now: now.time(),
            today: now.date_naive(),
            smooth_seconds,
            show_date,
            show_seconds,
            active_items: Vec::new(),
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

    /// Replace the active alarm/timer summaries available to the face overlay.
    pub fn set_active_items(&mut self, active_items: Vec<FaceActiveItem>) {
        self.active_items = active_items;
        self.cache.clear();
    }

    /// Current active alarm/timer summaries wired into the face.
    #[cfg(test)]
    pub fn active_items(&self) -> &[FaceActiveItem] {
        &self.active_items
    }
}

// -- Canvas Program implementation ----------------------------------------

impl canvas::Program<Message> for ClockFace {
    type State = ();

    fn update(
        &self,
        _state: &mut (),
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        // Only respond to events when the cursor is inside the clock bounds.
        let cursor_pos = cursor.position_in(bounds)?;

        // Check if the click is inside the circular face.
        let centre = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let radius = bounds.width.min(bounds.height) / 2.0 * 0.95;
        let dx = cursor_pos.x - centre.x;
        let dy = cursor_pos.y - centre.y;
        if dx * dx + dy * dy > radius * radius {
            return None;
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                Some(Action::publish(Message::StartDrag))
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                Some(Action::publish(Message::ToggleContextMenu))
            }
            _ => None,
        }
    }

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
            self.draw_overlay(frame, centre, radius);
        });

        vec![clock]
    }

    fn mouse_interaction(
        &self,
        _state: &(),
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            mouse::Interaction::Grab
        } else {
            mouse::Interaction::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::alarm::{Alarm, AlarmKind, AlertAction};
    use crate::theme::ThemeConfig;

    use super::ClockFace;

    #[test]
    fn set_active_items_replaces_projection() {
        let mut face = ClockFace::new(ThemeConfig::default(), false, true, true);
        let first = Alarm::new("Tea", AlarmKind::from_now(60), AlertAction::Both)
            .face_active_item()
            .expect("active alarm should project onto the clock face");
        let second = Alarm::new("Meeting", AlarmKind::from_now(300), AlertAction::Both)
            .face_active_item()
            .expect("active alarm should project onto the clock face");

        face.set_active_items(vec![first.clone()]);
        assert_eq!(face.active_items(), &[first]);

        face.set_active_items(vec![second.clone()]);
        assert_eq!(face.active_items(), &[second]);
    }
}
