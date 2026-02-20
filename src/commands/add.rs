use anyhow::Result;
use chrono::NaiveDate;
use colored::Colorize;

use crate::models::{Priority, Recurrence, Task};
use crate::storage::Storage;
use crate::validation;

pub fn execute(
    storage: &impl Storage,
    text: String,
    priority: Priority,
    tags: Vec<String>,
    project: Option<String>,
    due: Option<NaiveDate>,
    recur: Option<Recurrence>,
) -> Result<()> {
    validation::validate_task_text(&text)?;
    validation::validate_tags(&tags)?;

    if let Some(ref p) = project {
        validation::validate_project_name(p)?;
    }

    validation::validate_due_date(due, false)?;
    validation::validate_recurrence(recur, due)?;

    let task = Task::new(text, priority, tags, project, due, recur);
    let mut tasks = storage.load()?;
    tasks.push(task);

    let id = tasks.len();

    storage.save(&tasks)?;

    let ok = "✓".green();

    if let Some(pattern) = recur {
        println!("{} Added task #{} with {} recurrence", ok, id, pattern);
    } else {
        println!("{} Added task #{}", ok, id);
    }

    Ok(())
}
