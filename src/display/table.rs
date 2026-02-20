use colored::Colorize;

use crate::models::{Recurrence, Task};

use super::formatting::{get_due_colored, get_due_text, render_checkbox};

const ID_WIDTH: usize = 4;
const PRIORITY_WIDTH: usize = 1;
const STATUS_WIDTH: usize = 3;
const RECUR_WIDTH: usize = 1;

pub struct TableLayout {
    id: usize,
    priority: usize,
    status: usize,
    recur: usize,
    task: usize,
    project: usize,
    tags: usize,
    due: usize,
    show_recur: bool,
    show_project: bool,
    show_tags: bool,
    show_due: bool,
}

impl TableLayout {
    pub fn new(tasks: &[(usize, &Task)]) -> Self {
        let (task, project, tags, due) = calculate_column_widths(tasks);

        let show_recur = tasks.iter().any(|(_, t)| t.recurrence.is_some());
        let show_project = tasks.iter().any(|(_, t)| t.project.is_some());
        let show_tags = tasks.iter().any(|(_, t)| !t.tags.is_empty());
        let show_due = tasks.iter().any(|(_, t)| t.due_date.is_some());

        Self {
            id: ID_WIDTH,
            priority: PRIORITY_WIDTH,
            status: STATUS_WIDTH,
            recur: RECUR_WIDTH,
            task,
            project,
            tags,
            due,
            show_recur,
            show_project,
            show_tags,
            show_due,
        }
    }

    pub fn total_width(&self) -> usize {
        let mut width = self.id + self.priority + self.status + self.task + 8;
        if self.show_recur {
            width += self.recur + 2;
        }
        if self.show_project {
            width += self.project + 2;
        }
        if self.show_tags {
            width += self.tags + 2;
        }
        if self.show_due {
            width += self.due + 2;
        }
        width
    }

    pub fn display_header(&self) {
        print!("{:>id_width$} ", "ID".dimmed(), id_width = self.id);
        print!(" {:<p$} ", "P".dimmed(), p = self.priority);
        print!(" {:<s$} ", " S".dimmed(), s = self.status);
        if self.show_recur {
            print!(" {:<r$}  ", "R".dimmed(), r = self.recur);
        }
        print!("{:<t$}", "Task".dimmed(), t = self.task);
        if self.show_project {
            print!("  {:<p$}", "Project".dimmed(), p = self.project);
        }
        if self.show_tags {
            print!("  {:<t$}", "Tags".dimmed(), t = self.tags);
        }
        if self.show_due {
            print!("  {}", "Due".dimmed());
        }
        println!();
    }

    pub fn display_separator(&self) {
        println!("{}", "─".repeat(self.total_width()).dimmed());
    }

    pub fn display_task(&self, number: usize, task: &Task) {
        let checkbox = render_checkbox(task.completed);
        let letter = task.priority.letter();

        let task_text = truncate(&task.text, self.task);

        let project_str = task
            .project
            .as_deref()
            .map(|p| truncate(p, self.project))
            .unwrap_or_default();

        let tags_str = if task.tags.is_empty() {
            String::new()
        } else {
            truncate(&task.tags.join(", "), self.tags)
        };

        let due_text = get_due_text(task);
        let due_colored = get_due_colored(task, &due_text);

        let recur_indicator = match task.recurrence {
            Some(Recurrence::Daily) => "D".cyan(),
            Some(Recurrence::Weekly) => "W".cyan(),
            Some(Recurrence::Monthly) => "M".cyan(),
            None => " ".normal(),
        };

        let (text_colored, tags_colored, project_colored) = if task.completed {
            (task_text.green(), tags_str.dimmed(), project_str.dimmed())
        } else {
            (
                task_text.bright_white(),
                tags_str.cyan(),
                project_str.magenta(),
            )
        };

        print!(
            "{:>id_width$} ",
            number.to_string().dimmed(),
            id_width = self.id
        );
        print!(" {:<p$} ", letter, p = self.priority);
        print!(" {:<s$} ", checkbox, s = self.status);
        if self.show_recur {
            print!(" {:<r$}  ", recur_indicator, r = self.recur);
        }
        print!("{:<t$}", text_colored, t = self.task);
        if self.show_project {
            print!("  {:<p$}", project_colored, p = self.project);
        }
        if self.show_tags {
            print!("  {:<t$}", tags_colored, t = self.tags);
        }
        if self.show_due {
            print!("  {}", due_colored);
        }
        println!();
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max.saturating_sub(3)])
    } else {
        s.to_owned()
    }
}

fn calculate_column_widths(tasks: &[(usize, &Task)]) -> (usize, usize, usize, usize) {
    let mut max_task = 10;
    let mut max_project = 7; // "Project" header width
    let mut max_tags = 4;
    let mut max_due = 3;

    for (_, task) in tasks {
        max_task = max_task.max(task.text.len());
        if let Some(p) = &task.project {
            max_project = max_project.max(p.len());
        }
        if !task.tags.is_empty() {
            max_tags = max_tags.max(task.tags.join(", ").len());
        }
        let due_text = get_due_text(task);
        if !due_text.is_empty() {
            max_due = max_due.max(due_text.len());
        }
    }

    (
        max_task.min(40),
        max_project.min(24),
        max_tags.min(20),
        max_due.min(20),
    )
}

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
