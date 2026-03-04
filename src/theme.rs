//! Clock theme definitions — colours and styles for the clock face and hands.
//!
//! Themes can be loaded from TOML configuration or selected by name from
//! the set of built-in presets.

use iced::Color;
use serde::{Deserialize, Serialize};

// -- Style enums ----------------------------------------------------------

/// How the hour numerals are displayed on the face.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NumeralStyle {
    /// Western Arabic numerals (1–12).
    #[default]
    Arabic,
    /// Roman numerals (I–XII).
    Roman,
    /// Small dots at hour positions instead of numbers.
    Dots,
    /// No hour indicators at all.
    None,
}

/// Visual style of the clock hands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum HandStyle {
    /// Uniform-width hands with rounded caps.
    #[default]
    Classic,
    /// Tapered hands — wider at the centre, narrowing to the tip.
    Modern,
    /// Outlined (hollow) hands.
    Skeleton,
}

// -- Serialisable colour helper -------------------------------------------

/// A colour represented as `[r, g, b, a]` floats (0.0–1.0) for TOML serialisation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Colour(pub [f32; 4]);

impl Colour {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self([r, g, b, a])
    }
}

impl From<Colour> for Color {
    fn from(c: Colour) -> Self {
        Color::from_rgba(c.0[0], c.0[1], c.0[2], c.0[3])
    }
}

impl From<Color> for Colour {
    fn from(c: Color) -> Self {
        Self([c.r, c.g, c.b, c.a])
    }
}

// -- Theme configuration (TOML-serialisable) ------------------------------

/// All the colours and styles that define the appearance of the clock.
///
/// This struct is designed to be embedded as `[theme]` inside `AppConfig`
/// and will round-trip through TOML cleanly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Background fill of the circular face.
    #[serde(default = "defaults::face_colour")]
    pub face_colour: Colour,
    /// Outer border ring.
    #[serde(default = "defaults::border_colour")]
    pub border_colour: Colour,
    /// Minute and hour tick marks.
    #[serde(default = "defaults::tick_colour")]
    pub tick_colour: Colour,
    /// Hour numerals.
    #[serde(default = "defaults::numeral_colour")]
    pub numeral_colour: Colour,
    /// Hour hand.
    #[serde(default = "defaults::hour_hand_colour")]
    pub hour_hand_colour: Colour,
    /// Minute hand.
    #[serde(default = "defaults::minute_hand_colour")]
    pub minute_hand_colour: Colour,
    /// Second hand (typically an accent colour).
    #[serde(default = "defaults::second_hand_colour")]
    pub second_hand_colour: Colour,
    /// Small dot at the centre where hands meet.
    #[serde(default = "defaults::centre_dot_colour")]
    pub centre_dot_colour: Colour,
    /// Drop shadow colour for hands.
    #[serde(default = "defaults::shadow_colour")]
    pub shadow_colour: Colour,
    /// Date text colour.
    #[serde(default = "defaults::date_text_colour")]
    pub date_text_colour: Colour,
    /// Border ring width in logical pixels.
    #[serde(default = "defaults::border_width")]
    pub border_width: f32,

    /// How hour positions are labelled.
    #[serde(default)]
    pub numeral_style: NumeralStyle,
    /// Visual style of the hands.
    #[serde(default)]
    pub hand_style: HandStyle,
}

// -- Serde default functions (classic theme values) -----------------------

mod defaults {
    use super::Colour;

    pub fn face_colour() -> Colour {
        Colour::new(1.0, 1.0, 1.0, 0.90)
    }
    pub fn border_colour() -> Colour {
        Colour::new(0.20, 0.20, 0.20, 1.0)
    }
    pub fn tick_colour() -> Colour {
        Colour::new(0.15, 0.15, 0.15, 1.0)
    }
    pub fn numeral_colour() -> Colour {
        Colour::new(0.0, 0.0, 0.0, 1.0)
    }
    pub fn hour_hand_colour() -> Colour {
        Colour::new(0.10, 0.10, 0.10, 1.0)
    }
    pub fn minute_hand_colour() -> Colour {
        Colour::new(0.15, 0.15, 0.15, 1.0)
    }
    pub fn second_hand_colour() -> Colour {
        Colour::new(0.85, 0.10, 0.10, 1.0)
    }
    pub fn centre_dot_colour() -> Colour {
        Colour::new(0.85, 0.10, 0.10, 1.0)
    }
    pub fn shadow_colour() -> Colour {
        Colour::new(0.0, 0.0, 0.0, 0.25)
    }
    pub fn date_text_colour() -> Colour {
        Colour::new(0.30, 0.30, 0.30, 1.0)
    }
    pub fn border_width() -> f32 {
        2.0
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self::classic()
    }
}

// -- Built-in theme presets -----------------------------------------------

