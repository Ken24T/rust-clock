//! Application configuration — load/save from TOML at the XDG config path.
//!
//! Falls back to sensible defaults when the file is missing or malformed.

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::theme::ThemeConfig;

pub const MIN_CLOCK_SIZE: u32 = 50;
pub const MAX_CLOCK_SIZE: u32 = 500;
pub const MIN_OPACITY_PERCENT: u8 = 5;
pub const MAX_OPACITY_PERCENT: u8 = 100;
pub const OPACITY_STEP_PERCENT: i8 = 5;
pub const SIZE_ADJUST_STEP_PERCENT: i8 = 10;
pub const MAX_SIZE_ADJUST_PERCENT: i8 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ClockSizePreset {
    Small,
    #[default]
    Medium,
    Large,
}

impl ClockSizePreset {
    pub fn label(self) -> &'static str {
        match self {
            Self::Small => "Small",
            Self::Medium => "Medium",
            Self::Large => "Large",
        }
    }

    pub fn base_size(self) -> u32 {
        match self {
            Self::Small => 150,
            Self::Medium => 250,
            Self::Large => 350,
        }
    }
}

/// Persisted application settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Window width and height in logical pixels (50–500).
    #[serde(default = "default_size")]
    pub size: u32,

    /// Named size preset used by the settings UI.
    #[serde(default)]
    pub size_preset: Option<ClockSizePreset>,

    /// Relative adjustment from the preset base size.
    #[serde(default)]
    pub size_adjust_percent: Option<i8>,

    /// Optional fixed window position `[x, y]`.
    #[serde(default)]
    pub position: Option<(i32, i32)>,

    /// Name of a built-in theme preset (classic, dark, minimal, transparent).
    /// Ignored when a `[theme]` section is present.
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Enable smooth (60 fps) second hand sweep.
    #[serde(default = "default_true")]
    pub smooth_seconds: bool,

    /// Show the day-of-month on the clock face.
    #[serde(default = "default_true")]
    pub show_date: bool,

    /// Show the second hand on the clock face.
    #[serde(default = "default_true")]
    pub show_seconds: bool,

    /// Global opacity multiplier for the clock face theme.
    #[serde(default = "default_opacity_percent")]
    pub opacity_percent: u8,

    /// Full theme customisation. When present, overrides the `theme` name.
    #[serde(default)]
    pub theme_config: Option<ThemeConfig>,
}

// -- Serde defaults -------------------------------------------------------

fn default_size() -> u32 {
    250
}

fn default_opacity_percent() -> u8 {
    100
}

fn default_theme() -> String {
    "classic".to_string()
}

fn default_true() -> bool {
    true
}

// -- Trait impls -----------------------------------------------------------

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            size: default_size(),
            size_preset: Some(ClockSizePreset::default()),
            size_adjust_percent: Some(0),
            position: None,
            theme: default_theme(),
            smooth_seconds: true,
            show_date: true,
            show_seconds: true,
            opacity_percent: default_opacity_percent(),
            theme_config: None,
        }
    }
}

// -- Load / Save -----------------------------------------------------------

