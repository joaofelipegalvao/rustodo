//! Handlers for `todo export` and `todo import`.
//!
//! Export serializes all data to a JSON file (same envelope format as the
//! legacy todos.json). Import reads that file and upserts everything into
//! the SQLite database.
//!
//! # Import integrity
//!
//! Before writing anything, the import validates referential integrity:
//! - Tasks whose `project_id` points to a UUID not present in the file have
//!   their `project_id` cleared (with a warning) rather than being silently
//!   stored with a dangling reference.
//! - Notes whose `task_id` or `project_id` are dangling are cleared the
//!   same way.
//! - Notes whose `resource_ids` contain unknown UUIDs have those entries
//!   removed.

use anyhow::{Context, Result, bail};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use uuid::Uuid;

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
pub fn execute_export(storage: &impl Storage, file: Option<PathBuf>) -> Result<()> {
    let (tasks, projects, notes, resources) = storage.load_all_with_resources()?;

    let envelope = Envelope {
        tasks,
        projects,
        notes,
        resources,
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
        envelope.tasks.len().to_string().dimmed(),
        envelope.projects.len().to_string().dimmed(),
        envelope.notes.len().to_string().dimmed(),
        envelope.resources.len().to_string().dimmed(),
    );

    Ok(())
}

// ── import ────────────────────────────────────────────────────────────────────

/// `todo import <FILE>` — reads a JSON export and upserts into SQLite.
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

    let mut envelope: Envelope = serde_json::from_str(&content)
        .context("Failed to parse export file — is it a valid rustodo JSON export?")?;

    let task_count = envelope.tasks.len();
    let project_count = envelope.projects.len();
    let note_count = envelope.notes.len();
    let resource_count = envelope.resources.len();

    if task_count + project_count + note_count + resource_count == 0 {
        println!("{}", "\nNothing to import — file is empty.\n".dimmed());
        return Ok(());
    }

    // ── Referential integrity check ───────────────────────────────────────────
    let warnings = validate_and_repair(&mut envelope);

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

    if !warnings.is_empty() {
        println!();
        for w in &warnings {
            println!("  {} {}", "⚠".yellow(), w.yellow());
        }
    }

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
        storage.save(&envelope.tasks)?;
        storage.save_projects(&envelope.projects)?;
        storage.save_notes(&envelope.notes)?;
        storage.save_resources(&envelope.resources)?;
    } else {
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

// ── integrity validation ──────────────────────────────────────────────────────

/// Repairs dangling foreign-key references within the envelope and returns
/// human-readable warnings for each fix applied.
///
/// This runs entirely in memory before any data is written, so a corrupt
/// export file cannot produce an inconsistent database.
fn validate_and_repair(envelope: &mut Envelope) -> Vec<String> {
    let mut warnings = Vec::new();

    let project_uuids: HashSet<Uuid> = envelope.projects.iter().map(|p| p.uuid).collect();
    let task_uuids: HashSet<Uuid> = envelope.tasks.iter().map(|t| t.uuid).collect();
    let resource_uuids: HashSet<Uuid> = envelope.resources.iter().map(|r| r.uuid).collect();

    // Tasks: clear project_id if the project is not in the envelope
    for task in &mut envelope.tasks {
        if let Some(pid) = task.project_id
            && !project_uuids.contains(&pid)
        {
            warnings.push(format!(
                "Task \"{}\": project_id {} not found — cleared.",
                task.text, pid
            ));
            task.project_id = None;
        }
    }

    // Notes: clear project_id, task_id, and unknown resource_ids
    for note in &mut envelope.notes {
        if let Some(pid) = note.project_id
            && !project_uuids.contains(&pid)
        {
            let label = note.title.as_deref().unwrap_or("<untitled>");
            warnings.push(format!(
                "Note \"{}\": project_id {} not found — cleared.",
                label, pid
            ));
            note.project_id = None;
        }

        if let Some(tid) = note.task_id
            && !task_uuids.contains(&tid)
        {
            let label = note.title.as_deref().unwrap_or("<untitled>");
            warnings.push(format!(
                "Note \"{}\": task_id {} not found — cleared.",
                label, tid
            ));
            note.task_id = None;
        }

        let before = note.resource_ids.len();
        note.resource_ids.retain(|rid| resource_uuids.contains(rid));
        let removed = before - note.resource_ids.len();
        if removed > 0 {
            let label = note.title.as_deref().unwrap_or("<untitled>");
            warnings.push(format!(
                "Note \"{}\": {} unknown resource link(s) removed.",
                label, removed
            ));
        }
    }

    warnings
}
