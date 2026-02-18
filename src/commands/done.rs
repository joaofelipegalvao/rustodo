use anyhow::Result;
use colored::Colorize;

use crate::error::TodoError;
use crate::storage::Storage;
use crate::validation::validate_task_id;

pub fn execute(storage: &impl Storage, id: usize) -> Result<()> {
    let mut tasks = storage.load()?;
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

    if let Some(next_task) = tasks[index].create_next_recurrence(id) {
        let next_due = next_task.due_date.unwrap();

        // Check for existing task with same due date that:
        // 1. Has the same parent_id (part of same recurring chain), OR
        // 2. Has the same text (fallback for backward compatibility)
        let already_exists = tasks.iter().any(|t| {
            !t.completed
                && t.due_date == Some(next_due)
                && (t.parent_id == Some(id) || t.text == next_task.text)
        });

        if !already_exists {
            tasks.push(next_task);
            storage.save(&tasks)?;
            let next_id = tasks.len();
            println!("{}", "✓ Task marked as completed".green());
            println!(
                "{} Task #{} created (due {})",
                "↻".cyan(),
                next_id,
                next_due.format("%Y-%m-%d")
            );
        } else {
            storage.save(&tasks)?;
            println!("{}", "✓ Task marked as completed".green());
            println!(
                "{}",
                "Next recurrence already exists, skipping creation.".dimmed()
            );
        }
    } else {
        storage.save(&tasks)?;
        println!("{}", "✓ Task marked as completed".green());
    }
    Ok(())
}
