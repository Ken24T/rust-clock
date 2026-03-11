//! Drawing helpers for the clock face — face, numerals, hands.
//!
//! Extracted from `clock_face/mod.rs` to keep each file under ~300 lines.

use std::f32::consts::PI;

use chrono::Timelike;

use iced::alignment;
use iced::widget::canvas::{self, stroke, Frame, LineCap, Path, Stroke};
use iced::{Color, Point};

use crate::theme::{HandStyle, NumeralStyle};

use super::ClockFace;

impl ClockFace {
    /// Draw the static clock face: background circle, tick marks, numerals, and optional date.
    pub(super) fn draw_face(&self, frame: &mut Frame, centre: Point, radius: f32) {
        // Face background
        let face_circle = Path::circle(centre, radius);
        frame.fill(&face_circle, Color::from(self.theme.face_colour));

        // Border ring
        frame.stroke(
            &face_circle,
            Stroke {
                style: stroke::Style::Solid(self.theme.border_colour.into()),
                width: self.theme.border_width,
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
            let tick_width = if is_hour_mark {
                radius * 0.021 // ~2.5 px at 250 px window
            } else {
                radius * 0.008 // ~1.0 px at 250 px window
            };

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
                    style: stroke::Style::Solid(self.theme.tick_colour.into()),
                    width: tick_width,
                    ..Stroke::default()
                },
            );
        }

        // Hour indicators (style-dependent)
        self.draw_numerals(frame, centre, radius);

