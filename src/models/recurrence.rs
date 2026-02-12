use chrono::NaiveDate;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

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
    /// Repeat monthly (same day next montch)
    Monthly,
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
    /// let date = NaiveDate::from_ymd_opt(2025, 2, 10).unwrap();
    ///
    /// // Daily: 2025-02-10 → 2025-02-11
    /// assert_eq!(Recurrence::Daily.next_date(date),
    ///            NaiveDate::from_ymd_opt(2025, 2, 11).unwrap());
    ///
    /// // Weekly: 2025-02-10 → 2025-02-17
    /// assert_eq!(Recurrence::Weekly.next_date(date),
    ///            NaiveDate::from_ymd_opt(2025, 2, 17).unwrap());
    ///
    /// // Monthly: 2025-02-10 → 2025-03-10
    /// assert_eq!(Recurrence::Monthly.next_date(date),
    ///            NaiveDate::from_ymd_opt(2025, 3, 10).unwrap());
    /// ```
    pub fn next_date(&self, from_date: NaiveDate) -> NaiveDate {
        use chrono::Duration;

        match self {
            Recurrence::Daily => from_date + Duration::days(1),
            Recurrence::Weekly => from_date + Duration::days(7),
            Recurrence::Monthly => {
                // Using a stopwatch method to deal with months of different sizes
                from_date
                    .checked_add_months(chrono::Months::new(1))
                    .unwrap_or(from_date)
            }
        }
    }

    /// Returns a human-readable description of the recurrence pattern.
    pub fn description(&self) -> &'static str {
        match self {
            Recurrence::Daily => "daily",
            Recurrence::Weekly => "weekly",
            Recurrence::Monthly => "monthly",
        }
    }
}
