use anyhow::Result;
use colored::Colorize;

use crate::storage::Storage;
use crate::utils::confirm;
use crate::validation::validate_task_id;

pub fn execute(storage: &impl Storage, id: usize, yes: bool) -> Result<()> {
    let mut tasks = storage.load()?;
    validate_task_id(id, tasks.len())?;

    let index = id - 1;
    let task_text = &tasks[index].text;

    if !yes {
        println!("\n{} {}", "".yellow(), task_text.bright_white());

        if !confirm("Are you sure? [y/N]:")? {
            println!("{} Removal cancelled.", "".yellow());
            return Ok(());
        }
    }

    let removed_task = tasks.remove(index);
    storage.save(&tasks)?;
    println!(
        "{} {}",
        "✓".green(),
        format!("Task removed: {}", removed_task.text).dimmed()
    );
    Ok(())
}
