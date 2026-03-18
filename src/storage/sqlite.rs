//! SQLite-backed storage implementation for rustodo.
//!
//! Uses `rusqlite` (synchronous) — no async runtime needed for a CLI.
//! UUIDs are stored as TEXT, arrays (tags, tech) as JSON TEXT via `JsonVec<T>`.
//!
//! # Connection strategy
//!
//! A single `Connection` is held inside a `RefCell` and reused across all
//! operations. This avoids the overhead of opening a new file handle on every
//! read/write call, which is especially noticeable in the TUI render loop.
//!
//! # Transaction strategy
//!
//! Every write method wraps its loop in an explicit `conn.transaction()` →
//! `tx.commit()`. This guarantees atomicity: either all rows are written or
//! none are, preventing partial updates that could corrupt relational integrity.
//!
//! # Event log
//!
//! Every domain action (create, complete, delete, etc.) records a row in the
//! `events` table. This makes `stats_history` accurate even after `purge`
//! physically removes tombstones — the event log is append-only and never
//! cleaned up automatically.

use std::cell::RefCell;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc};
use directories::ProjectDirs;
use rusqlite::{
    Connection, Row, params,
    types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef},
};
use serde::{Serialize, de::DeserializeOwned};
use uuid::Uuid;

use super::{EntityType, EventStat, EventType, Storage};
use crate::models::StatusFilter;
use crate::models::{
    Difficulty, Note, NoteFormat, Priority, Project, Recurrence, Resource, ResourceType, Task,
};

// ── JsonVec<T> ────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct JsonVec<T>(pub Vec<T>);

impl<T: Serialize> ToSql for JsonVec<T> {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let json = serde_json::to_string(&self.0)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        Ok(ToSqlOutput::from(json))
    }
}

impl<T: DeserializeOwned> FromSql for JsonVec<T> {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s = value.as_str()?;
        serde_json::from_str(s)
            .map(JsonVec)
            .map_err(|e| FromSqlError::Other(Box::new(e)))
    }
}

// ── timestamp helpers ─────────────────────────────────────────────────────────

fn to_unix(dt: DateTime<Utc>) -> i64 {
    dt.timestamp()
}

fn from_unix(ts: i64) -> DateTime<Utc> {
    Utc.timestamp_opt(ts, 0)
        .single()
        .unwrap_or(DateTime::UNIX_EPOCH)
}

fn opt_to_unix(dt: Option<DateTime<Utc>>) -> Option<i64> {
    dt.map(to_unix)
}

fn opt_from_unix(ts: Option<i64>) -> Option<DateTime<Utc>> {
    ts.map(from_unix)
}

// ── SqliteStorage ─────────────────────────────────────────────────────────────

pub struct SqliteStorage {
    conn: RefCell<Connection>,
    path: PathBuf,
}

impl SqliteStorage {
    pub fn new() -> Result<Self> {
        let path = get_db_path()?;
        Self::open_at(path)
    }

    #[cfg(test)]
    pub fn with_path(path: PathBuf) -> Result<Self> {
        Self::open_at(path)
    }

    fn open_at(path: PathBuf) -> Result<Self> {
        let conn = Connection::open(&path).context("Failed to open SQLite database")?;
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")
            .context("Failed to set SQLite pragmas")?;
        let storage = Self {
            conn: RefCell::new(conn),
            path,
        };
        storage.initialize()?;
        Ok(storage)
    }

    fn initialize(&self) -> Result<()> {
        self.conn
            .borrow()
            .execute_batch(SCHEMA)
            .context("Failed to initialize schema")?;
        Ok(())
    }
}

// ── schema ────────────────────────────────────────────────────────────────────

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS projects (
    uuid        TEXT PRIMARY KEY NOT NULL,
    name        TEXT NOT NULL,
    completed   INTEGER NOT NULL DEFAULT 0,
    difficulty  TEXT NOT NULL DEFAULT 'medium',
    tech        TEXT NOT NULL DEFAULT '[]',
    due_date    TEXT,
    completed_at TEXT,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER,
    deleted_at  INTEGER
);

CREATE TABLE IF NOT EXISTS tasks (
    uuid        TEXT PRIMARY KEY NOT NULL,
    text        TEXT NOT NULL,
    completed   INTEGER NOT NULL DEFAULT 0,
    priority    TEXT NOT NULL DEFAULT 'medium'
                    CHECK(priority IN ('low','medium','high')),
    due_date    TEXT,
    recurrence  TEXT,
    project_id  TEXT REFERENCES projects(uuid),
    parent_id   TEXT REFERENCES tasks(uuid),
    tags        TEXT NOT NULL DEFAULT '[]',
    completed_at INTEGER,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER,
    deleted_at  INTEGER
);

CREATE TABLE IF NOT EXISTS task_dependencies (
    task_uuid        TEXT NOT NULL REFERENCES tasks(uuid),
    depends_on_uuid  TEXT NOT NULL REFERENCES tasks(uuid),
    PRIMARY KEY (task_uuid, depends_on_uuid)
);

CREATE TABLE IF NOT EXISTS notes (
    uuid        TEXT PRIMARY KEY NOT NULL,
    title       TEXT,
    body        TEXT NOT NULL,
    format      TEXT NOT NULL DEFAULT 'plain',
    language    TEXT,
    project_id  TEXT REFERENCES projects(uuid),
    task_id     TEXT REFERENCES tasks(uuid),
    tags        TEXT NOT NULL DEFAULT '[]',
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER,
    deleted_at  INTEGER
);

