//! Handler for `todo holidays refresh`.

use anyhow::Result;
use chrono::{Datelike, Local};
use colored::Colorize;

use crate::config::Config;
use crate::services::holidays;

pub fn execute_refresh() -> Result<()> {
    let cfg = Config::load().unwrap_or_default();

    if cfg.holidays_locale == "none" || cfg.holidays_locale.is_empty() {
        println!(
            "\n  {}\n",
            "No holidays locale configured. Set holidays_locale in config.toml.".dimmed()
        );
        println!("  Example: holidays_locale = \"pt-BR\"  or  \"en-US\"\n");
        return Ok(());
    }

    let year = Local::now().year();
    println!(
        "\n  Fetching holidays for {} {}...",
        cfg.holidays_locale.bright_white(),
        year
    );
    holidays::refresh(&cfg.holidays_locale, year)?;
    // Also prefetch next year (may not be available yet)
    println!(
        "  Fetching holidays for {} {}...",
        cfg.holidays_locale.bright_white(),
        year + 1
    );
    match holidays::refresh(&cfg.holidays_locale, year + 1) {
        Ok(()) => {}
        Err(e) if e.to_string().contains("404") || e.to_string().contains("http status") => {
            println!(
                "  {} {} data not yet available for {}.",
                "⚠".yellow(),
                cfg.holidays_locale.dimmed(),
                year + 1
            );
        }
        Err(e) => return Err(e),
    }

    println!();
    Ok(())
}
