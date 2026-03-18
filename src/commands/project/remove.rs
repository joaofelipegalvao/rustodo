//! Handler for `todo project remove <ID>`.

use anyhow::Result;
use colored::Colorize;

use crate::storage::{EntityType, EventType, Storage};
use crate::utils::validation::resolve_visible_index;

pub fn execute(storage: &impl Storage, id: usize, yes: bool) -> Result<()> {
    execute_inner(storage, id, yes, false)?;
    Ok(())
}

pub fn execute_silent(storage: &impl Storage, id: usize) -> Result<String> {
    execute_inner(storage, id, true, true)
}

fn execute_inner(storage: &impl Storage, id: usize, yes: bool, silent: bool) -> Result<String> {
    let (mut tasks, mut projects, mut notes) = storage.load_all()?;

    let real_index = resolve_visible_index(&projects, id, |p| p.is_deleted())
        .map_err(|_| anyhow::anyhow!("Project #{} not found", id))?;

    let project_uuid = projects[real_index].uuid;
    let name = projects[real_index].name.clone();

    if !yes && !silent {
        println!(
            "{} Remove project #{}: {}? [y/N] ",
            "!".yellow(),
            id,
            name.bold()
        );
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{} Cancelled.", "".dimmed());
            return Ok("Cancelled.".to_string());
        }
    }

    projects[real_index].soft_delete();

    for task in tasks.iter_mut().filter(|t| !t.is_deleted()) {
        if task.project_id == Some(project_uuid) {
            task.project_id = None;
            task.touch();
        }
    }
    for note in notes.iter_mut().filter(|n| !n.is_deleted()) {
        if note.project_id == Some(project_uuid) {
            note.project_id = None;
            note.touch();
        }
    }

    storage.save_all(&tasks, &projects, &notes)?;
    storage.record_event(EntityType::Project, project_uuid, EventType::Deleted)?;

    let msg = format!("Project #{} ({}) removed.", id, name);
    if !silent {
        println!("{} {}", "✓".green(), msg.as_str().cyan());
    }
    Ok(msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Note, Priority, Project, Task};
    use crate::storage::InMemoryStorage;

    fn make_project(name: &str) -> Project {
        Project::new(name.into())
    }

    #[test]
    fn test_project_remove_soft_deletes() {
        let storage = InMemoryStorage::default();
        storage.save_projects(&[make_project("Rustodo")]).unwrap();

        execute_silent(&storage, 1).unwrap();

        assert!(storage.load_projects().unwrap()[0].is_deleted());
    }

    #[test]
    fn test_project_remove_invalid_id_fails() {
        let storage = InMemoryStorage::default();
        storage.save_projects(&[make_project("Project")]).unwrap();

        assert!(execute_silent(&storage, 99).is_err());
    }

    #[test]
    fn test_project_remove_does_not_affect_others() {
        let storage = InMemoryStorage::default();
        storage
            .save_projects(&[make_project("A"), make_project("B")])
            .unwrap();

        execute_silent(&storage, 1).unwrap();

        let projects = storage.load_projects().unwrap();
        assert!(projects[0].is_deleted());
        assert!(!projects[1].is_deleted());
    }

    #[test]
    fn test_project_remove_clears_task_project_id() {
        let storage = InMemoryStorage::default();
        let p = make_project("Rustodo");
        let uuid = p.uuid;
        storage.save_projects(&[p]).unwrap();
        let task = Task::new(
            "Task".into(),
            Priority::Medium,
            vec![],
            Some(uuid),
            None,
            None,
        );
        storage.save(&[task]).unwrap();

        execute_silent(&storage, 1).unwrap();

        assert!(storage.load().unwrap()[0].project_id.is_none());
    }

    #[test]
    fn test_project_remove_clears_note_project_id() {
        let storage = InMemoryStorage::default();
        let p = make_project("Rustodo");
        let uuid = p.uuid;
        storage.save_projects(&[p]).unwrap();
        let mut note = Note::new("Body".into());
        note.project_id = Some(uuid);
        storage.save_notes(&[note]).unwrap();

        execute_silent(&storage, 1).unwrap();

        assert!(storage.load_notes().unwrap()[0].project_id.is_none());
    }
}
