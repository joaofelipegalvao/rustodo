use anyhow::Result;
use colored::Colorize;

use crate::error::TodoError;
use crate::storage::{load_tasks, save_tasks};
use crate::validation::validate_task_id;

pub fn execute(id: usize) -> Result<()> {
    let mut tasks = load_tasks()?;
    validate_task_id(id, tasks.len())?;
    let index = id - 1;

    if !tasks[index].completed {
        return Err(TodoError::TaskAlreadyInStatus {
            id,
            status: "pending".to_owned(),
        }
        .into());
    }

    tasks[index].mark_undone();
    save_tasks(&tasks)?;
    println!("{}", "✓ Task unmarked".yellow());
    Ok(())
}
