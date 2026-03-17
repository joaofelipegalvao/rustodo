//! Handler for `todo norecur <ID>`.
//!
//! Removes the recurrence pattern from a single task without deleting it.
//! The task remains and can still be completed manually.

use anyhow::Result;
use colored::Colorize;

use crate::storage::Storage;
use crate::utils::validation::resolve_visible_index;

pub fn execute(storage: &impl Storage, id: usize) -> Result<()> {
    let mut tasks = storage.load()?;

    let index = resolve_visible_index(&tasks, id, |t| t.is_deleted())
        .map_err(|_| anyhow::anyhow!("invalid task ID: {}", id))?;

    let task = &mut tasks[index];

    let Some(old_pattern) = task.recurrence.take() else {
        println!("{} Task #{} has no recurrence", "".yellow(), id);
        return Ok(());
    };

    task.touch();

    storage.save(&tasks)?;

    println!(
        "{} Removed {} recurrence from task #{}",
        "✓".green(),
        old_pattern,
        id,
    );

    Ok(())
}
