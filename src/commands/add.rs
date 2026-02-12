use anyhow::Result;
use chrono::NaiveDate;
use colored::Colorize;

use crate::models::{Priority, Recurrence, Task};
use crate::storage::{load_tasks, save_tasks};

pub fn execute(
    text: String,
    priority: Priority,
    tags: Vec<String>,
    due: Option<NaiveDate>,
    recur: Option<Recurrence>,
) -> Result<()> {
    if recur.is_some() && due.is_none() {
        return Err(anyhow::anyhow!(
            "Recurring tasks must have a due date. Use --due YYYY-MM-DD"
        ));
    }

    let task = Task::new(text, priority, tags, due, recur);
    let mut tasks = load_tasks()?;
    tasks.push(task);

    let id = tasks.len();

    save_tasks(&tasks)?;

    let ok = "✓".green();

    if let Some(pattern) = recur {
        println!(
            "{} Added task #{} with {} recurrence",
            ok,
            id,
            pattern.description()
        );
    } else {
        println!("{} Added task #{}", ok, id);
    }

    Ok(())
}
