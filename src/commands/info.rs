use std::fs;

use anyhow::Result;
use colored::Colorize;

use crate::storage::get_data_file_path;

pub fn execute() -> Result<()> {
    let path = get_data_file_path()?;
    let exists = path.exists();

    println!("\n{}\n", "Todo-List Information".bold());
    println!("{} {}", "Data file:".dimmed(), path.display());
    println!(
        "{} {}",
        "Status:".dimmed(),
        if exists {
            "exists ✓".green()
        } else {
            "not created yet".yellow()
        }
    );

    if exists {
        let metadata = fs::metadata(&path)?;
        let size = metadata.len();
        println!("{} {} bytes", "Size:".dimmed(), size);
    }

    println!();
    Ok(())
}
