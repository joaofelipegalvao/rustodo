//! Handler for `todo edit <ID>`.

use anyhow::Result;
use colored::Colorize;
use uuid::Uuid;

use crate::cli::EditArgs;
use crate::error::TodoError;
use crate::models::{Project, detect_cycle};
use crate::storage::{EntityType, EventType, Storage};
use crate::utils::date_parser;
use crate::utils::validation::{self, validate_task_id, visible_indices};

pub fn execute(storage: &impl Storage, args: EditArgs) -> Result<()> {
    execute_inner(storage, args, false)?;
    Ok(())
}

pub fn execute_silent(storage: &impl Storage, args: EditArgs) -> Result<String> {
    execute_inner(storage, args, true)
}

fn execute_inner(storage: &impl Storage, args: EditArgs, silent: bool) -> Result<String> {
    let due = if let Some(ref due_str) = args.due {
        Some(date_parser::parse_date(due_str)?)
    } else {
        None
    };

    let mut tasks = storage.load()?;
    let vis = visible_indices(&tasks, |t| t.is_deleted());
    validate_task_id(args.id, vis.len())?;
    let real_index = vis[args.id - 1];

    let add_dep_uuids: Vec<Uuid> = args
        .add_dep
        .iter()
        .map(|&id| validation::resolve_uuid_visible(id, &tasks))
        .collect::<Result<_, _>>()
        .map_err(anyhow::Error::from)?;

    let remove_dep_uuids: Vec<Uuid> = args
        .remove_dep
        .iter()
        .map(|&id| validation::resolve_uuid_visible(id, &tasks))
        .collect::<Result<_, _>>()
        .map_err(anyhow::Error::from)?;

    let current_deps_display = tasks[real_index]
        .depends_on
        .iter()
        .filter_map(|uuid| {
            let real_pos = tasks.iter().position(|t| t.uuid == *uuid)?;
            let vis_id = vis.iter().position(|&i| i == real_pos).map(|p| p + 1)?;
            Some(format!("#{}", vis_id))
        })
        .collect::<Vec<_>>()
        .join(", ");

    let mut changes = Vec::new();

    for (dep_id, &dep_uuid) in args.add_dep.iter().zip(add_dep_uuids.iter()) {
        if *dep_id == args.id {
            return Err(TodoError::SelfDependency { task_id: args.id }.into());
        }
        validate_task_id(*dep_id, vis.len())?;
        detect_cycle(&tasks, tasks[real_index].uuid, dep_uuid)
            .map_err(TodoError::DependencyCycle)?;
        if tasks[real_index].depends_on.contains(&dep_uuid) {
            return Err(TodoError::DuplicateDependency {
                task_id: args.id,
                dep_id: *dep_id,
            }
            .into());
        }
    }
    for (dep_id, dep_uuid) in args.remove_dep.iter().zip(remove_dep_uuids.iter()) {
        if !tasks[real_index].depends_on.contains(dep_uuid) {
            return Err(TodoError::DependencyNotFound {
                task_id: args.id,
                dep_id: *dep_id,
            }
            .into());
        }
    }

    let task = &mut tasks[real_index];

    if let Some(new_text) = args.text {
        if new_text.trim().is_empty() {
            return Err(anyhow::anyhow!("Task text cannot be empty"));
        }
        if task.text != new_text {
            task.text = new_text.clone();
            changes.push(format!("text → {}", new_text.bright_white()));
        }
    }

    if let Some(new_priority) = args.priority
        && task.priority != new_priority
    {
        task.priority = new_priority;
        changes.push(format!("priority → {}", new_priority.letter()));
    }

    if args.clear_project {
        if task.project_id.is_some() {
            task.project_id = None;
            changes.push("project → cleared".dimmed().to_string());
        }
    } else if let Some(ref new_project_name) = args.project {
        validation::validate_project_name(new_project_name)?;
        let projects = storage.load_projects()?;
        let new_uuid = Project::resolve_or_create(storage, &projects, new_project_name)?;
        if task.project_id != Some(new_uuid) {
            task.project_id = Some(new_uuid);
            changes.push(format!("project → {}", new_project_name.cyan()));
        }
    }

    if args.clear_tags {
        if !task.tags.is_empty() {
            let old_tags = task.tags.clone();
            task.tags.clear();
            changes.push(format!(
                "tags cleared → was [{}]",
                old_tags.join(", ").dimmed()
            ));
        }
    } else {
        if !args.remove_tag.is_empty() {
            let before_len = task.tags.len();
            let mut removed = Vec::new();
            task.tags.retain(|t| {
                if args.remove_tag.contains(t) {
                    removed.push(t.clone());
                    false
                } else {
                    true
                }
            });
            if !removed.is_empty() {
                changes.push(format!("removed tags → [{}]", removed.join(", ").red()));
            } else if before_len > 0 {
                return Err(anyhow::anyhow!(
                    "None of the specified tags [{}] exist in task #{}",
                    args.remove_tag.join(", "),
                    args.id
                ));
            }
        }
        if !args.add_tag.is_empty() {
            validation::validate_tags(&args.add_tag)?;
            let mut added = Vec::new();
            for new_tag in &args.add_tag {
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

    if args.clear_due {
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

    if args.clear_deps {
        if !task.depends_on.is_empty() {
            task.depends_on.clear();
            changes.push(format!(
                "dependencies cleared → was [{}]",
                current_deps_display.dimmed()
            ));
        }
    } else {
        if !args.remove_dep.is_empty() {
            task.depends_on.retain(|d| !remove_dep_uuids.contains(d));
            let removed = args
                .remove_dep
                .iter()
                .map(|id| format!("#{}", id))
                .collect::<Vec<_>>()
                .join(", ");
            changes.push(format!("removed deps → [{}]", removed.red()));
        }
        if !args.add_dep.is_empty() {
            for dep_uuid in &add_dep_uuids {
                task.depends_on.push(*dep_uuid);
            }
            let added = args
                .add_dep
                .iter()
                .map(|id| format!("#{}", id))
                .collect::<Vec<_>>()
                .join(", ");
            changes.push(format!("added deps → [{}]", added.cyan()));
        }
    }

    if changes.is_empty() {
        if !silent {
            println!(
                "{} No changes made (values are already set to the specified values).",
                "".blue()
            );
        }
        return Ok("No changes made.".to_string());
    }

    let task_uuid = tasks[real_index].uuid;
    tasks[real_index].touch();
    storage.upsert_task(&tasks[real_index])?;
    storage.record_event(EntityType::Task, task_uuid, EventType::Edited)?;

    if !silent {
        println!("{} Task #{} updated:", "✓".green(), args.id);
        for change in &changes {
            println!("  • {}", change);
        }
    }

    Ok(format!("Task #{} updated.", args.id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Priority, Task};
    use crate::storage::InMemoryStorage;

    fn args(id: usize) -> EditArgs {
        EditArgs {
            id,
            text: None,
            priority: None,
            add_tag: vec![],
            remove_tag: vec![],
            project: None,
            clear_project: false,
            due: None,
            clear_due: false,
            clear_tags: false,
            add_dep: vec![],
            remove_dep: vec![],
            clear_deps: false,
        }
    }

    fn make_task(text: &str) -> Task {
        Task::new(text.into(), Priority::Medium, vec![], None, None, None)
    }

    // ── text ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_edit_text() {
        let storage = InMemoryStorage::default();
        storage.save(&[make_task("Old text")]).unwrap();

        execute_silent(
            &storage,
            EditArgs {
                text: Some("New text".into()),
                ..args(1)
            },
        )
        .unwrap();

        assert_eq!(storage.load().unwrap()[0].text, "New text");
    }

    #[test]
    fn test_edit_empty_text_fails() {
        let storage = InMemoryStorage::default();
        storage.save(&[make_task("Task")]).unwrap();

        let err = execute_silent(
            &storage,
            EditArgs {
                text: Some("  ".into()),
                ..args(1)
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn test_edit_same_text_no_changes() {
        let storage = InMemoryStorage::default();
        storage.save(&[make_task("Same text")]).unwrap();

        let result = execute_silent(
            &storage,
            EditArgs {
                text: Some("Same text".into()),
                ..args(1)
            },
        )
        .unwrap();
        assert!(result.contains("No changes"));
    }

    // ── priority ──────────────────────────────────────────────────────────────

    #[test]
    fn test_edit_priority() {
        let storage = InMemoryStorage::default();
        storage.save(&[make_task("Task")]).unwrap();

        execute_silent(
            &storage,
            EditArgs {
                priority: Some(Priority::High),
                ..args(1)
            },
        )
        .unwrap();

        assert_eq!(storage.load().unwrap()[0].priority, Priority::High);
    }

    #[test]
    fn test_edit_same_priority_no_changes() {
        let storage = InMemoryStorage::default();
        storage.save(&[make_task("Task")]).unwrap();

        let result = execute_silent(
            &storage,
            EditArgs {
                priority: Some(Priority::Medium),
                ..args(1)
            },
        )
        .unwrap();
        assert!(result.contains("No changes"));
    }

    // ── tags ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_edit_add_tag() {
        let storage = InMemoryStorage::default();
        storage.save(&[make_task("Task")]).unwrap();

        execute_silent(
            &storage,
            EditArgs {
                add_tag: vec!["rust".into()],
                ..args(1)
            },
        )
        .unwrap();

        assert!(
            storage.load().unwrap()[0]
                .tags
                .contains(&"rust".to_string())
        );
    }

    #[test]
    fn test_edit_add_duplicate_tag_no_changes() {
        let storage = InMemoryStorage::default();
        let mut task = make_task("Task");
        task.tags = vec!["rust".into()];
        storage.save(&[task]).unwrap();

        let result = execute_silent(
            &storage,
            EditArgs {
                add_tag: vec!["rust".into()],
                ..args(1)
            },
        )
        .unwrap();
        assert!(result.contains("No changes"));
        assert_eq!(storage.load().unwrap()[0].tags.len(), 1);
    }

    #[test]
    fn test_edit_remove_tag() {
        let storage = InMemoryStorage::default();
        let mut task = make_task("Task");
        task.tags = vec!["rust".into(), "backend".into()];
        storage.save(&[task]).unwrap();

        execute_silent(
            &storage,
            EditArgs {
                remove_tag: vec!["rust".into()],
                ..args(1)
            },
        )
        .unwrap();

        let tags = &storage.load().unwrap()[0].tags;
        assert!(!tags.contains(&"rust".to_string()));
        assert!(tags.contains(&"backend".to_string()));
    }

    #[test]
    fn test_edit_remove_nonexistent_tag_fails() {
        let storage = InMemoryStorage::default();
        let mut task = make_task("Task");
        task.tags = vec!["rust".into()];
        storage.save(&[task]).unwrap();

        let err = execute_silent(
            &storage,
            EditArgs {
                remove_tag: vec!["python".into()],
                ..args(1)
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("python"));
    }

    #[test]
    fn test_edit_clear_tags() {
        let storage = InMemoryStorage::default();
        let mut task = make_task("Task");
        task.tags = vec!["rust".into(), "backend".into()];
        storage.save(&[task]).unwrap();

        execute_silent(
            &storage,
            EditArgs {
                clear_tags: true,
                ..args(1)
            },
        )
        .unwrap();

        assert!(storage.load().unwrap()[0].tags.is_empty());
    }

    #[test]
    fn test_edit_clear_tags_already_empty_no_changes() {
        let storage = InMemoryStorage::default();
        storage.save(&[make_task("Task")]).unwrap();

        let result = execute_silent(
            &storage,
            EditArgs {
                clear_tags: true,
                ..args(1)
            },
        )
        .unwrap();
        assert!(result.contains("No changes"));
    }

    // ── due date ──────────────────────────────────────────────────────────────

    #[test]
    fn test_edit_clear_due() {
        let storage = InMemoryStorage::default();
        let mut task = make_task("Task");
        task.due_date = Some(chrono::NaiveDate::from_ymd_opt(2099, 12, 31).unwrap());
        storage.save(&[task]).unwrap();

        execute_silent(
            &storage,
            EditArgs {
                clear_due: true,
                ..args(1)
            },
        )
        .unwrap();

        assert!(storage.load().unwrap()[0].due_date.is_none());
    }

    #[test]
    fn test_edit_clear_due_already_none_no_changes() {
        let storage = InMemoryStorage::default();
        storage.save(&[make_task("Task")]).unwrap();

        let result = execute_silent(
            &storage,
            EditArgs {
                clear_due: true,
                ..args(1)
            },
        )
        .unwrap();
        assert!(result.contains("No changes"));
    }

    // ── project ───────────────────────────────────────────────────────────────

    #[test]
    fn test_edit_clear_project() {
        let storage = InMemoryStorage::default();
        let project = crate::models::Project::new("MyProject".into());
        let proj_uuid = project.uuid;
        storage.save_projects(&[project]).unwrap();
        let mut task = make_task("Task");
        task.project_id = Some(proj_uuid);
        storage.save(&[task]).unwrap();

        execute_silent(
            &storage,
            EditArgs {
                clear_project: true,
                ..args(1)
            },
        )
        .unwrap();

        assert!(storage.load().unwrap()[0].project_id.is_none());
    }

    #[test]
    fn test_edit_clear_project_already_none_no_changes() {
        let storage = InMemoryStorage::default();
        storage.save(&[make_task("Task")]).unwrap();

        let result = execute_silent(
            &storage,
            EditArgs {
                clear_project: true,
                ..args(1)
            },
        )
        .unwrap();
        assert!(result.contains("No changes"));
    }

    // ── invalid id ────────────────────────────────────────────────────────────

    #[test]
    fn test_edit_invalid_id_returns_error() {
        let storage = InMemoryStorage::default();
        storage.save(&[make_task("Task")]).unwrap();

        assert!(execute_silent(&storage, args(99)).is_err());
    }

    #[test]
    fn test_edit_skips_deleted_in_id_resolution() {
        let storage = InMemoryStorage::default();
        let mut deleted = make_task("Deleted");
        deleted.soft_delete();
        let active = make_task("Active");
        storage.save(&[deleted, active]).unwrap();

        execute_silent(
            &storage,
            EditArgs {
                text: Some("Edited".into()),
                ..args(1)
            },
        )
        .unwrap();

        let tasks = storage.load().unwrap();
        assert_eq!(tasks[0].text, "Deleted"); // untouched
        assert_eq!(tasks[1].text, "Edited"); // #1 resolved to active
    }

    // ── no changes ────────────────────────────────────────────────────────────

    #[test]
    fn test_edit_no_args_no_changes() {
        let storage = InMemoryStorage::default();
        storage.save(&[make_task("Task")]).unwrap();

        let result = execute_silent(&storage, args(1)).unwrap();
        assert!(result.contains("No changes"));
    }

    // ── touch on save ─────────────────────────────────────────────────────────

    #[test]
    fn test_edit_updates_updated_at() {
        let storage = InMemoryStorage::default();
        let task = make_task("Task");
        let original_updated_at = task.updated_at;
        storage.save(&[task]).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10));
        execute_silent(
            &storage,
            EditArgs {
                text: Some("New text".into()),
                ..args(1)
            },
        )
        .unwrap();

        let updated = storage.load().unwrap();
        assert!(updated[0].updated_at > original_updated_at);
    }
}
