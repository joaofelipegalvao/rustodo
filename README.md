# Todo CLI 🦀

> A minimalist command-line task manager built to learn Rust

A simple, colorful, and functional task manager developed to learn Rust in practice, focusing on CLI design, file manipulation, error handling, type safety, and visual UX.

## Features

- **Modular Refactoring** - Transform monolithic code into professional architecture
- **Edit Command** - Modify existing tasks without deleting them
- **Interactive Confirmation** - Safe prompts for destructive operations (remove/clear)
- **Global Data Directory** - OS-appropriate storage (XDG on Linux, Application Support on macOS, AppData on Windows)
- **Error Handling with `anyhow` and `thiserror`** - Professional error messages with context chains
- **Professional CLI with Clap** - Auto-generated help, type-safe parsing, shell completions
- **Type-safe architecture** with structs and enums
- **Tags system** for task categorization
- **Priority system** (high, medium, low)
- **Due dates** with automatic validation
- **Advanced search and filters**
- **Progress statistics**
- **Colorful interface**
- **JSON storage** with automatic serialization
- **Fast and lightweight**

## Quick Start

```bash
# Install
git clone https://github.com/joaofelipegalvao/todo-cli
cd todo-cli
cargo install --path .

# Use
todo add "Learn Rust" --priority high --tag programming --tag learning --due 2026-02-20
todo list --status pending --sort priority
todo done 1
todo tags

# Edit tasks
todo edit 1 --text "Learn Rust properly" --priority high
todo edit 2 --due 2026-03-01

# Find where your data is stored
todo info
```

## Data Storage

Tasks are automatically saved to platform-specific locations:

- **Linux:** `~/.local/share/todo-cli/todos.json`
- **macOS:** `~/Library/Application Support/todo-cli/todos.json`
- **Windows:** `%APPDATA%\todo-cli\todos.json`

Use `todo info` to see your exact data file location.

## Commands

```bash
todo add <description> [options]              # Add a new task
todo edit <id> [options]                      # Edit an existing task
todo list [options]                           # List and filter tasks
todo search <query> [--tag <n>]               # Search tasks by text
todo done <id>                                # Mark task as completed
todo undone <id>                              # Mark task as pending
todo remove <id> [--yes]                      # Remove a task (with confirmation)
todo clear [--yes]                            # Remove all tasks (with confirmation)
todo tags                                     # List all tags with counts
todo info                                     # Show data file location
```

### Add Command

```bash
todo add "Task description" [options]

Options:
  --priority <PRIORITY>   Task priority: high, medium (default), low
  -t, --tag <TAG>        Add tags (repeatable: -t work -t urgent)
  --due <DATE>           Due date in YYYY-MM-DD format
```

**Examples:**

```bash
todo add "Deploy to production" --priority high --tag work --due 2026-02-15
todo add "Buy groceries" -t personal -t shopping
```

### Edit Command

```bash
todo edit <id> [options]

Options:
  --text <TEXT>          New task description
  --priority <PRIORITY>  New priority (high|medium|low)
  -t, --tag <TAG>       Replace tags (repeatable)
  --due <DATE>          New due date (YYYY-MM-DD)
  --clear-due           Remove due date
  --clear-tags          Remove all tags
```

**Examples:**

```bash
# Fix a typo
todo edit 5 --text "Updated description"

# Change priority and due date
todo edit 3 --priority high --due 2026-03-15

# Replace tags
todo edit 1 --tag work --tag urgent

# Clear due date
todo edit 2 --clear-due

# Multiple changes at once
todo edit 3 --text "New text" --priority low --due 2026-04-01
```

**What's preserved:**

- ✅ Task ID (stays the same)
- ✅ Creation date (`created_at`)
- ✅ Completion status

### List Command

```bash
todo list [options]

Filters:
  --status <STATUS>      Filter by status: all (default), pending, done
  --priority <PRIORITY>  Filter by priority: high, medium, low
  --due <DUE_FILTER>    Filter by due date: overdue, soon, with-due, no-due
  --tag <TAG>           Filter by tag name

Sorting:
  -s, --sort <SORT_BY>  Sort by: priority, due, created
```

**Examples:**

```bash
# Pending high-priority tasks, sorted by due date
todo list --status pending --priority high --sort due

# Tasks due soon
todo list --due soon

# Combine multiple filters
todo list --status pending --priority high --tag work --sort priority
```

### Remove & Clear Commands

Both commands now prompt for confirmation to prevent accidental deletion:

```bash
# Remove with confirmation
$ todo remove 5
Remove task 'Important meeting'? [y/N]: y
✓ Task removed: Important meeting

# Skip confirmation (for scripts)
$ todo remove 5 --yes
$ todo remove 5 -y

# Clear with warning
$ todo clear
WARNING: 10 tasks will be permanently deleted!
Are you sure? [y/N]: y
✓ All tasks have been removed

# Skip confirmation
$ todo clear --yes
```

