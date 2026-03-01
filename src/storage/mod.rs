//! Storage abstraction layer for task persistence.
//!
//! This module defines the [`Storage`] trait and re-exports two built-in
//! implementations:
//!
//! | Type | Description |
//! |---|---|
//! | [`JsonStorage`] | Persists tasks to a JSON file in the OS data directory |
//! | [`InMemoryStorage`] | Stores tasks in memory — ideal for tests |
//!
//! ## Implementing a custom storage backend
//!
//! Implement [`Storage`] to add your own persistence layer (SQLite, cloud
//! sync, encrypted file, etc.):
//!
//! ```no_run
//! use anyhow::Result;
//! use rustodo::models::Task;
//! use rustodo::storage::Storage;
//!
//! struct MyStorage;
//!
//! impl Storage for MyStorage {
//!     fn load(&self) -> Result<Vec<Task>> {
//!         // read from your backend
//!         Ok(vec![])
//!     }
//!
//!     fn save(&self, tasks: &[Task]) -> Result<()> {
//!         // write to your backend
//!         Ok(())
//!     }
//!
//!     fn location(&self) -> String {
//!         "my-backend".to_string()
//!     }
//! }
//! ```

use crate::models::Task;
use anyhow::Result;

/// Trait defining storage operations for tasks.
///
/// All command handlers depend on this trait rather than a concrete type,
/// making it easy to swap backends or inject fakes in tests.
pub trait Storage {
    /// Load all tasks from storage.
    ///
    /// Returns an empty `Vec` if no tasks have been saved yet (i.e. the
    /// backing file does not exist).
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying store cannot be read or if the
    /// serialized data is malformed.
    fn load(&self) -> Result<Vec<Task>>;

    /// Persist all tasks to storage, replacing any previously saved state.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying store cannot be written (e.g.
    /// permission denied, disk full).
    fn save(&self, tasks: &[Task]) -> Result<()>;

    /// Returns a human-readable description of the storage location.
    ///
    /// Used by `todo info` to display where data is being persisted.
    /// Examples: a filesystem path or `"memory"` for the in-memory backend.
    #[allow(dead_code)]
    fn location(&self) -> String;
}

pub mod json;
pub mod memory;

pub use json::JsonStorage;
pub use memory::InMemoryStorage;

pub use json::get_data_file_path;
