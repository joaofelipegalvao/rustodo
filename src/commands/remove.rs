//! Handler for `todo remove <ID>`.
//!
//! Shows the task text, optionally prompts for confirmation, then soft-deletes
//! the task by setting its `deleted_at` timestamp. The task remains in storage
//! so that sync can propagate the deletion to other devices via last-write-wins.
//!
//! IDs are 1-based positions in the **visible** (non-deleted) task list.
use crate::storage::Storage;
use crate::utils::confirm;
use crate::validation::validate_task_id;
use anyhow::Result;
use colored::Colorize;

pub fn execute(storage: &impl Storage, id: usize, yes: bool) -> Result<()> {
    let mut tasks = storage.load()?;
    let visible_indices: Vec<usize> = tasks
        .iter()
        .enumerate()
        .filter(|(_, t)| !t.is_deleted())
        .map(|(i, _)| i)
        .collect();
    validate_task_id(id, visible_indices.len())?;
    let real_index = visible_indices[id - 1];
    let task_text = tasks[real_index].text.clone();

    if !yes {
        println!("\n{} {}", "".yellow(), task_text.bright_white());
        if !confirm("Are you sure? [y/N]:")? {
            println!("{} Removal cancelled.", "".yellow());
            return Ok(());
        }
    }

    tasks[real_index].soft_delete();
    storage.save(&tasks)?;

    println!(
        "{} {}",
        "✓".green(),
        format!("Task removed: {}", task_text).dimmed()
    );

    Ok(())
}

/// TUI variant: no stdout, returns the task text for the status message.
pub fn execute_silent(storage: &impl Storage, id: usize) -> Result<String> {
    let mut tasks = storage.load()?;
    let visible_indices: Vec<usize> = tasks
        .iter()
        .enumerate()
        .filter(|(_, t)| !t.is_deleted())
        .map(|(i, _)| i)
        .collect();
    validate_task_id(id, visible_indices.len())?;
    let real_index = visible_indices[id - 1];
    let task_text = tasks[real_index].text.clone();

    tasks[real_index].soft_delete();
    storage.save(&tasks)?;

    Ok(format!("Removed: {}", task_text))
}