CREATE TABLE IF NOT EXISTS resources (
    uuid          TEXT PRIMARY KEY NOT NULL,
    title         TEXT NOT NULL,
    resource_type TEXT,
    url           TEXT,
    description   TEXT,
    tags          TEXT NOT NULL DEFAULT '[]',
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER,
    deleted_at    INTEGER
);

CREATE TABLE IF NOT EXISTS note_resources (
    note_uuid     TEXT NOT NULL REFERENCES notes(uuid),
    resource_uuid TEXT NOT NULL REFERENCES resources(uuid),
    PRIMARY KEY (note_uuid, resource_uuid)
);

-- Event log: append-only, never purged automatically.
-- Records every domain action so stats_history stays accurate
-- even after tombstones are physically removed by 'todo purge'.
CREATE TABLE IF NOT EXISTS events (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_type TEXT NOT NULL CHECK(entity_type IN ('task','project','note','resource')),
    entity_uuid TEXT NOT NULL,
    event_type  TEXT NOT NULL CHECK(event_type IN ('created','completed','uncompleted','edited','deleted','purged')),
    occurred_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_tasks_active
    ON tasks(created_at) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_tasks_project_active
    ON tasks(project_id, created_at) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_tasks_completed
    ON tasks(completed_at);
CREATE INDEX IF NOT EXISTS idx_notes_active
    ON notes(created_at) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_notes_project
    ON notes(project_id, created_at) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_notes_task
    ON notes(task_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_resources_active
    ON resources(created_at) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_projects_active
    ON projects(created_at) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_task_deps
    ON task_dependencies(depends_on_uuid);
CREATE INDEX IF NOT EXISTS idx_events_occurred
    ON events(occurred_at);
CREATE INDEX IF NOT EXISTS idx_events_entity
    ON events(entity_uuid);
";

// ── row mappers ───────────────────────────────────────────────────────────────

fn row_to_task(row: &Row, conn: &Connection, uuid_str: &str) -> rusqlite::Result<Task> {
    let uuid = Uuid::parse_str(uuid_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;

    let priority_str: String = row.get("priority")?;
    let priority = match priority_str.as_str() {
        "high" => Priority::High,
        "low" => Priority::Low,
        _ => Priority::Medium,
    };

    let recurrence_str: Option<String> = row.get("recurrence")?;
    let recurrence = recurrence_str.as_deref().and_then(|s| match s {
        "daily" => Some(Recurrence::Daily),
        "weekly" => Some(Recurrence::Weekly),
        "monthly" => Some(Recurrence::Monthly),
        _ => None,
    });

    let project_id_str: Option<String> = row.get("project_id")?;
    let project_id = project_id_str
        .as_deref()
        .and_then(|s| Uuid::parse_str(s).ok());

    let due_date_str: Option<String> = row.get("due_date")?;
    let due_date = due_date_str
        .as_deref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let tags: JsonVec<String> = row.get("tags")?;
    let created_at = from_unix(row.get("created_at")?);
    let updated_at = opt_from_unix(row.get("updated_at")?);
    let deleted_at = opt_from_unix(row.get("deleted_at")?);
    let completed_at_ts: Option<i64> = row.get("completed_at")?;
    let completed_at = completed_at_ts
        .map(from_unix)
        .map(|dt| dt.naive_local().date());

    let mut dep_stmt =
        conn.prepare_cached("SELECT depends_on_uuid FROM task_dependencies WHERE task_uuid = ?1")?;
    let depends_on: Vec<Uuid> = dep_stmt
        .query_map(params![uuid_str], |r| r.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .filter_map(|s| Uuid::parse_str(&s).ok())
        .collect();

    let parent_id_str: Option<String> = row.get("parent_id")?;
    let parent_id = parent_id_str
        .as_deref()
        .and_then(|s| Uuid::parse_str(s).ok());

    Ok(Task {
        uuid,
        text: row.get("text")?,
        completed: row.get::<_, i64>("completed")? != 0,
        priority,
        due_date,
        recurrence,
        project_id,
        project_name_legacy: None,
        parent_id,
        tags: tags.0,
        depends_on,
        created_at,
        updated_at,
        deleted_at,
        completed_at,
    })
}

fn row_to_project(row: &Row) -> rusqlite::Result<Project> {
    let uuid_str: String = row.get("uuid")?;
    let uuid = Uuid::parse_str(&uuid_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;

    let difficulty_str: String = row.get("difficulty")?;
    let difficulty = match difficulty_str.as_str() {
        "easy" => Difficulty::Easy,
        "hard" => Difficulty::Hard,
        _ => Difficulty::Medium,
    };

    let tech: JsonVec<String> = row.get("tech")?;
    let due_date_str: Option<String> = row.get("due_date")?;
    let due_date = due_date_str
        .as_deref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
    let completed_at_str: Option<String> = row.get("completed_at")?;
    let completed_at = completed_at_str
        .as_deref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    Ok(Project {
        uuid,
        name: row.get("name")?,
        completed: row.get::<_, i64>("completed")? != 0,
        difficulty,
        tech: tech.0,
        due_date,
        completed_at,
        created_at: from_unix(row.get("created_at")?),
        updated_at: opt_from_unix(row.get("updated_at")?),
        deleted_at: opt_from_unix(row.get("deleted_at")?),
    })
}

fn row_to_note(row: &Row, conn: &Connection) -> rusqlite::Result<Note> {
    let uuid_str: String = row.get("uuid")?;
    let uuid = Uuid::parse_str(&uuid_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;

    let format_str: String = row.get("format")?;
    let format = match format_str.as_str() {
        "markdown" => NoteFormat::Markdown,
        _ => NoteFormat::Plain,
    };

    let project_id_str: Option<String> = row.get("project_id")?;
    let project_id = project_id_str
        .as_deref()
        .and_then(|s| Uuid::parse_str(s).ok());
    let task_id_str: Option<String> = row.get("task_id")?;
    let task_id = task_id_str.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let tags: JsonVec<String> = row.get("tags")?;

    let mut res_stmt =
        conn.prepare_cached("SELECT resource_uuid FROM note_resources WHERE note_uuid = ?1")?;
    let resource_ids: Vec<Uuid> = res_stmt
        .query_map(params![uuid_str], |r| r.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .filter_map(|s| Uuid::parse_str(&s).ok())
        .collect();

    Ok(Note {
        uuid,
        title: row.get("title")?,
        body: row.get("body")?,
        format,
        language: row.get("language")?,
        project_id,
        task_id,
        tags: tags.0,
        resource_ids,
        created_at: from_unix(row.get("created_at")?),
        updated_at: opt_from_unix(row.get("updated_at")?),
        deleted_at: opt_from_unix(row.get("deleted_at")?),
    })
}

fn row_to_resource(row: &Row) -> rusqlite::Result<Resource> {
    let uuid_str: String = row.get("uuid")?;
    let uuid = Uuid::parse_str(&uuid_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;

    let resource_type_str: Option<String> = row.get("resource_type")?;
    let resource_type = resource_type_str.as_deref().and_then(|s| match s {
        "docs" => Some(ResourceType::Docs),
        "article" => Some(ResourceType::Article),
        "video" => Some(ResourceType::Video),
        "repo" => Some(ResourceType::Repo),
        "crate" => Some(ResourceType::Crate),
        "book" => Some(ResourceType::Book),
        "spec" => Some(ResourceType::Spec),
        "tool" => Some(ResourceType::Tool),
        _ => None,
    });

    let tags: JsonVec<String> = row.get("tags")?;

    Ok(Resource {
        uuid,
        title: row.get("title")?,
        resource_type,
        url: row.get("url")?,
        description: row.get("description")?,
        tags: tags.0,
        created_at: from_unix(row.get("created_at")?),
        updated_at: opt_from_unix(row.get("updated_at")?),
        deleted_at: opt_from_unix(row.get("deleted_at")?),
    })
}

// ── Storage impl ──────────────────────────────────────────────────────────────

impl Storage for SqliteStorage {
    fn load(&self) -> Result<Vec<Task>> {
        let conn = self.conn.borrow();
        let mut stmt = conn.prepare("SELECT * FROM tasks ORDER BY created_at")?;
        let tasks = stmt
            .query_map([], |row| {
                let uuid_str: String = row.get("uuid")?;
                row_to_task(row, &conn, &uuid_str)
            })?
            .collect::<rusqlite::Result<Vec<_>>>()
            .context("Failed to load tasks")?;
        Ok(tasks)
    }

    fn upsert_task(&self, task: &Task) -> Result<()> {
        let mut conn = self.conn.borrow_mut();
        let tx = conn
            .transaction()
            .context("Failed to begin upsert_task transaction")?;
        let uuid_str = task.uuid.to_string();
        tx.execute(
            "INSERT INTO tasks (uuid, text, completed, priority, due_date, recurrence,
                      project_id, parent_id, tags, completed_at, created_at,
                      updated_at, deleted_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)
             ON CONFLICT(uuid) DO UPDATE SET
               text=excluded.text, completed=excluded.completed,
               priority=excluded.priority, due_date=excluded.due_date,
               recurrence=excluded.recurrence, project_id=excluded.project_id,
               parent_id=excluded.parent_id, tags=excluded.tags,
               completed_at=excluded.completed_at, updated_at=excluded.updated_at,
               deleted_at=excluded.deleted_at",
            params![
                uuid_str,
                task.text,
                task.completed as i64,
                priority_to_str(task.priority),
                task.due_date.map(|d| d.format("%Y-%m-%d").to_string()),
                task.recurrence.map(recurrence_to_str),
                task.project_id.map(|u| u.to_string()),
                task.parent_id.map(|u| u.to_string()),
                JsonVec(task.tags.clone()),
                task.completed_at.map(|d| {
                    let dt: DateTime<Utc> = Utc.from_utc_datetime(&d.and_hms_opt(0, 0, 0).unwrap());
                    to_unix(dt)
                }),
                to_unix(task.created_at),
                opt_to_unix(task.updated_at),
                opt_to_unix(task.deleted_at),
            ],
        )
        .context("Failed to upsert task")?;

        tx.execute(
            "DELETE FROM task_dependencies WHERE task_uuid = ?1",
            params![uuid_str],
        )?;
        for dep_uuid in &task.depends_on {
            tx.execute(
                "INSERT OR IGNORE INTO task_dependencies
                 (task_uuid, depends_on_uuid) VALUES (?1, ?2)",
                params![uuid_str, dep_uuid.to_string()],
            )?;
        }
        tx.commit()
            .context("Failed to commit upsert_task transaction")?;
        Ok(())
    }

    fn upsert_project(&self, project: &Project) -> Result<()> {
        self.conn
            .borrow()
            .execute(
                "INSERT INTO projects (uuid, name, completed, difficulty, tech, due_date,
                          completed_at, created_at, updated_at, deleted_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)
                 ON CONFLICT(uuid) DO UPDATE SET
                   name=excluded.name, completed=excluded.completed,
                   difficulty=excluded.difficulty, tech=excluded.tech,
                   due_date=excluded.due_date, completed_at=excluded.completed_at,
                   updated_at=excluded.updated_at, deleted_at=excluded.deleted_at",
                params![
                    project.uuid.to_string(),
                    project.name,
                    project.completed as i64,
                    difficulty_to_str(project.difficulty),
                    JsonVec(project.tech.clone()),
                    project.due_date.map(|d| d.format("%Y-%m-%d").to_string()),
                    project
                        .completed_at
                        .map(|d| d.format("%Y-%m-%d").to_string()),
                    to_unix(project.created_at),
                    opt_to_unix(project.updated_at),
                    opt_to_unix(project.deleted_at),
                ],
            )
            .context("Failed to upsert project")?;
        Ok(())
    }

    fn upsert_note(&self, note: &Note) -> Result<()> {
        let mut conn = self.conn.borrow_mut();
        let tx = conn
            .transaction()
            .context("Failed to begin upsert_note transaction")?;
        let uuid_str = note.uuid.to_string();
        tx.execute(
            "INSERT INTO notes (uuid, title, body, format, language, project_id,
                      task_id, tags, created_at, updated_at, deleted_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)
             ON CONFLICT(uuid) DO UPDATE SET
               title=excluded.title, body=excluded.body, format=excluded.format,
               language=excluded.language, project_id=excluded.project_id,
               task_id=excluded.task_id, tags=excluded.tags,
               updated_at=excluded.updated_at, deleted_at=excluded.deleted_at",
            params![
                uuid_str,
                note.title,
                note.body,
                format_to_str(note.format),
                note.language,
                note.project_id.map(|u| u.to_string()),
                note.task_id.map(|u| u.to_string()),
                JsonVec(note.tags.clone()),
                to_unix(note.created_at),
                opt_to_unix(note.updated_at),
                opt_to_unix(note.deleted_at),
            ],
        )
        .context("Failed to upsert note")?;
        tx.execute(
            "DELETE FROM note_resources WHERE note_uuid = ?1",
            params![uuid_str],
        )?;
        for resource_id in &note.resource_ids {
            tx.execute(
                "INSERT OR IGNORE INTO note_resources
                 (note_uuid, resource_uuid) VALUES (?1, ?2)",
                params![uuid_str, resource_id.to_string()],
            )?;
        }
        tx.commit()
            .context("Failed to commit upsert_note transaction")?;
        Ok(())
    }

    fn upsert_resource(&self, resource: &Resource) -> Result<()> {
        self.conn
            .borrow()
            .execute(
                "INSERT INTO resources (uuid, title, resource_type, url, description,
                          tags, created_at, updated_at, deleted_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)
                 ON CONFLICT(uuid) DO UPDATE SET
                   title=excluded.title, resource_type=excluded.resource_type,
                   url=excluded.url, description=excluded.description,
                   tags=excluded.tags, updated_at=excluded.updated_at,
                   deleted_at=excluded.deleted_at",
                params![
                    resource.uuid.to_string(),
                    resource.title.clone(),
                    resource.resource_type.map(resource_type_to_str),
                    resource.url,
                    resource.description,
                    JsonVec(resource.tags.clone()),
                    to_unix(resource.created_at),
                    opt_to_unix(resource.updated_at),
                    opt_to_unix(resource.deleted_at),
                ],
            )
            .context("Failed to upsert resource")?;
        Ok(())
    }

    fn search_tasks(
        &self,
        q: &str,
        tags: &[String],
        project_id: Option<Uuid>,
        status: StatusFilter,
    ) -> Result<Vec<Task>> {
        let conn = self.conn.borrow();
        let pattern = format!("%{}%", q.to_lowercase());

        let status_clause = match status {
            StatusFilter::Pending => " AND completed = 0",
            StatusFilter::Done => " AND completed = 1",
            StatusFilter::All => "",
        };
        let proj_clause = if project_id.is_some() {
            " AND project_id = ?2"
        } else {
            ""
        };

        let sql = format!(
            "SELECT * FROM tasks WHERE deleted_at IS NULL AND LOWER(text) LIKE ?1{}{} ORDER BY created_at",
            status_clause, proj_clause
        );

        let mut stmt = conn.prepare(&sql)?;
        let tasks: Vec<Task> = if let Some(uuid) = project_id {
            stmt.query_map(rusqlite::params![pattern, uuid.to_string()], |row| {
                let uuid_str: String = row.get("uuid")?;
                row_to_task(row, &conn, &uuid_str)
            })?
            .collect::<rusqlite::Result<Vec<_>>>()
            .context("Failed to search tasks")?
        } else {
            stmt.query_map(rusqlite::params![pattern], |row| {
                let uuid_str: String = row.get("uuid")?;
                row_to_task(row, &conn, &uuid_str)
            })?
            .collect::<rusqlite::Result<Vec<_>>>()
            .context("Failed to search tasks")?
        };

        Ok(tasks
            .into_iter()
            .filter(|t| tags.is_empty() || tags.iter().all(|tag| t.tags.contains(tag)))
            .collect())
    }

    fn search_notes(
        &self,
        q: &str,
        tags: &[String],
        project_id: Option<Uuid>,
    ) -> Result<Vec<Note>> {
        let conn = self.conn.borrow();
        let pattern = format!("%{}%", q.to_lowercase());
        let proj_clause = if project_id.is_some() {
            " AND project_id = ?2"
        } else {
            ""
        };

        let sql = format!(
            "SELECT * FROM notes WHERE deleted_at IS NULL AND (
               LOWER(COALESCE(title,'')) LIKE ?1 OR
               LOWER(body) LIKE ?1 OR
               LOWER(COALESCE(language,'')) LIKE ?1
             ){} ORDER BY created_at",
            proj_clause
        );

        let mut stmt = conn.prepare(&sql)?;
        let notes: Vec<Note> = if let Some(uuid) = project_id {
            stmt.query_map(rusqlite::params![pattern, uuid.to_string()], |row| {
                row_to_note(row, &conn)
            })?
            .collect::<rusqlite::Result<Vec<_>>>()
            .context("Failed to search notes")?
        } else {
            stmt.query_map(rusqlite::params![pattern], |row| row_to_note(row, &conn))?
                .collect::<rusqlite::Result<Vec<_>>>()
                .context("Failed to search notes")?
        };

        Ok(notes
            .into_iter()
            .filter(|n| tags.is_empty() || tags.iter().all(|tag| n.tags.contains(tag)))
            .collect())
    }

    fn search_projects(&self, q: &str) -> Result<Vec<Project>> {
        let conn = self.conn.borrow();
        let pattern = format!("%{}%", q.to_lowercase());

        let mut stmt = conn.prepare(
            "SELECT * FROM projects WHERE deleted_at IS NULL AND (
               LOWER(name) LIKE ?1 OR LOWER(tech) LIKE ?1
             ) ORDER BY created_at",
        )?;

        stmt.query_map(rusqlite::params![pattern], row_to_project)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .context("Failed to search projects")
    }

    fn search_resources(&self, q: &str, tags: &[String]) -> Result<Vec<Resource>> {
        let conn = self.conn.borrow();
        let pattern = format!("%{}%", q.to_lowercase());

        let mut stmt = conn.prepare(
            "SELECT * FROM resources WHERE deleted_at IS NULL AND (
               LOWER(title) LIKE ?1 OR
               LOWER(COALESCE(url,'')) LIKE ?1 OR
               LOWER(COALESCE(description,'')) LIKE ?1 OR
               EXISTS (
                   SELECT 1 FROM json_each(tags)
                   WHERE LOWER(json_each.value) LIKE ?1
               )
             ) ORDER BY created_at",
        )?;

        let resources: Vec<Resource> = stmt
            .query_map(rusqlite::params![pattern], row_to_resource)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .context("Failed to search resources")?;

        Ok(resources
            .into_iter()
            .filter(|r| tags.is_empty() || tags.iter().all(|tag| r.tags.contains(tag)))
            .collect())
    }

    fn save(&self, tasks: &[Task]) -> Result<()> {
        let mut conn = self.conn.borrow_mut();
        let tx = conn.transaction().context("Failed to begin transaction")?;

        for task in tasks {
            let uuid_str = task.uuid.to_string();
            tx.execute(
                "INSERT INTO tasks (uuid, text, completed, priority, due_date, recurrence,
                          project_id, parent_id, tags, completed_at, created_at,
                          updated_at, deleted_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)
                 ON CONFLICT(uuid) DO UPDATE SET
                   text=excluded.text, completed=excluded.completed,
                   priority=excluded.priority, due_date=excluded.due_date,
                   recurrence=excluded.recurrence, project_id=excluded.project_id,
                   parent_id=excluded.parent_id, tags=excluded.tags,
                   completed_at=excluded.completed_at, updated_at=excluded.updated_at,
                   deleted_at=excluded.deleted_at",
                params![
                    uuid_str,
                    task.text,
                    task.completed as i64,
                    priority_to_str(task.priority),
                    task.due_date.map(|d| d.format("%Y-%m-%d").to_string()),
                    task.recurrence.map(recurrence_to_str),
                    task.project_id.map(|u| u.to_string()),
                    task.parent_id.map(|u| u.to_string()),
                    JsonVec(task.tags.clone()),
                    task.completed_at.map(|d| {
                        let dt: DateTime<Utc> =
                            Utc.from_utc_datetime(&d.and_hms_opt(0, 0, 0).unwrap());
                        to_unix(dt)
                    }),
                    to_unix(task.created_at),
                    opt_to_unix(task.updated_at),
                    opt_to_unix(task.deleted_at),
                ],
            )?;

            tx.execute(
                "DELETE FROM task_dependencies WHERE task_uuid = ?1",
                params![uuid_str],
            )?;
            for dep_uuid in &task.depends_on {
                tx.execute(
                    "INSERT OR IGNORE INTO task_dependencies
                     (task_uuid, depends_on_uuid) VALUES (?1, ?2)",
                    params![uuid_str, dep_uuid.to_string()],
                )?;
            }
        }

        tx.commit().context("Failed to commit tasks transaction")?;
        Ok(())
    }

    fn load_projects(&self) -> Result<Vec<Project>> {
        let conn = self.conn.borrow();
        let mut stmt = conn.prepare("SELECT * FROM projects ORDER BY created_at")?;
        let projects = stmt
            .query_map([], row_to_project)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .context("Failed to load projects")?;
        Ok(projects)
    }

    fn save_projects(&self, projects: &[Project]) -> Result<()> {
        let mut conn = self.conn.borrow_mut();
        let tx = conn.transaction().context("Failed to begin transaction")?;

        for project in projects {
            tx.execute(
                "INSERT INTO projects (uuid, name, completed, difficulty, tech, due_date,
                          completed_at, created_at, updated_at, deleted_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)
                 ON CONFLICT(uuid) DO UPDATE SET
                   name=excluded.name, completed=excluded.completed,
                   difficulty=excluded.difficulty, tech=excluded.tech,
                   due_date=excluded.due_date, completed_at=excluded.completed_at,
                   updated_at=excluded.updated_at, deleted_at=excluded.deleted_at",
                params![
                    project.uuid.to_string(),
                    project.name,
                    project.completed as i64,
                    difficulty_to_str(project.difficulty),
                    JsonVec(project.tech.clone()),
                    project.due_date.map(|d| d.format("%Y-%m-%d").to_string()),
                    project
                        .completed_at
                        .map(|d| d.format("%Y-%m-%d").to_string()),
                    to_unix(project.created_at),
                    opt_to_unix(project.updated_at),
                    opt_to_unix(project.deleted_at),
                ],
            )?;
        }

        tx.commit()
            .context("Failed to commit projects transaction")?;
        Ok(())
    }

    fn load_notes(&self) -> Result<Vec<Note>> {
        let conn = self.conn.borrow();
        let mut stmt = conn.prepare("SELECT * FROM notes ORDER BY created_at")?;
        let notes = stmt
            .query_map([], |row| row_to_note(row, &conn))?
            .collect::<rusqlite::Result<Vec<_>>>()
            .context("Failed to load notes")?;
        Ok(notes)
    }

    fn save_notes(&self, notes: &[Note]) -> Result<()> {
        let mut conn = self.conn.borrow_mut();
        let tx = conn.transaction().context("Failed to begin transaction")?;

        for note in notes {
            let uuid_str = note.uuid.to_string();
            tx.execute(
                "INSERT INTO notes (uuid, title, body, format, language, project_id,
                          task_id, tags, created_at, updated_at, deleted_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)
                 ON CONFLICT(uuid) DO UPDATE SET
                   title=excluded.title, body=excluded.body, format=excluded.format,
                   language=excluded.language, project_id=excluded.project_id,
                   task_id=excluded.task_id, tags=excluded.tags,
                   updated_at=excluded.updated_at, deleted_at=excluded.deleted_at",
                params![
                    uuid_str,
                    note.title,
                    note.body,
                    format_to_str(note.format),
                    note.language,
                    note.project_id.map(|u| u.to_string()),
                    note.task_id.map(|u| u.to_string()),
                    JsonVec(note.tags.clone()),
                    to_unix(note.created_at),
                    opt_to_unix(note.updated_at),
                    opt_to_unix(note.deleted_at),
                ],
            )?;

            tx.execute(
                "DELETE FROM note_resources WHERE note_uuid = ?1",
                params![uuid_str],
            )?;
            for resource_id in &note.resource_ids {
                tx.execute(
                    "INSERT OR IGNORE INTO note_resources
                     (note_uuid, resource_uuid) VALUES (?1, ?2)",
                    params![uuid_str, resource_id.to_string()],
                )?;
            }
        }

        tx.commit().context("Failed to commit notes transaction")?;
        Ok(())
    }

    fn load_resources(&self) -> Result<Vec<Resource>> {
        let conn = self.conn.borrow();
        let mut stmt = conn.prepare("SELECT * FROM resources ORDER BY created_at")?;
        let resources = stmt
            .query_map([], row_to_resource)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .context("Failed to load resources")?;
        Ok(resources)
    }

    fn save_resources(&self, resources: &[Resource]) -> Result<()> {
        let mut conn = self.conn.borrow_mut();
        let tx = conn.transaction().context("Failed to begin transaction")?;

        for resource in resources {
            tx.execute(
                "INSERT INTO resources (uuid, title, resource_type, url, description,
                          tags, created_at, updated_at, deleted_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)
                 ON CONFLICT(uuid) DO UPDATE SET
                   title=excluded.title, resource_type=excluded.resource_type,
                   url=excluded.url, description=excluded.description,
                   tags=excluded.tags, updated_at=excluded.updated_at,
                   deleted_at=excluded.deleted_at",
                params![
                    resource.uuid.to_string(),
                    resource.resource_type.map(resource_type_to_str),
                    resource.url,
                    resource.description,
                    JsonVec(resource.tags.clone()),
                    to_unix(resource.created_at),
                    opt_to_unix(resource.updated_at),
                    opt_to_unix(resource.deleted_at),
                ],
            )?;
        }

        tx.commit()
            .context("Failed to commit resources transaction")?;
        Ok(())
    }

    fn record_event(
        &self,
        entity_type: EntityType,
        entity_uuid: Uuid,
        event_type: EventType,
    ) -> Result<()> {
        self.conn
            .borrow()
            .execute(
                "INSERT INTO events (entity_type, entity_uuid, event_type, occurred_at)
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    entity_type.as_str(),
                    entity_uuid.to_string(),
                    event_type.as_str(),
                    to_unix(Utc::now()),
                ],
            )
            .context("Failed to record event")?;
        Ok(())
    }

    fn clear_events(&self, older_than_days: Option<u32>) -> Result<usize> {
        let conn = self.conn.borrow();
        let deleted = match older_than_days {
            None => conn
                .execute("DELETE FROM events", [])
                .context("Failed to clear all events")?,
            Some(days) => {
                let cutoff = to_unix(Utc::now() - chrono::Duration::days(days as i64));
                conn.execute("DELETE FROM events WHERE occurred_at < ?1", params![cutoff])
                    .context("Failed to clear old events")?
            }
        };
        Ok(deleted)
    }

    fn load_event_stats(&self, months: usize) -> Result<Vec<EventStat>> {
        let conn = self.conn.borrow();
        let now = chrono::Local::now();

        // Build cutoff date for the oldest month we want
        let cutoff = {
            let mut y = now.year();
            let mut m = now.month() as i32 - (months as i32 - 1);
            while m <= 0 {
                m += 12;
                y -= 1;
            }
            chrono::NaiveDate::from_ymd_opt(y, m as u32, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
        };
        let cutoff_ts = Utc.from_utc_datetime(&cutoff).timestamp();

        // Pre-populate all months so empty ones still appear in the chart
        let mut map: std::collections::BTreeMap<(i32, u32), EventStat> =
            std::collections::BTreeMap::new();
        let mut cursor = chrono::NaiveDate::from_ymd_opt(cutoff.year(), cutoff.month(), 1).unwrap();
        let end = chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap();
        while cursor <= end {
            map.entry((cursor.year(), cursor.month()))
                .or_insert_with(|| EventStat {
                    year: cursor.year(),
                    month: cursor.month(),
                    ..Default::default()
                });
            let next_month = cursor.month() % 12 + 1;
            let next_year = if cursor.month() == 12 {
                cursor.year() + 1
            } else {
                cursor.year()
            };
            cursor = chrono::NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap();
        }

        let mut stmt = conn.prepare(
            "SELECT event_type, occurred_at
             FROM events
             WHERE entity_type = 'task'
               AND occurred_at >= ?1
             ORDER BY occurred_at",
        )?;

        let rows = stmt
            .query_map(params![cutoff_ts], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()
            .context("Failed to load event stats")?;

        for (event_type, occurred_at) in rows {
            let local = from_unix(occurred_at).with_timezone(&chrono::Local);
            let key = (local.year(), local.month());
            let stat = map.entry(key).or_insert_with(|| EventStat {
                year: key.0,
                month: key.1,
                ..Default::default()
            });
            match event_type.as_str() {
                "created" => stat.created += 1,
                "completed" => stat.completed += 1,
                "deleted" | "purged" => stat.deleted += 1,
                _ => {}
            }
        }

        Ok(map.into_values().collect())
    }

    fn location(&self) -> String {
        self.path.display().to_string()
    }

    fn delete_tasks(&self, uuids: &[Uuid]) -> Result<()> {
        let mut conn = self.conn.borrow_mut();
        let tx = conn.transaction().context("Failed to begin transaction")?;
        for uuid in uuids {
            let s = uuid.to_string();
            tx.execute(
                "DELETE FROM task_dependencies WHERE task_uuid = ?1 OR depends_on_uuid = ?1",
                params![s],
            )?;
            tx.execute("DELETE FROM tasks WHERE uuid = ?1", params![s])?;
        }
        tx.commit()
            .context("Failed to commit delete_tasks transaction")?;
        Ok(())
    }

    fn delete_projects(&self, uuids: &[Uuid]) -> Result<()> {
        let mut conn = self.conn.borrow_mut();
        let tx = conn.transaction().context("Failed to begin transaction")?;
        for uuid in uuids {
            tx.execute(
                "DELETE FROM projects WHERE uuid = ?1",
                params![uuid.to_string()],
            )?;
        }
        tx.commit()
            .context("Failed to commit delete_projects transaction")?;
        Ok(())
    }

    fn delete_notes(&self, uuids: &[Uuid]) -> Result<()> {
        let mut conn = self.conn.borrow_mut();
        let tx = conn.transaction().context("Failed to begin transaction")?;
        for uuid in uuids {
            let s = uuid.to_string();
            tx.execute(
                "DELETE FROM note_resources WHERE note_uuid = ?1",
                params![s],
            )?;
            tx.execute("DELETE FROM notes WHERE uuid = ?1", params![s])?;
        }
        tx.commit()
            .context("Failed to commit delete_notes transaction")?;
        Ok(())
    }

    fn delete_resources(&self, uuids: &[Uuid]) -> Result<()> {
        let mut conn = self.conn.borrow_mut();
        let tx = conn.transaction().context("Failed to begin transaction")?;
        for uuid in uuids {
            let s = uuid.to_string();
            tx.execute(
                "DELETE FROM note_resources WHERE resource_uuid = ?1",
                params![s],
            )?;
            tx.execute("DELETE FROM resources WHERE uuid = ?1", params![s])?;
        }
        tx.commit()
            .context("Failed to commit delete_resources transaction")?;
        Ok(())
    }
}

// ── enum helpers ──────────────────────────────────────────────────────────────

fn priority_to_str(p: Priority) -> &'static str {
    match p {
        Priority::High => "high",
        Priority::Medium => "medium",
        Priority::Low => "low",
    }
}

