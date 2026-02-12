use anyhow::Result;
use colored::Colorize;

use crate::models::Recurrence;
use crate::storage::{load_tasks, save_tasks};
use crate::validation::validate_task_id;

pub fn execute(id: usize, pattern: Recurrence) -> Result<()> {
    let mut tasks = load_tasks()?;
    validate_task_id(id, tasks.len())?;

    let index = id - 1;
    let task = &mut tasks[index];

    if task.due_date.is_none() {
        return Err(anyhow::anyhow!(
            "Task #{} has no due date. Add one with: todo edit {} --due YYYY-MM-DD",
            id,
            id
        ));
    }

    let old_recurrence = task.recurrence;
    task.recurrence = Some(pattern);

    save_tasks(&tasks)?;

    match old_recurrence {
        Some(old) if old == pattern => {
            println!(
                "{} Recurrence already set to {} for task #{}",
                "".yellow(),
                pattern.description(),
                id,
            );
        }
        Some(old) => {
            println!(
                "{} Updated recurrence for task #{}: {} → {}",
                "✓".green(),
                id,
                old.description(),
                pattern.description()
            );
        }
        None => {
            println!(
                "{} Set {} recurrence for task #{}",
                "✓".green(),
                id,
                pattern.description()
            );
        }
    }

    Ok(())
}
