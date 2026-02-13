use anyhow::Result;
use chrono::NaiveDate;
use colored::Colorize;

use crate::models::Priority;
use crate::storage::{load_tasks, save_tasks};
use crate::validation::validate_task_id;

#[allow(clippy::too_many_arguments)]
pub fn execute(
    id: usize,
    text: Option<String>,
    priority: Option<Priority>,
    add_tag: Vec<String>,
    remove_tag: Vec<String>,
    due: Option<NaiveDate>,
    clear_due: bool,
    clear_tags: bool,
) -> Result<()> {
    let mut tasks = load_tasks()?;
    validate_task_id(id, tasks.len())?;

    let index = id - 1;
    let task = &mut tasks[index];

    let mut changes = Vec::new();

    // Update text if provided AND different
    if let Some(new_text) = text {
        if new_text.trim().is_empty() {
            return Err(anyhow::anyhow!("Task text cannot be empty"));
        }
        if task.text != new_text {
            task.text = new_text.clone();
            changes.push(format!("text → {}", new_text.bright_white()));
        }
    }

    // Update priority if provided AND different
    if let Some(new_priority) = priority
        && task.priority != new_priority
    {
        task.priority = new_priority;
        changes.push(format!("priority → {}", new_priority.letter()));
    }

    // Update tags
    if clear_tags {
        if !task.tags.is_empty() {
            let old_tags = task.tags.clone();
            task.tags.clear();
            changes.push(format!(
                "tags cleared → was [{}]",
                old_tags.join(", ").dimmed()
            ));
        }
    } else {
        // Remove specific tags
        if !remove_tag.is_empty() {
            let before_len = task.tags.len();
            let mut removed = Vec::new();

            task.tags.retain(|t| {
                if remove_tag.contains(t) {
                    removed.push(t.clone());
                    false
                } else {
                    true
                }
            });

            if !removed.is_empty() {
                changes.push(format!("removed tags → [{}]", removed.join(", ").red()));
            } else if before_len > 0 {
                // User tried to remove tags that don't exist
                return Err(anyhow::anyhow!(
                    "None of the specified tags [{}] exist in task #{}",
                    remove_tag.join(", "),
                    id
                ));
            }
        }

        // Add new tags
        if !add_tag.is_empty() {
            let mut added = Vec::new();

            for new_tag in &add_tag {
                if !task.tags.contains(new_tag) {
                    task.tags.push(new_tag.clone());
                    added.push(new_tag.clone());
                }
            }

            if !added.is_empty() {
                changes.push(format!("added tags → [{}]", added.join(", ").cyan()));
            }
        }
    }

    // Update due date
    if clear_due {
        if task.due_date.is_some() {
            task.due_date = None;
            changes.push("due date → cleared".dimmed().to_string());
        }
    } else if let Some(new_due) = due
        && task.due_date != Some(new_due)
    {
        task.due_date = Some(new_due);
        changes.push(format!("due date → {}", new_due.to_string().cyan()));
    }

    // Check if anything was actually changed
    if changes.is_empty() {
        println!(
            "{} No changes made (values are already set to the specified values).",
            "".blue()
        );
        return Ok(());
    }

    save_tasks(&tasks)?;

    println!("{} Task #{} updated:", "✓".green(), id);
    for change in changes {
        println!("  • {}", change);
    }

    Ok(())
}
