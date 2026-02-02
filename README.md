# Todo CLI 🦀

> A minimalist command-line task manager built to learn Rust

A simple, colorful, and functional task manager developed to learn Rust in practice, focusing on CLI design, file manipulation, error handling, type safety, and visual UX.

## Features

- **Type-safe architecture** with structs and enums
- **Tags system** for task categorization
- **Priority system** (high, medium, low)
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
todo add "Learn Rust" --high --tag programming --tag learning
todo list --pending --sort
todo done 1
todo tags
```

## Commands

```bash
todo add <task> [--high|--medium|--low] [--tag <n>]...  # Add task (default: medium priority)
todo list [flags]                                        # List tasks
todo search <term> [--tag <n>]                          # Search tasks
todo done <number>                                       # Mark as completed
todo undone <number>                                     # Unmark task
todo remove <number>                                     # Remove task
todo clear                                               # Remove all tasks
todo tags                                                # List all tags
```

### List Flags

- `--pending` / `--done` - Filter by status
- `--high` / `--medium` / `--low` - Filter by priority
- `--tag <n>` - Filter by tag
- `--sort` - Sort by priority
- Combine flags: `todo list --pending --high --tag work --sort`

## Documentation

- **[Complete Guide](docs/GUIDE.md)** - All commands and examples
- **[Learning Journey](docs/LEARNING.md)** - How this project evolved, version by version
- **[Changelog](CHANGELOG.md)** - Version history

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

[See full evolution →](docs/LEARNING.md)

### For Students

- Clone, read the code, explore version diffs  
- Each tag documents what was learned  
- Perfect for understanding CLI design in Rust
- Study the evolution from strings to type-safe structs to JSON

### For End Users

- The code works but lacks comprehensive automated tests  
- May have unhandled edge cases  
- Use at your own risk, or contribute improvements!  

## Roadmap

### Completed ✅

- Basic CRUD operations
- Priority system with visual indicators
- Advanced filters and search
- Sorting by priority
- Optimized pipeline architecture
- **Type-safe architecture with structs/enums**
- **JSON serialization with serde**
- **Tags system for categorization**
- **Critical bug fix: preserved numbering in filtered views**

### Planned 🚀

- Due dates with `chrono`
- Edit command
- Recurring tasks
- Subtasks/nested tasks
- Export/import commands
- Unit tests
- TUI (Terminal User Interface)

## Contributing

Contributions are welcome! This is a learning project, so feel free to:

- **Report bugs** - Open an issue with details
- **Suggest features** - Share your ideas
- **Improve documentation** - Fix typos, add examples
- **Submit PRs** - Fix bugs or add features
- **Share learning insights** - Add to LEARNING.md

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
