//! Handlers for `todo backup` and `todo restore`.

use anyhow::{Context, Result, bail};
use colored::Colorize;
use std::path::PathBuf;

use crate::storage::{backup, get_db_path};

// ── backup ────────────────────────────────────────────────────────────────────

/// `todo backup` — creates a manual snapshot immediately.
pub fn execute_backup() -> Result<()> {
    let db_path = get_db_path()?;
    let backup_dir = db_path
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .join("backups");

    let backup_path =
        backup::create_backup(&db_path, &backup_dir).context("Failed to create backup")?;

    println!(
        "{} Backup created: {}",
        "✓".green(),
        backup_path.display().to_string().cyan()
    );

    Ok(())
}

// ── restore ───────────────────────────────────────────────────────────────────

/// `todo restore [FILE]` — restores from a backup file.
///
/// If no file is given, lists available backups and lets the user pick one.
pub fn execute_restore(file: Option<PathBuf>, yes: bool) -> Result<()> {
    let db_path = get_db_path()?;
    let backup_dir = db_path
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .join("backups");

    let backup_path = match file {
        Some(p) => p,
        None => pick_backup(&backup_dir)?,
    };

    if !backup_path.exists() {
        bail!("Backup file not found: {}", backup_path.display());
    }

    println!(
        "\n{} This will replace your current database with:\n  {}\n",
        "!".yellow(),
        backup_path.display().to_string().cyan()
    );

    if !yes && !crate::utils::confirm("Restore from this backup? [y/N]:")? {
        println!("{}", "Restore cancelled.".dimmed());
        return Ok(());
    }

    // Create a safety backup of current state before overwriting
    if db_path.exists() {
        let safety_dir = backup_dir.clone();
        if let Ok(safety_path) = backup::create_backup(&db_path, &safety_dir) {
            println!(
                "{} Safety backup of current state saved to: {}",
                "".blue(),
                safety_path.display().to_string().dimmed()
            );
        }
    }

    std::fs::copy(&backup_path, &db_path).context("Failed to restore backup")?;

    println!(
        "{} Restored from: {}",
        "✓".green(),
        backup_path.display().to_string().cyan()
    );

    Ok(())
}

// ── list backups ──────────────────────────────────────────────────────────────

/// `todo backup list` — lists all available backups.
pub fn execute_list() -> Result<()> {
    let db_path = get_db_path()?;
    let backup_dir = db_path
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .join("backups");

    let mut backups = list_backups(&backup_dir)?;

    if backups.is_empty() {
        println!("{}", "\nNo backups found.\n".dimmed());
        return Ok(());
    }

    backups.sort();

    println!("\n{}\n", "Available backups:".bright_white().bold());
    for (i, path) in backups.iter().enumerate() {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let size = std::fs::metadata(path)
            .map(|m| format_size(m.len()))
            .unwrap_or_else(|_| "?".to_string());
        println!(
            "  {}  {}  {}",
            format!("{:>2}.", i + 1).dimmed(),
            name.bright_white(),
            size.dimmed()
        );
    }
    println!(
        "\n  {}\n",
        format!("Directory: {}", backup_dir.display()).dimmed()
    );

    Ok(())
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn list_backups(backup_dir: &std::path::Path) -> Result<Vec<PathBuf>> {
    if !backup_dir.exists() {
        return Ok(vec![]);
    }
    let backups = std::fs::read_dir(backup_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "db"))
        .collect();
    Ok(backups)
}

/// Picks the most recent backup automatically if only one exists,
/// otherwise shows a numbered list and asks the user to choose.
fn pick_backup(backup_dir: &std::path::Path) -> Result<PathBuf> {
    let mut backups = list_backups(backup_dir)?;

    if backups.is_empty() {
        bail!(
            "No backups found in {}. Run 'todo backup' first.",
            backup_dir.display()
        );
    }

    backups.sort();

    if backups.len() == 1 {
        return Ok(backups.remove(0));
    }

    println!("\n{}\n", "Available backups:".bright_white().bold());
    for (i, path) in backups.iter().enumerate() {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let size = std::fs::metadata(path)
            .map(|m| format_size(m.len()))
            .unwrap_or_else(|_| "?".to_string());
        println!(
            "  {}  {}  {}",
            format!("{:>2}.", i + 1).dimmed(),
            name.bright_white(),
            size.dimmed()
        );
    }
    println!();

    let choice = prompt_number(backups.len())?;
    Ok(backups.remove(choice - 1))
}

fn prompt_number(max: usize) -> Result<usize> {
    use std::io::{self, Write};
    loop {
        print!("Select backup [1-{}]: ", max);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().parse::<usize>() {
            Ok(n) if n >= 1 && n <= max => return Ok(n),
            _ => println!("Please enter a number between 1 and {}.", max),
        }
    }
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
