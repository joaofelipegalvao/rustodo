use std::fs;

use anyhow::{Context, Result};
use colored::Colorize;

use crate::storage::{Storage, get_data_file_path};
use crate::utils::confirm;

pub fn execute(storage: &impl Storage, yes: bool) -> Result<()> {
    let path = get_data_file_path()?;

    if !path.exists() {
        println!("{} No tasks to remove", "".blue());
        return Ok(());
    }

    let tasks = storage.load()?;
    let count = tasks.len();

    if !yes {
        println!(
            "\n{} {} tasks will be permanently deleted!",
            "".yellow().bold(),
            count
        );

        if !confirm("Type 'yes' to confirm:")? {
            println!("{} Clear cancelled.", "".yellow());
            return Ok(());
        }
    }

    fs::remove_file(&path).context(format!("Failed to remove {}", path.display()))?;
    println!("{} All tasks have been removed", "✓".green().bold());
    Ok(())
}
