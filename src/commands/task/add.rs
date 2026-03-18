//! Handler for `todo add`.

use anyhow::Result;
use colored::Colorize;

use crate::cli::AddArgs;
use crate::error::TodoError;
use crate::models::{Project, Task};
use crate::services::tag_service::collect_all_tag_names;
use crate::storage::{EntityType, EventType, Storage};
use crate::utils::tag_normalizer::normalize_tags;
use crate::utils::validation::{self, resolve_uuid_visible, visible_indices};
use crate::{utils::date_parser, utils::validation::validate_task_id};

pub fn execute(storage: &impl Storage, args: AddArgs) -> Result<()> {
    execute_inner(storage, args, false)?;
    Ok(())
}

pub fn execute_silent(storage: &impl Storage, args: AddArgs) -> Result<()> {
    execute_inner(storage, args, true)?;
    Ok(())
}

fn execute_inner(storage: &impl Storage, args: AddArgs, silent: bool) -> Result<usize> {
    validation::validate_task_text(&args.text)?;
    validation::validate_tags(&args.tag)?;
    if let Some(ref p) = args.project {
        validation::validate_project_name(p)?;
    }

    let due = if let Some(ref due_str) = args.due {
        Some(date_parser::parse_date_not_in_past(due_str)?)
    } else {
        None
    };

    validation::validate_due_date(due, false)?;
    validation::validate_recurrence(args.recurrence, due)?;

    let mut tasks = storage.load()?;

    // ── Duplicate check ───────────────────────────────────────────────────────
    if !silent && args.recurrence.is_none() {
        let vis = visible_indices(&tasks, |t| t.is_deleted());
        let duplicate = vis.iter().enumerate().find(|&(_, &real_idx)| {
            tasks[real_idx].text.to_lowercase() == args.text.to_lowercase()
        });
        if let Some((vis_pos, _)) = duplicate {
            let vis_id = vis_pos + 1;
            eprintln!(
                "{} Task \"{}\" already exists (#{}). Add anyway? [y/N] ",
                "".yellow(),
                args.text,
                vis_id,
            );
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim().to_lowercase() != "y" {
                println!("{}", "Cancelled.".dimmed());
                return Ok(0);
            }
        }
    }

    // ── Dependency validation ─────────────────────────────────────────────────
    let vis = visible_indices(&tasks, |t| t.is_deleted());
    let new_vis_id = vis.len() + 1;

    for &dep_id in &args.depends_on {
        if dep_id == new_vis_id {
            return Err(TodoError::SelfDependency {
                task_id: new_vis_id,
            }
            .into());
        }
        validate_task_id(dep_id, vis.len())?;
    }

    let dep_uuids: Vec<uuid::Uuid> = args
        .depends_on
        .iter()
        .map(|&dep_id| resolve_uuid_visible(dep_id, &tasks))
        .collect::<Result<_, _>>()
        .map_err(anyhow::Error::from)?;

    // ── Tags & project ────────────────────────────────────────────────────────
    let notes = storage.load_notes()?;
    let resources = storage.load_resources()?;
    let existing_tags = collect_all_tag_names(&tasks, &notes, &resources);
    let (normalized_tags, normalization_messages) = normalize_tags(args.tag, &existing_tags);

    let project_id = if let Some(ref name) = args.project {
        let projects = storage.load_projects()?;
        Some(Project::resolve_or_create(storage, &projects, name)?)
    } else {
        None
    };

    // ── Build & persist ───────────────────────────────────────────────────────
    let mut task = Task::new(
        args.text,
        args.priority,
        normalized_tags,
        project_id,
        due,
        args.recurrence,
    );
    task.depends_on = dep_uuids;
    let task_uuid = task.uuid;
    tasks.push(task);

    let id = vis.len() + 1;
    storage.save(&tasks)?;
    storage.record_event(EntityType::Task, task_uuid, EventType::Created)?;

    if !silent {
        let ok = "✓".green();
        for msg in &normalization_messages {
            println!("  {} Tag normalized: {}", "~".yellow(), msg.yellow());
        }
        if let Some(pattern) = args.recurrence {
            println!("{} Added task #{} with {} recurrence", ok, id, pattern);
        } else {
            println!("{} Added task #{}", ok, id);
        }
    }

    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Priority, Recurrence};
    use crate::storage::InMemoryStorage;

    fn args(text: &str) -> AddArgs {
        AddArgs {
            text: text.into(),
            priority: Priority::Medium,
            tag: vec![],
            project: None,
            due: None,
            recurrence: None,
            depends_on: vec![],
        }
    }

    #[test]
    fn test_add_creates_task() {
        let storage = InMemoryStorage::default();
        execute_silent(&storage, args("Buy milk")).unwrap();

        let tasks = storage.load().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].text, "Buy milk");
    }

    #[test]
    fn test_add_with_priority() {
        let storage = InMemoryStorage::default();
        execute_silent(
            &storage,
            AddArgs {
                priority: Priority::High,
                ..args("Task")
            },
        )
        .unwrap();

        assert_eq!(storage.load().unwrap()[0].priority, Priority::High);
    }

    #[test]
    fn test_add_with_tags() {
        let storage = InMemoryStorage::default();
        execute_silent(
            &storage,
            AddArgs {
                tag: vec!["rust".into(), "backend".into()],
                ..args("Task")
            },
        )
        .unwrap();

        let tags = &storage.load().unwrap()[0].tags;
        assert!(tags.contains(&"rust".to_string()));
        assert!(tags.contains(&"backend".to_string()));
    }

    #[test]
    fn test_add_with_project_creates_project() {
        let storage = InMemoryStorage::default();
        execute_silent(
            &storage,
            AddArgs {
                project: Some("Rustodo".into()),
                ..args("Task")
            },
        )
        .unwrap();

        let projects = storage.load_projects().unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Rustodo");

        let task = &storage.load().unwrap()[0];
        assert_eq!(task.project_id, Some(projects[0].uuid));
    }

    #[test]
    fn test_add_with_existing_project_reuses_it() {
        let storage = InMemoryStorage::default();
        let project = crate::models::Project::new("Rustodo".into());
        let proj_uuid = project.uuid;
        storage.save_projects(&[project]).unwrap();

        execute_silent(
            &storage,
            AddArgs {
                project: Some("Rustodo".into()),
                ..args("Task")
            },
        )
        .unwrap();

        assert_eq!(storage.load_projects().unwrap().len(), 1);
        assert_eq!(storage.load().unwrap()[0].project_id, Some(proj_uuid));
    }

    #[test]
    fn test_add_empty_text_fails() {
        let storage = InMemoryStorage::default();
        assert!(execute_silent(&storage, args("")).is_err());
    }

    #[test]
    fn test_add_whitespace_only_text_fails() {
        let storage = InMemoryStorage::default();
        assert!(execute_silent(&storage, args("   ")).is_err());
    }

    #[test]
    fn test_add_multiple_increments_count() {
        let storage = InMemoryStorage::default();
        execute_silent(&storage, args("Task A")).unwrap();
        execute_silent(&storage, args("Task B")).unwrap();
        execute_silent(&storage, args("Task C")).unwrap();

        assert_eq!(storage.load().unwrap().len(), 3);
    }

    #[test]
    fn test_add_with_recurrence() {
        let storage = InMemoryStorage::default();
        execute_silent(
            &storage,
            AddArgs {
                recurrence: Some(Recurrence::Weekly),
                due: Some("2099-12-31".into()),
                ..args("Weekly task")
            },
        )
        .unwrap();

        assert_eq!(
            storage.load().unwrap()[0].recurrence,
            Some(Recurrence::Weekly)
        );
    }

    #[test]
    fn test_add_normalizes_tags_to_existing() {
        let storage = InMemoryStorage::default();
        execute_silent(
            &storage,
            AddArgs {
                tag: vec!["rust".into()],
                ..args("Task A")
            },
        )
        .unwrap();

        execute_silent(
            &storage,
            AddArgs {
                tag: vec!["Rust".into()],
                ..args("Task B")
            },
        )
        .unwrap();

        let tasks = storage.load().unwrap();
        assert_eq!(tasks[1].tags[0], "rust");
    }
}
