use std::{env, error::Error, fs, process};

use chrono::{Local, NaiveDate};
use colored::{ColoredString, Colorize};
use serde::{Deserialize, Serialize};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task {
    text: String,
    completed: bool,
    priority: Priority,
    tags: Vec<String>,
    due_date: Option<NaiveDate>,
    created_at: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize)]
enum Priority {
    High,
    Medium,
    Low,
}

impl Task {
    fn new(
        text: String,
        priority: Priority,
        tags: Vec<String>,
        due_date: Option<NaiveDate>,
    ) -> Self {
        Task {
            text,
            completed: false,
            priority,
            tags,
            due_date,
            created_at: Local::now().naive_local().date(),
        }
    }

    fn mark_done(&mut self) {
        self.completed = true;
    }

    fn mark_undone(&mut self) {
        self.completed = false;
    }

    fn is_overdue(&self) -> bool {
        if let Some(due) = self.due_date {
            let today = Local::now().naive_local().date();
            due < today && !self.completed
        } else {
            false
        }
    }

    fn is_due_soon(&self, days: i64) -> bool {
        if let Some(due) = self.due_date {
            let today = Local::now().naive_local().date();
            let days_until = (due - today).num_days();
            days_until >= 0 && days_until <= days && !self.completed
        } else {
            false
        }
    }
}

impl Priority {
    fn order(&self) -> u8 {
        match self {
            Priority::High => 0,
            Priority::Medium => 1,
            Priority::Low => 2,
        }
    }

    fn letter(&self) -> ColoredString {
        match self {
            Priority::High => "H".red(),
            Priority::Medium => "M".yellow(),
            Priority::Low => "L".green(),
        }
    }
}

fn load_tasks() -> Result<Vec<Task>, Box<dyn Error>> {
    match fs::read_to_string("todos.json") {
        Ok(content) => {
            let tasks: Vec<Task> = serde_json::from_str(&content)?;
            Ok(tasks)
        }
        Err(_) => Ok(Vec::new()),
    }
}

fn save_tasks(tasks: &[Task]) -> Result<(), Box<dyn Error>> {
    let json = serde_json::to_string_pretty(tasks)?;
    fs::write("todos.json", json)?;
    Ok(())
}

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

    max_task_len = max_task_len.min(40);
    max_tags_len = max_tags_len.min(20);
    max_due_len = max_due_len.min(20);

    (max_task_len, max_tags_len, max_due_len)
}

fn get_due_text(task: &Task) -> String {
    if let Some(due) = task.due_date {
        let today = Local::now().naive_local().date();
        let days_until = (due - today).num_days();

        if task.completed {
            String::new()
        } else if days_until < 0 {
            format!(
                "late {} day{}",
                -days_until,
                if -days_until == 1 { "" } else { "s" }
            )
        } else if days_until == 0 {
            "due today".to_string()
        } else if days_until <= 7 {
            format!(
                "in {} day{}",
                days_until,
                if days_until == 1 { "" } else { "s" }
            )
        } else {
            format!(
                "in {} day{}",
                days_until,
                if days_until == 1 { "" } else { "s" }
            )
        }
    } else {
        String::new()
    }
}

