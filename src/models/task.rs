use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};

use super::filters::{DueFilter, StatusFilter};
use super::priority::Priority;

/// Represents a single task in the todo list.
///
/// Each task contains a description, completion status, priority level,
/// optional tags for organization, and optional due date for deadline
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
}

impl Task {
    /// Creates a new task with the given parameters.
    pub fn new(
        text: String,
        priority: Priority,
        tags: Vec<String>,
        due_date: Option<NaiveDate>,
    ) -> Self {
        Task {
            text,
            completed: false,
            priority,
            tags,
            due_date,
            created_at: Local::now().naive_local().date(),
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
    /// A task is considered overdue if it has a due in the past
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
}
