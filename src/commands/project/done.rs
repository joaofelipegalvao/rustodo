//! Handler for `todo project done <ID>`.

use anyhow::Result;
use colored::Colorize;

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
    let mut projects = storage.load_projects()?;

    let real_index = resolve_visible_index(&projects, id, |p| p.is_deleted())
        .map_err(|_| anyhow::anyhow!("Project #{} not found", id))?;

    let project = &mut projects[real_index];

    if project.completed {
        let msg = format!("Project {} is already done.", format!("#{}", id).green());
        if !silent {
            println!("{}", msg);
        }
        return Ok(msg);
    }

    let project_uuid = project.uuid;
    project.mark_done();
    storage.upsert_project(&projects[real_index])?;
    storage.record_event(EntityType::Project, project_uuid, EventType::Completed)?;

    let msg = format!("Project {} marked as done.", format!("#{}", id).green());
    if !silent {
        println!("{}", msg);
    }
    Ok(msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Project;
    use crate::storage::InMemoryStorage;

    #[test]
    fn test_project_done_marks_completed() {
        let storage = InMemoryStorage::default();
        storage
            .save_projects(&[Project::new("My Project".into())])
            .unwrap();

        execute_silent(&storage, 1).unwrap();

        let projects = storage.load_projects().unwrap();
        assert!(projects[0].completed);
        assert!(projects[0].completed_at.is_some());
    }

    #[test]
    fn test_project_done_invalid_id_returns_error() {
        let storage = InMemoryStorage::default();
        storage
            .save_projects(&[Project::new("Project".into())])
            .unwrap();

        assert!(execute_silent(&storage, 99).is_err());
    }

    #[test]
    fn test_project_done_already_completed_is_idempotent() {
        let storage = InMemoryStorage::default();
        let mut p = Project::new("Project".into());
        p.mark_done();
        storage.save_projects(&[p]).unwrap();

        let result = execute_silent(&storage, 1).unwrap();
        assert!(result.contains("already done"));
    }

    #[test]
    fn test_project_done_does_not_affect_other_projects() {
        let storage = InMemoryStorage::default();
        storage
            .save_projects(&[
                Project::new("Project A".into()),
                Project::new("Project B".into()),
            ])
            .unwrap();

        execute_silent(&storage, 1).unwrap();

        let projects = storage.load_projects().unwrap();
        assert!(projects[0].completed);
        assert!(!projects[1].completed);
    }
}
