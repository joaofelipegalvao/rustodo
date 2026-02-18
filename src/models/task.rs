use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};

use super::filters::{DueFilter, StatusFilter};
use super::priority::Priority;
use super::recurrence::Recurrence;

/// Represents a single task in the todo list.
///
/// Each task contains a description, completion status, priority level,
/// optional tags for organization, optional due date for deadline tracking,
/// and recurrence pattern for repeating tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// The task description/content
    pub text: String,
    /// Whether the task has been completed
    pub completed: bool,
    /// Priority level of the task
    pub priority: Priority,
    /// List of tags for categorization
    pub tags: Vec<String>,
    /// Optional due date for deadline tracking
    pub due_date: Option<NaiveDate>,
    /// Date when the task was created
    pub created_at: NaiveDate,
    /// Optional recurrence pattern (daily, weekly, monthly)
    pub recurrence: Option<Recurrence>,
    /// ID of the parent task (for recurring task chains)
    ///
    /// This links recurring tasks together, allowing:
    /// - Perfect deduplication even if text is edited
    /// - Tracking "families" of recurring tasks
    /// - Future features like `todo history <id>`
    #[serde(default)]
    pub parent_id: Option<usize>,
}

impl Task {
    /// Creates a new task with the given parameters.
    pub fn new(
        text: String,
        priority: Priority,
        tags: Vec<String>,
        due_date: Option<NaiveDate>,
        recurrence: Option<Recurrence>,
    ) -> Self {
        Task {
            text,
            completed: false,
            priority,
            tags,
            due_date,
            created_at: Local::now().naive_local().date(),
            recurrence,
            parent_id: None,
        }
    }

    /// Marks this task as completed.
    pub fn mark_done(&mut self) {
        self.completed = true;
    }

    /// Marks this task as pending (not completed).
    pub fn mark_undone(&mut self) {
        self.completed = false;
    }

    /// Checks if this is overdue.
    ///
    /// A task is considered overdue if it has a due date in the past
    /// and is not yet completed.
    pub fn is_overdue(&self) -> bool {
        if let Some(due) = self.due_date {
            let today = Local::now().naive_local().date();
            due < today && !self.completed
        } else {
            false
        }
    }

    /// Checks if this task is due soon (within the specified number of days).
    ///
    /// # Arguments
    ///
    /// * `days` - Number of days to look ahead
    ///
    /// # Returns
    ///
    /// `true` if the task is due within the specified number of days and
    /// is not yet completed, `false` otherwise.
    pub fn is_due_soon(&self, days: i64) -> bool {
        if let Some(due) = self.due_date {
            let today = Local::now().naive_local().date();
            let days_until = (due - today).num_days();
            days_until >= 0 && days_until <= days && !self.completed
        } else {
            false
        }
    }

    /// Checks if this task matches the given status filter.
    pub fn matches_status(&self, status: StatusFilter) -> bool {
        match status {
            StatusFilter::Pending => !self.completed,
            StatusFilter::Done => self.completed,
            StatusFilter::All => true,
        }
    }

    /// Checks if this task matches the given due date filter.
    pub fn matches_due_filter(&self, filter: DueFilter) -> bool {
        match filter {
            DueFilter::Overdue => self.is_overdue(),
            DueFilter::Soon => self.is_due_soon(7),
            DueFilter::WithDue => self.due_date.is_some(),
            DueFilter::NoDue => self.due_date.is_none(),
        }
    }

