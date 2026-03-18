use anyhow::Result;
use colored::Colorize;

use crate::storage::{EntityType, EventType, Storage};
use crate::utils::confirm;
use crate::utils::validation::resolve_visible_index;

pub fn execute(storage: &impl Storage, id: usize, yes: bool) -> Result<()> {
    execute_inner(storage, id, yes, false)?;
    Ok(())
}

pub fn execute_silent(storage: &impl Storage, id: usize) -> Result<String> {
    execute_inner(storage, id, true, true)
}

fn execute_inner(storage: &impl Storage, id: usize, yes: bool, silent: bool) -> Result<String> {
    let (mut tasks, projects, mut notes) = storage.load_all()?;

    let real_index = resolve_visible_index(&tasks, id, |t| t.is_deleted())
        .map_err(|_| anyhow::anyhow!("invalid task ID: {}", id))?;

    let task_uuid = tasks[real_index].uuid;
    let task_text = tasks[real_index].text.clone();

    if !yes && !silent {
        println!("\n{} {}", "".yellow(), task_text.bright_white());
        if !confirm("Are you sure? [y/N]:")? {
            println!("{} Removal cancelled.", "".yellow());
            return Ok("Cancelled.".to_string());
        }
    }

    tasks[real_index].soft_delete();

    for note in notes.iter_mut().filter(|n| !n.is_deleted()) {
        if note.task_id == Some(task_uuid) {
            note.task_id = None;
            note.touch();
        }
    }

    storage.save_all(&tasks, &projects, &notes)?;
    storage.record_event(EntityType::Task, task_uuid, EventType::Deleted)?;

    let msg = format!("Task removed: {}", task_text);
    if !silent {
        println!("{} {}", "✓".green(), msg.as_str().dimmed());
    }
    Ok(msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Priority, Task};
    use crate::storage::InMemoryStorage;

    fn make_task(text: &str) -> Task {
        Task::new(text.into(), Priority::Medium, vec![], None, None, None)
    }

    #[test]
    fn test_remove_soft_deletes_task() {
        let storage = InMemoryStorage::default();
        storage.save(&[make_task("Task A")]).unwrap();

        execute_silent(&storage, 1).unwrap();

        let tasks = storage.load().unwrap();
        assert!(tasks[0].is_deleted());
    }

    #[test]
    fn test_remove_invalid_id_returns_error() {
        let storage = InMemoryStorage::default();
        storage.save(&[make_task("Task")]).unwrap();

        assert!(execute_silent(&storage, 99).is_err());
    }

    #[test]
    fn test_remove_does_not_affect_other_tasks() {
        let storage = InMemoryStorage::default();
        storage
            .save(&[make_task("Task A"), make_task("Task B")])
            .unwrap();

        execute_silent(&storage, 1).unwrap();

        let tasks = storage.load().unwrap();
        assert!(tasks[0].is_deleted());
        assert!(!tasks[1].is_deleted());
    }

    #[test]
    fn test_remove_clears_note_task_id() {
        let storage = InMemoryStorage::default();
        let task = make_task("Task");
        let task_uuid = task.uuid;
        storage.save(&[task]).unwrap();

        let mut note = crate::models::Note::new("Note body".into());
        note.task_id = Some(task_uuid);
        storage.save_notes(&[note]).unwrap();

        execute_silent(&storage, 1).unwrap();

        let notes = storage.load_notes().unwrap();
        assert!(notes[0].task_id.is_none());
    }

    #[test]
    fn test_remove_skips_deleted_in_id_resolution() {
        let storage = InMemoryStorage::default();
        let mut deleted = make_task("Deleted");
        deleted.soft_delete();
        let active = make_task("Active");
        storage.save(&[deleted, active]).unwrap();

        execute_silent(&storage, 1).unwrap();

        let tasks = storage.load().unwrap();
        assert!(tasks[0].is_deleted()); // was already deleted
        assert!(tasks[1].is_deleted()); // #1 resolved to active, now deleted
    }
}
