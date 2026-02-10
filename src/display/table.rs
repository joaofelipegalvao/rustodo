use colored::Colorize;

use crate::models::Task;

use super::formatting::{get_due_colored, get_due_text, render_checkbox};

const ID_WIDTH: usize = 4;
const PRIORITY_WIDTH: usize = 1;
const STATUS_WIDTH: usize = 3; // [x] or [ ]

pub struct TableLayout {
    id: usize,
    priority: usize,
    status: usize,
    task: usize,
    tags: usize,
    due: usize,
}

impl TableLayout {
    pub fn new(tasks: &[(usize, &Task)]) -> Self {
        let (task, tags, due) = calculate_column_widths(tasks);

        Self {
            id: ID_WIDTH,
            priority: PRIORITY_WIDTH,
            status: STATUS_WIDTH,
            task,
            tags,
            due,
        }
    }

    pub fn total_width(&self) -> usize {
        self.id + self.priority + self.status + self.task + self.tags + self.due + 10
    }

    /// Displays a single task in tabular format.
    pub fn display_task(&self, number: usize, task: &Task) {
        let checkbox = render_checkbox(task.completed);
        let letter = task.priority.letter();

        let task_text = if task.text.len() > self.task {
            format!("{}...", &task.text[..self.task - 3])
        } else {
            task.text.to_owned()
        };

        let tags_str = if task.tags.is_empty() {
            String::new()
        } else {
            let joined = task.tags.join(", ");
            if joined.len() > self.tags {
                format!("{}...", &joined[..self.tags - 3])
            } else {
                joined
            }
        };

        let due_text = get_due_text(task);
        let due_colored = get_due_colored(task, &due_text);

        if task.completed {
            print!(
                "{:>id_width$} ",
                number.to_string().dimmed(),
                id_width = self.id,
            );
            print!(
                " {:<priority_width$} ",
                letter,
                priority_width = self.priority,
            );
            print!(" {:<status_width$} ", checkbox, status_width = self.status,);
            print!("{:<task_width$}", task_text.green(), task_width = self.task,);
            print!(
                "  {:<tags_width$}",
                tags_str.dimmed(),
                tags_width = self.tags,
            );
            println!("  {}", due_colored);
        } else {
            print!(
                "{:>id_width$} ",
                number.to_string().dimmed(),
                id_width = self.id,
            );
            print!(
                " {:<priority_width$} ",
                letter,
                priority_width = self.priority,
            );
            print!(" {:<status_width$} ", checkbox, status_width = self.status,);
            print!(
                "{:<task_width$}",
                task_text.bright_white(),
                task_width = self.task,
            );
            print!("  {:<tags_width$}", tags_str.cyan(), tags_width = self.tags,);
            println!("  {}", due_colored);
        }
    }

    /// Displays the table header.
    pub fn display_header(&self) {
        print!("{:>id_width$} ", "ID".dimmed(), id_width = self.id,);
        print!(
            " {:<priority_width$} ",
            "P".dimmed(),
            priority_width = self.priority,
        );
        print!(
            " {:<status_width$} ",
            " S".dimmed(),
            status_width = self.status,
        );
        print!("{:<task_width$}", "Task".dimmed(), task_width = self.task,);
        print!("  {:<tags_width$}", "Tags".dimmed(), tags_width = self.tags,);
        println!("  {}", "Due".dimmed());
    }

    /// Displays a separator line.
    pub fn display_separator(&self) {
        println!("{}", "─".repeat(self.total_width()).dimmed());
    }
}

/// Calculates optimal column widths for tabular display.
///
/// Analyzes all tasks to determine the maximum width needed for each column,
/// with upper limits to prevent excessively wide output.
///
/// # Returns
///
/// A tuple of (task_width, tags_width, due_width)
fn calculate_column_widths(tasks: &[(usize, &Task)]) -> (usize, usize, usize) {
    let mut max_task_len = 10;
    let mut max_tags_len = 4;
    let mut max_due_len = 3;

    for (_, task) in tasks {
        max_task_len = max_task_len.max(task.text.len());

        if !task.tags.is_empty() {
            let tags_str = task.tags.join(", ");
            max_tags_len = max_tags_len.max(tags_str.len());
        }

        let due_text = get_due_text(task);
        if !due_text.is_empty() {
            max_due_len = max_due_len.max(due_text.len());
        }
    }

    // Cap maximum widths to keep output reasonable
    max_task_len = max_task_len.min(40);
    max_tags_len = max_tags_len.min(20);
    max_due_len = max_due_len.min(20);

    (max_task_len, max_tags_len, max_due_len)
}

/// Displays a list of tasks in a formatted table with statistics.
///
/// # Arguments
///
/// * `tasks` - List of (task_number, task) tuples to display
/// * `title` - Title to show above the table
pub fn display_lists(tasks: &[(usize, &Task)], title: &str) {
    println!("\n{}:\n", title);

    let layout = TableLayout::new(tasks);

    layout.display_header();
    layout.display_separator();

    let mut completed = 0;
    let total = tasks.len();

    for (number, task) in tasks {
        layout.display_task(*number, task);

        if task.completed {
            completed += 1;
        }
    }

    layout.display_separator();

    let percentage = if total > 0 {
        (completed as f32 / total as f32 * 100.0) as u32
    } else {
        0
    };

    let stats = format!("{} of {} completed ({}%)", completed, total, percentage);

    if percentage == 100 {
        println!("{}", stats.green().bold());
    } else if percentage >= 50 {
        println!("{}", stats.yellow());
    } else {
        println!("{}", stats.red());
    }

    println!();
}