impl AppConfig {
    /// Resolve the config file path: `~/.config/rust-clock/config.toml`.
    pub fn config_path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "rust-clock").map(|dirs| dirs.config_dir().join("config.toml"))
    }

    /// Load configuration from disk, falling back to defaults on any error.
    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            eprintln!("Could not determine config directory, using defaults");
            return Self::default();
        };

        if !path.exists() {
            return Self::default();
        }

        match fs::read_to_string(&path) {
            Ok(contents) => match toml::from_str(&contents) {
                Ok(config) => {
                    let mut config: AppConfig = config;
                    config.normalise();
                    config
                }
                Err(e) => {
                    eprintln!("Failed to parse config at {}: {e}", path.display());
                    Self::default()
                }
            },
            Err(e) => {
                eprintln!("Failed to read config at {}: {e}", path.display());
                Self::default()
            }
        }
    }

    /// Write the current configuration to disk, creating directories as needed.
    #[allow(dead_code)]
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let Some(path) = Self::config_path() else {
            return Err("Could not determine config directory".into());
        };

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self)?;
        fs::write(&path, contents)?;
        Ok(())
    }

    /// Resolve the effective theme: explicit `[theme_config]` overrides the
    /// named preset in `theme`.
    pub fn resolved_theme(&self) -> ThemeConfig {
        self.theme_config
            .clone()
            .unwrap_or_else(|| ThemeConfig::by_name(&self.theme))
    }

    pub fn resolved_clock_theme(&self) -> ThemeConfig {
        self.resolved_theme().with_opacity(self.opacity_factor())
    }

    pub fn opacity_factor(&self) -> f32 {
        self.opacity_percent
            .clamp(MIN_OPACITY_PERCENT, MAX_OPACITY_PERCENT) as f32
            / 100.0
    }

    pub fn active_size_preset(&self) -> ClockSizePreset {
        self.size_preset.unwrap_or_default()
    }

    pub fn active_size_adjust_percent(&self) -> i8 {
        self.size_adjust_percent
            .unwrap_or(0)
            .clamp(-MAX_SIZE_ADJUST_PERCENT, MAX_SIZE_ADJUST_PERCENT)
    }

    pub fn set_size_preset(&mut self, preset: ClockSizePreset) {
        self.size_preset = Some(preset);
        self.sync_size_from_controls();
    }

    pub fn adjust_size_adjust_percent(&mut self, delta: i8) -> bool {
        let next = (self.active_size_adjust_percent() + delta)
            .clamp(-MAX_SIZE_ADJUST_PERCENT, MAX_SIZE_ADJUST_PERCENT);

        if next == self.active_size_adjust_percent() {
            return false;
        }

        self.size_adjust_percent = Some(next);
        self.sync_size_from_controls();
        true
    }

    pub fn can_adjust_size_adjust_percent(&self, delta: i8) -> bool {
        let next = self.active_size_adjust_percent() + delta;
        (-MAX_SIZE_ADJUST_PERCENT..=MAX_SIZE_ADJUST_PERCENT).contains(&next)
    }

    pub fn size_adjustment_label(&self) -> String {
        match self.active_size_adjust_percent().cmp(&0) {
            std::cmp::Ordering::Greater => format!("+{}%", self.active_size_adjust_percent()),
            std::cmp::Ordering::Less => format!("{}%", self.active_size_adjust_percent()),
            std::cmp::Ordering::Equal => "Base".to_string(),
        }
    }

    pub fn adjust_opacity_percent(&mut self, delta: i8) -> bool {
        let next = (self.opacity_percent as i16 + delta as i16)
            .clamp(MIN_OPACITY_PERCENT as i16, MAX_OPACITY_PERCENT as i16) as u8;

        if next == self.opacity_percent {
            return false;
        }

        self.opacity_percent = next;
        true
    }

    pub fn can_adjust_opacity_percent(&self, delta: i8) -> bool {
        let next = self.opacity_percent as i16 + delta as i16;
        (MIN_OPACITY_PERCENT as i16..=MAX_OPACITY_PERCENT as i16).contains(&next)
    }

    fn normalise(&mut self) {
        self.opacity_percent = self
            .opacity_percent
            .clamp(MIN_OPACITY_PERCENT, MAX_OPACITY_PERCENT);

        if self.size_preset.is_some() {
            self.size_adjust_percent = Some(self.active_size_adjust_percent());
            self.sync_size_from_controls();
        } else {
            let (preset, adjust, size) =
                infer_size_settings(self.size.clamp(MIN_CLOCK_SIZE, MAX_CLOCK_SIZE));
            self.size_preset = Some(preset);
            self.size_adjust_percent = Some(adjust);
            self.size = size;
        }
    }

    fn sync_size_from_controls(&mut self) {
        self.size = effective_size(self.active_size_preset(), self.active_size_adjust_percent());
    }
}

