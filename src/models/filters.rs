use clap::ValueEnum;

/// Filter for task completion status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum StatusFilter {
    /// Show only pending tasks
    Pending,
    /// Show only completed tasks
    Done,
    /// Show all tasks (default)
    All,
}

/// Filter for task due dates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum DueFilter {
    /// Tasks past their due date
    Overdue,
    /// Tasks due in the next 7 days
    Soon,
    /// Tasks with any due date set
    WithDue,
    /// Tasks without a due date
    NoDue,
}

/// Filter for task recurrence patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum RecurrenceFilter {
    /// Show only tasks that repeat daily
    Daily,
    /// Show only tasks that repeat weekly
    Weekly,
    /// Show only tasks that repeat monthly
    Monthly,
    /// Show any recurring task (daily, weekly, or monthly)
    Recurring,
    /// Show only non-recurring tasks
    NonRecurring,
}

/// Sorting options for task lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SortBy {
    /// Sort by priority (High -> Medium -> Low)
    Priority,
    /// Sort by due date (earliest first)
    Due,
    /// Sort by creation date (oldest first)
    Created,
}
