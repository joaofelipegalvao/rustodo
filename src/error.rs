use thiserror::Error;

/// Custom error types for the todo application.
///
/// These errors provide specific, user-friendly messages for common
/// error conditions that can occur during task management.
#[derive(Error, Debug)]
pub enum TodoError {
    #[error("Task ID {id} is invalid (valid range: 1-{max})")]
    InvalidTaskId { id: usize, max: usize },

    #[error("Task #{id} is already marked as {status}")]
    TaskAlreadyInStatus { id: usize, status: String },

    #[error("Tag '{0}' not found in any task")]
    TagNotFound(String),

    #[error("No tasks found matching the specified filters")]
    NoTasksFound,

    #[error("No tags found in any task")]
    NoTagsFound,

    #[error("Search returned no results for query: '{0}'")]
    NoSearchResults(String),
}