fn get_due_colored(task: &Task, text: &str) -> ColoredString {
    if text.is_empty() {
        return "".normal();
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

fn display_task_tabular(number: usize, task: &Task, task_width: usize, tags_width: usize) {
    let number_str = format!("{:>3}", number);
    let letter = task.priority.letter();
    let checkbox = if task.completed {
        "".green()
    } else {
        "".bright_white()
    };

    let task_text = if task.text.len() > task_width {
        format!("{}...", &task.text[..task_width - 3])
    } else {
        task.text.clone()
    };

    let tags_str = if task.tags.is_empty() {
        String::new()
    } else {
        let joined = task.tags.join(", ");
        if joined.len() > tags_width {
            format!("{}...", &joined[..tags_width - 3])
        } else {
            joined
        }
    };

    let due_text = get_due_text(task);
    let due_colored = get_due_colored(task, &due_text);

    if task.completed {
        print!("{:>4} ", number_str.dimmed());
        print!(" {} ", letter);
        print!(" {} ", checkbox);
        print!("{:<width$}", task_text.green(), width = task_width);
        print!("  {:<width$}", tags_str.dimmed(), width = tags_width);
        println!("  {}", due_colored);
    } else {
        print!("{:>4} ", number_str.dimmed());
        print!(" {} ", letter);
        print!(" {} ", checkbox);
        print!("{:<width$}", task_text.bright_white(), width = task_width);
        print!("  {:<width$}", tags_str.cyan(), width = tags_width);
        println!("  {}", due_colored);
    }
}

fn display_lists(tasks: &[(usize, &Task)], title: &str) {
    println!("\n{}:\n", title);

    if tasks.is_empty() {
        println!("No tasks");
        return;
    }

    let (task_width, tags_width, due_width) = calculate_column_widths(tasks);

    print!("{:>4} ", "ID".dimmed());
    print!(" {} ", "P".dimmed());
    print!(" {} ", "S".dimmed());
    print!("{:<width$}", "Task".dimmed(), width = task_width);
    print!("  {:<width$}", "Tags".dimmed(), width = tags_width);
    println!("  {}", "Due".dimmed());

    let total_width = task_width + tags_width + due_width + 19;

    println!("{}", "─".repeat(total_width).dimmed());

    let mut completed = 0;
    let total = tasks.len();

    for (number, task) in tasks {
        display_task_tabular(*number, task, task_width, tags_width);

        if task.completed {
            completed += 1;
        }
    }

    println!("\n{}", "─".repeat(total_width).dimmed());

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

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err("Usage: todo <command> [arguments]\n\
         Commands: add, list [--pending|--done|--high|--medium|--low|\
         --sort <field>|--overdue|--due-soon|--with-due|--without-due], \
         done, undone, remove, clear, search, tags"
            .into());
    }

    let command = &args[1];

    match command.as_str() {
        "add" => {
            if args.len() < 3 {
                return Err(
                    "Usage: todo add <task> [--high|--medium|--low] [--tag <n>] [--due YYYY-MM-DD]"
                        .into(),
                );
            }

            let text = args[2].clone();

            let mut priority = Priority::Medium;
            let mut tags: Vec<String> = Vec::new();
            let mut due_date: Option<NaiveDate> = None;

            let mut i = 3;

            while i < args.len() {
                match args[i].as_str() {
                    "--high" => priority = Priority::High,
                    "--medium" => priority = Priority::Medium,
                    "--low" => priority = Priority::Low,
                    "--tag" => {
                        if i + 1 >= args.len() {
                            return Err("--tag requires a value".into());
                        }
                        tags.push(args[i + 1].clone());
                        i += 1;
                    }

                    "--due" => {
                        if i + 1 >= args.len() {
                            return Err("--due requires a date in format YYYY-MM-DD".into());
                        }

                        let date_str = &args[i + 1];
                        match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                            Ok(date) => due_date = Some(date),
                            Err(_) => {
                                return Err(format!(
                                    "Invalid date format: '{}'. Use YYYY-MM-DD",
                                    date_str
                                )
                                .into());
                            }
                        }

                        i += 1;
                    }

                    _ => {
                        return Err(format!("Invalid flag: {}", args[i]).into());
                    }
                }

                i += 1;
            }

            let task = Task::new(text, priority, tags, due_date);

            let mut tasks = load_tasks()?;
            tasks.push(task);
            save_tasks(&tasks)?;

            println!("{}", "✓ Task added".green());
        }

        "list" => {
            let mut status_filter = "all";
            let mut priority_filter: Option<Priority> = None;
            let mut tag_filter: Option<String> = None;
            let mut sort_by = "none";
            let mut overdue = false;
            let mut due_soon = false;
            let mut with_due = false;
            let mut without_due = false;

            let mut i = 2;

            while i < args.len() {
                match args[i].as_str() {
                    "--pending" => {
                        if status_filter != "all" {
                            return Err("Use only one status filter (--pending or --done)".into());
                        }
                        status_filter = "pending";
                    }
                    "--done" => {
                        if status_filter != "all" {
                            return Err("Use only one status filter (--done or --pending)".into());
                        }
                        status_filter = "done";
                    }
                    "--high" => {
                        if priority_filter.is_some() {
                            return Err(
                                "Use only one priority filter (--high, --medium or --low)".into()
                            );
                        }
                        priority_filter = Some(Priority::High);
                    }
                    "--medium" => {
                        if priority_filter.is_some() {
                            return Err(
                                "Use only one priority filter (--high, --medium or --low)".into()
                            );
                        }
                        priority_filter = Some(Priority::Medium);
                    }
                    "--low" => {
                        if priority_filter.is_some() {
                            return Err(
                                "Use only one priority filter (--high, --medium or --low)".into()
                            );
                        }
                        priority_filter = Some(Priority::Low);
                    }
                    "--sort" => {
                        if sort_by != "none" {
                            return Err("Use --sort only once".into());
                        }

                        if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                            match args[i + 1].as_str() {
                                "priority" => sort_by = "priority",
                                "due" => sort_by = "due",
                                "created" => sort_by = "created",
                                _ => {
                                    return Err(format!(
                                        "Invalid sort field: '{}'. Use: priority, due, or created",
                                        args[i + 1]
                                    )
                                    .into());
                                }
                            }
                            i += 1;
                        } else {
                            sort_by = "priority";
                        }
                    }
                    "--tag" => {
                        if tag_filter.is_some() {
                            return Err("Use only one --tag filter".into());
                        }

                        if i + 1 >= args.len() {
                            return Err("--tag requires a value".into());
                        }
                        tag_filter = Some(args[i + 1].clone());
                        i += 1;
                    }
                    "--overdue" => {
                        if overdue || due_soon || with_due || without_due {
                            return Err("Use only one date filter".into());
                        }
                        overdue = true;
                    }
                    "--due-soon" => {
                        if overdue || due_soon || with_due || without_due {
                            return Err("Use only one date filter".into());
                        }
                        due_soon = true;
                    }

                    "--with-due" => {
                        if overdue || due_soon || with_due || without_due {
                            return Err("Use only one date filter".into());
                        }
                        with_due = true;
                    }

                    "--without-due" => {
                        if overdue || due_soon || with_due || without_due {
                            return Err("Use only one date filter".into());
                        }
                        without_due = true;
                    }
                    _ => return Err(format!("Invalid filter: {}", args[i]).into()),
                }
                i += 1;
            }

            let all_tasks = load_tasks()?;

            if all_tasks.is_empty() {
                println!("No tasks");
                return Ok(());
            }

            let mut indexed_tasks: Vec<(usize, &Task)> = all_tasks
                .iter()
                .enumerate()
                .map(|(i, task)| (i + 1, task))
                .collect();

            match status_filter {
                "pending" => indexed_tasks.retain(|(_, t)| !t.completed),
                "done" => indexed_tasks.retain(|(_, t)| t.completed),
                _ => {}
            }

            if let Some(pri) = priority_filter {
                indexed_tasks.retain(|(_, t)| t.priority == pri);
            }

            if let Some(tag) = &tag_filter {
                indexed_tasks.retain(|(_, t)| t.tags.contains(tag));
            }

            if overdue {
                indexed_tasks.retain(|(_, t)| t.is_overdue());
            }
            if due_soon {
                indexed_tasks.retain(|(_, t)| t.is_due_soon(7));
            }
            if with_due {
                indexed_tasks.retain(|(_, t)| t.due_date.is_some());
            }
            if without_due {
                indexed_tasks.retain(|(_, t)| t.due_date.is_none());
            }

            if indexed_tasks.is_empty() {
                println!("No tasks found with these filters");
                return Ok(());
            }

            match sort_by {
                "priority" => {
                    indexed_tasks
                        .sort_by(|(_, a), (_, b)| a.priority.order().cmp(&b.priority.order()));
                }
                "due" => {
                    indexed_tasks.sort_by(|(_, a), (_, b)| match (a.due_date, b.due_date) {
                        (Some(date_a), Some(date_b)) => date_a.cmp(&date_b),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => std::cmp::Ordering::Equal,
                    });
                }
                "created" => {
                    indexed_tasks.sort_by(|(_, a), (_, b)| a.created_at.cmp(&b.created_at));
                }
                _ => {}
            }

            let title = if with_due {
                "Tasks with due date"
            } else if without_due {
                "Tasks without due date"
            } else if overdue {
                "Overdue tasks"
            } else if due_soon {
                "Due soon (next 7 days)"
            } else {
                match (status_filter, priority_filter) {
                    ("pending", Some(Priority::High)) => "High priority pending tasks",
                    ("pending", Some(Priority::Medium)) => "Medium priority pending tasks",
                    ("pending", Some(Priority::Low)) => "Low priority pending tasks",
                    ("pending", None) => "Pending tasks",
                    ("done", Some(Priority::High)) => "High priority completed tasks",
                    ("done", Some(Priority::Medium)) => "Medium priority completed tasks",
                    ("done", Some(Priority::Low)) => "Low priority completed tasks",
                    ("done", None) => "Completed tasks",
                    (_, Some(Priority::High)) => "High priority",
                    (_, Some(Priority::Medium)) => "Medium priority",
                    (_, Some(Priority::Low)) => "Low priority",
                    _ => "Tasks",
                }
            };

            display_lists(&indexed_tasks, title);
        }

        "done" => {
            if args.len() < 3 {
                return Err("Usage: todo done <number>".into());
            }

            let number: usize = args[2].parse()?;

            let mut tasks = load_tasks()?;

            if number == 0 || number > tasks.len() {
                return Err("Invalid task number".into());
            }

            let index = number - 1;

            if tasks[index].completed {
                return Err("Task is already marked as completed".into());
            }

            tasks[index].mark_done();

            save_tasks(&tasks)?;

            println!("{}", "✓ Task marked as completed".green());
        }

        "undone" => {
            if args.len() < 3 {
                return Err("Usage: todo undone <number>".into());
            }

            let number: usize = args[2].parse()?;

            let mut tasks = load_tasks()?;

            if number == 0 || number > tasks.len() {
                return Err("Invalid task number".into());
            }

            let index = number - 1;

            if !tasks[index].completed {
                return Err("Task is already unmarked".into());
            }

            tasks[index].mark_undone();

            save_tasks(&tasks)?;

            println!("{}", "✓ Task unmarked".yellow());
        }

        "search" => {
            if args.len() < 3 {
                return Err("Usage: todo search <term> [--tag <n>]".into());
            }

            let term = &args[2];
            let mut tag_filter: Option<String> = None;

            let mut i = 3;

            while i < args.len() {
                match args[i].as_str() {
                    "--tag" => {
                        if i + 1 >= args.len() {
                            return Err("--tag requires a value".into());
                        }

                        tag_filter = Some(args[i + 1].clone());
                        i += 1;
                    }

                    _ => return Err(format!("Invalid flag: {}", args[i]).into()),
                }

                i += 1;
            }

            let tasks = load_tasks()?;

            if tasks.is_empty() {
                println!("No tasks");
                return Ok(());
            }

            let mut results: Vec<(usize, &Task)> = tasks
                .iter()
                .enumerate()
                .filter(|(_, task)| task.text.to_lowercase().contains(&term.to_lowercase()))
                .map(|(i, task)| (i + 1, task))
                .collect();

            if let Some(tag) = &tag_filter {
                results.retain(|(_, task)| task.tags.contains(tag));
            }

            if results.is_empty() {
                println!("No results for '{}'", term);
            } else {
                display_lists(&results, &format!("Search results for \"{}\"", term));
            }
        }

        "tags" => {
            let tasks = load_tasks()?;

            if tasks.is_empty() {
                println!("No tasks");
                return Ok(());
            }

            let mut all_tags: Vec<String> = Vec::new();
            for task in &tasks {
                for tag in &task.tags {
                    if !all_tags.contains(tag) {
                        all_tags.push(tag.clone());
                    }
                }
            }

            if all_tags.is_empty() {
                println!("No tags found");
                return Ok(());
            }

            all_tags.sort();

            println!("\nTags:\n");
            for tag in &all_tags {
                let count = tasks.iter().filter(|t| t.tags.contains(tag)).count();
                println!(
                    "  {} ({} task{})",
                    tag.cyan(),
                    count,
                    if count == 1 { "" } else { "s" }
                );
            }

            println!()
        }

        "remove" => {
            if args.len() < 3 {
                return Err("Usage: todo remove <number>".into());
            }

            let number: usize = args[2].parse()?;

            let mut tasks = load_tasks()?;

            if number == 0 || number > tasks.len() {
                return Err("This task doesn't exist or was already removed".into());
            }

            let index = number - 1;
            tasks.remove(index);

            save_tasks(&tasks)?;

            println!("{}", "✓ Task removed".red());
        }

        "clear" => {
            if fs::metadata("todos.json").is_ok() {
                fs::remove_file("todos.json")?;
                println!("{}", "✓ All tasks have been removed".red().bold());
            } else {
                println!("No tasks to remove");
            }
        }

        _ => {
            return Err(format!("Unknown command: {}", command).into());
        }
    }

    Ok(())
}
