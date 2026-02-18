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
    due: Option<NaiveDate>,
    recur: Option<Recurrence>,
) -> Result<()> {
    validation::validate_task_text(&text)?;
    validation::validate_tags(&tags)?;
    validation::validate_due_date(due, false)?;
    validation::validate_recurrence(recur, due)?;

    let task = Task::new(text, priority, tags, due, recur);
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
