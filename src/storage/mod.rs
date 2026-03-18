//! Storage abstraction layer for task, project, note, and resource persistence.
//!
//! | Type | Description |
//! |---|---|
//! | [`SqliteStorage`]   | Persists to a SQLite database in the OS data directory |
//! | [`InMemoryStorage`] | Stores in memory — ideal for tests |

use crate::models::StatusFilter;
use crate::models::{Note, Project, Resource, Task};
use anyhow::Result;
use uuid::Uuid;

// ── EntityType / EventType ────────────────────────────────────────────────────

/// The kind of entity an event refers to.
pub enum EntityType {
    Task,
    Project,
    Note,
    Resource,
}

impl EntityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityType::Task => "task",
            EntityType::Project => "project",
            EntityType::Note => "note",
            EntityType::Resource => "resource",
        }
    }
}

/// The kind of action that occurred on an entity.
pub enum EventType {
    Created,
    Completed,
    Uncompleted,
    Edited,
    Deleted,
    Purged,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::Created => "created",
            EventType::Completed => "completed",
            EventType::Uncompleted => "uncompleted",
            EventType::Edited => "edited",
            EventType::Deleted => "deleted",
            EventType::Purged => "purged",
        }
    }
}

// ── EventStat ─────────────────────────────────────────────────────────────────

/// Aggregated event count for a single month.
#[derive(Debug, Default)]
pub struct EventStat {
    pub year: i32,
    pub month: u32,
    pub created: usize,
    pub completed: usize,
    pub deleted: usize,
}

/// Trait defining storage operations for tasks, projects, notes, and resources.
pub trait Storage {
    // ── tasks ─────────────────────────────────────────────────────────────────

    /// Load all tasks from storage.
    fn load(&self) -> Result<Vec<Task>>;

    /// Persist all tasks (upsert by UUID).
    fn save(&self, tasks: &[Task]) -> Result<()>;

    /// Permanently delete tasks by UUID.
    fn delete_tasks(&self, uuids: &[Uuid]) -> Result<()>;

    // ── projects ──────────────────────────────────────────────────────────────

    /// Load all projects from storage.
    fn load_projects(&self) -> Result<Vec<Project>>;

    /// Persist all projects (upsert by UUID).
    fn save_projects(&self, projects: &[Project]) -> Result<()>;

    /// Permanently delete projects by UUID.
    fn delete_projects(&self, uuids: &[Uuid]) -> Result<()>;

    // ── notes ─────────────────────────────────────────────────────────────────

    /// Load all notes from storage.
    fn load_notes(&self) -> Result<Vec<Note>>;

    /// Persist all notes (upsert by UUID).
    fn save_notes(&self, notes: &[Note]) -> Result<()>;

    /// Permanently delete notes by UUID.
    fn delete_notes(&self, uuids: &[Uuid]) -> Result<()>;

    // ── resources ─────────────────────────────────────────────────────────────

    /// Load all resources from storage.
    fn load_resources(&self) -> Result<Vec<Resource>>;

    /// Persist all resources (upsert by UUID).
    fn save_resources(&self, resources: &[Resource]) -> Result<()>;

    /// Permanently delete resources by UUID.
    fn delete_resources(&self, uuids: &[Uuid]) -> Result<()>;

    // ── events ────────────────────────────────────────────────────────────────

    /// Record a domain event (created, completed, deleted, etc.).
    ///
    /// Should be called within the same logical operation as the data write
    /// so that the event log stays consistent with the entity state.
    fn record_event(
        &self,
        entity_type: EntityType,
        entity_uuid: Uuid,
        event_type: EventType,
    ) -> Result<()>;

    /// Delete events from the log.
    ///
    /// -  — deletes all events.
    /// -  — deletes only events older than n days.
    ///
    /// Returns the number of rows deleted.
    fn clear_events(&self, older_than_days: Option<u32>) -> Result<usize>;

    /// Load aggregated monthly event stats for the last `months` months.
    ///
    /// Used by `stats_history` to build the activity chart. Returns one
    /// `EventStat` per month, oldest first, covering only `task` events.
    fn load_event_stats(&self, months: usize) -> Result<Vec<EventStat>>;

    // ── search ────────────────────────────────────────────────────────────────

