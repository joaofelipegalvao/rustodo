use chrono::Local;
use colored::{ColoredString, Colorize};

use crate::models::Task;

/// Generates a human-readable due date description.
///
/// # Returns
///
/// A string like "in 3 days", "due today", or "late 2 days"
pub fn get_due_text(task: &Task) -> String {
    let Some(due) = task.due_date else {
        return String::new();
    };

    let today = Local::now().naive_local().date();
    let days = (due - today).num_days();

    match days {
        d if d < 0 => {
            let abs_d = d.abs();
            format!("late {} day{}", abs_d, if abs_d == 1 { "" } else { "s" })
        }
        0 => "due today".to_string(),
        d => format!("in {} day{}", d, if d == 1 { "" } else { "s" }),
    }
}

/// Returns a colored version of the due date text based on urgency.
///
/// Color coding:
/// - Red (bold): Overdue
/// - Yellow (bold): Due today
/// - Yellow: Due within 7 days
/// - Cyan: Due later
/// - Dimmed: Completed tasks
pub fn get_due_colored(task: &Task, text: &str) -> ColoredString {
    if text.is_empty() {
        return "".normal();
    }

    if task.completed {
        return text.dimmed();
    }

    if let Some(due) = task.due_date {
        let today = Local::now().naive_local().date();
        let days_until = (due - today).num_days();

        if days_until < 0 {
            text.red().bold()
        } else if days_until == 0 {
            text.yellow().bold()
        } else if days_until <= 7 {
            text.yellow()
        } else {
            text.cyan()
        }
    } else {
        text.normal()
    }
}

/// Renders a checkbox for task completion status.
pub fn render_checkbox(completed: bool) -> ColoredString {
    if completed {
        "[x]".green()
    } else {
        "[ ]".bright_white()
    }
}
