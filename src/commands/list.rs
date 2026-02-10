use anyhow::Result;

use crate::display::display_lists;
use crate::error::TodoError;
use crate::models::{DueFilter, Priority, SortBy, StatusFilter};
use crate::storage::load_tasks;

#[allow(clippy::too_many_arguments)]
pub fn execute(
    status: StatusFilter,
    priority: Option<Priority>,
    due: Option<DueFilter>,
    sort: Option<SortBy>,
    tag: Option<String>,
) -> Result<()> {
    let all_tasks = load_tasks()?;

    // Create indexed view of tasks (1-based numbering)
    let mut indexed_tasks: Vec<(usize, &_)> = all_tasks
        .iter()
        .enumerate()
        .map(|(i, task)| (i + 1, task))
        .collect();

    // Apply filters sequentially
    indexed_tasks.retain(|(_, t)| t.matches_status(status));

    if let Some(pri) = priority {
        indexed_tasks.retain(|(_, t)| t.priority == pri);
    }

    if let Some(due_filter) = due {
        indexed_tasks.retain(|(_, t)| t.matches_due_filter(due_filter));
    }

    if let Some(tag_name) = &tag {
        let count_before = indexed_tasks.len();
        indexed_tasks.retain(|(_, t)| t.tags.contains(tag_name));

        if indexed_tasks.is_empty() && count_before > 0 {
            return Err(TodoError::TagNotFound(tag_name.to_owned()).into());
        }
    }

    if indexed_tasks.is_empty() {
        return Err(TodoError::NoTasksFound.into());
    }

    // Apply sorting if requested
    if let Some(sort_by) = sort {
        match sort_by {
            SortBy::Priority => {
                indexed_tasks.sort_by(|(_, a), (_, b)| a.priority.order().cmp(&b.priority.order()));
            }
            SortBy::Due => {
                indexed_tasks.sort_by(|(_, a), (_, b)| match (a.due_date, b.due_date) {
                    (Some(date_a), Some(date_b)) => date_a.cmp(&date_b),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                });
            }
            SortBy::Created => {
                indexed_tasks.sort_by(|(_, a), (_, b)| a.created_at.cmp(&b.created_at));
            }
        }
    }

    // Determine appropriate title based on active filters
    let title = match (status, priority, due) {
        (StatusFilter::Pending, Some(Priority::High), None) => "High priority pending tasks",
        (StatusFilter::Pending, Some(Priority::Medium), None) => "Medium priority pending tasks",
        (StatusFilter::Pending, Some(Priority::Low), None) => "Low priority pending tasks",
        (StatusFilter::Pending, None, Some(DueFilter::Overdue)) => "Pending overdue tasks",
        (StatusFilter::Pending, None, Some(DueFilter::Soon)) => "Pending tasks due soon",
        (StatusFilter::Pending, None, None) => "Pending tasks",
        (StatusFilter::Done, _, _) => "Completed tasks",
        (StatusFilter::All, Some(Priority::High), _) => "High priority tasks",
        (StatusFilter::All, Some(Priority::Medium), _) => "Medium priority tasks",
        (StatusFilter::All, Some(Priority::Low), _) => "Low priority tasks",
        (StatusFilter::All, None, Some(DueFilter::Overdue)) => "Overdue tasks",
        (StatusFilter::All, None, Some(DueFilter::Soon)) => "Tasks due soon",
        (StatusFilter::All, None, Some(DueFilter::WithDue)) => "Tasks with due date",
        (StatusFilter::All, None, Some(DueFilter::NoDue)) => "Tasks without due date",
        _ => "Tasks",
    };

    display_lists(&indexed_tasks, title);
    Ok(())
}
