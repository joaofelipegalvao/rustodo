//! Handler for `todo done <ID>`.
//!
//! Marks a task as completed. If the task has pending dependencies it is
//! rejected with a [`TodoError::TaskBlocked`]
//! error. For recurring tasks a new occurrence is automatically created via
//! [`Task::create_next_recurrence`](crate::models::Task::create_next_recurrence).

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

    // Block completion if dependencies are still pending
    let blocking = tasks[index].blocking_deps(&tasks);
    if !blocking.is_empty() {
        let ids = blocking
            .iter()
            .filter_map(|uuid| tasks.iter().position(|t| t.uuid == *uuid).map(|i| i + 1))
            .map(|num_id| {
                let text = tasks
                    .get(num_id - 1)
                    .map(|t| format!("\"{}\"", t.text))
                    .unwrap_or_default();
                format!("#{} {}", num_id, text)
            })
            .collect::<Vec<_>>()
            .join(", ");
        return Err(TodoError::TaskBlocked(id, ids).into());
    }

    tasks[index].mark_done();

    let task_uuid = tasks[index].uuid;

    if let Some(next_task) = tasks[index].create_next_recurrence(task_uuid) {
        let next_due = next_task.due_date.unwrap();

        let already_exists = tasks.iter().any(|t| {
            !t.completed
                && t.due_date == Some(next_due)
                && (t.parent_id == Some(task_uuid) || t.text == next_task.text)
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
