//! Handler for `todo clear`.

use anyhow::Result;
use colored::Colorize;
use uuid::Uuid;

use crate::storage::{EntityType, EventType, Storage};
use crate::utils::confirm;

pub fn execute(storage: &impl Storage, yes: bool) -> Result<()> {
    let (mut tasks, projects, mut notes) = storage.load_all()?;

    let visible_count = tasks.iter().filter(|t| !t.is_deleted()).count();

    if visible_count == 0 {
        println!("{} No tasks to remove", "".blue());
        return Ok(());
    }

    if !yes {
        println!(
            "\n{} {} tasks will be permanently deleted!",
            "".yellow().bold(),
            visible_count
        );
        if !confirm("Type 'yes' to confirm:")? {
            println!("{} Clear cancelled.", "".yellow());
            return Ok(());
        }
    }

    let deleted_uuids: Vec<Uuid> = tasks
        .iter()
        .filter(|t| !t.is_deleted())
        .map(|t| t.uuid)
        .collect();

    for task in tasks.iter_mut().filter(|t| !t.is_deleted()) {
        task.soft_delete();
    }

    let mut notes_updated = 0;
    for note in notes.iter_mut().filter(|n| !n.is_deleted()) {
        if let Some(tid) = note.task_id
            && deleted_uuids.contains(&tid)
        {
            note.task_id = None;
            note.touch();
            notes_updated += 1;
        }
    }

    storage.save_all(&tasks, &projects, &notes)?;

    // Record one Deleted event per task
    for uuid in &deleted_uuids {
        storage.record_event(EntityType::Task, *uuid, EventType::Deleted)?;
    }

    println!(
        "{} {} tasks have been removed",
        "✓".green().bold(),
        visible_count
    );
    if notes_updated > 0 {
        println!(
            "  {} {} note{} unlinked",
            "·".dimmed(),
            notes_updated,
            if notes_updated == 1 { "" } else { "s" }
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Note, Priority, Task};
    use crate::storage::InMemoryStorage;

    fn make_task(text: &str) -> Task {
        Task::new(text.into(), Priority::Medium, vec![], None, None, None)
    }

    #[test]
    fn test_clear_soft_deletes_all_tasks() {
        let storage = InMemoryStorage::default();
        storage
            .save(&[
                make_task("Task A"),
                make_task("Task B"),
                make_task("Task C"),
            ])
            .unwrap();

        execute(&storage, true).unwrap();

        let tasks = storage.load().unwrap();
        assert!(tasks.iter().all(|t| t.is_deleted()));
    }

    #[test]
    fn test_clear_empty_storage_is_ok() {
        let storage = InMemoryStorage::default();
        execute(&storage, true).unwrap();
        assert!(storage.load().unwrap().is_empty());
    }

    #[test]
    fn test_clear_does_not_remove_already_deleted() {
        let storage = InMemoryStorage::default();
        let mut deleted = make_task("Already deleted");
        deleted.soft_delete();
        storage.save(&[deleted, make_task("Active")]).unwrap();

        execute(&storage, true).unwrap();

        // Both end up deleted — the active one gets soft-deleted too
        let tasks = storage.load().unwrap();
        assert!(tasks.iter().all(|t| t.is_deleted()));
    }

    #[test]
    fn test_clear_unlinks_note_task_ids() {
        let storage = InMemoryStorage::default();
        let task = make_task("Task");
        let task_uuid = task.uuid;
        storage.save(&[task]).unwrap();

        let mut note = Note::new("Note body".into());
        note.task_id = Some(task_uuid);
        storage.save_notes(&[note]).unwrap();

        execute(&storage, true).unwrap();

        assert!(storage.load_notes().unwrap()[0].task_id.is_none());
    }

    #[test]
    fn test_clear_preserves_notes_themselves() {
        let storage = InMemoryStorage::default();
        storage.save(&[make_task("Task")]).unwrap();
        storage.save_notes(&[Note::new("Note".into())]).unwrap();

        execute(&storage, true).unwrap();

        assert!(!storage.load_notes().unwrap().is_empty());
    }
}
