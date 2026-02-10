use anyhow::Result;
use colored::Colorize;

use crate::error::TodoError;
use crate::storage::{load_tasks, save_tasks};
use crate::validation::validate_task_id;

pub fn execute(id: usize) -> Result<()> {
    let mut tasks = load_tasks()?;
    validate_task_id(id, tasks.len())?;
    let index = id - 1;

    if tasks[index].completed {
        return Err(TodoError::TaskAlreadyInStatus {
            id,
            status: "completed".to_owned(),
        }
        .into());
    }

    tasks[index].mark_done();
    save_tasks(&tasks)?;
    println!("{}", "✓ Task marked as completed".green());
    Ok(())
}
