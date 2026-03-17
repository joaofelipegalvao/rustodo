//! Storage abstraction layer for task, project, note, and resource persistence.
//!
//! | Type | Description |
//! |---|---|
//! | [`SqliteStorage`]   | Persists to a SQLite database in the OS data directory |
//! | [`InMemoryStorage`] | Stores in memory — ideal for tests |

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
