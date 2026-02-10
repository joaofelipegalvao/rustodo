use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use directories::ProjectDirs;

use crate::models::Task;

/// Loads tasks from the todos.json file.
///
/// If the file doesn't exist, returns an empty vector.
/// If the file exists but is corrupted, returns an error.
///
/// # Errors
///
/// Returns an error if:
/// - The file exists but cannot be read
/// - The file contains invalid JSON
pub fn load_tasks() -> Result<Vec<Task>> {
    let path = get_data_file_path()?;

    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content)
            .context("Failed to parse todos.json - file may be corrupted"),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e).context(format!(
            "Failed to read todos.json from: {}",
            path.display()
        )),
    }
}

/// Saves tasks to the todos.json file.
///
/// The tasks are serialized to pretty-printed JSON format.
///
/// # Errors
///
/// Returns an error if:
/// - The tasks cannot be serialized to JSON
/// - The file cannot be written (e.g., permission issues)
pub fn save_tasks(tasks: &[Task]) -> Result<()> {
    let path = get_data_file_path()?;

    let json = serde_json::to_string_pretty(tasks).context("Failed to serialize tasks to JSON")?;

    fs::write(&path, json).context(format!(
        "Failed to write to {} - check file permission",
        path.display()
    ))?;

    Ok(())
}

/// Returns the path to the todos.json file in the user's config directory.
///
/// Creates the config directory if it doesn't exist.
///
/// # Platform-specific locations:
/// - Linux: `~/.config/todo-cli/todos.json`
/// - macOS: `~/Library/Application Support/todo-cli/todos.json`
/// - Windows: `C:\Users\{user}\AppData\Roaming\todo-cli\todos.json`
pub fn get_data_file_path() -> Result<PathBuf> {
    let project_dirs =
        ProjectDirs::from("", "", "todo-cli").context("Failed to determine project directories")?;

    let data_dir = project_dirs.data_dir();

    // Create directory if it doesn't exist
    fs::create_dir_all(data_dir).context(format!(
        "Failed to create data directory: {}",
        data_dir.display()
    ))?;

    let mut path = data_dir.to_path_buf();
    path.push("todos.json");

    Ok(path)
}
