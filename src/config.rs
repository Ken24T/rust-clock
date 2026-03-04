//! Application configuration — load/save from TOML at the XDG config path.
//!
//! Falls back to sensible defaults when the file is missing or malformed.

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Persisted application settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Window width and height in logical pixels.
    #[serde(default = "default_size")]
    pub size: u32,

    /// Optional fixed window position `[x, y]`.
    #[serde(default)]
    pub position: Option<(i32, i32)>,

    /// Name of the colour theme to use.
    #[serde(default = "default_theme")]
    pub theme: String,
}

// -- Serde defaults -------------------------------------------------------

fn default_size() -> u32 {
    250
}

fn default_theme() -> String {
    "classic".to_string()
}

// -- Trait impls -----------------------------------------------------------

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            size: default_size(),
            position: None,
            theme: default_theme(),
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
                Ok(config) => config,
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
        assert!(config.position.is_none());
    }
}