impl ThemeConfig {
    /// Classic — white face, dark hands, red second hand.
    pub fn classic() -> Self {
        Self {
            face_colour: defaults::face_colour(),
            border_colour: defaults::border_colour(),
            tick_colour: defaults::tick_colour(),
            numeral_colour: defaults::numeral_colour(),
            hour_hand_colour: defaults::hour_hand_colour(),
            minute_hand_colour: defaults::minute_hand_colour(),
            second_hand_colour: defaults::second_hand_colour(),
            centre_dot_colour: defaults::centre_dot_colour(),
            shadow_colour: defaults::shadow_colour(),
            date_text_colour: defaults::date_text_colour(),
            border_width: defaults::border_width(),
            numeral_style: NumeralStyle::Arabic,
            hand_style: HandStyle::Classic,
        }
    }

    /// Dark — dark face, light hands, cyan second hand.
    pub fn dark() -> Self {
        Self {
            face_colour: Colour::new(0.12, 0.12, 0.15, 0.92),
            border_colour: Colour::new(0.40, 0.40, 0.45, 1.0),
            tick_colour: Colour::new(0.60, 0.60, 0.65, 1.0),
            numeral_colour: Colour::new(0.85, 0.85, 0.85, 1.0),
            hour_hand_colour: Colour::new(0.90, 0.90, 0.90, 1.0),
            minute_hand_colour: Colour::new(0.80, 0.80, 0.82, 1.0),
            second_hand_colour: Colour::new(0.0, 0.85, 0.85, 1.0),
            centre_dot_colour: Colour::new(0.0, 0.85, 0.85, 1.0),
            shadow_colour: Colour::new(0.0, 0.0, 0.0, 0.40),
            date_text_colour: Colour::new(0.65, 0.65, 0.65, 1.0),
            border_width: 2.0,
            numeral_style: NumeralStyle::Arabic,
            hand_style: HandStyle::Classic,
        }
    }

    /// Minimal — no numerals, thin markers, grey tones.
    pub fn minimal() -> Self {
        Self {
            face_colour: Colour::new(0.95, 0.95, 0.95, 0.85),
            border_colour: Colour::new(0.70, 0.70, 0.70, 1.0),
            tick_colour: Colour::new(0.55, 0.55, 0.55, 1.0),
            numeral_colour: Colour::new(0.55, 0.55, 0.55, 1.0),
            hour_hand_colour: Colour::new(0.35, 0.35, 0.35, 1.0),
            minute_hand_colour: Colour::new(0.45, 0.45, 0.45, 1.0),
            second_hand_colour: Colour::new(0.55, 0.55, 0.55, 1.0),
            centre_dot_colour: Colour::new(0.45, 0.45, 0.45, 1.0),
            shadow_colour: Colour::new(0.0, 0.0, 0.0, 0.15),
            date_text_colour: Colour::new(0.55, 0.55, 0.55, 1.0),
            border_width: 1.0,
            numeral_style: NumeralStyle::None,
            hand_style: HandStyle::Modern,
        }
    }

    /// Transparent — no face fill, outline-only ticks, ghost hands.
    pub fn transparent() -> Self {
        Self {
            face_colour: Colour::new(1.0, 1.0, 1.0, 0.05),
            border_colour: Colour::new(1.0, 1.0, 1.0, 0.30),
            tick_colour: Colour::new(1.0, 1.0, 1.0, 0.50),
            numeral_colour: Colour::new(1.0, 1.0, 1.0, 0.60),
            hour_hand_colour: Colour::new(1.0, 1.0, 1.0, 0.70),
            minute_hand_colour: Colour::new(1.0, 1.0, 1.0, 0.55),
            second_hand_colour: Colour::new(1.0, 0.40, 0.40, 0.60),
            centre_dot_colour: Colour::new(1.0, 0.40, 0.40, 0.60),
            shadow_colour: Colour::new(0.0, 0.0, 0.0, 0.10),
            date_text_colour: Colour::new(1.0, 1.0, 1.0, 0.50),
            border_width: 1.5,
            numeral_style: NumeralStyle::Dots,
            hand_style: HandStyle::Skeleton,
        }
    }

    /// Look up a built-in theme by name, falling back to Classic.
    pub fn by_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "classic" => Self::classic(),
            "dark" => Self::dark(),
            "minimal" => Self::minimal(),
            "transparent" => Self::transparent(),
            other => {
                eprintln!("Unknown theme \"{other}\", falling back to classic");
                Self::classic()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_config_round_trips() {
        let theme = ThemeConfig::dark();
        let serialised = toml::to_string_pretty(&theme).expect("serialise");
        let deser: ThemeConfig = toml::from_str(&serialised).expect("deserialise");

        assert_eq!(deser.numeral_style, NumeralStyle::Arabic);
        assert_eq!(deser.hand_style, HandStyle::Classic);
    }

    #[test]
    fn by_name_falls_back_to_classic() {
        let theme = ThemeConfig::by_name("nonexistent");
        // Classic has a white-ish face
        assert!(theme.face_colour.0[0] > 0.9);
    }

    #[test]
    fn empty_toml_uses_defaults() {
        let theme: ThemeConfig = toml::from_str("").expect("parse empty");
        assert_eq!(theme.numeral_style, NumeralStyle::Arabic);
        assert_eq!(theme.hand_style, HandStyle::Classic);
    }
}
