use clap::ValueEnum;

/// Filters tasks by completion status.
///
/// Used by `todo list --status` and `todo search --status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum StatusFilter {
    /// Show only pending tasks.
    Pending,
    /// Show only completed tasks.
    Done,
    /// Show all tasks (default).
    All,
}

/// Filters tasks by their due-date window.
///
/// Used by `todo list --due`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum DueFilter {
    /// Tasks whose due date is strictly before today.
    Overdue,
    /// Tasks due within the next 7 days (inclusive of today).
    Soon,
    /// Tasks that have any due date set.
    WithDue,
    /// Tasks that have no due date.
    NoDue,
}

/// Filters tasks by recurrence pattern.
///
/// Used by `todo list --recurrence`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum RecurrenceFilter {
    /// Only tasks with a daily recurrence.
    Daily,
    /// Only tasks with a weekly recurrence.
    Weekly,
    /// Only tasks with a monthly recurrence.
    Monthly,
    /// Any task that has a recurrence set (daily, weekly, or monthly).
    Recurring,
    /// Tasks with no recurrence pattern.
    NonRecurring,
}

/// Sort order for `todo list`.
///
/// Used by `todo list --sort`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SortBy {
    /// Sort by priority: High → Medium → Low.
    Priority,
    /// Sort by due date, earliest first. Tasks without a due date appear last.
    Due,
    /// Sort by creation date, oldest first.
    Created,
}
