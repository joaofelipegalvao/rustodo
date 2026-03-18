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
        storage.search_projects(&query)?
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
