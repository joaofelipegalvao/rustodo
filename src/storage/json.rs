//! JSON file-based storage implementation.
//!
//! Tasks are stored as a pretty-printed JSON array at a platform-specific
//! path resolved by [`get_data_file_path`]:
//!
//! | Platform | Path |
//! |----------|------|
//! | Linux | `~/.local/share/rustodo/todos.json` |
//! | macOS | `~/Library/Application Support/rustodo/todos.json` |
//! | Windows | `%APPDATA%\rustodo\todos.json` |
//!
//! The directory is created automatically on first use.
use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::{fs, path::PathBuf};

use super::Storage;
use crate::models::Task;

/// JSON file-based storage implementation
pub struct JsonStorage {
    file_path: PathBuf,
}

impl JsonStorage {
    /// Creates a new [`JsonStorage`] pointing to the default OS data directory.
    ///
    /// The data directory is created automatically if it does not exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the OS data directory cannot be determined or
    /// if the directory cannot be created.
    pub fn new() -> Result<Self> {
        let file_path = get_data_file_path()?;
        Ok(Self { file_path })
    }

    /// Creates a [`JsonStorage`] at an arbitrary path.
    ///
    /// Intended for use in tests where a [`tempfile::TempDir`] provides an
    /// isolated, automatically-cleaned directory.
    #[cfg(test)]
    pub fn with_path(file_path: PathBuf) -> Self {
        Self { file_path }
    }
}

impl Storage for JsonStorage {
    fn load(&self) -> Result<Vec<Task>> {
        match fs::read_to_string(&self.file_path) {
            Ok(content) => serde_json::from_str(&content)
                .context("Failed to parse todos.json - file may be corrupted"),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(e) => Err(e).context(format!(
                "Failed to read todos.json from: {}",
                self.file_path.display()
            )),
        }
    }

    fn save(&self, tasks: &[Task]) -> Result<()> {
        let json =
            serde_json::to_string_pretty(tasks).context("Failed to serialize tasks to JSON")?;

        fs::write(&self.file_path, json).context(format!(
            "Failed to write to {} - check file permissions",
            self.file_path.display()
        ))?;

        Ok(())
    }

    fn location(&self) -> String {
        self.file_path.display().to_string()
    }
}

/// Returns the path to the todos.json file (re-exported for compatibility)
pub fn get_data_file_path() -> Result<PathBuf> {
    let project_dirs =
        ProjectDirs::from("", "", "rustodo").context("Failed to determine project directories")?;

    let data_dir = project_dirs.data_dir();
    fs::create_dir_all(data_dir).context(format!(
        "Failed to create data directory: {}",
        data_dir.display()
    ))?;

    let mut path = data_dir.to_path_buf();
    path.push("todos.json");

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Priority;
    use tempfile::TempDir;

    #[test]
    fn test_json_storage_save_and_load() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.json");

        let storage = JsonStorage::with_path(path.clone());

        let tasks = vec![crate::models::Task::new(
            "Test task".to_string(),
            Priority::Medium,
            vec![],
            None,
            None,
            None,
        )];
        storage.save(&tasks).unwrap();

        assert!(path.exists());

        let loaded = storage.load().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].text, "Test task");
    }

    #[test]
    fn test_json_storage_empty_file() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("empty.json");

        let storage = JsonStorage::with_path(path);

        let tasks = storage.load().unwrap();
        assert_eq!(tasks.len(), 0);
    }
}