        // Optional weekday and day-of-month display at the 3 o'clock position
        if self.show_date {
            let date_x = centre.x + radius * 0.38;
            let weekday_y = centre.y - radius * 0.055;
            let date_y = centre.y + radius * 0.055;
            let weekday_size = radius * 0.08;
            let date_size = radius * 0.12;

            frame.fill_text(canvas::Text {
                content: self.today.format("%a").to_string().to_uppercase(),
                position: Point::new(date_x, weekday_y),
                size: weekday_size.into(),
                color: self.theme.date_text_colour.into(),
                align_x: alignment::Horizontal::Center.into(),
                align_y: alignment::Vertical::Center,
                ..canvas::Text::default()
            });

            frame.fill_text(canvas::Text {
                content: self.today.format("%d").to_string(),
                position: Point::new(date_x, date_y),
                size: date_size.into(),
                color: self.theme.date_text_colour.into(),
                align_x: alignment::Horizontal::Center.into(),
                align_y: alignment::Vertical::Center,
                ..canvas::Text::default()
            });
        }
    }

    /// Draw hour indicators according to the configured `NumeralStyle`.
    fn draw_numerals(&self, frame: &mut Frame, centre: Point, radius: f32) {
        let numeral_radius = radius * 0.72;
        let numeral_size = radius * 0.15;
        let colour: Color = self.theme.numeral_colour.into();

        match self.theme.numeral_style {
            NumeralStyle::Arabic => {
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
                        color: colour,
                        align_x: alignment::Horizontal::Center.into(),
                        align_y: alignment::Vertical::Center,
                        ..canvas::Text::default()
                    });
                }
            }
            NumeralStyle::Roman => {
                const ROMAN: [&str; 12] = [
                    "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X", "XI", "XII",
                ];
                for (idx, label) in ROMAN.iter().enumerate() {
                    let i = idx + 1;
                    let angle = (i as f32) * 2.0 * PI / 12.0 - PI / 2.0;
                    let position = Point::new(
                        centre.x + numeral_radius * angle.cos(),
                        centre.y + numeral_radius * angle.sin(),
                    );
                    // Slightly smaller text for Roman numerals to fit
                    let size = numeral_size * 0.85;
                    frame.fill_text(canvas::Text {
                        content: (*label).to_string(),
                        position,
                        size: size.into(),
                        color: colour,
                        align_x: alignment::Horizontal::Center.into(),
                        align_y: alignment::Vertical::Center,
                        ..canvas::Text::default()
                    });
                }
            }
            NumeralStyle::Dots => {
                let dot_radius = radius * 0.02;
                for i in 1..=12 {
                    let angle = (i as f32) * 2.0 * PI / 12.0 - PI / 2.0;
                    let position = Point::new(
                        centre.x + numeral_radius * angle.cos(),
                        centre.y + numeral_radius * angle.sin(),
                    );
                    let dot = Path::circle(position, dot_radius);
                    frame.fill(&dot, colour);
                }
            }
            NumeralStyle::None => {} // No indicators
        }
    }

    /// Draw the hour, minute, and second hands with drop shadows, plus the centre dot.
    pub(super) fn draw_hands(&self, frame: &mut Frame, centre: Point, radius: f32) {
        let hour = self.now.hour() as f32;
        let minute = self.now.minute() as f32;
        let second = self.now.second() as f32;
        let nano = self.now.nanosecond() as f32;

        // Proportional dimensions (reference: 250 px window ≈ 119 px radius)
        let scale = radius / 119.0;

        // Shadow offset scales with size
        let shadow_off = 1.5 * scale;
        let shadow_centre = Point::new(centre.x + shadow_off, centre.y + shadow_off);
        let shadow_colour: Color = self.theme.shadow_colour.into();

        let hour_width = 4.5 * scale;
        let minute_width = 3.0 * scale;
        let second_width = 1.5 * scale;
        let dot_radius = 4.0 * scale;

        // Hour hand — short and thick
        let hour_angle = ((hour % 12.0) + minute / 60.0) * 2.0 * PI / 12.0 - PI / 2.0;
        self.draw_hand(
            frame,
            shadow_centre,
            hour_angle,
            radius * 0.50,
            hour_width,
            shadow_colour,
        );
        self.draw_hand(
            frame,
            centre,
            hour_angle,
            radius * 0.50,
            hour_width,
            self.theme.hour_hand_colour.into(),
        );

        // Minute hand — medium
        let minute_angle = (minute + second / 60.0) * 2.0 * PI / 60.0 - PI / 2.0;
        self.draw_hand(
            frame,
            shadow_centre,
            minute_angle,
            radius * 0.70,
            minute_width,
            shadow_colour,
        );
        self.draw_hand(
            frame,
            centre,
            minute_angle,
            radius * 0.70,
            minute_width,
            self.theme.minute_hand_colour.into(),
        );

        // Second hand — thin, accent colour, optionally smooth
        if self.show_seconds {
            let second_frac = if self.smooth_seconds {
                second + nano / 1_000_000_000.0
            } else {
                second
            };
            let second_angle = second_frac * 2.0 * PI / 60.0 - PI / 2.0;
            self.draw_hand(
                frame,
                shadow_centre,
                second_angle,
                radius * 0.78,
                second_width,
                shadow_colour,
            );
            self.draw_hand(
                frame,
                centre,
                second_angle,
                radius * 0.78,
                second_width,
                self.theme.second_hand_colour.into(),
            );
        }

        // Centre dot (shadow then real)
        let shadow_dot = Path::circle(shadow_centre, dot_radius);
        frame.fill(&shadow_dot, shadow_colour);
        let dot = Path::circle(centre, dot_radius);
        frame.fill(&dot, Color::from(self.theme.centre_dot_colour));
    }

    /// Draw a single clock hand from a short tail through the centre to the tip.
    /// The rendering varies according to `self.theme.hand_style`.
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

        match self.theme.hand_style {
            HandStyle::Classic => {
                // Uniform-width line with rounded caps
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
            HandStyle::Modern => {
                // Tapered: wide at centre, narrow at tip — drawn as a filled triangle
                let perp_x = -angle.sin();
                let perp_y = angle.cos();
                let half_base = width * 0.8;
                let half_tip = width * 0.15;

                let hand = Path::new(|builder| {
                    // Base (near centre)
                    builder.move_to(Point::new(
                        tail.x + perp_x * half_base,
                        tail.y + perp_y * half_base,
                    ));
                    builder.line_to(Point::new(
                        tail.x - perp_x * half_base,
                        tail.y - perp_y * half_base,
                    ));
                    // Tip (narrow)
                    builder.line_to(Point::new(
                        tip.x - perp_x * half_tip,
                        tip.y - perp_y * half_tip,
                    ));
                    builder.line_to(Point::new(
                        tip.x + perp_x * half_tip,
                        tip.y + perp_y * half_tip,
                    ));
                    builder.close();
                });
                frame.fill(&hand, colour);
            }
            HandStyle::Skeleton => {
                // Outlined (hollow) hand — stroked rectangle/lozenge shape
                let perp_x = -angle.sin();
                let perp_y = angle.cos();
                let half_w = width * 0.6;

                let hand = Path::new(|builder| {
                    builder.move_to(Point::new(
                        tail.x + perp_x * half_w,
                        tail.y + perp_y * half_w,
                    ));
                    builder.line_to(Point::new(tip.x + perp_x * half_w, tip.y + perp_y * half_w));
                    builder.line_to(Point::new(tip.x - perp_x * half_w, tip.y - perp_y * half_w));
                    builder.line_to(Point::new(
                        tail.x - perp_x * half_w,
                        tail.y - perp_y * half_w,
                    ));
                    builder.close();
                });
                frame.stroke(
                    &hand,
                    Stroke {
                        style: stroke::Style::Solid(colour),
                        width: (width * 0.3).max(1.0),
                        line_cap: LineCap::Round,
                        ..Stroke::default()
                    },
                );
            }
        }
    }
}
