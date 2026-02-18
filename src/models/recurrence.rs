use chrono::NaiveDate;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Recurrence pattern for tasks.
///
/// Defines how often a task should repeat when marked as completed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Recurrence {
    /// Repeat daily (next day)
    Daily,
    /// Repeat weekly (same day next week)
    Weekly,
    /// Repeat monthly (same day next month)
    Monthly,
}

impl fmt::Display for Recurrence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Recurrence::Daily => "daily",
            Recurrence::Weekly => "weekly",
            Recurrence::Monthly => "monthly",
        };
        write!(f, "{}", s)
    }
}

impl Recurrence {
    /// Calculates the next occurrence date based on the pattern.
    ///
    /// # Arguments
    ///
    /// * `from_date` - The base date to calculate from (usually current due date)
    ///
    /// # Returns
    ///
    /// The next occurrence date
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::NaiveDate;
    /// use todo_cli::models::Recurrence;
    ///
    /// let date = NaiveDate::from_ymd_opt(2025, 2, 10).unwrap();
    ///
    /// // Daily: 2025-02-10 → 2025-02-11
    /// assert_eq!(Recurrence::Daily.next_date(date),
    ///            NaiveDate::from_ymd_opt(2025, 2, 11).unwrap());
    /// ```
    pub fn next_date(&self, from_date: NaiveDate) -> NaiveDate {
        use chrono::Duration;

        match self {
            Recurrence::Daily => from_date + Duration::days(1),
            Recurrence::Weekly => from_date + Duration::days(7),
            Recurrence::Monthly => from_date
                .checked_add_months(chrono::Months::new(1))
                .unwrap_or(from_date),
        }
    }
}