    /// Search tasks by substring query with optional tag and project filters.
    fn search_tasks(
        &self,
        q: &str,
        tags: &[String],
        project_id: Option<Uuid>,
        status: StatusFilter,
    ) -> Result<Vec<Task>> {
        let q_lower = q.to_lowercase();
        Ok(self
            .load()?
            .into_iter()
            .filter(|t| !t.is_deleted())
            .filter(|t| t.text.to_lowercase().contains(&q_lower))
            .filter(|t| t.matches_status(status))
            .filter(|t| tags.is_empty() || tags.iter().all(|tag| t.tags.contains(tag)))
            .filter(|t| project_id.is_none_or(|uuid| t.project_id == Some(uuid)))
            .collect())
    }

    /// Search notes by substring query with optional tag and project filters.
    fn search_notes(
        &self,
        q: &str,
        tags: &[String],
        project_id: Option<Uuid>,
    ) -> Result<Vec<Note>> {
        let q_lower = q.to_lowercase();
        Ok(self
            .load_notes()?
            .into_iter()
            .filter(|n| !n.is_deleted())
            .filter(|n| {
                n.title
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&q_lower)
                    || n.body.to_lowercase().contains(&q_lower)
                    || n.tags.iter().any(|t| t.to_lowercase().contains(&q_lower))
                    || n.language
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&q_lower)
            })
            .filter(|n| tags.is_empty() || tags.iter().all(|tag| n.tags.contains(tag)))
            .filter(|n| project_id.is_none_or(|uuid| n.project_id == Some(uuid)))
            .collect())
    }

    /// Search projects by substring query.
    fn search_projects(&self, q: &str) -> Result<Vec<Project>> {
        let q_lower = q.to_lowercase();
        Ok(self
            .load_projects()?
            .into_iter()
            .filter(|p| !p.is_deleted())
            .filter(|p| {
                p.name.to_lowercase().contains(&q_lower)
                    || p.tech.iter().any(|t| t.to_lowercase().contains(&q_lower))
            })
            .collect())
    }

    /// Search resources by substring query with optional tag filter.
    fn search_resources(&self, q: &str, tags: &[String]) -> Result<Vec<Resource>> {
        let q_lower = q.to_lowercase();
        Ok(self
            .load_resources()?
            .into_iter()
            .filter(|r| !r.is_deleted())
            .filter(|r| {
                r.title.to_lowercase().contains(&q_lower)
                    || r.url
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&q_lower)
                    || r.description
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&q_lower)
                    || r.tags.iter().any(|t| t.to_lowercase().contains(&q_lower))
            })
            .filter(|r| tags.is_empty() || tags.iter().all(|tag| r.tags.contains(tag)))
            .collect())
    }

    // ── single-entity upserts ─────────────────────────────────────────────────

    /// Persist a single task by UUID (upsert). Avoids rewriting the full table.
    fn upsert_task(&self, task: &Task) -> Result<()> {
        self.save(std::slice::from_ref(task))
    }

    /// Persist a single project by UUID (upsert). Avoids rewriting the full table.
    fn upsert_project(&self, project: &Project) -> Result<()> {
        self.save_projects(std::slice::from_ref(project))
    }

    // ── combined ──────────────────────────────────────────────────────────────

    /// Load tasks, projects, and notes in a single call.
    fn load_all(&self) -> Result<(Vec<Task>, Vec<Project>, Vec<Note>)> {
        Ok((self.load()?, self.load_projects()?, self.load_notes()?))
    }

    /// Load everything including resources.
    #[allow(clippy::type_complexity)]
    fn load_all_with_resources(
        &self,
    ) -> Result<(Vec<Task>, Vec<Project>, Vec<Note>, Vec<Resource>)> {
        Ok((
            self.load()?,
            self.load_projects()?,
            self.load_notes()?,
            self.load_resources()?,
        ))
    }

    /// Persist tasks, projects, and notes in a single call.
    fn save_all(&self, tasks: &[Task], projects: &[Project], notes: &[Note]) -> Result<()> {
        self.save(tasks)?;
        self.save_projects(projects)?;
        self.save_notes(notes)
    }

    /// Returns a human-readable description of the storage location.
    #[allow(dead_code)]
    fn location(&self) -> String;
}

pub mod backup;
pub mod memory;
pub mod sqlite;

pub use memory::InMemoryStorage;
pub use sqlite::{SqliteStorage, get_db_path};
