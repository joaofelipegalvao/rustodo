use std::{env, error::Error, fs, process};

use colored::{ColoredString, Colorize};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

#[derive(Debug, Clone)]
struct Task {
    text: String,
    completed: bool,
    priority: Priority,
}

#[derive(Debug, Clone, PartialEq, Copy)]
enum Priority {
    High,
    Medium,
    Low,
}

impl Task {
    fn new(text: String, priority: Priority) -> Self {
        Task {
            text,
            completed: false,
            priority,
        }
    }

    fn mark_done(&mut self) {
        self.completed = true;
    }

    fn mark_undone(&mut self) {
        self.completed = false;
    }

    fn to_line(&self) -> String {
        let checkbox = if self.completed { "[x]" } else { "[ ]" };
        let priority_str = match self.priority {
            Priority::High => "(high)",
            Priority::Medium => "",
            Priority::Low => "(low)",
        };

        if priority_str.is_empty() {
            format!("{} {}", checkbox, self.text)
        } else {
            format!("{} {} {}", checkbox, priority_str, self.text)
        }
    }

    fn from_line(line: &str) -> Option<Self> {
        let completed = line.contains("[x]");

        let without_checkbox = line
            .replace("[ ]", "")
            .replace("[x]", "")
            .trim()
            .to_string();

        let (priority, text) = if without_checkbox.starts_with("(high)") {
            (
                Priority::High,
                without_checkbox.replace("(high)", "").trim().to_string(),
            )
        } else if without_checkbox.starts_with("(low)") {
            (
                Priority::Low,
                without_checkbox.replace("(low)", "").trim().to_string(),
            )
        } else {
            (Priority::Medium, without_checkbox)
        };

        Some(Task {
            text,
            completed,
            priority,
        })
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
    match fs::read_to_string("todos.txt") {
        Ok(content) => {
            let tasks: Vec<Task> = content
                .lines()
                .filter(|l| !l.trim().is_empty())
                .filter_map(Task::from_line)
                .collect();
            Ok(tasks)
        }
        Err(_) => Ok(Vec::new()),
    }
}

fn save_tasks(tasks: &[Task]) -> Result<(), Box<dyn Error>> {
    let lines: Vec<String> = tasks.iter().map(|t| t.to_line()).collect();
    fs::write("todos.txt", lines.join("\n") + "\n")?;
    Ok(())
}

fn display_task(number: usize, task: &Task) {
    let number_fmt = format!("{}.", number);
    let emoji = task.priority.emoji();

    if task.completed {
        println!(
            "{} {} {} {}",
            number_fmt.dimmed(),
            emoji,
            "󰄵".green(),
            task.text.green().strikethrough()
        );
    } else {
        println!(
            "{} {} {} {}",
            number_fmt.dimmed(),
            emoji,
            "".yellow(),
            task.text.bright_white()
        );
    }
}

fn display_lists(tasks: &[Task], title: &str) {
    println!("\n {}:\n", title);

    let mut completed = 0;
    let total = tasks.len();

    for (i, task) in tasks.iter().enumerate() {
        display_task(i + 1, task);

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
                return Err("Usage: todo add <task> [--high|--medium|--low]".into());
            }

            let text = args[2].clone();

            let priority = if args.len() >= 4 {
                match args[3].as_str() {
                    "--high" => Priority::High,
                    "--medium" => Priority::Medium,
                    "--low" => Priority::Low,
                    _ => {
                        return Err(format!(
                            "Invalid flag '{}'. Use --high, --medium or --low",
                            args[3]
                        )
                        .into());
                    }
                }
            } else {
                Priority::Medium
            };

            let task = Task::new(text, priority);

            let mut tasks = load_tasks()?;
            tasks.push(task);
            save_tasks(&tasks)?;

            println!("{}", "✓ Task added".green());
        }

        "list" => {
            let mut status_filter = "all";
            let mut priority_filter: Option<Priority> = None;
            let mut sort = false;

            for arg in &args[2..] {
                match arg.as_str() {
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
                    _ => return Err(format!("Invalid filter: {}", arg).into()),
                }
            }

            let mut tasks = load_tasks()?;

            if tasks.is_empty() {
                println!("No tasks");
                return Ok(());
            }

            // Filtrar por status
            match status_filter {
                "pending" => tasks.retain(|t| !t.completed),
                "done" => tasks.retain(|t| t.completed),
                _ => {}
            };

            // Filtrar por prioridade
            if let Some(pri) = priority_filter {
                tasks.retain(|t| t.priority == pri);
            }

            if tasks.is_empty() {
                println!("No tasks found with these filters");
                return Ok(());
            }

            if sort {
                tasks.sort_by(|a, b| a.priority.order().cmp(&b.priority.order()));
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

            display_lists(&tasks, title);
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

            save_tasks(&tasks)?; // ← CORRIGIDO (adicionado ?)

            println!("{}", "✓ Task unmarked".yellow());
        }

        "search" => {
            if args.len() < 3 {
                return Err("Usage: todo search <term>".into());
            }

            let term = &args[2];

            let tasks = load_tasks()?;

            if tasks.is_empty() {
                println!("No tasks");
                return Ok(());
            }

            let results: Vec<(usize, &Task)> = tasks
                .iter()
                .enumerate()
                .filter(|(_, task)| task.text.to_lowercase().contains(&term.to_lowercase()))
                .map(|(i, task)| (i + 1, task))
                .collect();

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
            if fs::metadata("todos.txt").is_ok() {
                fs::remove_file("todos.txt")?;
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
