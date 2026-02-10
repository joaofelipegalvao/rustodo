use anyhow::Result;
use chrono::NaiveDate;
use colored::Colorize;

use crate::models::{Priority, Task};
use crate::storage::{load_tasks, save_tasks};

pub fn execute(
    text: String,
    priority: Priority,
    tags: Vec<String>,
    due: Option<NaiveDate>,
) -> Result<()> {
    let task = Task::new(text, priority, tags, due);
    let mut tasks = load_tasks()?;
    tasks.push(task);
    save_tasks(&tasks)?;
    println!("{}", "✓ Task added".green());
    Ok(())
}
