//! Clock face rendering using iced's canvas widget.
//!
//! Draws a classic analog clock: circular face, 60 tick marks,
//! Arabic numerals (1–12), and hour/minute/second hands.

use std::f32::consts::PI;

use chrono::Timelike;
use iced::alignment;
use iced::mouse;
use iced::widget::canvas::{self, stroke, Cache, Frame, Geometry, LineCap, Path, Stroke};
use iced::{Color, Point, Rectangle, Renderer, Theme};

use crate::theme::ClockTheme;

/// Holds the clock state and rendering cache.
pub struct ClockFace {
    theme: ClockTheme,
    now: chrono::NaiveTime,
    cache: Cache,
}

impl ClockFace {
    /// Create a new clock face with the given theme, initialised to the current time.
    pub fn new(theme: ClockTheme) -> Self {
        Self {
            theme,
            now: chrono::Local::now().time(),
            cache: Cache::new(),
        }
    }

    /// Refresh the stored time and invalidate the drawing cache.
    pub fn update_time(&mut self) {
        self.now = chrono::Local::now().time();
        self.cache.clear();
    }

    // -- Drawing helpers --------------------------------------------------

    /// Draw the static clock face: background circle, tick marks, and numerals.
    fn draw_face(&self, frame: &mut Frame, centre: Point, radius: f32) {
        // Face background (semi-transparent white)
        let face_circle = Path::circle(centre, radius);
        frame.fill(&face_circle, self.theme.face_colour);

        // Border ring
        frame.stroke(
            &face_circle,
            Stroke {
                style: stroke::Style::Solid(self.theme.border_colour),
                width: 2.0,
                ..Stroke::default()
            },
        );

        // 60 tick marks (thicker at the hour positions)
        for i in 0..60 {
            let angle = (i as f32) * 2.0 * PI / 60.0 - PI / 2.0;
            let is_hour_mark = i % 5 == 0;

            let inner_r = if is_hour_mark {
                radius * 0.82
            } else {
                radius * 0.88
            };
            let outer_r = radius * 0.93;
            let tick_width = if is_hour_mark { 2.5 } else { 1.0 };

            let start = Point::new(
                centre.x + inner_r * angle.cos(),
                centre.y + inner_r * angle.sin(),
            );
            let end = Point::new(
                centre.x + outer_r * angle.cos(),
                centre.y + outer_r * angle.sin(),
            );

            let tick = Path::new(|builder| {
                builder.move_to(start);
                builder.line_to(end);
            });

            frame.stroke(
                &tick,
                Stroke {
                    style: stroke::Style::Solid(self.theme.tick_colour),
                    width: tick_width,
                    ..Stroke::default()
                },
            );
        }

        // Arabic numerals 1–12
        let numeral_radius = radius * 0.72;
        let numeral_size = radius * 0.15;

        for i in 1..=12 {
            let angle = (i as f32) * 2.0 * PI / 12.0 - PI / 2.0;
            let position = Point::new(
                centre.x + numeral_radius * angle.cos(),
                centre.y + numeral_radius * angle.sin(),
            );

            frame.fill_text(canvas::Text {
                content: i.to_string(),
                position,
                size: numeral_size.into(),
                color: self.theme.numeral_colour,
                align_x: alignment::Horizontal::Center.into(),
                align_y: alignment::Vertical::Center,
                ..canvas::Text::default()
            });
        }
    }

    /// Draw the hour, minute, and second hands plus the centre dot.
    fn draw_hands(&self, frame: &mut Frame, centre: Point, radius: f32) {
        let hour = self.now.hour() as f32;
        let minute = self.now.minute() as f32;
        let second = self.now.second() as f32;

        // Hour hand — short and thick
        let hour_angle = ((hour % 12.0) + minute / 60.0) * 2.0 * PI / 12.0 - PI / 2.0;
        self.draw_hand(
            frame,
            centre,
            hour_angle,
            radius * 0.50,
            4.5,
            self.theme.hour_hand_colour,
        );

        // Minute hand — medium
        let minute_angle = (minute + second / 60.0) * 2.0 * PI / 60.0 - PI / 2.0;
        self.draw_hand(
            frame,
            centre,
            minute_angle,
            radius * 0.70,
            3.0,
            self.theme.minute_hand_colour,
        );

        // Second hand — thin, red accent
        let second_angle = second * 2.0 * PI / 60.0 - PI / 2.0;
        self.draw_hand(
            frame,
            centre,
            second_angle,
            radius * 0.78,
            1.5,
            self.theme.second_hand_colour,
        );

        // Centre dot
        let dot = Path::circle(centre, 4.0);
        frame.fill(&dot, self.theme.centre_dot_colour);
    }

    /// Draw a single clock hand from a short tail through the centre to the tip.
    fn draw_hand(
        &self,
        frame: &mut Frame,
        centre: Point,
        angle: f32,
        length: f32,
        width: f32,
        colour: Color,
    ) {
        let tip = Point::new(
            centre.x + length * angle.cos(),
            centre.y + length * angle.sin(),
        );
        let tail_length = length * 0.15;
        let tail = Point::new(
            centre.x - tail_length * angle.cos(),
            centre.y - tail_length * angle.sin(),
        );

        let hand = Path::new(|builder| {
            builder.move_to(tail);
            builder.line_to(tip);
        });

        frame.stroke(
            &hand,
            Stroke {
                style: stroke::Style::Solid(colour),
                width,
                line_cap: LineCap::Round,
                ..Stroke::default()
            },
        );
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
