use colored::Colorize;

use crate::models::{Recurrence, Task};

use super::formatting::{get_due_colored, get_due_text, render_checkbox};

const ID_WIDTH: usize = 4;
const PRIORITY_WIDTH: usize = 1;
const STATUS_WIDTH: usize = 3; // [x] or [ ]
const RECUR_WIDTH: usize = 1; // D, W, M or space

pub struct TableLayout {
    id: usize,
    priority: usize,
    status: usize,
    recur: usize,
    task: usize,
    tags: usize,
    due: usize,
    // Conditional column flags
    show_recur: bool,
    show_tags: bool,
    show_due: bool,
}

impl TableLayout {
    pub fn new(tasks: &[(usize, &Task)]) -> Self {
        let (task, tags, due) = calculate_column_widths(tasks);

        // Determine which columns to show
        let show_recur = tasks.iter().any(|(_, t)| t.recurrence.is_some());
        let show_tags = tasks.iter().any(|(_, t)| !t.tags.is_empty());
        let show_due = tasks.iter().any(|(_, t)| t.due_date.is_some());

        Self {
            id: ID_WIDTH,
            priority: PRIORITY_WIDTH,
            status: STATUS_WIDTH,
            recur: RECUR_WIDTH,
            task,
            tags,
            due,
            show_recur,
            show_tags,
            show_due,
        }
    }

    pub fn total_width(&self) -> usize {
        let mut width = self.id + self.priority + self.status + self.task + 8; // Base spacing

        if self.show_recur {
            width += self.recur + 2; // R column + spacing
        }
        if self.show_tags {
            width += self.tags + 2; // Tags column + spacing
        }
        if self.show_due {
            width += self.due + 2; // Due column + spacing
        }

        width
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

        // Render recurrence indicator (single letter, colored)
        let recur_indicator = if let Some(recurrence) = task.recurrence {
            match recurrence {
                Recurrence::Daily => "D".cyan(),
                Recurrence::Weekly => "W".cyan(),
                Recurrence::Monthly => "M".cyan(),
            }
        } else {
            " ".normal()
        };

        if task.completed {
            // ID, Priority, Status
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

            // Recurrence (conditional)
            if self.show_recur {
                print!(
                    " {:<recur_width$}  ",
                    recur_indicator,
                    recur_width = self.recur,
                );
            }

            // Task text
            print!("{:<task_width$}", task_text.green(), task_width = self.task,);

            // Tags (conditional)
            if self.show_tags {
                print!(
                    "  {:<tags_width$}",
                    tags_str.dimmed(),
                    tags_width = self.tags,
                );
            }

            // Due date (conditional)
            if self.show_due {
                print!("  {}", due_colored);
            }

            println!();
        } else {
            // ID, Priority, Status
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

            // Recurrence (conditional)
            if self.show_recur {
                print!(
                    " {:<recur_width$}  ",
                    recur_indicator,
                    recur_width = self.recur,
                );
            }

            // Task text
            print!(
                "{:<task_width$}",
                task_text.bright_white(),
                task_width = self.task,
            );

            // Tags (conditional)
            if self.show_tags {
                print!("  {:<tags_width$}", tags_str.cyan(), tags_width = self.tags,);
            }

            // Due date (conditional)
            if self.show_due {
                print!("  {}", due_colored);
            }

            println!();
        }
    }

    /// Displays the table header.
    pub fn display_header(&self) {
        // ID, Priority, Status
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

        // Recurrence (conditional)
        if self.show_recur {
            print!(
                " {:<recur_width$}  ",
                "R".dimmed(),
                recur_width = self.recur,
            );
        }

        // Task
        print!("{:<task_width$}", "Task".dimmed(), task_width = self.task,);

        // Tags (conditional)
        if self.show_tags {
            print!("  {:<tags_width$}", "Tags".dimmed(), tags_width = self.tags,);
        }

        // Due (conditional)
        if self.show_due {
            print!("  {}", "Due".dimmed());
        }

        println!();
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
