use anyhow::Result;
use chrono::NaiveDate;
use colored::Colorize;

use crate::error::TodoError;
use crate::models::{Priority, Recurrence, Task};
use crate::storage::Storage;
use crate::validation;

#[allow(clippy::too_many_arguments)]
pub fn execute(
    storage: &impl Storage,
    text: String,
    priority: Priority,
    tags: Vec<String>,
    project: Option<String>,
    due: Option<NaiveDate>,
    recur: Option<Recurrence>,
    depends_on: Vec<usize>,
) -> Result<()> {
    validation::validate_task_text(&text)?;
    validation::validate_tags(&tags)?;
    if let Some(ref p) = project {
        validation::validate_project_name(p)?;
    }
    validation::validate_due_date(due, false)?;
    validation::validate_recurrence(recur, due)?;

    let mut tasks = storage.load()?;

    let new_id = tasks.len() + 1;

    // Validate dependencies
    for &dep_id in &depends_on {
        if dep_id == new_id {
            return Err(TodoError::SelfDependency { task_id: new_id }.into());
        }
        validation::validate_task_id(dep_id, tasks.len())?;
        // No cycle detection needed for new tasks - they have no dependents yet.
    }

    let mut task = Task::new(text, priority, tags, project, due, recur);
    task.depends_on = depends_on;
    tasks.push(task);

    let id = tasks.len();
    storage.save(&tasks)?;

    let ok = "✓".green();

    if let Some(pattern) = recur {
        println!("{} Added task #{} with {} recurrence", ok, id, pattern);
    } else {
        println!("{} Added task #{}", ok, id);
    }

    Ok(())
}
