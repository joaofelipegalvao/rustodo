//! Handler for `todo note add`.

use anyhow::{Result, anyhow};
use colored::Colorize;

use crate::cli::NoteAddArgs;
use crate::models::{Note, Project};
use crate::services::tag_service::collect_all_tag_names;
use crate::storage::{EntityType, EventType, Storage};
use crate::utils::tag_normalizer::normalize_tags;
use crate::utils::validation::resolve_visible;

pub fn execute(storage: &impl Storage, args: NoteAddArgs) -> Result<()> {
    let (tasks, projects, notes) = storage.load_all()?;
    let resources = storage.load_resources()?;

    let (body, is_markdown) = match (args.body, args.editor, args.file) {
        (Some(text), false, None) => (text, false),
        (None, true, None) => {
            let content = edit::edit_with_builder("", edit::Builder::new().suffix(".md"))?;
            let trimmed = content.trim().to_string();
            if trimmed.is_empty() {
                return Err(anyhow!("Aborted: note body is empty."));
            }
            (trimmed, true)
        }
        (None, false, Some(path)) => {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| anyhow!("Failed to read file {}: {}", path.display(), e))?;
            (content, true)
        }
        (None, false, None) => {
            return Err(anyhow!(
                "Provide a note body: <BODY>, --editor, or --file <PATH>"
            ));
        }
        _ => {
            return Err(anyhow!(
                "Only one input source allowed: <BODY>, --editor, or --file <PATH>"
            ));
        }
    };

    let project_id = if let Some(ref name) = args.project {
        Some(Project::resolve_or_create(storage, &projects, name)?)
    } else {
        None
    };

    let task_id = if let Some(task_num) = args.task {
        let task = resolve_visible(&tasks, task_num, |t| t.is_deleted())
            .map_err(|_| anyhow!("Task #{} not found", task_num))?;
        Some(task.uuid)
    } else {
        None
    };

    let existing_tags = collect_all_tag_names(&tasks, &notes, &resources);
    let (normalized_tags, normalization_messages) = normalize_tags(args.tag, &existing_tags);

    let mut note = if is_markdown {
        Note::new_markdown(body)
    } else {
        Note::new(body)
    };
    note.title = args.title;
    note.tags = normalized_tags;
    note.language = args.language;
    note.project_id = project_id;
    note.task_id = task_id;

    let note_uuid = note.uuid;
    let id = notes.iter().filter(|n| !n.is_deleted()).count() + 1;
    storage.upsert_note(&note)?;
    storage.record_event(EntityType::Note, note_uuid, EventType::Created)?;

    for msg in &normalization_messages {
        println!("  {} Tag normalized: {}", "~".yellow(), msg.yellow());
    }
    println!("{} Added note #{}", "✓".green(), id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Priority, Task};
    use crate::storage::InMemoryStorage;

    fn args(body: &str) -> NoteAddArgs {
        NoteAddArgs {
            body: Some(body.into()),
            editor: false,
            file: None,
            title: None,
            tag: vec![],
            language: None,
            project: None,
            task: None,
        }
    }

    // ── basic add ─────────────────────────────────────────────────────────────

    #[test]
    fn test_note_add_creates_note() {
        let storage = InMemoryStorage::default();
        execute(&storage, args("My note body")).unwrap();

        let notes = storage.load_notes().unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].body, "My note body");
    }

    #[test]
    fn test_note_add_no_body_fails() {
        let storage = InMemoryStorage::default();
        let err = execute(
            &storage,
            NoteAddArgs {
                body: None,
                editor: false,
                file: None,
                title: None,
                tag: vec![],
                language: None,
                project: None,
                task: None,
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("body"));
    }

    #[test]
    fn test_note_add_with_title() {
        let storage = InMemoryStorage::default();
        execute(
            &storage,
            NoteAddArgs {
                title: Some("My title".into()),
                ..args("Body")
            },
        )
        .unwrap();

        assert_eq!(
            storage.load_notes().unwrap()[0].title.as_deref(),
            Some("My title")
        );
    }

    #[test]
    fn test_note_add_with_language() {
        let storage = InMemoryStorage::default();
        execute(
            &storage,
            NoteAddArgs {
                language: Some("rust".into()),
                ..args("fn main() {}")
            },
        )
        .unwrap();

        assert_eq!(
            storage.load_notes().unwrap()[0].language.as_deref(),
            Some("rust")
        );
    }

    #[test]
    fn test_note_add_with_tags() {
        let storage = InMemoryStorage::default();
        execute(
            &storage,
            NoteAddArgs {
                tag: vec!["rust".into(), "backend".into()],
                ..args("Body")
            },
        )
        .unwrap();

        let tags = &storage.load_notes().unwrap()[0].tags;
        assert!(tags.contains(&"rust".to_string()));
        assert!(tags.contains(&"backend".to_string()));
    }

    #[test]
    fn test_note_add_multiple_increments_count() {
        let storage = InMemoryStorage::default();
        execute(&storage, args("Note A")).unwrap();
        execute(&storage, args("Note B")).unwrap();
        execute(&storage, args("Note C")).unwrap();

        assert_eq!(storage.load_notes().unwrap().len(), 3);
    }

    // ── project ───────────────────────────────────────────────────────────────

    #[test]
    fn test_note_add_with_existing_project() {
        let storage = InMemoryStorage::default();
        let project = crate::models::Project::new("Rustodo".into());
        let proj_uuid = project.uuid;
        storage.save_projects(&[project]).unwrap();

        execute(
            &storage,
            NoteAddArgs {
                project: Some("Rustodo".into()),
                ..args("Body")
            },
        )
        .unwrap();

        assert_eq!(storage.load_notes().unwrap()[0].project_id, Some(proj_uuid));
    }

    #[test]
    fn test_note_add_with_new_project_creates_it() {
        let storage = InMemoryStorage::default();

        execute(
            &storage,
            NoteAddArgs {
                project: Some("NewProject".into()),
                ..args("Body")
            },
        )
        .unwrap();

        let projects = storage.load_projects().unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "NewProject");

        let note = &storage.load_notes().unwrap()[0];
        assert_eq!(note.project_id, Some(projects[0].uuid));
    }

    // ── task link ─────────────────────────────────────────────────────────────

    #[test]
    fn test_note_add_linked_to_task() {
        let storage = InMemoryStorage::default();
        let task = Task::new("My task".into(), Priority::Medium, vec![], None, None, None);
        let task_uuid = task.uuid;
        storage.save(&[task]).unwrap();

        execute(
            &storage,
            NoteAddArgs {
                task: Some(1),
                ..args("Body")
            },
        )
        .unwrap();

        assert_eq!(storage.load_notes().unwrap()[0].task_id, Some(task_uuid));
    }

    #[test]
    fn test_note_add_invalid_task_id_fails() {
        let storage = InMemoryStorage::default();
        storage
            .save(&[Task::new(
                "Task".into(),
                Priority::Medium,
                vec![],
                None,
                None,
                None,
            )])
            .unwrap();

        let err = execute(
            &storage,
            NoteAddArgs {
                task: Some(99),
                ..args("Body")
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    // ── tag normalization ─────────────────────────────────────────────────────

    #[test]
    fn test_note_add_normalizes_tags_to_existing() {
        let storage = InMemoryStorage::default();

        execute(
            &storage,
            NoteAddArgs {
                tag: vec!["rust".into()],
                ..args("Note A")
            },
        )
        .unwrap();

        execute(
            &storage,
            NoteAddArgs {
                tag: vec!["Rust".into()],
                ..args("Note B")
            },
        )
        .unwrap();

        let notes = storage.load_notes().unwrap();
        assert_eq!(notes[1].tags[0], "rust");
    }
}
