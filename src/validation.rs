//! Input validation for task data
//!
//! This module provides comprehensive validation functions to ensure data integrity
//! before persisting tasks to storage.

use crate::error::TodoError;
use crate::models::{Recurrence, Task};
use chrono::NaiveDate;

/// Validates that a task ID is within valid range
///
/// Task IDs are 1-based (displayed to users as 1, 2, 3, etc.)
/// but stored in Vec at indices 0, 1, 2, etc.
///
/// # Arguments
///
/// * `id` - The task ID to validate (1-based)
/// * `max` - The maximum valid ID (total number of tasks)
///
/// # Errors
///
/// Returns `TodoError::InvalidTaskId` if the ID is 0 or greater than max.
pub fn validate_task_id(id: usize, max: usize) -> Result<(), TodoError> {
    if id == 0 || id > max {
        return Err(TodoError::InvalidTaskId { id, max });
    }
    Ok(())
}

/// Validates task text is not empty and within length limits
///
/// # Rules
///
/// - Text cannot be empty or whitespace-only
/// - Text cannot exceed 500 characters (trimmed)
///
/// # Errors
///
/// Returns:
/// - `TodoError::EmptyTaskText` if text is empty
/// - `TodoError::TaskTextTooLong` if text exceeds 500 characters
pub fn validate_task_text(text: &str) -> Result<(), TodoError> {
    let trimmed = text.trim();

    if trimmed.is_empty() {
        return Err(TodoError::EmptyTaskText);
    }

    const MAX_LENGTH: usize = 500;
    if trimmed.len() > MAX_LENGTH {
        return Err(TodoError::TaskTextTooLong {
            max: MAX_LENGTH,
            actual: trimmed.len(),
        });
    }

    Ok(())
}

/// Validates tags are properly formatted and unique
///
/// # Rules
///
/// - Tags cannot be empty or whitespace-only
/// - Tags cannot exceed 50 characters
/// - Tags can only contain alphanumeric characters, hyphens, and underscores
/// - No duplicate tags (case-insensitive)
///
/// # Errors
///
/// Returns:
/// - `TodoError::EmptyTag` if any tag is empty
/// - `TodoError::TagTooLong` if any tag exceeds 50 characters
/// - `TodoError::InvalidTagFormat` if any tag contains invalid characters
/// - `TodoError::DuplicateTag` if there are duplicate tags
pub fn validate_tags(tags: &[String]) -> Result<(), TodoError> {
    use std::collections::HashSet;

    const MAX_TAG_LENGTH: usize = 50;

    for tag in tags {
        let trimmed = tag.trim();

        // Empty check
        if trimmed.is_empty() {
            return Err(TodoError::EmptyTag);
        }

        // Length check
        if trimmed.len() > MAX_TAG_LENGTH {
            return Err(TodoError::TagTooLong {
                max: MAX_TAG_LENGTH,
                actual: trimmed.len(),
            });
        }

        // Format check: only alphanumeric, hyphen, underscore
        let valid_chars = trimmed
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_');

        if !valid_chars {
            return Err(TodoError::InvalidTagFormat {
                tag: trimmed.to_string(),
            });
        }
    }

    // Duplicate check (case-insensitive)
    let mut seen = HashSet::new();
    for tag in tags {
        let lowercase = tag.to_lowercase();
        if !seen.insert(lowercase.clone()) {
            return Err(TodoError::DuplicateTag { tag: tag.clone() });
        }
    }

    Ok(())
}

/// Validates due date is not in the past (for new tasks)
///
/// # Arguments
///
/// * `due_date` - Optional due date to validate
/// * `allow_past` - If true, allows past dates (for editing existing tasks)
///
/// # Errors
///
/// Returns `TodoError::DueDateInPast` if date is in the past and `allow_past` is false
pub fn validate_due_date(due_date: Option<NaiveDate>, allow_past: bool) -> Result<(), TodoError> {
    if let Some(due) = due_date
        && !allow_past
    {
        let today = chrono::Local::now().naive_local().date();
        if due < today {
            return Err(TodoError::DueDateInPast { date: due });
        }
    }
    Ok(())
}

/// Validates recurrence pattern has a due date
///
/// Recurring tasks MUST have a due date to calculate the next occurrence.
///
/// # Errors
///
/// Returns `TodoError::RecurrenceRequiresDueDate` if recurrence is set but due_date is None
pub fn validate_recurrence(
    recurrence: Option<Recurrence>,
    due_date: Option<NaiveDate>,
) -> Result<(), TodoError> {
    if recurrence.is_some() && due_date.is_none() {
        return Err(TodoError::RecurrenceRequiresDueDate);
    }
    Ok(())
}

