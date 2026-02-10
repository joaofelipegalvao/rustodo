use std::fs;

use anyhow::{Context, Result};
use colored::Colorize;

use crate::storage::{get_data_file_path, load_tasks};
use crate::utils::confirm;

pub fn execute(yes: bool) -> Result<()> {
    let path = get_data_file_path()?;

    if !path.exists() {
        println!("No tasks to remove");
        return Ok(());
    }

    let tasks = load_tasks()?;
    let count = tasks.len();

    if !yes {
        println!(
            "\n{} {} tasks will be permanently deleted!",
            "WARNING:".red().bold(),
            count
        );

        if !confirm("Type 'yes' to confirm:")? {
            println!("{}", "Clear cancelled.".yellow());
            return Ok(());
        }
    }

    fs::remove_file(&path).context(format!("Failed to remove {}", path.display()))?;
    println!("{}", "✓ All tasks have been removed".red().bold());
    Ok(())
}
