use clap::{Args, Parser, Subcommand};

use crate::models::{DueFilter, Priority, Recurrence, RecurrenceFilter, SortBy, StatusFilter};

#[derive(Parser)]
#[command(name = "todo-list")]
#[command(author = "github.com/joaofelipegalvao")]
#[command(version = "2.7.0")]
#[command(about = "A modern, powerful task manager built with Rust", long_about = None)]
#[command(after_help = "EXAMPLES:\n    \
    # Add a task to a project with a natural language date\n    \
    todo add \"Fix login bug\" --project \"Backend\" --priority high --due \"next friday\"\n\n    \
    # Add a task due in 3 days\n    \
    todo add \"Review PR\" --due \"in 3 days\"\n\n    \
    # Add a task with strict date format\n    \
    todo add \"Project deadline\" --due 2026-03-15\n\n    \
    # List all tasks in a project\n    \
    todo list --project \"Backend\"\n\n    \
    # List pending tasks in a project, sorted by due date\n    \
    todo list --project \"Backend\" --status pending --sort due\n\n    \
    # List all projects\n    \
    todo projects\n\n    \
    # Search within a project\n    \
    todo search \"bug\" --project \"Backend\"\n\n    \
    # Edit a task project\n    \
    todo edit 3 --project \"Frontend\"\n\n    \
    # Remove a task from its project\n    \
    todo edit 3 --clear-project\n\n    \
    # Add a recurring task in a project\n    \
    todo add \"Weekly review\" --project \"Management\" --due \"next monday\" --recurrence weekly\n\n    \
    # List overdue tasks sorted by due date\n    \
    todo list --due overdue --sort due\n\n    \
    # Search for tasks\n    \
    todo search rust\n\n    \
    # Mark task as completed (auto-creates next recurrence)\n    \
    todo done 3\n\n    \
    # Set recurrence pattern\n    \
    todo recur 5 daily\n\n\
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
        tags, and due date. Tasks are saved immediately to todos.json.\n\n\
        Due dates accept both natural language and strict YYYY-MM-DD format:\n  \
        todo add \"Meeting\" --due tomorrow\n  \
        todo add \"Deploy\" --due \"next friday\"\n  \
        todo add \"Report\" --due \"in 3 days\"\n  \
        todo add \"Deadline\" --due 2026-03-15\n\n\
        Assign to a project:\n  \
        todo add \"Fix bug\" --project \"Backend\"\n  \
        todo add \"Write docs\" --project \"Documentation\" --tag work\n\n\
        Use --recurrence to make the task repeat automatically when completed.")]
    Add(AddArgs),

    /// List and filter tasks
    #[command(visible_alias = "ls")]
    #[command(
        long_about = "List and filter tasks with powerful filtering options\n\n\
        Examples:\n  \
        todo list --project \"Backend\"\n  \
        todo list --project \"Backend\" --status pending\n  \
        todo list --recurrence daily\n  \
        todo list --status pending --priority high --sort due"
    )]
    List {
        #[arg(long, value_enum, default_value_t = StatusFilter::All)]
        status: StatusFilter,
        #[arg(long, value_enum)]
        priority: Option<Priority>,
        #[arg(long, value_enum)]
        due: Option<DueFilter>,
        #[arg(long, short = 's', value_enum)]
        sort: Option<SortBy>,
        #[arg(long, short = 't')]
        tag: Option<String>,
        /// Filter by project name
        #[arg(long, short = 'p')]
        project: Option<String>,
        #[arg(long, short = 'r', value_enum)]
        recurrence: Option<RecurrenceFilter>,
    },

    /// Mark a task as completed
    #[command(visible_alias = "complete")]
    Done {
        #[arg(value_name = "ID")]
        id: usize,
    },

    /// Mark a completed task as pending
    #[command(visible_alias = "undo")]
    Undone {
        #[arg(value_name = "ID")]
        id: usize,
    },

    /// Remove a task permanently
    #[command(visible_aliases = ["rm", "delete"])]
    Remove {
        #[arg(value_name = "ID")]
        id: usize,
        #[arg(long, short = 'y')]
        yes: bool,
    },

    /// Edit an existing task
    #[command(visible_alias = "e")]
    #[command(long_about = "Edit an existing task\n\n\
        Modify task properties like text, priority, tags, or due date.\n\
        Only specify the fields you want to change.\n\n\
        Due dates accept natural language or YYYY-MM-DD:\n  \
        todo edit 3 --due tomorrow\n  \
        todo edit 3 --due \"next monday\"\n  \
        todo edit 3 --due \"in 5 days\"\n  \
        todo edit 3 --due 2026-04-01\n\n\
        Tag operations:\n  \
        todo edit 1 --add-tag urgent,critical     # Add multiple tags\n  \
        todo edit 1 --remove-tag work,team        # Remove multiple tags\n  \
        todo edit 1 --add-tag urgent --remove-tag team  # Combine operations\n\n\
        Project operations:\n  \
        todo edit 3 --project \"Backend\"   # Assign to a project\n  \
        todo edit 3 --clear-project         # Remove from project")]
    Edit {
        #[arg(value_name = "ID")]
        id: usize,

        /// New task description
        #[arg(long)]
        text: Option<String>,
        #[arg(long, value_enum)]
        priority: Option<Priority>,
        /// Add tags (comma-separated or repeat flag)
        #[arg(long, value_delimiter = ',')]
        add_tag: Vec<String>,
        /// Remove tags (comma-separated or repeat flag)
        #[arg(long, value_delimiter = ',')]
        remove_tag: Vec<String>,

        /// Assign task to a project
        #[arg(long, short = 'p', conflicts_with = "clear_project")]
        project: Option<String>,
        /// Remove task from its current project
        #[arg(long, conflicts_with = "project")]
        clear_project: bool,
        /// Due date — accepts natural language or YYYY-MM-DD format.
        /// Examples: tomorrow, "next friday", "in 3 days", "in 2 weeks", 2026-02-20
        #[arg(long, value_name = "DATE|EXPRESSION")]
        due: Option<String>,
        #[arg(long, conflicts_with = "due")]
        clear_due: bool,

        /// Remove all tags
        #[arg(long, conflicts_with_all = ["add_tag", "remove_tag"])]
        clear_tags: bool,
        /// Add task IDs as dependencies (repeat or space-separated)
        #[arg(long, value_name = "ID", conflicts_with = "clear_deps")]
        add_dep: Vec<usize>,
        /// Remove task IDs from dependencies
        #[arg(long, value_name = "ID", conflicts_with = "clear_deps")]
        remove_dep: Vec<usize>,
        /// Remove all dependencies from this task
        #[arg(long, conflicts_with_all = ["add_dep", "remove_dep"])]
        clear_deps: bool,
    },

    /// Clear all tasks
    #[command(visible_alias = "reset")]
    Clear {
        #[arg(long, short = 'y')]
        yes: bool,
    },

    /// Search for tasks by text content
    #[command(visible_alias = "find")]
    Search {
        #[arg(value_name = "QUERY")]
        query: String,
        #[arg(long, short = 't')]
        tag: Option<String>,
        /// Filter results by project
        #[arg(long, short = 'p')]
        project: Option<String>,
        #[arg(long, value_enum, default_value_t = StatusFilter::All)]
        status: StatusFilter,
    },

    /// Show productivity statistics and activity chart
    Stats,

    /// List all tags with task counts
    Tags,

    /// List all projects with task counts
    #[command(long_about = "List all projects used across your tasks\n\n\
        Shows each project name with the count of pending and completed tasks.\n\n\
        Use 'todo list --project <NAME>' to see tasks within a specific project.")]
    Projects,

    /// Show dependency graph for a task
    #[command(long_about = "Show the dependency graph for a task\n\n
        Displays:\n  \
        • Tasks this task depends on (with completion status)\n  \
        • Tasks that depend on this one\n  \
        • Whether the task is currently blocked\n\n\
        Examples:\n  \
        todo deps 5\n  \
        todo deps 1")]
    Deps {
        #[arg(value_name = "ID")]
        id: usize,
    },

    /// Show information about data file location
    Info,

    /// Set or change recurrence pattern for a task
    Recur {
        #[arg(value_name = "ID")]
        id: usize,
        #[arg(value_enum)]
        pattern: Recurrence,
    },

    /// Remove recurrence pattern from a task
    #[command(visible_alias = "norecur")]
    #[command(long_about = "Remove recurrence pattern from a task\n\n\
        Stops a task from repeating automatically. The task will remain\n\
        but won't create new occurrences when completed.\n\n\
        Aliases: norecur")]
    ClearRecur {
        #[arg(value_name = "ID")]
        id: usize,
    },
}

#[derive(Args)]
pub struct AddArgs {
    #[arg(value_name = "DESCRIPTION")]
    pub text: String,
    #[arg(long, value_enum, default_value_t = Priority::Medium)]
    pub priority: Priority,
    /// Add tags (comma-separated or repeat flag)
    #[arg(long, short = 't', value_name = "TAG", value_delimiter = ',')]
    pub tag: Vec<String>,
    /// Assign task to a project (e.g. --project "Backend")
    #[arg(long, short = 'p', value_name = "PROJECT")]
    pub project: Option<String>,
    /// Due date — accepts natural language or YYYY-MM-DD format.
    /// Examples: tomorrow, "next friday", "in 3 days", "in 2 weeks", 2026-02-20
    #[arg(long, value_name = "DATE|EXPRESSION")]
    pub due: Option<String>,
    /// Recurrence pattern (daily, weekly, monthly). Requires a due date.
    #[arg(long, value_enum)]
    pub recurrence: Option<Recurrence>,
    /// Task IDs this task depends on (must be completed first)
    #[arg(long, value_name = "ID")]
    pub depends_on: Vec<usize>,
}
