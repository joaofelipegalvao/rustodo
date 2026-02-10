/*
A morden, powerful task manager built with Rust.

This CLI application provides a simple yet feature-rich interface for managing
todo tasks with support for priorities, tags, due dates, and advanced filtering.
*/

use std::{fs, path::PathBuf, process};

use anyhow::{Context, Result};
use chrono::{Local, NaiveDate};
use clap::{Args, Parser, Subcommand, ValueEnum};
use colored::{ColoredString, Colorize};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Custom error types for the todo application.
///
/// These erros provide specific, user-friendly messages for common
/// error conditions that can occur during task management.
#[derive(Error, Debug)]
pub enum TodoError {
    #[error("Task ID {id} is invalid (valid range: 1-{max})")]
    InvalidTaskId { id: usize, max: usize },

    #[error("Task #{id} is already marked as {status}")]
    TaskAlreadyInStatus { id: usize, status: String },

    #[error("Tag '{0}' not found in any task")]
    TagNotFound(String),

    #[error("No tasks found matching the specified filters")]
    NoTasksFound,

    #[error("No tags found in any task")]
    NoTagsFound,

    #[error("Search returned no results for query: '{0}'")]
    NoSearchResults(String),
}

/// Main entry point for the todo-list application.
///
/// Parses command-line arguments and executes the requested command.
/// If an error occurs, it prints a formatted error message and exits
/// with a non-zero status code.
fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("{} {}", "Error:".red().bold(), format!("{}", e).red());

        // Print the full error chain for better debugging
        let mut source = e.source();
        while let Some(cause) = source {
            eprintln!("{} {}", "Caused by:".red(), cause);
            source = cause.source();
        }

        process::exit(1);
    }
}

/// Represents a single task in the todo list.
///
/// Each task contains a description, completion status, priority level,
/// optional tags for organization, and optional due date for deadline
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task {
    /// The task description/content
    text: String,
    /// Wheter the task has been completed
    completed: bool,
    /// Priority level of the task
    priority: Priority,
    /// List of tags for categorization
    tags: Vec<String>,
    /// Optional due date for deadline tracking
    due_date: Option<NaiveDate>,
    /// Date when the task was created
    created_at: NaiveDate,
}

/// Priority levels for tasks.
///
/// Tasks can be categorized as High, Medium, or Low priority,
/// which affects their sorting order and visual presentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
enum Priority {
    /// High priority - urgent and important tasks
    High,
    /// Medium priority - default for most tasks
    Medium,
    /// Low priority - nice to have, not urgent
    Low,
}

/// Filter for task completion status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum StatusFilter {
    /// Show only pending tasks
    Pending,
    /// Show only completed tasks
    Done,
    /// Show all tasks (default)
    All,
}

/// Filter for task due dates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum DueFilter {
    /// Tasks past their due date
    Overdue,
    /// Tasks due in the next 7 days
    Soon,
    /// Tasks with any due date set
    WithDue,
    /// Tasks without a due date
    NoDue,
}

/// Sorting options for task lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum SortBy {
    /// Sort by priority (High -> Medium -> Low)
    Priority,
    /// Sort by due date (earliest first)
    Due,
    /// Sort by creation date (oldest first)
    Created,
}