### Info Command

```bash
# Show data file location and status
todo info
```

**Example output:**

```
Todo-List Information

Data file: /home/user/.local/share/todo-cli/todos.json
Status: exists ✓
Size: 1245 bytes
```

### Command Aliases

For faster typing:

```bash
todo a "Task"          # alias for 'add'
todo e 1 --text "New"  # alias for 'edit'
todo ls                # alias for 'list'
todo rm 3              # alias for 'remove'
todo delete 3          # also works for 'remove'
```

### Getting Help

```bash
todo --help            # Show all commands
todo add --help        # Help for specific command
todo edit --help       # Edit command options
todo list --help       # Detailed filtering options
```

## Documentation

- **[Complete Guide](docs/GUIDE.md)** - All commands and examples
- **[Complete Documentation](https://joaofelipegalvao.github.io/todo-cli/)** - Full learning journey, concepts, and examples
- **[Changelog](CHANGELOG.md)** - Version history

---

## Educational Project

This project was developed as a Rust learning exercise, documenting each incremental step. Each version represents a learning milestone:

| Version | Feature | Key Concepts |
|---------|---------|--------------|
| v0.1.0 | Basic CLI | `OpenOptions`, `match`, `?` operator |
| v0.2.0 | Mark as done | `.map()`, `.collect()`, `Vec<String>` |
| v0.8.0 | Priorities + Filters | `Option<T>`, pattern matching, pipeline |
| v1.0.0 | Search + Refactoring | Atomic functions, separation of concerns |
| v1.2.0 | Type-safe structs | `struct`, `enum`, `impl`, derive macros |
| v1.3.0 | JSON serialization | `serde`, automatic serialization, 91% I/O reduction |
| v1.4.0 | Tags system | `Vec<String>`, `.retain()`, bug fixes |
| v1.5.0 | Due dates | `chrono`, `NaiveDate`, date validation |
| v1.6.0 | Professional CLI | `clap`, derive macros, type-safe enums, auto-help |
| v1.7.0 | Error Handling | `anyhow`, `thiserror`, error chains |
| v1.8.0 | Global Data Directory | `directories` crate, `PathBuf`, XDG compliance |
| v1.9.0 | Edit + Confirmations + Refactoring | `todo edit`, let-chains, `confirm()`, `--yes`, `TableLayout` struct |
| v2.0.0 | Modular Refactoring | Modules, re-exports, separation of concerns, Command pattern | ~95 main.rs (was 1200) |

[See full evolution →](CHANGELOG.md)

### For Students

- Clone, read the code, explore version diffs  
- Each tag documents what was learned  
- Perfect for understanding CLI design in Rust
- Study the evolution from manual parsing to professional CLI with Clap
- Learn cross-platform development with OS-appropriate storage

### For End Users

- The code works but lacks comprehensive automated tests  
- May have unhandled edge cases  
- Use at your own risk, or contribute improvements!  

## Roadmap

### Completed ✅

- Basic CRUD operations
- Priority system with visual indicators
- Advanced filters and search
- Sorting by priority and due date
- Optimized pipeline architecture
- Type-safe architecture with structs/enums
- JSON serialization with serde
- Tags system for categorization
- Due dates with automatic validation
- Professional CLI with Clap framework
- Type-safe filtering with enums
- Auto-generated help and documentation
- Command aliases for productivity
- Professional error handling with anyhow + thiserror
- Global data directory with OS-appropriate storage
- Edit command for modifying existing tasks
- Interactive confirmation for destructive operations
- TableLayout architecture for cleaner display code
- Modular Refactoring for a professional structure

### Planned 🚀

- Recurring tasks
- Subtasks/nested tasks
- Export/import commands
- Shell completions (bash, zsh, fish)
- Unit tests
- TUI (Terminal User Interface)

## Contributing

Contributions are welcome! This is a learning project, so feel free to:

- **Report bugs** - Open an issue with details
- **Suggest features** - Share your ideas
- **Improve documentation** - Fix typos, add examples
- **Submit PRs** - Fix bugs or add features
- **Share learning insights** - Add to the wiki

### Development

```bash
# Clone and build
git clone https://github.com/joaofelipegalvao/todo-cli
cd todo-cli
cargo build

# Run tests (when available)
cargo test

# Run with logging
RUST_LOG=debug cargo run -- add "Test task"

# Check code quality
cargo clippy
cargo fmt --check
```

## License

**MIT License** - Educational project developed to learn Rust 🦀

See [LICENSE](https://github.com/joaofelipegalvao/todo-cli/blob/main/LICENSE) for full details.

---

**Built with ❤️ to learn Rust - Each commit represents a step in the learning journey**
