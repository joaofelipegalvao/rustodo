//! Handler for `todo search <QUERY>`.
//!
//! Case-insensitive substring search over tasks, notes, projects, and resources.
//! Searches across: task text, note title/body, project name/tech, resource title/url/description.
//!
//! Filter behaviour:
//! - `--project` filters tasks, notes, and projects (resources have no project_id).
//! - `--tag`     filters tasks, notes, and resources (projects have no tags — hidden when --tag is passed).
//! - `--status`  filters tasks only.

use anyhow::Result;
use colored::Colorize;
use uuid::Uuid;

use crate::error::TodoError;
use crate::models::StatusFilter;
use crate::render::display_lists;
use crate::render::note_table::display_notes;
use crate::render::project_table::display_projects;
use crate::render::resource_table::display_resources;
use crate::storage::Storage;

pub fn execute(
    storage: &impl Storage,
    query: String,
    tags: Vec<String>,
    project: Option<String>,
    status: StatusFilter,
) -> Result<()> {
    // ── Resolve project UUID ───────────────────────────────────────────────────
    let proj_uuid: Option<Uuid> = if let Some(ref project_name) = project {
        let projects = storage.load_projects()?;
        let uuid = projects
            .iter()
            .find(|p| p.name.to_lowercase() == project_name.to_lowercase() && !p.is_deleted())
            .map(|p| p.uuid);

        if uuid.is_none() {
            return Err(TodoError::ProjectNotFound(project_name.clone()).into());
        }
        uuid
    } else {
        None
    };

    // ── Search each entity via storage (SQLite uses WHERE LIKE) ───────────────
    let task_results = storage.search_tasks(&query, &tags, proj_uuid, status)?;
    let note_results = storage.search_notes(&query, &tags, proj_uuid)?;
    let project_results = if tags.is_empty() {
        storage
            .search_projects(&query)?
            .into_iter()
            .filter(|p| proj_uuid.is_none_or(|uuid| p.uuid == uuid))
            .collect()
    } else {
        vec![]
    };
    let resource_results = storage.search_resources(&query, &tags)?;

    if task_results.is_empty()
        && note_results.is_empty()
        && project_results.is_empty()
        && resource_results.is_empty()
    {
        return Err(TodoError::NoSearchResults(query).into());
    }

    // ── Render ────────────────────────────────────────────────────────────────
    let found_total =
        task_results.len() + note_results.len() + project_results.len() + resource_results.len();

    println!(
        "\nSearch results for \"{}\"  ({})\n",
        query,
        format!("{} found", found_total).dimmed()
    );

    if !task_results.is_empty() {
        // Render context: full task list needed for deps/blocking display
        let all_tasks = storage.load()?;
        let all_projects = storage.load_projects()?;
        let all_notes = storage.load_notes()?;
        let all_resources = storage.load_resources()?;

        let visible_tasks: Vec<_> = all_tasks
            .iter()
            .filter(|t| !t.is_deleted())
            .cloned()
            .collect();
        let task_pairs: Vec<(usize, &_)> = task_results
            .iter()
            .filter_map(|t| {
                visible_tasks
                    .iter()
                    .position(|v| v.uuid == t.uuid)
                    .map(|i| (i + 1, t))
            })
            .collect();

        let title = format!("Tasks  ({})", task_pairs.len());
        display_lists(
            &task_pairs,
            &title,
            &visible_tasks,
            &all_projects,
            &all_notes,
            &all_resources,
        );

        if !project_results.is_empty() {
            display_projects(
                &project_results.iter().collect::<Vec<_>>(),
                &all_tasks,
                &all_notes,
            );
        }
        if !note_results.is_empty() {
            display_notes(
                &note_results.iter().collect::<Vec<_>>(),
                &all_projects,
                &all_resources,
            );
        }
        if !resource_results.is_empty() {
            display_resources(&resource_results.iter().collect::<Vec<_>>(), &all_notes);
        }
    } else {
        // No tasks — load only what's needed for each render section
        if !project_results.is_empty() {
            let all_tasks = storage.load()?;
            let all_notes = storage.load_notes()?;
            display_projects(
                &project_results.iter().collect::<Vec<_>>(),
                &all_tasks,
                &all_notes,
            );
        }
        if !note_results.is_empty() {
            let all_projects = storage.load_projects()?;
            let all_resources = storage.load_resources()?;
            display_notes(
                &note_results.iter().collect::<Vec<_>>(),
                &all_projects,
                &all_resources,
            );
        }
        if !resource_results.is_empty() {
            let all_notes = storage.load_notes()?;
            display_resources(&resource_results.iter().collect::<Vec<_>>(), &all_notes);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::StatusFilter;
    use crate::models::{Note, Priority, Project, Resource, Task};
    use crate::storage::InMemoryStorage;

    fn make_task(text: &str) -> Task {
        Task::new(text.into(), Priority::Medium, vec![], None, None, None)
    }

    fn search(storage: &InMemoryStorage, query: &str) -> Result<()> {
        execute(storage, query.into(), vec![], None, StatusFilter::All)
    }

    #[test]
    fn test_search_finds_task_by_text() {
        let storage = InMemoryStorage::default();
        storage.save(&[make_task("Buy milk")]).unwrap();

        assert!(search(&storage, "milk").is_ok());
    }

    #[test]
    fn test_search_no_results_fails() {
        let storage = InMemoryStorage::default();
        storage.save(&[make_task("Buy milk")]).unwrap();

        assert!(search(&storage, "xyz").is_err());
    }

    #[test]
    fn test_search_case_insensitive() {
        let storage = InMemoryStorage::default();
        storage.save(&[make_task("Buy Milk")]).unwrap();

        assert!(search(&storage, "buy milk").is_ok());
    }

    #[test]
    fn test_search_finds_note_by_body() {
        let storage = InMemoryStorage::default();
        storage
            .save_notes(&[Note::new("Interesting note body".into())])
            .unwrap();

        assert!(search(&storage, "interesting").is_ok());
    }

    #[test]
    fn test_search_finds_project_by_name() {
        let storage = InMemoryStorage::default();
        storage
            .save_projects(&[Project::new("Rustodo".into())])
            .unwrap();

        assert!(search(&storage, "rustodo").is_ok());
    }

    #[test]
    fn test_search_finds_resource_by_title() {
        let storage = InMemoryStorage::default();
        storage
            .save_resources(&[Resource::new("SQLx docs".into())])
            .unwrap();

        assert!(search(&storage, "sqlx").is_ok());
    }

    #[test]
    fn test_search_with_tag_filter() {
        let storage = InMemoryStorage::default();
        let mut task = make_task("Task");
        task.tags = vec!["rust".into()];
        storage.save(&[task]).unwrap();

        let result = execute(
            &storage,
            "task".into(),
            vec!["rust".into()],
            None,
            StatusFilter::All,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_tag_filter_no_match_fails() {
        let storage = InMemoryStorage::default();
        storage.save(&[make_task("Task")]).unwrap();

        let result = execute(
            &storage,
            "task".into(),
            vec!["nonexistent".into()],
            None,
            StatusFilter::All,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_search_status_pending_excludes_done() {
        let storage = InMemoryStorage::default();
        let mut done = make_task("Done task");
        done.mark_done();
        storage.save(&[done]).unwrap();

        let result = execute(&storage, "task".into(), vec![], None, StatusFilter::Pending);
        assert!(result.is_err());
    }

    #[test]
    fn test_search_with_project_filter() {
        let storage = InMemoryStorage::default();
        let p = Project::new("Rustodo".into());
        let uuid = p.uuid;
        storage.save_projects(&[p]).unwrap();
        let mut task = make_task("Task");
        task.project_id = Some(uuid);
        storage.save(&[task]).unwrap();

        let result = execute(
            &storage,
            "task".into(),
            vec![],
            Some("Rustodo".into()),
            StatusFilter::All,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_nonexistent_project_fails() {
        let storage = InMemoryStorage::default();
        storage.save(&[make_task("Task")]).unwrap();

        let result = execute(
            &storage,
            "task".into(),
            vec![],
            Some("NonExistent".into()),
            StatusFilter::All,
        );
        assert!(result.is_err());
    }
}
