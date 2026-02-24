//! Handler for `todo add`.
//!
//! Validates input, normalizes tags against existing ones via
//! [`tag_normalizer`](crate::tag_normalizer), creates a new [`Task`], appends
//! it to storage, and prints a confirmation message.
//!
//! [`Task`]: crate::models::Task

use anyhow::Result;
use chrono::NaiveDate;
use colored::Colorize;

use crate::error::TodoError;
use crate::models::{Priority, Recurrence, Task};
use crate::storage::Storage;
use crate::tag_normalizer::{collect_existing_tags, normalize_tags};
use crate::validation;

#[allow(clippy::too_many_arguments)]
pub fn execute(
    storage: &impl Storage,
    text: String,
    priority: Priority,
    tags: Vec<String>,
    project: Option<String>,
    due: Option<NaiveDate>,
    recur: Option<Recurrence>,
    depends_on: Vec<usize>,
) -> Result<()> {
    validation::validate_task_text(&text)?;
    validation::validate_tags(&tags)?;
    if let Some(ref p) = project {
        validation::validate_project_name(p)?;
    }
    validation::validate_due_date(due, false)?;
    validation::validate_recurrence(recur, due)?;

    let mut tasks = storage.load()?;

    let new_id = tasks.len() + 1;

    for &dep_id in &depends_on {
        if dep_id == new_id {
            return Err(TodoError::SelfDependency { task_id: new_id }.into());
        }
        validation::validate_task_id(dep_id, tasks.len())?;
    }

    let existing_tags = collect_existing_tags(&tasks);
    let (normalized_tags, normalization_messages) = normalize_tags(tags, &existing_tags);

    let mut task = Task::new(text, priority, normalized_tags, project, due, recur);
    task.depends_on = depends_on;
    tasks.push(task);

    let id = tasks.len();
    storage.save(&tasks)?;

    let ok = "✓".green();

    for msg in &normalization_messages {
        println!("  {} Tag normalized: {}", "~".yellow(), msg.yellow());
    }

    if let Some(pattern) = recur {
        println!("{} Added task #{} with {} recurrence", ok, id, pattern);
    } else {
        println!("{} Added task #{}", ok, id);
    }

    Ok(())
}
