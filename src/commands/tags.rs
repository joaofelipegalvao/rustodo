use anyhow::Result;
use colored::Colorize;

use crate::error::TodoError;
use crate::storage::load_tasks;

pub fn execute() -> Result<()> {
    let tasks = load_tasks()?;

    // Collect all unique tags
    let mut all_tags: Vec<String> = Vec::new();
    for task in &tasks {
        for tag in &task.tags {
            if !all_tags.contains(tag) {
                all_tags.push(tag.to_owned());
            }
        }
    }

    if all_tags.is_empty() {
        return Err(TodoError::NoTagsFound.into());
    }

    all_tags.sort();

    println!("\nTags:\n");
    for tag in &all_tags {
        let count = tasks.iter().filter(|t| t.tags.contains(tag)).count();
        println!(
            "  {} ({} task{})",
            tag.cyan(),
            count,
            if count == 1 { "" } else { "s" }
        );
    }

    println!();
    Ok(())
}
