use std::{env, error::Error, fs, process};

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
}

#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize)]
enum Priority {
    High,
    Medium,
    Low,
}

impl Task {
    fn new(text: String, priority: Priority, tags: Vec<String>) -> Self {
        Task {
            text,
            completed: false,
            priority,
            tags,
        }
    }

    fn mark_done(&mut self) {
        self.completed = true;
    }

    fn mark_undone(&mut self) {
        self.completed = false;
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

    fn emoji(&self) -> ColoredString {
        match self {
            Priority::High => "".red(),
            Priority::Medium => "".yellow(),
            Priority::Low => "".green(),
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

fn display_task(number: usize, task: &Task) {
    let number_fmt = format!("{}.", number);
    let emoji = task.priority.emoji();

    let tags_str = if task.tags.is_empty() {
        String::new()
    } else {
        format!(" [{}]", task.tags.join(", "))
    };

    if task.completed {
        println!(
            "{} {} {} {}{}",
            number_fmt.dimmed(),
            emoji,
            "󰄵".green(),
            task.text.green().strikethrough(),
            tags_str.dimmed()
        );
    } else {
        println!(
            "{} {} {} {}{}",
            number_fmt.dimmed(),
            emoji,
            "".yellow(),
            task.text.bright_white(),
            tags_str.cyan()
        );
    }
}

fn display_lists(tasks: &[(usize, &Task)], title: &str) {
    println!("\n {}:\n", title);

    let mut completed = 0;
    let total = tasks.len();

    for (number, task) in tasks {
        display_task(*number, task);

        if task.completed {
            completed += 1;
        }
    }

    println!("\n{}", "─".repeat(30).dimmed());

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
        return Err(
            "Usage: todo <command> [arguments]\nCommands: add, list [--pending|--done|--high|--medium|--low|--sort], done, undone, remove, clear, search"
                .into(),
        );
    }

    let command = &args[1];

    match command.as_str() {
        "add" => {
            if args.len() < 3 {
                return Err(
                    "Usage: todo add <task> [--high|--medium|--low] [--tag <name>]...".into(),
                );
            }

            let text = args[2].clone();

            let mut priority = Priority::Medium;
            let mut tags: Vec<String> = Vec::new();

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

                    _ => {
                        return Err(format!("Invalid flag: {}", args[i]).into());
                    }
                }

                i += 1;
            }

            let task = Task::new(text, priority, tags);

            let mut tasks = load_tasks()?;
            tasks.push(task);
            save_tasks(&tasks)?;

            println!("{}", "✓ Task added".green());
        }

        "list" => {
            let mut status_filter = "all";
            let mut priority_filter: Option<Priority> = None;
            let mut tag_filter: Option<String> = None;
            let mut sort = false;

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
                        if sort {
                            return Err("Use --sort only once".into());
                        }
                        sort = true;
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

            if indexed_tasks.is_empty() {
                println!("No tasks found with these filters");
                return Ok(());
            }

            if sort {
                indexed_tasks.sort_by(|(_, a), (_, b)| a.priority.order().cmp(&b.priority.order()));
            }

            let title = match (status_filter, priority_filter) {
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
                return Err("Usage: todo search <term> [--tag <name>]".into());
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
                println!("\n󱎸 Results for \"{}\":\n", term);

                for (number, task) in &results {
                    display_task(*number, task);
                }

                println!("\n{} result(s) found\n", results.len());
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

            println!("\n Tags:\n");
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
