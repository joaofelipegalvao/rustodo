use anyhow::Result;
use colored::Colorize;

use crate::storage::{load_tasks, save_tasks};
use crate::validation::validate_task_id;

pub fn execute(id: usize) -> Result<()> {
    let mut tasks = load_tasks()?;
    validate_task_id(id, tasks.len())?;

    let index = id - 1;
    let task = &mut tasks[index];

    if task.recurrence.is_none() {
        println!("{} Task #{} has no recurrence", "".yellow(), id);
        return Ok(());
    }

    let old_pattern = task.recurrence.unwrap();
    task.recurrence = None;

    save_tasks(&tasks)?;

    println!(
        "{} Removed {} recurrence from task #{}",
        "✓".green(),
        old_pattern.description(),
        id,
    );

    Ok(())
}