fn recurrence_to_str(r: Recurrence) -> &'static str {
    match r {
        Recurrence::Daily => "daily",
        Recurrence::Weekly => "weekly",
        Recurrence::Monthly => "monthly",
    }
}

fn difficulty_to_str(d: Difficulty) -> &'static str {
    match d {
        Difficulty::Easy => "easy",
        Difficulty::Medium => "medium",
        Difficulty::Hard => "hard",
    }
}

fn format_to_str(f: NoteFormat) -> &'static str {
    match f {
        NoteFormat::Plain => "plain",
        NoteFormat::Markdown => "markdown",
    }
}

fn resource_type_to_str(rt: ResourceType) -> &'static str {
    match rt {
        ResourceType::Docs => "docs",
        ResourceType::Article => "article",
        ResourceType::Video => "video",
        ResourceType::Repo => "repo",
        ResourceType::Crate => "crate",
        ResourceType::Book => "book",
        ResourceType::Spec => "spec",
        ResourceType::Tool => "tool",
    }
}

// ── path helper ───────────────────────────────────────────────────────────────

pub fn get_db_path() -> Result<PathBuf> {
    let data_dir = if let Ok(dir) = std::env::var("RUSTODO_DATA_DIR") {
        PathBuf::from(dir)
    } else {
        let proj_dirs =
            ProjectDirs::from("", "", "rustodo").context("Could not determine data directory")?;
        proj_dirs.data_dir().to_path_buf()
    };
    std::fs::create_dir_all(&data_dir).context("Failed to create data directory")?;
    Ok(data_dir.join("rustodo.db"))
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Priority;
    use tempfile::TempDir;

    fn make_storage() -> (SqliteStorage, TempDir) {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test.db");
        let storage = SqliteStorage::with_path(path).unwrap();
        (storage, tmp)
    }

    #[test]
    fn test_save_and_load_tasks() {
        let (storage, _tmp) = make_storage();
        let task = Task::new(
            "Buy milk".into(),
            Priority::Medium,
            vec![],
            None,
            None,
            None,
        );
        storage.save(&[task]).unwrap();
        assert_eq!(storage.load().unwrap()[0].text, "Buy milk");
    }

    #[test]
    fn test_record_and_load_events() {
        let (storage, _tmp) = make_storage();
        let task = Task::new("T".into(), Priority::Medium, vec![], None, None, None);
        let uuid = task.uuid;
        storage.save(&[task]).unwrap();
        storage
            .record_event(EntityType::Task, uuid, EventType::Created)
            .unwrap();
        storage
            .record_event(EntityType::Task, uuid, EventType::Completed)
            .unwrap();
        let stats = storage.load_event_stats(1).unwrap();
        let m = stats.last().unwrap();
        assert_eq!(m.created, 1);
        assert_eq!(m.completed, 1);
        assert_eq!(m.deleted, 0);
    }

    #[test]
    fn test_purged_events_survive_physical_delete() {
        let (storage, _tmp) = make_storage();
        let task = Task::new("T".into(), Priority::Medium, vec![], None, None, None);
        let uuid = task.uuid;
        storage.save(&[task]).unwrap();
        storage
            .record_event(EntityType::Task, uuid, EventType::Created)
            .unwrap();
        storage
            .record_event(EntityType::Task, uuid, EventType::Purged)
            .unwrap();
        storage.delete_tasks(&[uuid]).unwrap();
        // Row is gone but the event log must still count it
        let stats = storage.load_event_stats(1).unwrap();
        let m = stats.last().unwrap();
        assert_eq!(m.created, 1);
        assert_eq!(m.deleted, 1);
    }

    #[test]
    fn test_event_stats_all_months_present() {
        let (storage, _tmp) = make_storage();
        let stats = storage.load_event_stats(6).unwrap();
        assert_eq!(stats.len(), 6);
    }

    #[test]
    fn test_upsert_does_not_duplicate() {
        let (storage, _tmp) = make_storage();
        let task = Task::new("T".into(), Priority::Medium, vec![], None, None, None);
        storage.save(std::slice::from_ref(&task)).unwrap();
        storage.save(std::slice::from_ref(&task)).unwrap();
        assert_eq!(storage.load().unwrap().len(), 1);
    }

    #[test]
    fn test_soft_delete_preserved() {
        let (storage, _tmp) = make_storage();
        let mut task = Task::new("T".into(), Priority::Medium, vec![], None, None, None);
        task.soft_delete();
        storage.save(&[task]).unwrap();
        assert!(storage.load().unwrap()[0].is_deleted());
    }
}
