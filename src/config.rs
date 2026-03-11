//! User configuration for rustodo.
//!
//! Loaded from `~/.config/rustodo/config.toml` (created with defaults on first run).
//!
//! # Example config.toml
//!
//! ```toml
//! holidays_locale = "pt-BR"  # or "en-US", "none"
//!
//! [theme]
//! accent         = "#00ffff"
//! high           = "#ff5555"
//! medium         = "#ffff55"
//! low            = "#55ff55"
//! done           = "#555555"
//! blocked        = "#969696"
//! selected_bg    = "#282840"
//! search_bg      = "#1e1e32"
//! focused_bg     = "#1e2840"
//! focused_border = "#00ffff"
//! ```

use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::tui::style::Theme;

// ── Config ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    /// Locale for holiday data (e.g. "pt-BR", "en-US", or "none")
    #[serde(default = "default_holidays_locale")]
    pub holidays_locale: String,
    /// TUI color theme
    pub theme: Theme,
}

impl Config {
    /// Load config from disk. Returns defaults if the file doesn't exist yet.
    /// Creates the file with defaults on first run.
    pub fn load() -> Result<Self> {
        let path = config_path()?;

        if !path.exists() {
            let cfg = Config::default();
            cfg.save()?;
            return Ok(cfg);
        }

        let contents = fs::read_to_string(&path)?;
        let cfg: Config = toml::from_str(&contents)?;
        Ok(cfg)
    }

    /// Save current config to disk.
    pub fn save(&self) -> Result<()> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self)?;
        fs::write(&path, contents)?;
        Ok(())
    }

    /// Return the path to the config file (for `todo info`).
    pub fn path() -> Result<PathBuf> {
        config_path()
    }
}

fn default_holidays_locale() -> String {
    "none".to_string()
}

fn config_path() -> Result<PathBuf> {
    let config_dir = if let Ok(dir) = std::env::var("RUSTODO_CONFIG_DIR") {
        PathBuf::from(dir)
    } else {
        let dirs = ProjectDirs::from("", "", "rustodo")
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        dirs.config_dir().to_path_buf()
    };
    fs::create_dir_all(&config_dir)?;
    Ok(config_dir.join("config.toml"))
}
