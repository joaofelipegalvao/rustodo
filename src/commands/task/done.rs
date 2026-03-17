//! Handler for `todo done <ID>`.

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

    if tasks[index].completed {
        return Err(TodoError::TaskAlreadyInStatus {
            id,
            status: "completed".to_owned(),
        }
        .into());
    }

    let blocking = tasks[index].blocking_deps(&tasks);
    if !blocking.is_empty() {
        let vis: Vec<usize> = tasks
            .iter()
            .enumerate()
            .filter(|(_, t)| !t.is_deleted())
            .map(|(i, _)| i)
            .collect();
        let ids = blocking
            .iter()
            .filter_map(|uuid| {
                let real_pos = tasks.iter().position(|t| t.uuid == *uuid)?;
                let vis_id = vis.iter().position(|&i| i == real_pos).map(|p| p + 1)?;
                let text = tasks[real_pos].text.clone();
                Some(format!("#{} \"{}\"", vis_id, text))
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
            let next_uuid = next_task.uuid;
            tasks.push(next_task);
            let next_vis_id = tasks.iter().filter(|t| !t.is_deleted()).count();
            storage.save(&tasks)?;
            storage.record_event(EntityType::Task, task_uuid, EventType::Completed)?;
            storage.record_event(EntityType::Task, next_uuid, EventType::Created)?;

            let msg = format!(
                "Task #{} marked as done. Next recurrence: #{} (due {})",
                id,
                next_vis_id,
                next_due.format("%Y-%m-%d")
            );
            if !silent {
                println!("Task {} marked as done.", format!("#{}", id).green());
                println!(
                    "Task {} created (due {})",
                    format!("#{}", next_vis_id).yellow(),
                    next_due.format("%Y-%m-%d")
                );
            }
            Ok(msg)
        } else {
            storage.save(&tasks)?;
            storage.record_event(EntityType::Task, task_uuid, EventType::Completed)?;
            if !silent {
                println!("Task {} marked as done.", format!("#{}", id).green());
                println!(
                    "{}",
                    "Next recurrence already exists, skipping creation.".dimmed()
                );
            }
            Ok(format!("Task #{} marked as done.", id))
        }
    } else {
        storage.save(&tasks)?;
        storage.record_event(EntityType::Task, task_uuid, EventType::Completed)?;
        if !silent {
            println!("Task {} marked as done.", format!("#{}", id).green());
        }
        Ok(format!("Task #{} marked as done.", id))
    }
}
