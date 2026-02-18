use anyhow::Result;
use colored::Colorize;

use crate::models::Recurrence;
use crate::storage::Storage;
use crate::validation::validate_task_id;

pub fn execute(storage: &impl Storage, id: usize, pattern: Recurrence) -> Result<()> {
    let mut tasks = storage.load()?;
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

    storage.save(&tasks)?;

    match old_recurrence {
        Some(old) if old == pattern => {
            println!(
                "{} Recurrence already set to {} for task #{}",
                "".yellow(),
                pattern,
                id,
            );
        }
        Some(old) => {
            println!(
                "{} Updated recurrence for task #{}: {} → {}",
                "✓".green(),
                id,
                old,
                pattern
            );
        }
        None => {
            println!(
                "{} Set {} recurrence for task #{}",
                "✓".green(),
                id,
                pattern,
            );
        }
    }

    Ok(())
}
