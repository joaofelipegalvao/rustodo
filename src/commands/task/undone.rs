//! Handler for `todo undone <ID>`.

use anyhow::Result;
use colored::Colorize;

use crate::error::TodoError;
use crate::storage::{EntityType, EventType, Storage};
use crate::utils::validation::resolve_visible_index;

pub fn execute(storage: &impl Storage, id: usize) -> Result<()> {
    execute_inner(storage, id, false)?;
    Ok(())
}

pub fn execute_silent(storage: &impl Storage, id: usize) -> Result<String> {
    execute_inner(storage, id, true)
}

fn execute_inner(storage: &impl Storage, id: usize, silent: bool) -> Result<String> {
    let mut tasks = storage.load()?;

    let index = resolve_visible_index(&tasks, id, |t| t.is_deleted())
        .map_err(|_| anyhow::anyhow!("invalid task ID: {}", id))?;

    if !tasks[index].completed {
        return Err(TodoError::TaskAlreadyInStatus {
            id,
            status: "pending".to_owned(),
        }
        .into());
    }

    let task_uuid = tasks[index].uuid;
    tasks[index].mark_undone();
    storage.upsert_task(&tasks[index])?;
    storage.record_event(EntityType::Task, task_uuid, EventType::Uncompleted)?;

    if !silent {
        println!("Task {} marked as pending.", format!("#{}", id).yellow());
    }
    Ok(format!("Task #{} marked as pending.", id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Priority, Task};
    use crate::storage::InMemoryStorage;

    fn make_done_task(text: &str) -> Task {
        let mut t = Task::new(text.into(), Priority::Medium, vec![], None, None, None);
        t.mark_done();
        t
    }

    #[test]
    fn test_undone_marks_task_pending() {
        let storage = InMemoryStorage::default();
        storage.save(&[make_done_task("Task")]).unwrap();

        execute_silent(&storage, 1).unwrap();

        let tasks = storage.load().unwrap();
        assert!(!tasks[0].completed);
        assert!(tasks[0].completed_at.is_none());
    }

    #[test]
    fn test_undone_invalid_id_returns_error() {
        let storage = InMemoryStorage::default();
        storage.save(&[make_done_task("Task")]).unwrap();

        assert!(execute_silent(&storage, 99).is_err());
    }

    #[test]
    fn test_undone_already_pending_returns_error() {
        let storage = InMemoryStorage::default();
        let task = Task::new("Task".into(), Priority::Medium, vec![], None, None, None);
        storage.save(&[task]).unwrap();

        let err = execute_silent(&storage, 1).unwrap_err();
        assert!(err.to_string().contains("pending"));
    }

    #[test]
    fn test_undone_does_not_affect_other_tasks() {
        let storage = InMemoryStorage::default();
        storage
            .save(&[make_done_task("Task A"), make_done_task("Task B")])
            .unwrap();

        execute_silent(&storage, 1).unwrap();

        let tasks = storage.load().unwrap();
        assert!(!tasks[0].completed);
        assert!(tasks[1].completed);
    }
}
