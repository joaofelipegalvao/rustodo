//! Handler for `todo add`.
//!
//! Validates input, normalizes tags against existing ones via
//! [`tag_normalizer`](crate::tag_normalizer), creates a new [`Task`], appends
//! it to storage, and prints a confirmation message.
//!
//! [`Task`]: crate::models::Task

use anyhow::Result;
use colored::Colorize;

use crate::cli::AddArgs;
use crate::error::TodoError;
use crate::models::Task;
use crate::storage::Storage;
use crate::tag_normalizer::{collect_existing_tags, normalize_tags};
use crate::{date_parser, validation};

pub fn execute(storage: &impl Storage, args: AddArgs) -> Result<()> {
    validation::validate_task_text(&args.text)?;
    validation::validate_tags(&args.tag)?;
    if let Some(ref p) = args.project {
        validation::validate_project_name(p)?;
    }

    let due = if let Some(ref due_str) = args.due {
        Some(date_parser::parse_date_not_in_past(due_str)?)
    } else {
        None
    };

    validation::validate_due_date(due, false)?;
    validation::validate_recurrence(args.recurrence, due)?;

    let mut tasks = storage.load()?;
    let new_id = tasks.len() + 1;

    for &dep_id in &args.depends_on {
        if dep_id == new_id {
            return Err(TodoError::SelfDependency { task_id: new_id }.into());
        }
        validation::validate_task_id(dep_id, tasks.len())?;
    }

    let dep_uuids: Vec<uuid::Uuid> = args
        .depends_on
        .iter()
        .map(|&dep_id| validation::resolve_uuid(dep_id, &tasks))
        .collect::<Result<_, _>>()
        .map_err(anyhow::Error::from)?;

    let existing_tags = collect_existing_tags(&tasks);
    let (normalized_tags, normalization_messages) = normalize_tags(args.tag, &existing_tags);

    let mut task = Task::new(
        args.text,
        args.priority,
        normalized_tags,
        args.project,
        due,
        args.recurrence,
    );
    task.depends_on = dep_uuids;
    tasks.push(task);

    let id = tasks.len();
    storage.save(&tasks)?;

    let ok = "✓".green();

    for msg in &normalization_messages {
        println!("  {} Tag normalized: {}", "~".yellow(), msg.yellow());
    }

    if let Some(pattern) = args.recurrence {
        println!("{} Added task #{} with {} recurrence", ok, id, pattern);
    } else {
        println!("{} Added task #{}", ok, id);
    }

    Ok(())
}
