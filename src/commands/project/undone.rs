//! Handler for `todo project undone <ID>`.

use anyhow::Result;
use colored::Colorize;

use crate::storage::Storage;
use crate::utils::validation::resolve_visible_index;

pub fn execute(storage: &impl Storage, id: usize) -> Result<()> {
    execute_inner(storage, id, false)?;
    Ok(())
}

/// TUI variant: same logic, no stdout, returns a status string.
pub fn execute_silent(storage: &impl Storage, id: usize) -> Result<String> {
    execute_inner(storage, id, true)
}

fn execute_inner(storage: &impl Storage, id: usize, silent: bool) -> Result<String> {
    let mut projects = storage.load_projects()?;

    let real_index = resolve_visible_index(&projects, id, |p| p.is_deleted())
        .map_err(|_| anyhow::anyhow!("Project #{} not found", id))?;

    let project = &mut projects[real_index];

    if !project.completed {
        let msg = format!(
            "Project {} is already pending.",
            format!("#{}", id).yellow()
        );
        if !silent {
            println!("{}", msg);
        }
        return Ok(msg);
    }

    project.mark_undone();
    storage.upsert_project(&projects[real_index])?;

    let msg = format!("Project {} marked as pending.", format!("#{}", id).yellow());
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

    fn make_done_project(name: &str) -> Project {
        let mut p = Project::new(name.into());
        p.mark_done();
        p
    }

    #[test]
    fn test_project_undone_marks_pending() {
        let storage = InMemoryStorage::default();
        storage
            .save_projects(&[make_done_project("Project")])
            .unwrap();

        execute_silent(&storage, 1).unwrap();

        let projects = storage.load_projects().unwrap();
        assert!(!projects[0].completed);
        assert!(projects[0].completed_at.is_none());
    }

    #[test]
    fn test_project_undone_invalid_id_returns_error() {
        let storage = InMemoryStorage::default();
        storage
            .save_projects(&[make_done_project("Project")])
            .unwrap();

        assert!(execute_silent(&storage, 99).is_err());
    }

    #[test]
    fn test_project_undone_already_pending_is_idempotent() {
        let storage = InMemoryStorage::default();
        storage
            .save_projects(&[Project::new("Project".into())])
            .unwrap();

        let result = execute_silent(&storage, 1).unwrap();
        assert!(result.contains("already pending"));
    }
}