#[derive(Parser)]
#[command(name = "todo-list")]
#[command(author = "github.com/joaofelipegalvao")]
#[command(version = "1.9.0")]
#[command(about = "A modern, powerful task manager built with Rust", long_about = None)]
#[command(after_help = "EXAMPLES:\n    \
    # Add a high priority task\n    \
    todo add \"Complete Rust project\" --priority high --tag work --due 2025-02-15\n\n    \
    # List pending high priority tasks\n    \
    todo list --status pending --priority high\n\n    \
    # List overdue tasks sorted by due date\n    \
    todo list --due overdue --sort due\n\n    \
    # Search for tasks\n    \
    todo search rust\n\n    \
    # Mark task as completed\n    \
    todo done 3\n\n\
For more information, visit: https://github.com/joaofelipegalvao/todo-cli
")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new task to your todo list
    #[command(visible_alias = "a")]
    #[command(long_about = "Add a new task to your todo list\n\n\
        Creates a new task with the specified text and optional metadata like priority,\n\
        tags, and due date. Tasks are saved immediately to todos.json.")]
    Add(AddArgs),

    /// List and filter tasks
    #[command(visible_alias = "ls")]
    #[command(
        long_about = "List and filter tasks with powerful filtering options\n\n\
        Display your tasks with filtering and sorting capabilities.\n\
        All filters can be combined to find exactly what you need."
    )]
    List {
        /// Filter by completion status
        #[arg(long, value_enum, default_value_t = StatusFilter::All)]
        status: StatusFilter,

        /// Filter by priority level
        #[arg(long, value_enum)]
        priority: Option<Priority>,

        /// Filter by due date
        #[arg(long, value_enum)]
        due: Option<DueFilter>,

        /// Sort results by field
        #[arg(long, short = 's', value_enum)]
        sort: Option<SortBy>,

        /// Filter by tag name
        #[arg(long, short = 't')]
        tag: Option<String>,
    },

    /// Mark a task as completed
    #[command(visible_alias = "complete")]
    #[command(long_about = "Mark a task as completed\n\n\
        Marks the specified task as done. The task will be shown with a ✓ symbol\n\
        and appear in green when listing tasks.")]
    Done {
        /// Task ID number (from 'list' command)
        #[arg(value_name = "ID")]
        id: usize,
    },

    /// Mark a completed task as pending
    #[command(visible_alias = "undo")]
    #[command(long_about = "Mark a completed task as pending\n\n\
        Reverts a task back to pending status. Useful if you accidentally marked\n\
        a task as done or need to redo it.")]
    Undone {
        /// Task ID number
        #[arg(value_name = "ID")]
        id: usize,
    },

    /// Remove a task permanently
    #[command(visible_aliases = ["rm", "delete"])]
    #[command(long_about = "Remove a task permanently from your list\n\n\
        WARNING: This action cannot be undone. The task will be permanently deleted\n\
        from your todos.json file.")]
    Remove {
        /// Task ID number
        #[arg(value_name = "ID")]
        id: usize,

        #[arg(long, short = 'y')]
        yes: bool,
    },

    /// Edit an existing task
    #[command(visible_alias = "e")]
    #[command(long_about = "Edit an existing task\n\n\
        Modify task properties like text, priority, tags, or due date.\n\
        Only specify the fields you want to change.")]
    Edit {
        /// Task ID number
        #[arg(value_name = "ID")]
        id: usize,
        /// New Task description
        #[arg(long)]
        text: Option<String>,

        /// New priority level
        #[arg(long, value_enum)]
        priority: Option<Priority>,

        /// Replace tags (use multiple times: -t work -t urgent)
        #[arg(long, short = 't', value_name = "TAG")]
        tag: Vec<String>,

        /// New due date (YYYY-MM-DD)
        #[arg(long, value_parser = clap::value_parser!(NaiveDate))]
        due: Option<NaiveDate>,
        /// Remove due date
        #[arg(long, conflicts_with = "due")]
        clear_due: bool,

        /// Remove all tags
        #[arg(long)]
        clear_tags: bool,
    },

    /// Clear all tasks
    #[command(visible_alias = "reset")]
    #[command(long_about = "Clear all tasks (removes todos.json file)\n\n\
        WARNING: This will permanently delete ALL tasks. This action cannot be undone.\n\
        You will lose all your tasks, tags, and metadata.")]
    Clear {
        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
    },

    /// Search for tasks by text content
    #[command(visible_alias = "find")]
    #[command(long_about = "Search for tasks by text content\n\n\
        Performs a case-insensitive search through all task descriptions.\n\
        Returns all tasks that contain the search term.")]
    Search {
        /// The text to search for in task descriptions
        #[arg(value_name = "QUERY")]
        query: String,

        /// Filter results by tag
        #[arg(long, short = 't')]
        tag: Option<String>,
    },

    /// List all tags
    #[command(long_about = "List all tags used across your tasks\n\n\
        Shows a summary of all tags you've created, along with the count\n\
        of tasks associated with each tag.")]
    Tags,

    /// Show information about data file location
    #[command(long_about = "Show information about where your tasks are stored\n\n\
    Displays the path to the todos.json file and its status.")]
    Info,
}

