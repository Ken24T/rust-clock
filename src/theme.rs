//! Clock theme definitions — colours used for the clock face and hands.

use iced::Color;

/// All the colours that define the appearance of the clock.
#[derive(Debug, Clone)]
pub struct ClockTheme {
    /// Background fill of the circular face.
    pub face_colour: Color,
    /// Outer border ring.
    pub border_colour: Color,
    /// Minute and hour tick marks.
    pub tick_colour: Color,
    /// Arabic numerals (1–12).
    pub numeral_colour: Color,
    /// Hour hand.
    pub hour_hand_colour: Color,
    /// Minute hand.
    pub minute_hand_colour: Color,
    /// Second hand (typically a red accent).
    pub second_hand_colour: Color,
    /// Small dot at the centre where hands meet.
    pub centre_dot_colour: Color,
    /// Shadow colour for drop shadows on hands.
    pub shadow_colour: Color,
    /// Colour for the date text shown on the face.
    pub date_text_colour: Color,
}

impl ClockTheme {
    /// A traditional white-face clock with dark hands and a red second hand.
    pub fn classic() -> Self {
        Self {
            face_colour: Color::from_rgba(1.0, 1.0, 1.0, 0.90),
            border_colour: Color::from_rgb(0.20, 0.20, 0.20),
            tick_colour: Color::from_rgb(0.15, 0.15, 0.15),
            numeral_colour: Color::BLACK,
            hour_hand_colour: Color::from_rgb(0.10, 0.10, 0.10),
            minute_hand_colour: Color::from_rgb(0.15, 0.15, 0.15),
            second_hand_colour: Color::from_rgb(0.85, 0.10, 0.10),
            centre_dot_colour: Color::from_rgb(0.85, 0.10, 0.10),
            shadow_colour: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
            date_text_colour: Color::from_rgb(0.30, 0.30, 0.30),
        }
    }
}

impl Default for ClockTheme {
    fn default() -> Self {
        Self::classic()
    }
}
