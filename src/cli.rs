use chrono::NaiveDate;
use clap::{Args, Parser, Subcommand};

use crate::models::{DueFilter, Priority, SortBy, StatusFilter};

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
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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
pub struct AddArgs {
    /// Task description
    #[arg(value_name = "DESCRIPTION")]
    pub text: String,

    /// Task priority level
    #[arg(long, value_enum, default_value_t = Priority::Medium)]
    pub priority: Priority,

    /// Add tags (can be repeated: -t work -t urgent)
    #[arg(long, short = 't', value_name = "TAG")]
    pub tag: Vec<String>,

    /// Due date in format YYYY-MM-DD (example: --due 2026-02-09)
    #[arg(long, value_name = "DATE", value_parser = clap::value_parser!(NaiveDate))]
    pub due: Option<NaiveDate>,
}
