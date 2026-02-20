use clap::{Args, Parser, Subcommand};

use crate::models::{DueFilter, Priority, Recurrence, RecurrenceFilter, SortBy, StatusFilter};

#[derive(Parser)]
#[command(name = "todo-list")]
#[command(author = "github.com/joaofelipegalvao")]
#[command(version = "2.4.0")]
#[command(about = "A modern, powerful task manager built with Rust", long_about = None)]
#[command(after_help = "EXAMPLES:\n    \
    # Add a high priority task with a natural language date\n    \
    todo add \"Complete Rust project\" --priority high --tag work --due \"next friday\"\n\n    \
    # Add a task due in 3 days\n    \
    todo add \"Review PR\" --due \"in 3 days\"\n\n    \
    # Add a task with strict date format\n    \
    todo add \"Project deadline\" --due 2026-03-15\n\n    \
    # Add a recurring task\n    \
    todo add \"Weekly review\" --due \"next monday\" --recurrence weekly\n\n    \
    # List pending high priority tasks\n    \
    todo list --status pending --priority high\n\n    \
    # List only daily recurring tasks\n    \
    todo list --recurrence daily\n\n    \
    # List overdue tasks sorted by due date\n    \
    todo list --due overdue --sort due\n\n    \
    # Edit task due date with natural language\n    \
    todo edit 3 --due \"next monday\"\n\n    \
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
        Use --recurrence to make the task repeat automatically when completed.")]
    Add(AddArgs),

    /// List and filter tasks
    #[command(visible_alias = "ls")]
    #[command(
        long_about = "List and filter tasks with powerful filtering options\n\n\
        Display your tasks with filtering and sorting capabilities.\n\
        All filters can be combined to find exactly what you need.\n\n\
        Examples:\n  \
        todo list --recurrence daily           # Only daily tasks\n  \
        todo list --recurrence any             # Any recurring task\n  \
        todo list --recurrence none            # Non-recurring tasks\n  \
        todo list --status pending --recur weekly --sort due"
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

        /// Filter by recurrence pattern (daily, weekly, monthly, any, none)
        #[arg(long, short = 'r', value_enum)]
        recurrence: Option<RecurrenceFilter>,
    },

    /// Mark a task as completed
    #[command(visible_alias = "complete")]
    #[command(long_about = "Mark a task as completed\n\n\
        Marks the specified task as done. The task will be shown with a ✓ symbol\n\
        and appear in green when listing tasks.\n\n\
        If the task is recurring, the next occurrence will be automatically created.")]
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
        Only specify the fields you want to change.\n\n\
        Due dates accept natural language or YYYY-MM-DD:\n  \
        todo edit 3 --due tomorrow\n  \
        todo edit 3 --due \"next monday\"\n  \
        todo edit 3 --due \"in 5 days\"\n  \
        todo edit 3 --due 2026-04-01\n\n\
        Tag operations:\n  \
        todo edit 1 --add-tag urgent,critical     # Add multiple tags\n  \
        todo edit 1 --remove-tag work,team        # Remove multiple tags\n  \
        todo edit 1 --add-tag urgent --remove-tag team  # Combine operations")]
    Edit {
        /// Task ID number
        #[arg(value_name = "ID")]
        id: usize,

        /// New task description
        #[arg(long)]
        text: Option<String>,

        /// New priority level
        #[arg(long, value_enum)]
        priority: Option<Priority>,

        /// Add tags (comma-separated or repeat flag)
        /// Examples: --add-tag work,urgent  OR  --add-tag work --add-tag urgent
        #[arg(long, value_delimiter = ',')]
        add_tag: Vec<String>,

        /// Remove tags (comma-separated or repeat flag)
        /// Examples: --remove-tag work,team  OR  --remove-tag work --remove-tag team
        #[arg(long, value_delimiter = ',')]
        remove_tag: Vec<String>,

        /// Due date — accepts natural language or YYYY-MM-DD format.
        /// Examples: tomorrow, "next friday", "in 3 days", "in 2 weeks", 2026-02-20
        #[arg(long, value_name = "DATE|EXPRESSION")]
        due: Option<String>,

        /// Remove due date
        #[arg(long, conflicts_with = "due")]
        clear_due: bool,

        /// Remove all tags
        #[arg(long, conflicts_with_all = ["add_tag", "remove_tag"])]
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

        #[arg(long, value_enum, default_value_t = StatusFilter::All)]
        status: StatusFilter,
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

    /// Set or change recurrence pattern for a task
    #[command(long_about = "Set or change recurrence pattern for a task\n\n\
        Makes a task repeat automatically when marked as done.\n\
        The task must have a due date to use recurrence.\n\n\
        When you complete a recurring task, a new task will be created\n\
        with the next due date based on the pattern (daily, weekly, monthly).")]
    Recur {
        /// Task ID number
        #[arg(value_name = "ID")]
        id: usize,

        /// Recurrence pattern (daily, weekly, monthly)
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
        /// Task ID number
        #[arg(value_name = "ID")]
        id: usize,
    },
}

#[derive(Args)]
pub struct AddArgs {
    /// Task description
    #[arg(value_name = "DESCRIPTION")]
    pub text: String,

    /// Task priority level
    #[arg(long, value_enum, default_value_t = Priority::Medium)]
    pub priority: Priority,

    /// Add tags (comma-separated or repeat flag)
    /// Examples: -t work,urgent  OR  -t work -t urgent
    #[arg(long, short = 't', value_name = "TAG", value_delimiter = ',')]
    pub tag: Vec<String>,

    /// Due date — accepts natural language or YYYY-MM-DD format.
    /// Examples: tomorrow, "next friday", "in 3 days", "in 2 weeks", 2026-02-20
    #[arg(long, value_name = "DATE|EXPRESSION")]
    pub due: Option<String>,

    /// Recurrence pattern (daily, weekly, monthly)
    /// Requires a due date to be set
    #[arg(long, value_enum)]
    pub recurrence: Option<Recurrence>,
}
