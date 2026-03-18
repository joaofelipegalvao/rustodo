//! In-memory storage implementation for testing.

use anyhow::Result;
use chrono::TimeZone;
use std::cell::RefCell;
use uuid::Uuid;

use super::{EntityType, EventStat, EventType, Storage};
use crate::models::{Note, Project, Resource, Task};

#[derive(Debug, Default)]
struct EventRow {
    entity_type: String,
    #[allow(dead_code)]
    entity_uuid: Uuid,
    event_type: String,
    occurred_at: i64,
}

/// In-memory storage implementation.
#[derive(Default)]
pub struct InMemoryStorage {
    tasks: RefCell<Vec<Task>>,
    projects: RefCell<Vec<Project>>,
    notes: RefCell<Vec<Note>>,
    resources: RefCell<Vec<Resource>>,
    events: RefCell<Vec<EventRow>>,
}

#[allow(dead_code)]
impl InMemoryStorage {
    pub fn with_tasks(tasks: Vec<Task>) -> Self {
        Self {
            tasks: RefCell::new(tasks),
            ..Default::default()
        }
    }

    pub fn len(&self) -> usize {
        self.tasks.borrow().len()
    }
    pub fn is_empty(&self) -> bool {
        self.tasks.borrow().is_empty()
    }
}

impl Storage for InMemoryStorage {
    fn load(&self) -> Result<Vec<Task>> {
        Ok(self.tasks.borrow().clone())
    }

    fn upsert_task(&self, task: &Task) -> Result<()> {
        let mut tasks = self.tasks.borrow_mut();
        if let Some(existing) = tasks.iter_mut().find(|t| t.uuid == task.uuid) {
            *existing = task.clone();
        } else {
            tasks.push(task.clone());
        }
        Ok(())
    }

    fn upsert_project(&self, project: &Project) -> Result<()> {
        let mut projects = self.projects.borrow_mut();
        if let Some(existing) = projects.iter_mut().find(|p| p.uuid == project.uuid) {
            *existing = project.clone();
        } else {
            projects.push(project.clone());
        }
        Ok(())
    }

    fn save(&self, tasks: &[Task]) -> Result<()> {
        *self.tasks.borrow_mut() = tasks.to_vec();
        Ok(())
    }

    fn delete_tasks(&self, uuids: &[Uuid]) -> Result<()> {
        self.tasks.borrow_mut().retain(|t| !uuids.contains(&t.uuid));
        Ok(())
    }

    fn load_projects(&self) -> Result<Vec<Project>> {
        Ok(self.projects.borrow().clone())
    }

    fn save_projects(&self, projects: &[Project]) -> Result<()> {
        *self.projects.borrow_mut() = projects.to_vec();
        Ok(())
    }

    fn delete_projects(&self, uuids: &[Uuid]) -> Result<()> {
        self.projects
            .borrow_mut()
            .retain(|p| !uuids.contains(&p.uuid));
        Ok(())
    }

    fn load_notes(&self) -> Result<Vec<Note>> {
        Ok(self.notes.borrow().clone())
    }

    fn save_notes(&self, notes: &[Note]) -> Result<()> {
        *self.notes.borrow_mut() = notes.to_vec();
        Ok(())
    }

    fn delete_notes(&self, uuids: &[Uuid]) -> Result<()> {
        self.notes.borrow_mut().retain(|n| !uuids.contains(&n.uuid));
        Ok(())
    }

    fn load_resources(&self) -> Result<Vec<Resource>> {
        Ok(self.resources.borrow().clone())
    }

    fn save_resources(&self, resources: &[Resource]) -> Result<()> {
        *self.resources.borrow_mut() = resources.to_vec();
        Ok(())
    }

    fn delete_resources(&self, uuids: &[Uuid]) -> Result<()> {
        self.resources
            .borrow_mut()
            .retain(|r| !uuids.contains(&r.uuid));
        Ok(())
    }

    fn record_event(
        &self,
        entity_type: EntityType,
        entity_uuid: Uuid,
        event_type: EventType,
    ) -> Result<()> {
        self.events.borrow_mut().push(EventRow {
            entity_type: entity_type.as_str().to_string(),
            entity_uuid,
            event_type: event_type.as_str().to_string(),
            occurred_at: chrono::Utc::now().timestamp(),
        });
        Ok(())
    }

    fn clear_events(&self, older_than_days: Option<u32>) -> Result<usize> {
        let mut events = self.events.borrow_mut();
        let before = events.len();
        match older_than_days {
            None => events.clear(),
            Some(days) => {
                let cutoff = chrono::Utc::now().timestamp() - (days as i64 * 86400);
                events.retain(|e| e.occurred_at >= cutoff);
            }
        }
        Ok(before - events.len())
    }

    fn load_event_stats(&self, months: usize) -> Result<Vec<EventStat>> {
        use chrono::Datelike;
        let now = chrono::Local::now();

        // Pre-populate all months
        let mut map: std::collections::BTreeMap<(i32, u32), EventStat> =
            std::collections::BTreeMap::new();
        let cutoff = {
            let mut y = now.year();
            let mut m = now.month() as i32 - (months as i32 - 1);
            while m <= 0 {
                m += 12;
                y -= 1;
            }
            chrono::NaiveDate::from_ymd_opt(y, m as u32, 1).unwrap()
        };
        let mut cursor = cutoff;
        let end = chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap();
        while cursor <= end {
            map.entry((cursor.year(), cursor.month()))
                .or_insert_with(|| EventStat {
                    year: cursor.year(),
                    month: cursor.month(),
                    ..Default::default()
                });
            let nm = cursor.month() % 12 + 1;
            let ny = if cursor.month() == 12 {
                cursor.year() + 1
            } else {
                cursor.year()
            };
            cursor = chrono::NaiveDate::from_ymd_opt(ny, nm, 1).unwrap();
        }

        let cutoff_ts = chrono::Utc
            .from_utc_datetime(&cutoff.and_hms_opt(0, 0, 0).unwrap())
            .timestamp();

        for ev in self.events.borrow().iter() {
            if ev.entity_type != "task" || ev.occurred_at < cutoff_ts {
                continue;
            }
            let local = chrono::DateTime::<chrono::Utc>::from_timestamp(ev.occurred_at, 0)
                .unwrap()
                .with_timezone(&chrono::Local);
            let key = (local.year(), local.month());
            let stat = map.entry(key).or_insert_with(|| EventStat {
                year: key.0,
                month: key.1,
                ..Default::default()
            });
            match ev.event_type.as_str() {
                "created" => stat.created += 1,
                "completed" => stat.completed += 1,
                "deleted" | "purged" => stat.deleted += 1,
                _ => {}
            }
        }

        Ok(map.into_values().collect())
    }

    fn location(&self) -> String {
        "memory".to_string()
    }
}
