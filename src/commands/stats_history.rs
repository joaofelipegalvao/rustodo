//! Handler for `todo stats history`.
//!
//! Shows a monthly history chart of tasks created, completed, and deleted.
//! Data is read from the `events` table, which is append-only and survives
//! `todo purge` — unlike the previous approach of inferring history from
//! `created_at` / `deleted_at` fields on live task rows.

use anyhow::Result;
use colored::Colorize;

use crate::storage::Storage;

pub fn execute(storage: &impl Storage, months: usize) -> Result<()> {
    let rows = storage.load_event_stats(months)?;

    // Drop leading months that have zero activity
    let first_non_empty = rows
        .iter()
        .position(|r| r.created > 0 || r.completed > 0 || r.deleted > 0);

    let rows = match first_non_empty {
        Some(idx) => &rows[idx..],
        None => {
            println!("{}", "\nNo activity found.\n".dimmed());
            return Ok(());
        }
    };

    let max_count = rows
        .iter()
        .map(|r| r.created.max(r.completed).max(r.deleted))
        .max()
        .unwrap_or(1)
        .max(1);

    let bar_width = 24usize;

    println!("\n{}\n", "Monthly History".bright_white().bold());
    println!(
        "  {:<10}  {:<width$}  {}",
        "Month".dimmed(),
        "Added / Completed / Deleted".dimmed(),
        "Count".dimmed(),
        width = bar_width + 2,
    );
    println!("{}", "─".repeat(bar_width + 32).dimmed());

    for row in rows {
        let filled_a = (row.created * bar_width) / max_count;
        let filled_c = (row.completed * bar_width) / max_count;
        let filled_d = (row.deleted * bar_width) / max_count;
        let total_filled = (filled_a + filled_c + filled_d).min(bar_width);
        let empty = bar_width.saturating_sub(total_filled);

        let bar = format!(
            "{}{}{}{}",
            "█".repeat(filled_a).green(),
            "█".repeat(filled_c).yellow(),
            "█".repeat(filled_d).red(),
            "░".repeat(empty).dimmed(),
        );

        let detail = format!(
            "+{}  ✓{}  -{}",
            row.created.to_string().green(),
            row.completed.to_string().yellow(),
            row.deleted.to_string().red(),
        );

        let label = format!("{} {:04}", month_abbr(row.month), row.year);
        println!("  {:<10}  {}  {}", label.dimmed(), bar, detail);
    }

    println!("{}", "─".repeat(bar_width + 32).dimmed());
    println!(
        "\n  {}  {}  {}  {}  {}  {}  {}\n",
        "█".green(),
        "Added".dimmed(),
        "█".yellow(),
        "Completed".dimmed(),
        "█".red(),
        "Deleted".dimmed(),
        format!("(last {} months)", months).dimmed(),
    );

    Ok(())
}

pub fn execute_clear(
    storage: &impl Storage,
    all: bool,
    days: Option<u32>,
    yes: bool,
) -> Result<()> {
    if !all && days.is_none() {
        anyhow::bail!(
            "Specify --all to clear everything or --days N to clear events older than N days."
        );
    }

    let description = if all {
        "All event history will be permanently deleted.".to_string()
    } else {
        format!(
            "Events older than {} day{} will be permanently deleted.",
            days.unwrap(),
            if days.unwrap() == 1 { "" } else { "s" }
        )
    };

    println!(
        "
{}  {}
",
        "!".yellow(),
        description.yellow()
    );

    if !yes {
        print!("{} Proceed? [y/N]: ", "?".yellow());
        use std::io::Write;
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", "Cancelled.".dimmed());
            return Ok(());
        }
    }

    let older_than = if all { None } else { days };
    let removed = storage.clear_events(older_than)?;

    println!(
        "{} Removed {} event{}.",
        "✓".green(),
        removed.to_string().green(),
        if removed == 1 { "" } else { "s" }
    );

    Ok(())
}

fn month_abbr(month: u32) -> &'static str {
    match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => "???",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Priority, Task};
    use crate::storage::{EntityType, EventType, InMemoryStorage};

    #[test]
    fn test_stats_history_no_activity_is_ok() {
        let storage = InMemoryStorage::default();
        assert!(execute(&storage, 6).is_ok());
    }

    #[test]
    fn test_stats_history_with_events_is_ok() {
        let storage = InMemoryStorage::default();
        let task = Task::new("T".into(), Priority::Medium, vec![], None, None, None);
        let uuid = task.uuid;
        storage.save(&[task]).unwrap();
        storage
            .record_event(EntityType::Task, uuid, EventType::Created)
            .unwrap();
        storage
            .record_event(EntityType::Task, uuid, EventType::Completed)
            .unwrap();

        assert!(execute(&storage, 6).is_ok());
    }

    #[test]
    fn test_stats_history_clear_all_is_ok() {
        let storage = InMemoryStorage::default();
        let task = Task::new("T".into(), Priority::Medium, vec![], None, None, None);
        let uuid = task.uuid;
        storage.save(&[task]).unwrap();
        storage
            .record_event(EntityType::Task, uuid, EventType::Created)
            .unwrap();

        assert!(execute_clear(&storage, true, None, true).is_ok());
        assert!(
            storage
                .load_event_stats(1)
                .unwrap()
                .iter()
                .all(|r| r.created == 0)
        );
    }

    #[test]
    fn test_stats_history_clear_requires_flag() {
        let storage = InMemoryStorage::default();
        assert!(execute_clear(&storage, false, None, true).is_err());
    }
}
