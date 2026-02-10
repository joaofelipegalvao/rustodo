use anyhow::Result;
use colored::Colorize;

use crate::storage::{load_tasks, save_tasks};
use crate::utils::confirm;
use crate::validation::validate_task_id;

pub fn execute(id: usize, yes: bool) -> Result<()> {
    let mut tasks = load_tasks()?;
    validate_task_id(id, tasks.len())?;

    let index = id - 1;
    let task_text = &tasks[index].text;

    if !yes {
        println!(
            "\n{} {}",
            "Remove task:".red().bold(),
            task_text.bright_white()
        );

        if !confirm("Are you sure? [y/N]:")? {
            println!("{}", "Removal cancelled.".yellow());
            return Ok(());
        }
    }

    let removed_task = tasks.remove(index);
    save_tasks(&tasks)?;
    println!("{} {}", "✓ Task removed:".red(), removed_task.text.dimmed());
    Ok(())
}
