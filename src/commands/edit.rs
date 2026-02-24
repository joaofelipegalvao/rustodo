//! Handler for `todo edit <ID>`.
//!
//! Applies partial updates to an existing task. Only fields explicitly
//! provided by the caller are changed; everything else is preserved.
//! Validates dependency additions for self-references and cycles before
//! mutating any state.

use anyhow::Result;
use chrono::NaiveDate;
use colored::Colorize;

use crate::error::TodoError;
use crate::models::{Priority, detect_cycle};
use crate::storage::Storage;
use crate::validation::{self, validate_task_id};

#[allow(clippy::too_many_arguments)]
pub fn execute(
    storage: &impl Storage,
    id: usize,
    text: Option<String>,
    priority: Option<Priority>,
    add_tag: Vec<String>,
    remove_tag: Vec<String>,
    project: Option<String>,
    clear_project: bool,
    due: Option<NaiveDate>,
    clear_due: bool,
    clear_tags: bool,
    add_dep: Vec<usize>,
    remove_dep: Vec<usize>,
    clear_deps: bool,
) -> Result<()> {
    let mut tasks = storage.load()?;
    validate_task_id(id, tasks.len())?;

    let mut changes = Vec::new();

    // === Validate dependencies before mutating ===
    for &dep_id in &add_dep {
        if dep_id == id {
            return Err(TodoError::SelfDependency { task_id: id }.into());
        }
        validate_task_id(dep_id, tasks.len())?;
        detect_cycle(&tasks, id, dep_id).map_err(TodoError::DependencyCycle)?;
        if tasks[id - 1].depends_on.contains(&dep_id) {
            return Err(TodoError::DuplicateDependency {
                task_id: id,
                dep_id,
            }
            .into());
        }
    }
    for &dep_id in &remove_dep {
        if !tasks[id - 1].depends_on.contains(&dep_id) {
            return Err(TodoError::DependencyNotFound {
                task_id: id,
                dep_id,
            }
            .into());
        }
    }

    let task = &mut tasks[id - 1];

    // === Text ===
    if let Some(new_text) = text {
        if new_text.trim().is_empty() {
            return Err(anyhow::anyhow!("Task text cannot be empty"));
        }
        if task.text != new_text {
            task.text = new_text.clone();
            changes.push(format!("text → {}", new_text.bright_white()));
        }
    }

    // === Priority ===
    if let Some(new_priority) = priority
        && task.priority != new_priority
    {
        task.priority = new_priority;
        changes.push(format!("priority → {}", new_priority.letter()));
    }

    // === Project ===
    if clear_project {
        if task.project.is_some() {
            let old = task.project.take().unwrap();
            changes.push(format!("project cleared -> was {}", old.dimmed()));
        }
    } else if let Some(new_project) = project {
        validation::validate_project_name(&new_project)?;
        if task.project.as_deref() != Some(&new_project) {
            task.project = Some(new_project.clone());
            changes.push(format!("project -> {}", new_project.cyan()));
        }
    }

    // === Tags ===
    if clear_tags {
        if !task.tags.is_empty() {
            let old_tags = task.tags.clone();
            task.tags.clear();
            changes.push(format!(
                "tags cleared → was [{}]",
                old_tags.join(", ").dimmed()
            ));
        }
    } else {
        // Remove specific tags
        if !remove_tag.is_empty() {
            let before_len = task.tags.len();
            let mut removed = Vec::new();

            task.tags.retain(|t| {
                if remove_tag.contains(t) {
                    removed.push(t.clone());
                    false
                } else {
                    true
                }
            });

            if !removed.is_empty() {
                changes.push(format!("removed tags → [{}]", removed.join(", ").red()));
            } else if before_len > 0 {
                // User tried to remove tags that don't exist
                return Err(anyhow::anyhow!(
                    "None of the specified tags [{}] exist in task #{}",
                    remove_tag.join(", "),
                    id
                ));
            }
        }

        // Add new tags
        if !add_tag.is_empty() {
            validation::validate_tags(&add_tag)?;
            let mut added = Vec::new();

            for new_tag in &add_tag {
                if !task.tags.contains(new_tag) {
                    task.tags.push(new_tag.clone());
                    added.push(new_tag.clone());
                }
            }

            if !added.is_empty() {
                changes.push(format!("added tags → [{}]", added.join(", ").cyan()));
            }
        }
    }

    // === Due date ===
    if clear_due {
        if task.due_date.is_some() {
            task.due_date = None;
            changes.push("due date → cleared".dimmed().to_string());
        }
    } else if let Some(new_due) = due
        && task.due_date != Some(new_due)
    {
        task.due_date = Some(new_due);
        changes.push(format!("due date → {}", new_due.to_string().cyan()));
    }

    // === Dependencies ===
    if clear_deps {
        if !task.depends_on.is_empty() {
            let old = task
                .depends_on
                .drain(..)
                .map(|id| format!("#{}", id))
                .collect::<Vec<_>>()
                .join(", ");
            changes.push(format!("dependencies cleared → was [{}]", old.dimmed()));
        }
    } else {
        if !remove_dep.is_empty() {
            task.depends_on.retain(|d| !remove_dep.contains(d));
            let removed = remove_dep
                .iter()
                .map(|id| format!("#{}", id))
                .collect::<Vec<_>>()
                .join(", ");
            changes.push(format!("removed deps → [{}]", removed.red()));
        }
        if !add_dep.is_empty() {
            for dep_id in &add_dep {
                task.depends_on.push(*dep_id);
            }
            let added = add_dep
                .iter()
                .map(|id| format!("#{}", id))
                .collect::<Vec<_>>()
                .join(", ");
            changes.push(format!("added deps → [{}]", added.cyan()));
        }
    }

    // Check if anything was actually changed
    if changes.is_empty() {
        println!(
            "{} No changes made (values are already set to the specified values).",
            "".blue()
        );
        return Ok(());
    }

    storage.save(&tasks)?;

    println!("{} Task #{} updated:", "✓".green(), id);
    for change in changes {
        println!("  • {}", change);
    }

    Ok(())
}
