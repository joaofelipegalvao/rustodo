//! Natural language date parsing
//!
//! Parses dates from both natural language (e.g., "tomorrow", "next friday")
//! and strict format (YYYY-MM-DD).

use anyhow::{Context, Result};
use chrono::{Duration, Local, NaiveDate};
use chrono_english::{Dialect, parse_date_string};

/// Parses a date from either natural language or YYYY-MM-DD format
///
/// # Examples
///
/// ```
/// use todo_cli::date_parser::parse_date;
///
/// // Natural language
/// let date = parse_date("tomorrow").unwrap();
/// let date = parse_date("next friday").unwrap();
/// let date = parse_date("in 3 days").unwrap();
/// let date = parse_date("in 2 weeks").unwrap();
/// let date = parse_date("in 1 month").unwrap();
///
/// // Strict format
/// let date = parse_date("2026-02-20").unwrap();
/// ```
///
/// # Supported Natural Language Patterns
///
/// - **Relative dates:**
///   - `today`, `tomorrow`, `yesterday`
///   - `in N days` (e.g., `in 3 days`)
///   - `in N weeks` (e.g., `in 2 weeks`)
///   - `in N months` (e.g., `in 1 month`)
///
/// - **Weekdays:**
///   - `monday`, `tuesday`, `friday`, etc. (next occurrence)
///   - `next monday`, `next friday`
///
/// - **Specific dates:**
///   - `jan 15`, `march 20`, `dec 25`
///   - `15 january`, `20 march`
///   - `january 15 2027` (with year)
///
/// - **Strict format:**
///   - `YYYY-MM-DD` (e.g., `2026-02-20`)
///
/// # Errors
///
/// Returns an error if the date string cannot be parsed in any supported format.
pub fn parse_date(input: &str) -> Result<NaiveDate> {
    let trimmed = input.trim().to_lowercase();

    // Try parsing as strict YYYY-MM-DD first
    if let Ok(date) = NaiveDate::parse_from_str(&trimmed, "%Y-%m-%d") {
        return Ok(date);
    }

    // Try custom patterns that chrono-english doesn't support well
    if let Some(date) = try_parse_custom_patterns(&trimmed) {
        return Ok(date);
    }

    // Try natural language parsing with chrono-english
    parse_date_string(&trimmed, Local::now(), Dialect::Uk)
        .map(|dt| dt.date_naive())
        .with_context(|| {
            format!(
                "Failed to parse date: '{}'\n\nSupported formats:\n  \
                - Natural: tomorrow, next friday, in 3 days, in 2 weeks, in 1 month\n  \
                - Strict: YYYY-MM-DD (e.g., 2026-02-20)",
                input.trim()
            )
        })
}

/// Try parsing custom patterns that chrono-english doesn't handle well
fn try_parse_custom_patterns(input: &str) -> Option<NaiveDate> {
    let today = Local::now().date_naive();

    // Pattern: "in N days"
    if let Some(days) = parse_in_n_units(input, "day") {
        return Some(today + Duration::days(days));
    }

    // Pattern: "in N weeks"
    if let Some(weeks) = parse_in_n_units(input, "week") {
        return Some(today + Duration::weeks(weeks));
    }

    // Pattern: "in N months"
    if let Some(months) = parse_in_n_units(input, "month") {
        return today.checked_add_months(chrono::Months::new(months as u32));
    }

    None
}

/// Helper to parse "in N <unit>" patterns
fn parse_in_n_units(input: &str, unit: &str) -> Option<i64> {
    // Try both singular and plural
    let patterns = [
        format!("in {} {}", r"(\d+)", unit),
        format!("in {} {}s", r"(\d+)", unit),
    ];

    for pattern in &patterns {
        if let Some(caps) = regex::Regex::new(pattern).ok()?.captures(input)
            && let Some(num_str) = caps.get(1)
        {
            return num_str.as_str().parse::<i64>().ok();
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_strict_format() {
        let date = parse_date("2026-02-20").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2026, 2, 20).unwrap());
    }

    #[test]
    fn test_parse_today() {
        let today = Local::now().date_naive();
        let parsed = parse_date("today").unwrap();
        assert_eq!(parsed, today);
    }

    #[test]
    fn test_parse_tomorrow() {
        let tomorrow = Local::now().date_naive() + Duration::days(1);
        let parsed = parse_date("tomorrow").unwrap();
        assert_eq!(parsed, tomorrow);
    }

    #[test]
    fn test_parse_invalid() {
        let result = parse_date("not a date");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_in_n_days() {
        let expected = Local::now().date_naive() + Duration::days(3);
        let parsed = parse_date("in 3 days").unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_parse_in_n_weeks() {
        let expected = Local::now().date_naive() + Duration::weeks(2);
        let parsed = parse_date("in 2 weeks").unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_parse_in_n_months() {
        let today = Local::now().date_naive();
        let expected = today.checked_add_months(chrono::Months::new(1)).unwrap();
        let parsed = parse_date("in 1 month").unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_parse_next_week() {
        let result = parse_date("next monday");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_month_day_format() {
        let result = parse_date("march 15");
        assert!(result.is_ok());
    }
}
