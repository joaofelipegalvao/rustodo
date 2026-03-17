//! Handler for `todo purge`.
//!
//! Permanently removes soft-deleted tombstones older than N days across all
//! entity types: tasks, projects, notes, and resources.
//!
//! Tombstones must be kept long enough for sync to propagate deletions
//! across all devices — purging too early causes deleted tasks to reappear.

use anyhow::Result;
use chrono::{DateTime, Utc};
use colored::Colorize;
use uuid::Uuid;

use crate::storage::Storage;
use crate::utils::confirm;

// ── HasDeletedAt trait ────────────────────────────────────────────────────────

/// Implemented by any entity that carries a soft-deletion timestamp.
pub trait HasDeletedAt {
    fn deleted_at(&self) -> Option<DateTime<Utc>>;
    fn uuid(&self) -> Uuid;
    fn label(&self) -> String;
}

impl HasDeletedAt for crate::models::Task {
    fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at
    }
    fn uuid(&self) -> Uuid {
        self.uuid
    }
    fn label(&self) -> String {
        self.text.clone()
    }
}

impl HasDeletedAt for crate::models::Project {
    fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at
    }
    fn uuid(&self) -> Uuid {
        self.uuid
    }
    fn label(&self) -> String {
        self.name.clone()
    }
}

impl HasDeletedAt for crate::models::Note {
    fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at
    }
    fn uuid(&self) -> Uuid {
        self.uuid
    }
    fn label(&self) -> String {
        self.title.clone().unwrap_or_else(|| {
            self.body
                .lines()
                .find(|l| !l.trim().is_empty())
                .map(|l| l.trim_start_matches('#').trim().to_string())
                .unwrap_or_default()
        })
    }
}

impl HasDeletedAt for crate::models::Resource {
    fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at
    }
    fn uuid(&self) -> Uuid {
        self.uuid
    }
    fn label(&self) -> String {
        self.title.clone()
    }
}

// ── helper ────────────────────────────────────────────────────────────────────

/// Collects UUIDs and labels of tombstones older than `cutoff`.
fn collect_tombstones<T: HasDeletedAt>(items: &[T], cutoff: DateTime<Utc>) -> Vec<(Uuid, String)> {
    items
        .iter()
        .filter_map(|item| {
            item.deleted_at().and_then(|deleted_at| {
                if deleted_at <= cutoff {
                    Some((item.uuid(), item.label()))
                } else {
                    None
                }
            })
        })
        .collect()
}

// ── execute ───────────────────────────────────────────────────────────────────

pub fn execute(storage: &impl Storage, days: u32, dry_run: bool, yes: bool) -> Result<()> {
    let (tasks, projects, notes, resources) = storage.load_all_with_resources()?;

    let cutoff = Utc::now() - chrono::Duration::days(days as i64);

    let task_tombs = collect_tombstones(&tasks, cutoff);
    let project_tombs = collect_tombstones(&projects, cutoff);
    let note_tombs = collect_tombstones(&notes, cutoff);
    let resource_tombs = collect_tombstones(&resources, cutoff);

    let total = task_tombs.len() + project_tombs.len() + note_tombs.len() + resource_tombs.len();

    if total == 0 {
        println!(
            "{}",
            format!(
                "\nNo tombstones older than {} day{} found.\n",
                days,
                if days == 1 { "" } else { "s" }
            )
            .dimmed()
        );
        return Ok(());
    }

    // ── preview ───────────────────────────────────────────────────────────────

    println!(
        "\n{} tombstone{} older than {} day{} would be permanently removed:\n",
        total.to_string().yellow(),
        if total == 1 { "" } else { "s" },
        days,
        if days == 1 { "" } else { "s" },
    );

    let print_section = |label: &str, tombs: &[(Uuid, String)]| {
        if !tombs.is_empty() {
            println!("  {}:", label.dimmed());
            for (_, lbl) in tombs {
                println!("    {} {}", "✗".dimmed(), lbl.dimmed());
            }
        }
    };

    print_section("tasks", &task_tombs);
    print_section("projects", &project_tombs);
    print_section("notes", &note_tombs);
    print_section("resources", &resource_tombs);
    println!();

    if dry_run {
        println!("{}", "Dry run — nothing was removed.".dimmed());
        return Ok(());
    }

    if !yes && !confirm("Permanently delete these tombstones? [y/N]:")? {
        println!("{}", "Purge cancelled.".dimmed());
        return Ok(());
    }

    // ── purge — deleção física via storage ────────────────────────────────────

    let task_uuids: Vec<Uuid> = task_tombs.iter().map(|(u, _)| *u).collect();
    let project_uuids: Vec<Uuid> = project_tombs.iter().map(|(u, _)| *u).collect();
    let note_uuids: Vec<Uuid> = note_tombs.iter().map(|(u, _)| *u).collect();
    let resource_uuids: Vec<Uuid> = resource_tombs.iter().map(|(u, _)| *u).collect();

    if !task_uuids.is_empty() {
        storage.delete_tasks(&task_uuids)?;
    }
    if !project_uuids.is_empty() {
        storage.delete_projects(&project_uuids)?;
    }
    if !note_uuids.is_empty() {
        storage.delete_notes(&note_uuids)?;
    }
    if !resource_uuids.is_empty() {
        storage.delete_resources(&resource_uuids)?;
    }

    println!(
        "{} Permanently removed {} tombstone{}.",
        "✓".green(),
        total.to_string().green(),
        if total == 1 { "" } else { "s" },
    );

    Ok(())
}
