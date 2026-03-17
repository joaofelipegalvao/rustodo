//! Holiday data fetched from holidata.net and cached locally.
//!
//! Cache location: `~/.config/rustodo/holidays/<locale>/<year>.json`
//!
//! Usage:
//! ```no_run
//! use rustodo::services::holidays::HolidayCache;
//! use chrono::NaiveDate;
//! let holidays = HolidayCache::load("pt-BR", 2026)?;
//! let names = holidays.for_date(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
//! # Ok::<(), anyhow::Error>(())
//! ```

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HolidayEntry {
    pub date: String,
    pub description: String,
    #[serde(default)]
    pub region: String,
}

/// In-memory holiday map for a single locale+year.
pub struct HolidayCache {
    /// date → holiday name (national only, region == "")
    map: HashMap<NaiveDate, String>,
}

impl HolidayCache {
    /// Load holidays for `locale` and `year`.
    ///
    /// Strategy:
    /// 1. Check cache at `~/.config/rustodo/holidays/<locale>/<year>.json`
    /// 2. If missing, fetch from holidata.net and save to cache
    /// 3. Parse and return
    pub fn load(locale: &str, year: i32) -> Result<Self> {
        let path = cache_path(locale, year)?;

        let raw = if path.exists() {
            fs::read_to_string(&path)?
        } else {
            let data = fetch(locale, year)?;
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&path, &data)?;
            data
        };

        let map = parse_ndjson(&raw);
        Ok(Self { map })
    }

    /// Returns the holiday name for a date, if any.
    pub fn for_date(&self, date: NaiveDate) -> Option<&str> {
        self.map.get(&date).map(String::as_str)
    }

    /// Returns true if the date is a holiday.
    pub fn is_holiday(&self, date: NaiveDate) -> bool {
        self.map.contains_key(&date)
    }
}

/// Empty cache — used when holidays are disabled.
impl Default for HolidayCache {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}

// ── Fetch ─────────────────────────────────────────────────────────────────────

/// HTTP timeout for holiday data requests.
///
/// A stalled connection to holidata.net should never hang the CLI indefinitely.
/// 10 seconds is generous enough for slow connections while still being
/// responsive to network failures.
const HTTP_TIMEOUT_SECS: u64 = 10;

fn fetch(locale: &str, year: i32) -> Result<String> {
    let url = format!("https://holidata.net/{}/{}.json", locale, year);

    // ureq 3.x API: Agent::config_builder().timeout_global(...).build().into()
    // ureq 2.x used ureq::builder().timeout(...).build() — incompatible.
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(std::time::Duration::from_secs(HTTP_TIMEOUT_SECS)))
        .build()
        .into();

    let mut response = agent
        .get(&url)
        .call()
        .with_context(|| format!("Failed to fetch holiday data from {}", url))?;

    response
        .body_mut()
        .read_to_string()
        .context("Failed to read holiday response body")
}

// ── Parse ─────────────────────────────────────────────────────────────────────

/// Parse NDJSON (one JSON object per line). Only national holidays (region == "").
fn parse_ndjson(raw: &str) -> HashMap<NaiveDate, String> {
    let mut map = HashMap::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(entry) = serde_json::from_str::<HolidayEntry>(line) {
            if !entry.region.is_empty() {
                continue;
            }
            if let Ok(date) = NaiveDate::parse_from_str(&entry.date, "%Y-%m-%d") {
                map.insert(date, entry.description);
            }
        }
    }
    map
}

// ── Cache path ────────────────────────────────────────────────────────────────

fn cache_path(locale: &str, year: i32) -> Result<PathBuf> {
    let config_dir = if let Ok(dir) = std::env::var("RUSTODO_CONFIG_DIR") {
        PathBuf::from(dir)
    } else {
        let dirs = directories::ProjectDirs::from("", "", "rustodo")
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        dirs.config_dir().to_path_buf()
    };
    Ok(config_dir
        .join("holidays")
        .join(locale)
        .join(format!("{}.json", year)))
}

// ── Refresh command ───────────────────────────────────────────────────────────

/// Force re-download of holiday data for `locale` and `year`.
/// Called by `todo holidays refresh`.
pub fn refresh(locale: &str, year: i32) -> Result<()> {
    let path = cache_path(locale, year)?;
    let data = fetch(locale, year)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, &data)?;
    println!(
        "  Holidays updated: {} {} ({} entries)",
        locale,
        year,
        data.lines().filter(|l| !l.trim().is_empty()).count()
    );
    Ok(())
}