fn effective_size(preset: ClockSizePreset, adjust_percent: i8) -> u32 {
    let scale = 1.0 + adjust_percent as f32 / 100.0;
    ((preset.base_size() as f32) * scale)
        .round()
        .clamp(MIN_CLOCK_SIZE as f32, MAX_CLOCK_SIZE as f32) as u32
}

fn infer_size_settings(size: u32) -> (ClockSizePreset, i8, u32) {
    let mut best = (
        ClockSizePreset::default(),
        0,
        effective_size(ClockSizePreset::default(), 0),
    );
    let mut best_distance = u32::MAX;

    for preset in [
        ClockSizePreset::Small,
        ClockSizePreset::Medium,
        ClockSizePreset::Large,
    ] {
        for adjust in (-MAX_SIZE_ADJUST_PERCENT..=MAX_SIZE_ADJUST_PERCENT)
            .step_by(SIZE_ADJUST_STEP_PERCENT as usize)
        {
            let candidate_size = effective_size(preset, adjust);
            let distance = candidate_size.abs_diff(size);

            if distance < best_distance {
                best = (preset, adjust, candidate_size);
                best_distance = distance;
            }
        }
    }

    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_round_trips() {
        let config = AppConfig::default();
        let serialised = toml::to_string_pretty(&config).expect("serialise");
        let deserialised: AppConfig = toml::from_str(&serialised).expect("deserialise");

        assert_eq!(deserialised.size, config.size);
        assert_eq!(deserialised.size_preset, config.size_preset);
        assert_eq!(deserialised.size_adjust_percent, config.size_adjust_percent);
        assert_eq!(deserialised.theme, config.theme);
        assert_eq!(deserialised.position, config.position);
        assert_eq!(deserialised.opacity_percent, config.opacity_percent);
    }

    #[test]
    fn missing_fields_use_defaults() {
        let minimal = "";
        let config: AppConfig = toml::from_str(minimal).expect("parse empty");

        assert_eq!(config.size, 250);
        assert_eq!(config.active_size_preset(), ClockSizePreset::Medium);
        assert_eq!(config.active_size_adjust_percent(), 0);
        assert_eq!(config.theme, "classic");
        assert!(config.smooth_seconds);
        assert!(config.show_date);
        assert!(config.position.is_none());
        assert_eq!(config.opacity_percent, 100);
        assert!(config.theme_config.is_none());
    }

    #[test]
    fn resolved_theme_uses_name_when_no_config() {
        let mut config = AppConfig::default();
        config.theme = "dark".to_string();
        let theme = config.resolved_theme();
        // Dark theme has a dark face (low red channel)
        assert!(theme.face_colour.0[0] < 0.2);
    }

    #[test]
    fn resolved_theme_prefers_explicit_config() {
        use crate::theme::Colour;
        let mut config = AppConfig::default();
        config.theme = "dark".to_string();
        let mut custom = ThemeConfig::classic();
        custom.face_colour = Colour::new(0.5, 0.5, 0.5, 1.0);
        config.theme_config = Some(custom);
        let theme = config.resolved_theme();
        assert!((theme.face_colour.0[0] - 0.5).abs() < 0.01);
    }

    #[test]
    fn legacy_size_is_inferred_to_nearest_preset_step() {
        let mut config: AppConfig = toml::from_str("size = 150").expect("parse legacy size");
        config.normalise();

        assert_eq!(config.active_size_preset(), ClockSizePreset::Small);
        assert_eq!(config.active_size_adjust_percent(), 0);
        assert_eq!(config.size, 150);
    }

    #[test]
    fn resolved_clock_theme_applies_global_opacity() {
        let mut config = AppConfig::default();
        config.theme = "dark".to_string();
        config.opacity_percent = 50;

        let theme = config.resolved_clock_theme();

        assert!((theme.face_colour.0[3] - 0.46).abs() < 0.01);
        assert!((theme.border_colour.0[3] - 0.5).abs() < 0.01);
    }
}
