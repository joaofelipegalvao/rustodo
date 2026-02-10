use crate::error::TodoError;

/// Validates that a task ID is within the valid range.
///
/// # Arguments
///
/// * `id` - The task ID to validate (1-based indexing)
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
