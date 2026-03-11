//! TUI theme and style helpers for rustodo.
//!
//! Loaded from `~/.config/rustodo/config.toml` under `[theme]`.
//!
//! # Example config.toml
//!
//! ```toml
//! [theme]
//! accent         = "#00ffff"
//! high           = "#ff5555"
//! medium         = "#ffff55"
//! low            = "#55ff55"
//! done           = "#555555"
//! blocked        = "#aaaaaa"
//! selected_bg    = "#282840"
//! search_bg      = "#1e1e32"
//! focused_bg     = "#1e2840"
//! focused_border = "#00ffff"
//! ```
//!
//! Colors accept:
//!   - Hex:         `"#rrggbb"`
//!   - Named:       `"red"`, `"blue"`, `"cyan"`, `"white"`, etc.
//!   - 256-palette: `"color196"`

use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

// ── Theme ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Theme {
    /// Accent color — borders, highlights, keybind labels
    pub accent: String,
    /// High priority
    pub high: String,
    /// Medium priority
    pub medium: String,
    /// Low priority
    pub low: String,
    /// Completed tasks
    pub done: String,
    /// Blocked tasks
    pub blocked: String,
    /// Selected item background
    pub selected_bg: String,
    /// Search bar background
    pub search_bg: String,
    /// Focused input field background
    pub focused_bg: String,
    /// Focused input field border
    pub focused_border: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            accent: "#00ffff".into(),  // cyan
            high: "#ff5555".into(),    // red
            medium: "#ffff55".into(),  // yellow
            low: "#55ff55".into(),     // green
            done: "#555555".into(),    // dark gray
            blocked: "#969696".into(), // gray
            selected_bg: "#282840".into(),
            search_bg: "#1e1e32".into(),
            focused_bg: "#1e2840".into(),
            focused_border: "#00ffff".into(), // cyan
        }
    }
}

impl Theme {
    /// Resolve all theme colors into ratatui `Color` values.
    pub fn resolve(&self) -> ResolvedTheme {
        ResolvedTheme {
            accent: parse_color(&self.accent),
            high: parse_color(&self.high),
            medium: parse_color(&self.medium),
            low: parse_color(&self.low),
            done: parse_color(&self.done),
            blocked: parse_color(&self.blocked),
            selected_bg: parse_color(&self.selected_bg),
            search_bg: parse_color(&self.search_bg),
            focused_bg: parse_color(&self.focused_bg),
            focused_border: parse_color(&self.focused_border),
        }
    }
}

// ── ResolvedTheme ─────────────────────────────────────────────────────────────

/// Theme with all colors already resolved to `ratatui::style::Color`.
/// Created once at startup via `Theme::resolve()`.
#[derive(Debug, Clone)]
pub struct ResolvedTheme {
    pub accent: Color,
    pub high: Color,
    pub medium: Color,
    pub low: Color,
    pub done: Color,
    pub blocked: Color,
    pub selected_bg: Color,
    pub search_bg: Color,
    pub focused_bg: Color,
    pub focused_border: Color,
}

impl ResolvedTheme {
    /// `Style` with just the accent foreground.
    pub fn accent(&self) -> Style {
        Style::default().fg(self.accent)
    }

    /// `Style` with accent fg + BOLD.
    pub fn accent_bold(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    /// `Style` for a selected row (accent bg + BOLD).
    pub fn selected(&self) -> Style {
        Style::default()
            .bg(self.selected_bg)
            .add_modifier(Modifier::BOLD)
    }

    /// `Style` for a focused border.
    pub fn focused_border(&self) -> Style {
        Style::default().fg(self.focused_border)
    }

    /// `Style` for an unfocused border.
    pub fn inactive_border(&self) -> Style {
        Style::default().fg(Color::DarkGray)
    }

    /// `Style` for a focused input background.
    pub fn focused_input(&self) -> Style {
        Style::default().bg(self.focused_bg)
    }

    /// `Style` for the search bar background.
    pub fn search_bg(&self) -> Style {
        Style::default().bg(self.search_bg)
    }
}

// ── Color parser ──────────────────────────────────────────────────────────────

/// Parse a color string into a `ratatui::style::Color`.
/// Supports: `#rrggbb`, named colors, and `color0`–`color255`.
/// Falls back to `Color::Reset` on unknown input.
pub fn parse_color(s: &str) -> Color {
    let s = s.trim();

    // Hex: #rrggbb
    if let Some(hex) = s.strip_prefix('#')
        && let Some((r, g, b)) = parse_hex(hex)
    {
        return Color::Rgb(r, g, b);
    }

    // 256-palette: color0–color255
    if let Some(rest) = s.strip_prefix("color")
        && let Ok(n) = rest.parse::<u8>()
    {
        return Color::Indexed(n);
    }

    // Named colors
    match s {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "bright_black" | "dark_gray" => Color::DarkGray,
        "bright_red" | "light_red" => Color::LightRed,
        "bright_green" | "light_green" => Color::LightGreen,
        "bright_yellow" | "light_yellow" => Color::LightYellow,
        "bright_blue" | "light_blue" => Color::LightBlue,
        "bright_magenta" | "light_magenta" => Color::LightMagenta,
        "bright_cyan" | "light_cyan" => Color::LightCyan,
        "bright_white" | "light_white" => Color::White,
        "reset" | "" => Color::Reset,
        _ => Color::Reset,
    }
}

fn parse_hex(hex: &str) -> Option<(u8, u8, u8)> {
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}
