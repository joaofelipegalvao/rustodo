//! Storage abstraction layer for task persistence

use crate::models::Task;
use anyhow::Result;

/// Trait defining storage operations for tasks
pub trait Storage {
    /// Load all tasks from storage
    fn load(&self) -> Result<Vec<Task>>;

    /// Save all tasks to storage
    fn save(&self, tasks: &[Task]) -> Result<()>;

    /// Get the storage location description
    fn location(&self) -> String;
}

// Re-export implementations
pub mod json;
pub mod memory;
pub use memory::InMemoryStorage;

pub use json::JsonStorage;

// Re-export old functions for backward compatibility (temporary)
pub use json::get_data_file_path;

pub fn load_tasks() -> Result<Vec<Task>> {
    let storage = JsonStorage::new()?;
    storage.load()
}

pub fn save_tasks(tasks: &[Task]) -> Result<()> {
    let storage = JsonStorage::new()?;
    storage.save(tasks)
}