    /// Creates a new task for the next recurrence cycle.
    ///
    /// # Arguments
    ///
    /// * `parent_id` - The ID of the current task (to link recurring tasks)
    ///
    /// # Returns
    ///
    /// `Some(Task)` if the task is recurring and has a due date,
    /// `None` otherwise.
    ///
    /// # Behavior
    ///
    /// - Preserves: text, priority, tags, recurrence pattern
    /// - Resets: completed = false
    /// - Updates: due_date (calculated from recurrence), created_at (now)
    /// - Sets: parent_id (to link the chain)
    ///
    /// # Examples
    ///
    /// ```
    /// use todo_cli::models::{Task, Recurrence};
    /// use chrono::NaiveDate;
    ///
    /// let task = Task {
    ///     text: "Weekly review".to_string(),
    ///     completed: true,
    ///     priority: todo_cli::models::Priority::Medium,
    ///     tags: vec![],
    ///     due_date: Some(NaiveDate::from_ymd_opt(2025, 2, 10).unwrap()),
    ///     created_at: NaiveDate::from_ymd_opt(2025, 2, 1).unwrap(),
    ///     recurrence: Some(Recurrence::Weekly),
    ///     parent_id: None,
    /// };
    ///
    /// let next = task.create_next_recurrence(1);
    /// assert!(next.is_some());
    /// let next_task = next.unwrap();
    /// assert_eq!(next_task.due_date,
    ///            Some(NaiveDate::from_ymd_opt(2025, 2, 17).unwrap()));
    /// assert_eq!(next_task.parent_id, Some(1));
    /// ```
    pub fn create_next_recurrence(&self, parent_id: usize) -> Option<Task> {
        let recurrence = self.recurrence?;
        let current_due = self.due_date?;

        let next_due = recurrence.next_date(current_due);

        let mut next_task = Task::new(
            self.text.clone(),
            self.priority,
            self.tags.clone(),
            Some(next_due),
            Some(recurrence),
        );

        next_task.parent_id = Some(parent_id);

        Some(next_task)
    }

    #[allow(dead_code)]
    pub fn is_recurring(&self) -> bool {
        self.recurrence.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daily_recurrence() {
        let date = NaiveDate::from_ymd_opt(2026, 2, 10).unwrap();
        let task = Task::new(
            "Test".to_string(),
            Priority::Medium,
            vec![],
            Some(date),
            Some(Recurrence::Daily),
        );

        let next = task.create_next_recurrence(1).unwrap();
        assert_eq!(
            next.due_date,
            Some(NaiveDate::from_ymd_opt(2026, 2, 11).unwrap())
        );
        assert_eq!(next.parent_id, Some(1));
    }

    #[test]
    fn test_weekly_recurrence() {
        let date = NaiveDate::from_ymd_opt(2026, 2, 10).unwrap();
        let task = Task::new(
            "Test".to_string(),
            Priority::Medium,
            vec![],
            Some(date),
            Some(Recurrence::Weekly),
        );

        let next = task.create_next_recurrence(1).unwrap();
        assert_eq!(
            next.due_date,
            Some(NaiveDate::from_ymd_opt(2026, 2, 17).unwrap())
        );
    }

    #[test]
    fn test_monthly_recurrence() {
        let date = NaiveDate::from_ymd_opt(2026, 2, 10).unwrap();
        let task = Task::new(
            "Test".to_string(),
            Priority::Medium,
            vec![],
            Some(date),
            Some(Recurrence::Monthly),
        );

        let next = task.create_next_recurrence(1).unwrap();
        assert_eq!(
            next.due_date,
            Some(NaiveDate::from_ymd_opt(2026, 3, 10).unwrap())
        );
    }

    #[test]
    fn test_monthly_boundary_case() {
        // January 31 -> February should handle correctly
        let date = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();
        let task = Task::new(
            "Test".to_string(),
            Priority::Medium,
            vec![],
            Some(date),
            Some(Recurrence::Monthly),
        );

        let next = task.create_next_recurrence(1).unwrap();
        // February 2026 has 28 days
        assert_eq!(
            next.due_date,
            Some(NaiveDate::from_ymd_opt(2026, 2, 28).unwrap())
        );
    }

    #[test]
    fn test_no_recurrence_returns_none() {
        let task = Task::new(
            "Test".to_string(),
            Priority::Medium,
            vec![],
            Some(NaiveDate::from_ymd_opt(2026, 2, 10).unwrap()),
            None,
        );

        assert!(task.create_next_recurrence(1).is_none());
    }

    #[test]
    fn test_no_due_date_returns_none() {
        let task = Task::new(
            "Test".to_string(),
            Priority::Medium,
            vec![],
            None,
            Some(Recurrence::Daily),
        );

        assert!(task.create_next_recurrence(1).is_none());
    }

    #[test]
    fn test_parent_id_preserved() {
        let date = NaiveDate::from_ymd_opt(2026, 2, 10).unwrap();
        let task = Task::new(
            "Test".to_string(),
            Priority::Medium,
            vec![],
            Some(date),
            Some(Recurrence::Daily),
        );

        let next = task.create_next_recurrence(42).unwrap();
        assert_eq!(next.parent_id, Some(42));
    }
}
