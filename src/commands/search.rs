use anyhow::Result;

use crate::display::display_lists;
use crate::error::TodoError;
use crate::models::StatusFilter;
use crate::storage::Storage;

pub fn execute(
    storage: &impl Storage,
    query: String,
    tag: Option<String>,
    project: Option<String>,
    status: StatusFilter,
) -> Result<()> {
    let tasks = storage.load()?;

    // Perform case-insensitive search on task text
    let mut results: Vec<(usize, &_)> = tasks
        .iter()
        .enumerate()
        .filter(|(_, task)| task.text.to_lowercase().contains(&query.to_lowercase()))
        .filter(|(_, task)| task.matches_status(status))
        .map(|(i, task)| (i + 1, task))
        .collect();

    // Apply tag filter if specified
    if let Some(tag_name) = &tag {
        results.retain(|(_, task)| task.tags.contains(tag_name));
    }

    if let Some(project_name) = &project {
        results.retain(|(_, task)| {
            task.project
                .as_deref()
                .map(|p| p.to_lowercase() == project_name.to_lowercase())
                .unwrap_or(false)
        });
    }

    if results.is_empty() {
        return Err(TodoError::NoSearchResults(query).into());
    }

    display_lists(&results, &format!("Search results for \"{}\"", query));
    Ok(())
}
