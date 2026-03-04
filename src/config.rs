//! Application configuration — load/save from TOML at the XDG config path.
//!
//! Falls back to sensible defaults when the file is missing or malformed.

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::theme::ThemeConfig;

/// Persisted application settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Window width and height in logical pixels (50–500).
    #[serde(default = "default_size")]
    pub size: u32,

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

    /// Full theme customisation. When present, overrides the `theme` name.
    #[serde(default)]
    pub theme_config: Option<ThemeConfig>,
}

// -- Serde defaults -------------------------------------------------------

fn default_size() -> u32 {
    250
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
            position: None,
            theme: default_theme(),
            smooth_seconds: true,
            show_date: true,
            show_seconds: true,
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
                    config.size = config.size.clamp(50, 500);
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
        assert_eq!(deserialised.theme, config.theme);
        assert_eq!(deserialised.position, config.position);
    }

    #[test]
    fn missing_fields_use_defaults() {
        let minimal = "";
        let config: AppConfig = toml::from_str(minimal).expect("parse empty");

        assert_eq!(config.size, 250);
        assert_eq!(config.theme, "classic");
        assert!(config.smooth_seconds);
        assert!(config.show_date);
        assert!(config.position.is_none());
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
}
