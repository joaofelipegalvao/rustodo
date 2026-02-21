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
    /// Optional project this task belongs to
    #[serde(default)]
    pub project: Option<String>,
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
    /// IDs (1- based) of tasks that must be completed before this one
    #[serde(default)]
    pub depends_on: Vec<usize>,
}

impl Task {
    /// Creates a new task with the given parameters.
    pub fn new(
        text: String,
        priority: Priority,
        tags: Vec<String>,
        project: Option<String>,
        due_date: Option<NaiveDate>,
        recurrence: Option<Recurrence>,
    ) -> Self {
        Task {
            text,
            completed: false,
            priority,
            tags,
            project,
            due_date,
            created_at: Local::now().naive_local().date(),
            recurrence,
            parent_id: None,
            depends_on: Vec::new(),
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

    /// Returns true if any dependency task is still pending.
    ///
    /// `all_tasks` is the full 0-indexed task list; IDs in `depends_on` are 1-based.
    pub fn is_blocked(&self, all_tasks: &[Task]) -> bool {
        self.depends_on.iter().any(|&dep_id| {
            let idx = dep_id.saturating_sub(1);
            all_tasks.get(idx).map(|t| !t.completed).unwrap_or(false)
        })
    }

    /// Returns the IDs of blocking (still-pending) dependencies.
    pub fn blocking_deps(&self, all_tasks: &[Task]) -> Vec<usize> {
        self.depends_on
            .iter()
            .copied()
            .filter(|&dep_id| {
                let idx = dep_id.saturating_sub(1);
                all_tasks.get(idx).map(|t| !t.completed).unwrap_or(false)
            })
            .collect()
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
    /// use chrono::NaiveDate;
    /// use todo_cli::models::{Task, Priority, Recurrence};
    ///
    /// let task = Task::new(
    ///     "Weekly review".to_string(),
    ///     Priority::Medium,
    ///     vec![],
    ///     None,
    ///     Some(NaiveDate::from_ymd_opt(2025, 2, 10).unwrap()),
    ///     Some(Recurrence::Weekly),
    /// );
    ///
    /// let next = task.create_next_recurrence(1).unwrap();
    /// assert_eq!(
    ///     next.due_date,
    ///     Some(NaiveDate::from_ymd_opt(2025, 2, 17).unwrap())
    /// );
    /// assert_eq!(next.parent_id, Some(1));
    /// ```
    pub fn create_next_recurrence(&self, parent_id: usize) -> Option<Task> {
        let recurrence = self.recurrence?;
        let current_due = self.due_date?;
        let next_due = recurrence.next_date(current_due);

        let mut next_task = Task::new(
            self.text.clone(),
            self.priority,
            self.tags.clone(),
            self.project.clone(),
            Some(next_due),
            Some(recurrence),
        );

        next_task.parent_id = Some(parent_id);
        // Dependencies are NOT propagated to recurrences — each occurrence stands alone.
        Some(next_task)
    }

    #[allow(dead_code)]
    pub fn is_recurring(&self) -> bool {
        self.recurrence.is_some()
    }
}

/// Detects a dependency cycle using iterative DFS.
///
/// Returns `Err` with the cycle description if adding `dep_id → task_id`
/// would create a cycle, `Ok(())` otherwise.
///
/// `tasks` is the full 0-indexed list; IDs are 1-based.
pub fn detect_cycle(tasks: &[Task], task_id: usize, new_dep_id: usize) -> Result<(), String> {
    // Would adding "task_id depends on new_dep_id" create a cycle?
    // A cycle exists if task_id is reachable FROM new_dep_id via depends_on edges.
    // i.e. check if task_id appears in the transitive deps of new_dep_id.
    let mut visited = std::collections::HashSet::new();
    let mut stack = vec![new_dep_id];

    while let Some(current) = stack.pop() {
        if current == task_id {
            return Err(format!(
                "Adding this dependency would create a cycle: \
                task #{} → task #{} → ... → task #{}",
                task_id, new_dep_id, task_id
            ));
        }

        if visited.insert(current) {
            if let Some(t) = tasks.get(current.saturating_sub(1)) {
                for &d in &t.depends_on {
                    stack.push(d);
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    fn make_task(text: &str) -> Task {
        Task::new(text.to_string(), Priority::Medium, vec![], None, None, None)
    }

    #[test]
    fn test_is_blocked_no_deps() {
        let task = make_task("A");
        assert!(!task.is_blocked(&[]));
    }

    #[test]
    fn test_is_blocked_pending_dep() {
        let dep = make_task("Dep");
        let mut task = make_task("Task");
        task.depends_on = vec![1];
        assert!(task.is_blocked(&[dep]));
    }

    #[test]
    fn test_is_blocked_completed_dep() {
        let mut dep = make_task("Dep");
        dep.completed = true;
        let mut task = make_task("Task");
        task.depends_on = vec![1];
        assert!(!task.is_blocked(&[dep]));
    }

    #[test]
    fn test_detect_cycle_direct() {
        let mut task = vec![make_task("A"), make_task("B")];
        // A depends on B
        task[0].depends_on = vec![2];
        // Adding B depends on A should fail
        let result = detect_cycle(&task, 2, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_no_cycle() {
        let tasks = vec![make_task("A"), make_task("B"), make_task("C")];
        // A->B, adding C->A should be fine
        let result = detect_cycle(&tasks, 3, 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_detect_transitive_cycle() {
        let mut tasks = vec![make_task("A"), make_task("B"), make_task("C")];
        // A depends on B, B depends on C
        tasks[0].depends_on = vec![2];
        tasks[1].depends_on = vec![3];
        // Adding C depends on A should fail (C->A->B->C)
        let result = detect_cycle(&tasks, 3, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_blocking_deps_returns_pending_only() {
        let mut dep1 = make_task("Dep1");
        dep1.completed = true;
        let dep2 = make_task("Dep2");
        let mut task = make_task("Task");
        task.depends_on = vec![1, 2];
        let blocking = task.blocking_deps(&[dep1, dep2]);
        assert_eq!(blocking, vec![2]);
    }

    fn make_recurring(recurrence: Option<Recurrence>, due: Option<NaiveDate>) -> Task {
        Task::new(
            "Test".to_string(),
            Priority::Medium,
            vec![],
            None,
            due,
            recurrence,
        )
    }

    #[test]
    fn test_daily_recurrence() {
        let date = NaiveDate::from_ymd_opt(2026, 2, 10).unwrap();
        let task = make_recurring(Some(Recurrence::Daily), Some(date));
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
        let task = make_recurring(Some(Recurrence::Weekly), Some(date));
        let next = task.create_next_recurrence(1).unwrap();
        assert_eq!(
            next.due_date,
            Some(NaiveDate::from_ymd_opt(2026, 2, 17).unwrap())
        );
    }

    #[test]
    fn test_monthly_recurrence() {
        let date = NaiveDate::from_ymd_opt(2026, 2, 10).unwrap();
        let task = make_recurring(Some(Recurrence::Monthly), Some(date));
        let next = task.create_next_recurrence(1).unwrap();
        assert_eq!(
            next.due_date,
            Some(NaiveDate::from_ymd_opt(2026, 3, 10).unwrap())
        );
    }

    #[test]
    fn test_monthly_boundary_case() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();
        let task = make_recurring(Some(Recurrence::Monthly), Some(date));
        let next = task.create_next_recurrence(1).unwrap();
        assert_eq!(
            next.due_date,
            Some(NaiveDate::from_ymd_opt(2026, 2, 28).unwrap())
        );
    }

    #[test]
    fn test_no_recurrence_returns_none() {
        let task = make_recurring(None, Some(NaiveDate::from_ymd_opt(2026, 2, 10).unwrap()));
        assert!(task.create_next_recurrence(1).is_none());
    }

    #[test]
    fn test_no_due_date_returns_none() {
        let task = make_recurring(Some(Recurrence::Daily), None);
        assert!(task.create_next_recurrence(1).is_none());
    }

    #[test]
    fn test_project_preserved_in_recurrence() {
        let date = NaiveDate::from_ymd_opt(2026, 2, 10).unwrap();
        let mut task = make_recurring(Some(Recurrence::Daily), Some(date));
        task.project = Some("work".to_string());
        let next = task.create_next_recurrence(1).unwrap();
        assert_eq!(next.project, Some("work".to_string()));
    }

    #[test]
    fn test_deps_not_propagated_to_recurrence() {
        let date = NaiveDate::from_ymd_opt(2026, 2, 10).unwrap();
        let mut task = make_recurring(Some(Recurrence::Daily), Some(date));
        task.depends_on = vec![1, 2];
        let next = task.create_next_recurrence(1).unwrap();
        assert!(
            next.depends_on.is_empty(),
            "recurrences should not inherit dependencies"
        );
    }
}
