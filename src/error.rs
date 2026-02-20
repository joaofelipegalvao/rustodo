//! Custom error types for the todo application

use chrono::NaiveDate;
use thiserror::Error;

/// Custom error types for the todo application.
///
/// These errors provide specific, user-friendly messages for common
/// error conditions that can occur during task management.
#[derive(Error, Debug)]
pub enum TodoError {
    // === ID Validation Errors ===
    #[error("Task ID {id} is invalid (valid range: 1-{max})")]
    InvalidTaskId { id: usize, max: usize },

    // === State Transition Errors ===
    #[error("Task #{id} is already marked as {status}")]
    TaskAlreadyInStatus { id: usize, status: String },

    // === Search/Filter Errors ===
    #[error("Tag '{0}' not found in any task")]
    TagNotFound(String),

    #[error("Project '{0} not found in any task'")]
    ProjectNotFound(String),

    #[error("No tasks found matching the specified filters")]
    NoTasksFound,

    #[error("No tags found in any task")]
    NoTagsFound,

    #[error("No projects found in any task")]
    NoProjectsFound,

    #[error("Search returned no results for query: '{0}'")]
    NoSearchResults(String),

    // === Text Validation Errors ===
    #[error("Task text cannot be empty")]
    EmptyTaskText,

    #[error("Task text too long (max: {max} characters, actual: {actual} characters)")]
    TaskTextTooLong { max: usize, actual: usize },

    // === Tag Validation Errors ===
    #[error("Tag cannot be empty")]
    EmptyTag,

    #[error("Tag too long (max: {max} characters, actual: {actual} characters)")]
    TagTooLong { max: usize, actual: usize },

    #[error(
        "Invalid tag format: '{tag}' (tags can only contain alphanumeric characters, hyphens, and underscores)"
    )]
    InvalidTagFormat { tag: String },

    #[error("Duplicate tag: '{tag}' (tags must be unique, case-insensitive)")]
    DuplicateTag { tag: String },

    // === Project Validation Erros ===
    #[error("Project name cannot be empty")]
    EmptyProjectName,

    #[error("Project name too long (max: {max} characters, actual: {actual} characters)")]
    ProjectNameTooLong { max: usize, actual: usize },

    // === Date Validation Errors ===
    #[error("Due date cannot be in the past: {date}")]
    DueDateInPast { date: NaiveDate },

    // === Recurrence Validation Errors ===
    #[error("Recurring tasks must have a due date. Use --due YYYY-MM-DD")]
    RecurrenceRequiresDueDate,
}
