//! Natural language date parsing
//!
//! Parses dates from both natural language (e.g., "tomorrow", "next friday")
//! and strict format (YYYY-MM-DD).

use anyhow::{Context, Result, bail};
use chrono::{Duration, Local, NaiveDate};
use chrono_english::{Dialect, parse_date_string};
use std::sync::LazyLock;

// Regex compiled once via LazyLock, avoiding recompilation on every call
// to try_parse_custom_patterns.
static RE_IN_N_DAYS: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"in (\d+) days?").unwrap());

static RE_IN_N_WEEKS: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"in (\d+) weeks?").unwrap());

static RE_IN_N_MONTHS: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"in (\d+) months?").unwrap());

/// Parses a date from either natural language or YYYY-MM-DD format.
///
/// # Supported formats
///
/// **Natural language:**
/// - `today`, `tomorrow`, `yesterday`
/// - `in N days` (e.g., `in 3 days`)
/// - `in N weeks` (e.g., `in 2 weeks`)
/// - `in N months` (e.g., `in 1 month`)
/// - `monday`, `next friday`, `next monday`
/// - `jan 15`, `march 20`, `december 25`
///
/// **Strict format:**
/// - `YYYY-MM-DD` (e.g., `2026-02-20`)
///
/// # Errors
///
/// Returns a descriptive error if the input cannot be parsed in any
/// supported format, listing examples to guide the user.
///
/// # Examples
///
/// ```
/// use todo_cli::date_parser::parse_date;
///
/// let date = parse_date("tomorrow").unwrap();
/// let date = parse_date("next friday").unwrap();
/// let date = parse_date("in 3 days").unwrap();
/// let date = parse_date("in 2 weeks").unwrap();
/// let date = parse_date("in 1 month").unwrap();
/// let date = parse_date("2026-02-20").unwrap();
/// ```
pub fn parse_date(input: &str) -> Result<NaiveDate> {
    let trimmed = input.trim().to_lowercase();

    // Try strict YYYY-MM-DD format first
    if let Ok(date) = NaiveDate::parse_from_str(&trimmed, "%Y-%m-%d") {
        return Ok(date);
    }

    // Try custom patterns that chrono-english does not handle well
    if let Some(date) = try_parse_custom_patterns(&trimmed) {
        return Ok(date);
    }

    // Fall back to natural language parsing via chrono-english
    parse_date_string(&trimmed, Local::now(), Dialect::Uk)
        .map(|dt| dt.date_naive())
        .with_context(|| {
            format!(
                "Could not parse date: '{}'\n\n\
                Accepted formats:\n  \
                * Natural language: tomorrow, next friday, in 3 days, in 2 weeks, in 1 month\n  \
                * Weekdays:         monday, tuesday, next wednesday\n  \
                * Month and day:    jan 15, march 20, december 25\n  \
                * Strict format:    YYYY-MM-DD (e.g. 2026-02-20)",
                input.trim()
            )
        })
}

/// Parses a date and rejects values that fall in the past.
///
/// Separating the "past date" check from parsing allows the error message
/// to reference the original user input alongside the interpreted date,
/// rather than only showing the resolved value.
///
/// Use this for `add` (new tasks must have a future due date).
/// Use [`parse_date`] for `edit` (correcting an already-overdue task is valid).
///
/// # Errors
///
/// Returns an error if the date is before today, naming both what was typed
/// and what was interpreted.
pub fn parse_date_not_in_past(input: &str) -> Result<NaiveDate> {
    let date = parse_date(input)?;
    let today = Local::now().date_naive();

    if date < today {
        bail!(
            "The date interpreted from '{}' is {} -- which is already in the past.\n  \
            Please use a future date, e.g.: tomorrow, next monday, in 3 days, or YYYY-MM-DD.",
            input.trim(),
            date.format("%Y-%m-%d")
        );
    }

    Ok(date)
}

/// Tries custom date patterns that chrono-english does not handle well.
fn try_parse_custom_patterns(input: &str) -> Option<NaiveDate> {
    let today = Local::now().date_naive();

    // Uses static regexes instead of compiling on every call
    if let Some(caps) = RE_IN_N_DAYS.captures(input) {
        let days: i64 = caps[1].parse().ok()?;
        return Some(today + Duration::days(days));
    }

    if let Some(caps) = RE_IN_N_WEEKS.captures(input) {
        let weeks: i64 = caps[1].parse().ok()?;
        return Some(today + Duration::weeks(weeks));
    }

    if let Some(caps) = RE_IN_N_MONTHS.captures(input) {
        let months: u32 = caps[1].parse().ok()?;
        return today.checked_add_months(chrono::Months::new(months));
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
        // Error message must include usage examples
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("Accepted formats"));
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

    #[test]
    fn test_parse_date_not_in_past_future_ok() {
        // unwrap() fails the test with a clear message if the date is rejected
        let date = parse_date_not_in_past("tomorrow").unwrap();
        let tomorrow = Local::now().date_naive() + Duration::days(1);
        assert_eq!(date, tomorrow);
    }

    #[test]
    fn test_parse_date_not_in_past_past_fails() {
        let result = parse_date_not_in_past("yesterday");
        assert!(result.is_err());
        // Error must mention the original input and signal it is in the past
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("yesterday"));
        assert!(msg.contains("in the past"));
    }

    #[test]
    fn test_parse_date_not_in_past_strict_past_fails() {
        let result = parse_date_not_in_past("2020-01-01");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("2020-01-01"));
        assert!(msg.contains("in the past"));
    }

    #[test]
    fn test_lazy_lock_regex_reuse() {
        // Ensures static regexes work correctly across multiple calls
        let r1 = parse_date("in 1 day").unwrap();
        let r2 = parse_date("in 2 days").unwrap();
        let r3 = parse_date("in 1 week").unwrap();
        let r4 = parse_date("in 2 weeks").unwrap();
        let r5 = parse_date("in 1 month").unwrap();

        let today = Local::now().date_naive();
        assert_eq!(r1, today + Duration::days(1));
        assert_eq!(r2, today + Duration::days(2));
        assert_eq!(r3, today + Duration::weeks(1));
        assert_eq!(r4, today + Duration::weeks(2));
        assert_eq!(
            r5,
            today.checked_add_months(chrono::Months::new(1)).unwrap()
        );
    }
}
