//! Handler for `todo undone <ID>`.

use anyhow::Result;
use colored::Colorize;

use crate::error::TodoError;
use crate::storage::{EntityType, EventType, Storage};
use crate::utils::validation::resolve_visible_index;

pub fn execute(storage: &impl Storage, id: usize) -> Result<()> {
    execute_inner(storage, id, false)?;
    Ok(())
}

pub fn execute_silent(storage: &impl Storage, id: usize) -> Result<String> {
    execute_inner(storage, id, true)
}

fn execute_inner(storage: &impl Storage, id: usize, silent: bool) -> Result<String> {
    let mut tasks = storage.load()?;

    let index = resolve_visible_index(&tasks, id, |t| t.is_deleted())
        .map_err(|_| anyhow::anyhow!("invalid task ID: {}", id))?;

    if !tasks[index].completed {
        return Err(TodoError::TaskAlreadyInStatus {
            id,
            status: "pending".to_owned(),
        }
        .into());
    }

    let task_uuid = tasks[index].uuid;
    tasks[index].mark_undone();
    storage.save(&tasks)?;
    storage.record_event(EntityType::Task, task_uuid, EventType::Uncompleted)?;

    if !silent {
        println!("Task {} marked as pending.", format!("#{}", id).yellow());
    }
    Ok(format!("Task #{} marked as pending.", id))
}
