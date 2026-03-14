//! Handlers for `todo export` and `todo import`.
//!
//! Export serializes all data to a JSON file (same envelope format as the
//! legacy todos.json). Import reads that file and upserts everything into
//! the SQLite database.
use anyhow::{Context, Result, bail};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::models::{Note, Project, Resource, Task};
use crate::storage::Storage;

// ── envelope ──────────────────────────────────────────────────────────────────

/// The JSON envelope used for export/import.
#[derive(Serialize, Deserialize, Default)]
struct Envelope {
    #[serde(default)]
    tasks: Vec<Task>,
    #[serde(default)]
    projects: Vec<Project>,
    #[serde(default)]
    notes: Vec<Note>,
    #[serde(default)]
    resources: Vec<Resource>,
}

// ── export ────────────────────────────────────────────────────────────────────

/// `todo export [FILE]` — serializes all data to a JSON file.
///
/// Defaults to `rustodo-export-YYYY-MM-DD.json` in the current directory.
pub fn execute_export(storage: &impl Storage, file: Option<PathBuf>) -> Result<()> {
    let (tasks, projects, notes, resources) = storage.load_all_with_resources()?;

    let envelope = Envelope {
        tasks: tasks.clone(),
        projects: projects.clone(),
        notes: notes.clone(),
        resources: resources.clone(),
    };

    let json = serde_json::to_string_pretty(&envelope).context("Failed to serialize data")?;

    let path = file.unwrap_or_else(|| {
        let date = chrono::Local::now().format("%Y-%m-%d");
        PathBuf::from(format!("rustodo-export-{}.json", date))
    });

    std::fs::write(&path, &json)
        .context(format!("Failed to write export file: {}", path.display()))?;

    println!(
        "{} Exported to: {}",
        "✓".green(),
        path.display().to_string().cyan()
    );
    println!(
        "  {} tasks, {} projects, {} notes, {} resources",
        tasks.len().to_string().dimmed(),
        projects.len().to_string().dimmed(),
        notes.len().to_string().dimmed(),
        resources.len().to_string().dimmed(),
    );

    Ok(())
}

// ── import ────────────────────────────────────────────────────────────────────

/// `todo import <FILE>` — reads a JSON export and upserts into SQLite.
///
/// Existing data is preserved — import only adds or updates by UUID.
/// Use `--replace` to clear all data before importing.
pub fn execute_import(
    storage: &impl Storage,
    file: PathBuf,
    replace: bool,
    yes: bool,
) -> Result<()> {
    if !file.exists() {
        bail!("File not found: {}", file.display());
    }

    let content =
        std::fs::read_to_string(&file).context(format!("Failed to read {}", file.display()))?;

    let envelope: Envelope = serde_json::from_str(&content)
        .context("Failed to parse export file — is it a valid rustodo JSON export?")?;

    let task_count = envelope.tasks.len();
    let project_count = envelope.projects.len();
    let note_count = envelope.notes.len();
    let resource_count = envelope.resources.len();

    if task_count + project_count + note_count + resource_count == 0 {
        println!("{}", "\nNothing to import — file is empty.\n".dimmed());
        return Ok(());
    }

    println!(
        "\n{} Importing from: {}\n",
        "".blue(),
        file.display().to_string().cyan()
    );
    println!(
        "  {} tasks, {} projects, {} notes, {} resources",
        task_count.to_string().bright_white(),
        project_count.to_string().bright_white(),
        note_count.to_string().bright_white(),
        resource_count.to_string().bright_white(),
    );

    if replace {
        println!(
            "\n  {} {} All existing data will be replaced.\n",
            "!".yellow(),
            "Warning:".yellow()
        );
    } else {
        println!(
            "\n  {} Existing records with matching UUIDs will be updated.\n",
            "".dimmed()
        );
    }

    if !yes && !crate::utils::confirm("Proceed with import? [y/N]:")? {
        println!("{}", "Import cancelled.".dimmed());
        return Ok(());
    }

    if replace {
        // Load current data and soft-delete everything before importing
        // (we use save with empty slices to replace)
        storage.save(&envelope.tasks)?;
        storage.save_projects(&envelope.projects)?;
        storage.save_notes(&envelope.notes)?;
        storage.save_resources(&envelope.resources)?;
    } else {
        // Upsert — SqliteStorage.save() uses INSERT OR REPLACE by UUID
        if !envelope.tasks.is_empty() {
            storage.save(&envelope.tasks)?;
        }
        if !envelope.projects.is_empty() {
            storage.save_projects(&envelope.projects)?;
        }
        if !envelope.notes.is_empty() {
            storage.save_notes(&envelope.notes)?;
        }
        if !envelope.resources.is_empty() {
            storage.save_resources(&envelope.resources)?;
        }
    }

    println!(
        "{} Import complete: {} tasks, {} projects, {} notes, {} resources",
        "✓".green(),
        task_count.to_string().green(),
        project_count.to_string().green(),
        note_count.to_string().green(),
        resource_count.to_string().green(),
    );

    Ok(())
}