/// Validates a complete task before saving
///
/// Runs all validation checks on a task.
///
/// # Arguments
///
/// * `task` - The task to validate
/// * `is_new` - If true, disallows past due dates; if false, allows them
///
/// # Errors
///
/// Returns the first validation error encountered, or Ok(()) if all checks pass
pub fn validate_task(task: &Task, is_new: bool) -> Result<(), TodoError> {
    validate_task_text(&task.text)?;
    validate_tags(&task.tags)?;
    validate_due_date(task.due_date, !is_new)?;
    validate_recurrence(task.recurrence, task.due_date)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Priority;

    #[test]
    fn test_validate_task_id() {
        assert!(validate_task_id(1, 10).is_ok());
        assert!(validate_task_id(10, 10).is_ok());
        assert!(validate_task_id(0, 10).is_err());
        assert!(validate_task_id(11, 10).is_err());
    }

    #[test]
    fn test_validate_task_text() {
        assert!(validate_task_text("Valid task").is_ok());
        assert!(validate_task_text("  spaces  ").is_ok());
        assert!(validate_task_text("a").is_ok());

        assert!(validate_task_text("").is_err());
        assert!(validate_task_text("   ").is_err());
        assert!(validate_task_text("\t\n").is_err());

        // 501 characters
        let too_long = "x".repeat(501);
        assert!(validate_task_text(&too_long).is_err());

        // 500 characters (OK)
        let exactly_max = "x".repeat(500);
        assert!(validate_task_text(&exactly_max).is_ok());
    }

    #[test]
    fn test_validate_tags() {
        // Valid tags
        assert!(validate_tags(&vec!["work".to_string()]).is_ok());
        assert!(validate_tags(&vec!["work-urgent".to_string()]).is_ok());
        assert!(validate_tags(&vec!["task_1".to_string()]).is_ok());
        assert!(
            validate_tags(&vec![
                "work".to_string(),
                "urgent".to_string(),
                "high-priority".to_string(),
            ])
            .is_ok()
        );

        // Empty tag
        assert!(validate_tags(&vec!["".to_string()]).is_err());
        assert!(validate_tags(&vec!["work".to_string(), "".to_string()]).is_err());

        // Invalid characters
        assert!(validate_tags(&vec!["work@home".to_string()]).is_err());
        assert!(validate_tags(&vec!["tag with spaces".to_string()]).is_err());
        assert!(validate_tags(&vec!["tag/slash".to_string()]).is_err());

        // Too long
        let long_tag = "x".repeat(51);
        assert!(validate_tags(&vec![long_tag]).is_err());

        // Duplicates (case-insensitive)
        assert!(validate_tags(&vec!["work".to_string(), "work".to_string()]).is_err());
        assert!(validate_tags(&vec!["work".to_string(), "Work".to_string()]).is_err());
        assert!(validate_tags(&vec!["work".to_string(), "WORK".to_string()]).is_err());
    }

    #[test]
    fn test_validate_due_date() {
        use chrono::Local;

        let today = Local::now().naive_local().date();
        let yesterday = today - chrono::Duration::days(1);
        let tomorrow = today + chrono::Duration::days(1);

        // Future dates always OK
        assert!(validate_due_date(Some(tomorrow), false).is_ok());
        assert!(validate_due_date(Some(tomorrow), true).is_ok());

        // Today is OK
        assert!(validate_due_date(Some(today), false).is_ok());
        assert!(validate_due_date(Some(today), true).is_ok());

        // Past dates: allowed only if allow_past = true
        assert!(validate_due_date(Some(yesterday), false).is_err());
        assert!(validate_due_date(Some(yesterday), true).is_ok());

        // None is always OK
        assert!(validate_due_date(None, false).is_ok());
        assert!(validate_due_date(None, true).is_ok());
    }

    #[test]
    fn test_validate_recurrence() {
        use chrono::Local;

        let today = Local::now().naive_local().date();
        let future = today + chrono::Duration::days(7);

        // Recurrence without due date = error
        assert!(validate_recurrence(Some(Recurrence::Daily), None).is_err());
        assert!(validate_recurrence(Some(Recurrence::Weekly), None).is_err());
        assert!(validate_recurrence(Some(Recurrence::Monthly), None).is_err());

        // Recurrence with due date = OK
        assert!(validate_recurrence(Some(Recurrence::Daily), Some(future)).is_ok());
        assert!(validate_recurrence(Some(Recurrence::Weekly), Some(future)).is_ok());
        assert!(validate_recurrence(Some(Recurrence::Monthly), Some(future)).is_ok());

        // No recurrence = always OK
        assert!(validate_recurrence(None, None).is_ok());
        assert!(validate_recurrence(None, Some(future)).is_ok());
    }

    #[test]
    fn test_validate_task_new() {
        use chrono::Local;

        let today = Local::now().naive_local().date();
        let future = today + chrono::Duration::days(7);

        // Valid new task
        let task = Task::new(
            "Valid task".to_string(),
            Priority::Medium,
            vec!["work".to_string()],
            Some(future),
            None,
        );
        assert!(validate_task(&task, true).is_ok());

        // Invalid text
        let mut invalid = task.clone();
        invalid.text = "".to_string();
        assert!(validate_task(&invalid, true).is_err());

        // Invalid tags
        let mut invalid = task.clone();
        invalid.tags = vec!["invalid tag with spaces".to_string()];
        assert!(validate_task(&invalid, true).is_err());

        // Past due date (new task)
        let yesterday = today - chrono::Duration::days(1);
        let mut invalid = task.clone();
        invalid.due_date = Some(yesterday);
        assert!(validate_task(&invalid, true).is_err());

        // Recurrence without due date
        let mut invalid = task.clone();
        invalid.recurrence = Some(Recurrence::Daily);
        invalid.due_date = None;
        assert!(validate_task(&invalid, true).is_err());
    }

    #[test]
    fn test_validate_task_existing() {
        use chrono::Local;

        let today = Local::now().naive_local().date();
        let yesterday = today - chrono::Duration::days(1);

        // Existing task with past due date = OK
        let task = Task::new(
            "Past task".to_string(),
            Priority::High,
            vec![],
            Some(yesterday),
            None,
        );
        assert!(validate_task(&task, false).is_ok());
    }
}