#[derive(Args)]
struct AddArgs {
    /// Task description
    #[arg(value_name = "DESCRIPTION")]
    text: String,

    /// Task priority level
    #[arg(long, value_enum, default_value_t = Priority::Medium)]
    priority: Priority,

    /// Add tags (can be repeated: -t work -t urgent)
    #[arg(long, short = 't', value_name = "TAG")]
    tag: Vec<String>,

    /// Due date in format YYYY-MM-DD (example: --due 2025-12-31)
    #[arg(long, value_name = "DATE", value_parser = clap::value_parser!(NaiveDate))]
    due: Option<NaiveDate>,
}

/// Creates a new task with the given parameters.
///
/// The task is initially marked as not completed and the creation date
/// is set to the current date.
///
/// # Arguments
///
/// * `text` - The task description
/// * `priority` - Priority level of the task
/// * `tags` -  List of tags for categorization
/// * `due_date` - Optional due date for the task
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

    /// Marks this task as completed.
    fn mark_done(&mut self) {
        self.completed = true;
    }

    /// Marks this task as pending (not completed).
    fn mark_undone(&mut self) {
        self.completed = false;
    }

    /// Checks if this is overdue.
    ///
    /// A task is considered overdue if it has a due in the past
    /// and is not yet completed.
    fn is_overdue(&self) -> bool {
        if let Some(due) = self.due_date {
            let today = Local::now().naive_local().date();
            due < today && !self.completed
        } else {
            false
        }
    }

    /// Checks if this task is due soon (within the specified number of days).
    ///
    /// # Arguments
    ///
    /// * `days` - Number of days to look ahead
    ///
    /// # Returns
    ///
    /// `true` if the task is due within the specified number of days and
    /// is not yet completed, `false` otherwise.
    fn is_due_soon(&self, days: i64) -> bool {
        if let Some(due) = self.due_date {
            let today = Local::now().naive_local().date();
            let days_until = (due - today).num_days();
            days_until >= 0 && days_until <= days && !self.completed
        } else {
            false
        }
    }

    /// Checks if this task matches the given status filter.
    fn matches_status(&self, status: StatusFilter) -> bool {
        match status {
            StatusFilter::Pending => !self.completed,
            StatusFilter::Done => self.completed,
            StatusFilter::All => true,
        }
    }

    /// Checks if this task matches the given due date filter.
    fn matches_due_filter(&self, filter: DueFilter) -> bool {
        match filter {
            DueFilter::Overdue => self.is_overdue(),
            DueFilter::Soon => self.is_due_soon(7),
            DueFilter::WithDue => self.due_date.is_some(),
            DueFilter::NoDue => self.due_date.is_none(),
        }
    }
}

impl Priority {
    /// Returns the sort order for this priority level.
    ///
    /// Lower numbers indicate higher priority.
    fn order(&self) -> u8 {
        match self {
            Priority::High => 0,
            Priority::Medium => 1,
            Priority::Low => 2,
        }
    }

    /// Returns a colored single-letter representation of this priority.
    ///
    /// - High: Red 'H'
    /// - Medium: Yellow 'M'
    /// - Low: Green 'L'
    fn letter(&self) -> ColoredString {
        match self {
            Priority::High => "H".red(),
            Priority::Medium => "M".yellow(),
            Priority::Low => "L".green(),
        }
    }
}

/// Loads tasks from the todos.json file.
///
/// If the file doesn't exist, returns an empty vector.
/// If the file exists but is corrupted, returns an error.
///
/// # Errors
///
/// Returns an error if:
/// - The file exists but cannot be read
/// - The file contains invalid JSON
fn load_tasks() -> Result<Vec<Task>> {
    let path = get_data_file_path()?;

    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content)
            .context("Failed to parse todos.json - file may be corrupted"),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e).context(format!(
            "Failed to read todos.json from: {}",
            path.display()
        )),
    }
}

