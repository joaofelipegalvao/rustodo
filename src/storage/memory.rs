//! In-memory storage implementation for testing

use anyhow::Result;
use std::cell::RefCell;

use super::Storage;
use crate::models::Task;

/// In-memory storage implementation
///
/// Stores tasks in memory without any file I/O, making tests fast and isolated.
/// Uses `RefCell` for interior mutability since `Storage` trait methods take `&self`.
///
/// # Examples
///
/// ```
/// use todo_cli::storage::InMemoryStorage;
///
/// let storage = InMemoryStorage::default();
/// // Use in tests without touching filesystem
/// ```
#[derive(Default)]
pub struct InMemoryStorage {
    tasks: RefCell<Vec<Task>>,
}

#[allow(dead_code)]
impl InMemoryStorage {
    /// Create in-memory storage pre-populated with tasks
    ///
    /// Useful for setting up test fixtures.
    pub fn with_tasks(tasks: Vec<Task>) -> Self {
        Self {
            tasks: RefCell::new(tasks),
        }
    }

    /// Get current number of tasks (for assertions)
    pub fn len(&self) -> usize {
        self.tasks.borrow().len()
    }

    /// Check if storage is empty (for assertions)
    pub fn is_empty(&self) -> bool {
        self.tasks.borrow().is_empty()
    }
}

impl Storage for InMemoryStorage {
    fn load(&self) -> Result<Vec<Task>> {
        Ok(self.tasks.borrow().clone())
    }

    fn save(&self, tasks: &[Task]) -> Result<()> {
        *self.tasks.borrow_mut() = tasks.to_vec();
        Ok(())
    }

    fn location(&self) -> String {
        "memory".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Priority;

    #[test]
    fn test_memory_storage_starts_empty() {
        let storage = InMemoryStorage::default();

        assert_eq!(storage.len(), 0);
        assert!(storage.is_empty());
        assert_eq!(storage.load().unwrap().len(), 0);
    }

    #[test]
    fn test_memory_storage_save_and_load() {
        let storage = InMemoryStorage::default();

        // Save
        let tasks = vec![
            Task::new("Task 1".to_string(), Priority::High, vec![], None, None),
            Task::new("Task 2".to_string(), Priority::Low, vec![], None, None),
        ];
        storage.save(&tasks).unwrap();

        // Load
        let loaded = storage.load().unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].text, "Task 1");
        assert_eq!(loaded[1].text, "Task 2");
    }

    #[test]
    fn test_memory_storage_with_tasks() {
        let tasks = vec![Task::new(
            "Existing".to_string(),
            Priority::Medium,
            vec![],
            None,
            None,
        )];

        let storage = InMemoryStorage::with_tasks(tasks);

        assert_eq!(storage.len(), 1);
        assert!(!storage.is_empty());

        let loaded = storage.load().unwrap();
        assert_eq!(loaded[0].text, "Existing");
    }

    #[test]
    fn test_memory_storage_overwrite() {
        let storage = InMemoryStorage::default();

        // Save first batch
        let tasks1 = vec![Task::new(
            "Task 1".to_string(),
            Priority::Medium,
            vec![],
            None,
            None,
        )];
        storage.save(&tasks1).unwrap();
        assert_eq!(storage.len(), 1);

        // Save second batch (overwrites)
        let tasks2 = vec![
            Task::new("Task 2".to_string(), Priority::High, vec![], None, None),
            Task::new("Task 3".to_string(), Priority::Low, vec![], None, None),
        ];
        storage.save(&tasks2).unwrap();

        // Should have new tasks only
        assert_eq!(storage.len(), 2);
        let loaded = storage.load().unwrap();
        assert_eq!(loaded[0].text, "Task 2");
        assert_eq!(loaded[1].text, "Task 3");
    }

    #[test]
    fn test_memory_storage_location() {
        let storage = InMemoryStorage::default();
        assert_eq!(storage.location(), "memory");
    }
}