/// Saves tasks to the todos.json file.
///
/// The tasks are serialized to pretty-printed JSON format.
///
/// # Errors
///
/// Returns an error if:
/// - The tasks cannot be serialized to JSON
/// - The file cannot be written (e.g., permission issues)
fn save_tasks(tasks: &[Task]) -> Result<()> {
    let path = get_data_file_path()?;

    let json = serde_json::to_string_pretty(tasks).context("Failed to serialize tasks to JSON")?;

    fs::write(&path, json).context(format!(
        "Failed to write to {} - check file permission",
        path.display()
    ))?;

    Ok(())
}

/// Validates that a task ID is within the valid range.
///
/// # Arguments
///
/// * `id` - The task ID to validate (1-based indexing)
/// * `max` - The maximum valid ID (total number of tasks)
///
/// # Errors
///
/// Returns `TodoError::InvalidTaskId` if the ID is 0 or greater than max.
fn validate_task_id(id: usize, max: usize) -> Result<(), TodoError> {
    if id == 0 || id > max {
        return Err(TodoError::InvalidTaskId { id, max });
    }
    Ok(())
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

/// Generates a human-readable due date description.
///
/// # Returns
///
/// A string like "in 3 days", "due today", or "late 2 days"
fn get_due_text(task: &Task) -> String {
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
fn get_due_colored(task: &Task, text: &str) -> ColoredString {
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

/// Displays a single task in tabular format.
///
/// # Arguments
///
/// * `number` - The task number (1-based index)
/// * `task` - The task to display
/// * `task_width` - Column width for task text
/// * `tags_width` - Column width for tags
fn display_task_tabular(number: usize, task: &Task, task_width: usize, tags_width: usize) {
    let number_str = format!("{:>3}", number);
    let letter = task.priority.letter();
    let checkbox = if task.completed {
        "".green()
    } else {
        "".bright_white()
    };

    // Truncate task text if it exceeds the column width
    let task_text = if task.text.len() > task_width {
        format!("{}...", &task.text[..task_width - 3])
    } else {
        task.text.to_owned()
    };

    // Format tags for display
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

    // Apply different styling for completed vs pending tasks
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

/// Displays a list of tasks in a formatted table with statistics.
///
/// # Arguments
///
/// * `tasks` - List of (task_number, task) tuples to display
/// * `title` - Title to show above the table
fn display_lists(tasks: &[(usize, &Task)], title: &str) {
    println!("\n{}:\n", title);

    let (task_width, tags_width, due_width) = calculate_column_widths(tasks);

    // Print table header
    print!("{:>4} ", "ID".dimmed());
    print!(" {} ", "P".dimmed());
    print!(" {} ", "S".dimmed());
    print!("{:<width$}", "Task".dimmed(), width = task_width);
    print!("  {:<width$}", "Tags".dimmed(), width = tags_width);
    println!("  {}", "Due".dimmed());

    let total_width = task_width + tags_width + due_width + 19;

    println!("{}", "─".repeat(total_width).dimmed());

    // Print each task
    let mut completed = 0;
    let total = tasks.len();

    for (number, task) in tasks {
        display_task_tabular(*number, task, task_width, tags_width);

        if task.completed {
            completed += 1;
        }
    }

    // Print footer with statistics
    println!("\n{}", "─".repeat(total_width).dimmed());

    let percentage = if total > 0 {
        (completed as f32 / total as f32 * 100.0) as u32
    } else {
        0
    };

    let stats = format!("{} of {} completed ({}%)", completed, total, percentage);

    // Color-code the statistics based on completion percentage
    if percentage == 100 {
        println!("{}", stats.green().bold());
    } else if percentage >= 50 {
        println!("{}", stats.yellow());
    } else {
        println!("{}", stats.red());
    }

    println!();
}

/// Returns the path to the todos.json file in the user's config directory.
///
/// Creates the config directory if it doesn't exist.
///
/// # Platform-specific locations:
/// - Linux: `~/.config/todo-cli/todos.json`
/// - macOS: `~/Library/Application Support/todo-cli/todos.json`
/// - Windows: `C:\Users\{user}\AppData\Roaming\todo-cli\todos.json`
fn get_data_file_path() -> Result<PathBuf> {
    let project_dirs =
        ProjectDirs::from("", "", "todo-cli").context("Failed to determine project directories")?;

    let data_dir = project_dirs.data_dir();

    // Create directory if it doesn't exist
    fs::create_dir_all(data_dir).context(format!(
        "Failed to create data directory: {}",
        data_dir.display()
    ))?;

    let mut path = data_dir.to_path_buf();
    path.push("todos.json");

    Ok(path)
}

/// Prompts the user for confirmation.
///
/// # Arguments
///
/// * `message` - The confirmation message to display
///
/// # Returns
///
/// `true` if the user confirms (y/Y/yes), `false` otherwise
fn confirm(message: &str) -> Result<bool> {
    use std::io::{self, Write};

    print!("{} ", message.yellow());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let response = input.trim().to_lowercase();
    Ok(matches!(response.as_str(), "y" | "yes"))
}

/// Main application logic dispatcher.
///
/// This function processes the parsed CLI arguments and executes the
/// appropriate command. It handles all the core functionality of the
/// todo application including adding, listing, completing, and managing tasks.
///
/// # Errors
///
/// Returns an error if:
/// - File I/O operations fail
/// - Task validation fails
/// - No tasks match the specified filters
fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Add(args) => {
            let task = Task::new(args.text, args.priority, args.tag, args.due);
            let mut tasks = load_tasks()?;
            tasks.push(task);
            save_tasks(&tasks)?;
            println!("{}", "✓ Task added".green())
        }

        Commands::List {
            status,
            priority,
            due,
            sort,
            tag,
        } => {
            let all_tasks = load_tasks()?;

            // Create indexed view of tasks (1-based numbering)
            let mut indexed_tasks: Vec<(usize, &Task)> = all_tasks
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
                        indexed_tasks
                            .sort_by(|(_, a), (_, b)| a.priority.order().cmp(&b.priority.order()));
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
                (StatusFilter::Pending, Some(Priority::High), None) => {
                    "High priority pending tasks"
                }
                (StatusFilter::Pending, Some(Priority::Medium), None) => {
                    "Medium priority pending tasks"
                }
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
        }

        Commands::Done { id } => {
            let mut tasks = load_tasks()?;
            validate_task_id(id, tasks.len())?;
            let index = id - 1;

            if tasks[index].completed {
                return Err(TodoError::TaskAlreadyInStatus {
                    id,
                    status: "completed".to_owned(),
                }
                .into());
            }

            tasks[index].mark_done();
            save_tasks(&tasks)?;
            println!("{}", "✓ Task marked as completed".green());
        }

        Commands::Undone { id } => {
            let mut tasks = load_tasks()?;
            validate_task_id(id, tasks.len())?;
            let index = id - 1;

            if !tasks[index].completed {
                return Err(TodoError::TaskAlreadyInStatus {
                    id,
                    status: "pending".to_owned(),
                }
                .into());
            }

            tasks[index].mark_undone();
            save_tasks(&tasks)?;
            println!("{}", "✓ Task unmarked".yellow());
        }

        Commands::Remove { id, yes } => {
            let mut tasks = load_tasks()?;
            validate_task_id(id, tasks.len())?;

            let index = id - 1;
            let task_text = &tasks[index].text;

            if !yes {
                println!(
                    "\n{} {}",
                    "Remove task:".red().bold(),
                    task_text.bright_white()
                );

                if !confirm("Are you sure? [y/N]:")? {
                    println!("{}", "Removal cancelled.".yellow());
                    return Ok(());
                }
            }

            let removed_task = tasks.remove(index);
            save_tasks(&tasks)?;
            println!("{} {}", "✓ Task removed:".red(), removed_task.text.dimmed());
        }

        Commands::Edit {
            id,
            text,
            priority,
            tag,
            due,
            clear_due,
            clear_tags,
        } => {
            let mut tasks = load_tasks()?;
            validate_task_id(id, tasks.len())?;

            let index = id - 1;
            let task = &mut tasks[index];

            let mut changes = Vec::new();

            // Update text if provided AND different
            if let Some(new_text) = text {
                if new_text.trim().is_empty() {
                    return Err(anyhow::anyhow!("Task text cannot be empty"));
                }
                if task.text != new_text {
                    task.text = new_text.clone();
                    changes.push(format!("text → {}", new_text.bright_white()));
                }
            }

            // Update priority if provided AND different
            if let Some(new_priority) = priority
                && task.priority != new_priority
            {
                task.priority = new_priority;
                changes.push(format!("priority → {}", new_priority.letter()));
            }

            // Update tags
            if clear_tags {
                if !task.tags.is_empty() {
                    task.tags.clear();
                    changes.push("tags → cleared".dimmed().to_string());
                }
            } else if !tag.is_empty() && task.tags != tag {
                task.tags = tag;
                changes.push(format!("tags → [{}]", task.tags.join(", ").cyan()));
            }

            // Update due date
            if clear_due {
                if task.due_date.is_some() {
                    task.due_date = None;
                    changes.push("due date → cleared".dimmed().to_string());
                }
            } else if let Some(new_due) = due
                && task.due_date != Some(new_due)
            {
                task.due_date = Some(new_due);
                changes.push(format!("due date → {}", new_due.to_string().cyan()));
            }

            // Check if anything was actually changed
            if changes.is_empty() {
                println!(
                    "{}",
                    "No changes made (values are already set to the specified values).".yellow()
                );
                return Ok(());
            }

            save_tasks(&tasks)?;

            println!("{} Task #{} updated:", "✓".green(), id);
            for change in changes {
                println!("  • {}", change);
            }
        }

        Commands::Clear { yes } => {
            let path = get_data_file_path()?;

            if !path.exists() {
                println!("No tasks to remove");
                return Ok(());
            }

            let tasks = load_tasks()?;
            let count = tasks.len();

            if !yes {
                println!(
                    "\n{} {} tasks will be permanently deleted!",
                    "WARNING:".red().bold(),
                    count
                );

                if !confirm("Type 'yes' to confirm:")? {
                    println!("{}", "Clear cancelled.".yellow());
                    return Ok(());
                }
            }

            fs::remove_file(&path).context(format!("Failed to remove {}", path.display()))?;
            println!("{}", "✓ All tasks have been removed".red().bold());
        }

        Commands::Search { query, tag } => {
            let tasks = load_tasks()?;

            // Perform case-insensitive search on task text
            let mut results: Vec<(usize, &Task)> = tasks
                .iter()
                .enumerate()
                .filter(|(_, task)| task.text.to_lowercase().contains(&query.to_lowercase()))
                .map(|(i, task)| (i + 1, task))
                .collect();

            // Apply tag filter if specified
            if let Some(tag_name) = &tag {
                results.retain(|(_, task)| task.tags.contains(tag_name));
            }

            if results.is_empty() {
                return Err(TodoError::NoSearchResults(query).into());
            } else {
                display_lists(&results, &format!("Search results for \"{}\"", query));
            }
        }

        Commands::Tags => {
            let tasks = load_tasks()?;

            // Collect all unique tags
            let mut all_tags: Vec<String> = Vec::new();
            for task in &tasks {
                for tag in &task.tags {
                    if !all_tags.contains(tag) {
                        all_tags.push(tag.to_owned());
                    }
                }
            }

            if all_tags.is_empty() {
                return Err(TodoError::NoTagsFound.into());
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

        Commands::Info => {
            let path = get_data_file_path()?;
            let exists = path.exists();

            println!("\n{}\n", "Todo-List Information".bold());
            println!("{} {}", "Data file:".dimmed(), path.display());
            println!(
                "{} {}",
                "Status:".dimmed(),
                if exists {
                    "exists ✓".green()
                } else {
                    "not created yet".yellow()
                }
            );

            if exists {
                let metadata = fs::metadata(&path)?;
                let size = metadata.len();
                println!("{} {} bytes", "Size:".dimmed(), size);
            }

            println!();
        }
    }

    Ok(())
}
