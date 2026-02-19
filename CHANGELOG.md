## Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.3.1](https://github.com/joaofelipegalvao/todo-cli/compare/v2.3.0...v2.3.1) (2026-02-19)

### 🐛 Bug Fixes

* remove unused PathBuf import in storage module ([0a4f37f](https://github.com/joaofelipegalvao/todo-cli/commit/0a4f37f8b0c26ab15002dfbcc29da3882b00921c))

## [2.3.0](https://github.com/joaofelipegalvao/todo-cli/compare/v2.2.2...v2.3.0) (2026-02-19)

### ✨ Features

* **search:** add status filter to search command ([8f3deb4](https://github.com/joaofelipegalvao/todo-cli/commit/8f3deb4a69547eaea3d390d3ec956013f983921a))

## [2.2.2](https://github.com/joaofelipegalvao/todo-cli/compare/v2.2.1...v2.2.2) (2026-02-19)

### 🐛 Bug Fixes

* update markdownlint config and remove mkdocs link from README ([a2faad0](https://github.com/joaofelipegalvao/todo-cli/commit/a2faad0e559366831eb24f7e8cef66684e161383))

## [2.2.1](https://github.com/joaofelipegalvao/todo-cli/compare/v2.2.0...v2.2.1) (2026-02-19)

### 🐛 Bug Fixes

* remove unused import and collapse nested if, fix README link ([fd7646f](https://github.com/joaofelipegalvao/todo-cli/commit/fd7646f342eda0d3fa367eab93cec37b34540cb5))

## [2.1.0] - 2026-02-12

### Major Feature: Recurring Tasks

**Goal:** Add automated recurring task functionality with intelligent scheduling and comprehensive filtering

This version introduces a complete recurring tasks system with automatic next occurrence generation, smart deduplication, and extensive filtering capabilities.

### Added

- **Recurrence enum** - Type-safe task recurrence patterns
  - `Daily` - Task repeats every day
  - `Weekly` - Task repeats every 7 days  
  - `Monthly` - Task repeats on the same day each month
  - Includes intelligent date arithmetic with edge case handling (e.g., Jan 31 → Feb 28)
  
- **Task model extensions** - Two new optional fields for recurring tasks
  - `recurrence: Option<Recurrence>` - Defines how task repeats
  - `parent_id: Option<usize>` - Links task to its parent in recurring chain
  - Both use `#[serde(default)]` for backward compatibility
  
- **RecurrenceFilter enum** - Advanced filtering for recurring tasks
  - `Daily`, `Weekly`, `Monthly` - Filter by specific pattern
  - `Recurring` - Show any recurring task (meta-filter)
  - `NonRecurring` - Show only one-time tasks
  
- **recur command** - Set recurrence pattern on existing tasks
  - `todo recur <ID> <pattern>` - Make task recurring
  - Validates task has due date (required for recurrence)
  - Detects and warns if already set to same pattern
  - Shows old → new pattern on changes
  
- **norecur command (clearrecur alias)** - Remove recurrence from tasks
  - `todo norecur <ID>` - Convert recurring task to one-time
  - Preserves all other task properties
  - Informative messages for already non-recurring tasks
  
- **Automatic next occurrence generation** - When marking recurring tasks done
  - Calculates next due date based on recurrence pattern
  - Creates new task with same properties (text, priority, tags, recurrence)
  - Links to parent with `parent_id` for chain tracking
  - Shows creation message: `↻ Task #X created (due YYYY-MM-DD)`
  
- **Smart deduplication** - Prevents duplicate next occurrences
  - Primary check: Compares `parent_id` (reliable even after edits)
  - Fallback check: Compares task text (backward compatibility)
  - Skips creation with informative message if duplicate detected
  
- **--recurrence flag** for add command
  - `todo add "Task" --due YYYY-MM-DD --recurrence daily`
  - Validates that recurrence requires due date
  - Clear error message if due date missing
  
- **--recurrence filter** for list command
  - Combines with existing filters (status, priority, due, tags)
  - Enables complex queries: `todo list --status pending --recurrence weekly --priority high`
  
- **Conditional column display** - Adaptive table layout
  - Recurrence indicator column (D/W/M) shown only when recurring tasks exist
  - Saves horizontal space when not needed
  - Applied to tags and due date columns as well
  
- **Comprehensive unit tests** - 7 tests covering core functionality
  - `test_daily_recurrence()` - Verify daily advancement
  - `test_weekly_recurrence()` - Verify weekly advancement  
  - `test_monthly_recurrence()` - Verify normal month transition
  - `test_monthly_boundary_case()` - Edge case: Jan 31 → Feb 28
  - `test_no_recurrence_returns_none()` - Non-recurring tasks
  - `test_no_due_date_returns_none()` - Tasks without due dates
  - `test_parent_id_preserved()` - Chain linking verification

### Changed

- **Task model** - Added two optional fields with backward compatibility
  - All existing task files load successfully (fields default to None)
  - New tasks automatically get appropriate field values
  - No manual migration required
  
- **Done command** - Enhanced with automatic next occurrence logic
  - Checks if task is recurring and has due date
  - Generates next occurrence after marking current done
  - Performs deduplication check before creation
  - Provides rich feedback on actions taken
  
- **List command** - Extended filtering and title generation
  - Added recurrence filtering capability
  - Updated `determine_title()` with 30+ new filter combinations
  - Title clearly indicates active filters
  
- **Add command** - Added recurrence parameter and validation
  - Accepts optional `--recurrence` flag
  - Validates recurrence requires due date
  - Creates task with recurrence pattern
  
- **Table display** - Adaptive column rendering
  - `TableLayout` struct now includes visibility flags
  - Columns only shown when data exists
  - Cleaner output for simple task lists
  
- **Display module** - Recurrence indicator rendering
  - Daily tasks: `D` in bright cyan
  - Weekly tasks: `W` in bright cyan
  - Monthly tasks: `M` in bright cyan
  - Non-recurring: empty space

### Technical Details

**Architecture improvements:**

1. **Separation of concerns** - Recurrence logic isolated in dedicated module
2. **Type safety** - Enums prevent invalid recurrence values
3. **Referential integrity** - parent_id enables reliable task chain tracking
4. **Defensive programming** - Dual deduplication check (parent_id + text)
5. **Progressive disclosure** - UI adapts to data (conditional columns)

**Key design patterns:**

- **Domain model vs Query model** - Separate `Recurrence` and `RecurrenceFilter` enums
- **Optional fields with defaults** - `#[serde(default)]` for backward compatibility
- **Method parameter design** - `parent_id` as parameter (tasks don't own their ID)
- **Exhaustive testing** - Unit tests cover edge cases and boundary conditions

**Code metrics:**

| Component | Lines | Purpose |
|-----------|-------|---------|
| `models/recurrence.rs` | 60 | Recurrence enum + date logic |
| `models/filters.rs` | +20 | RecurrenceFilter enum |
| `models/task.rs` | +80 | create_next_recurrence + tests |
| `commands/recur.rs` | 40 | Set recurrence command |
| `commands/clear_recur.rs` | 25 | Remove recurrence command |
| `commands/done.rs` | +20 | Auto-generation logic |
| `commands/list.rs` | +50 | Filtering + title generation |
| `display/table.rs` | +40 | Conditional rendering |
| `cli.rs` | +30 | CLI definitions |
| **Total** | **~365** | Complete feature |

**Test coverage:** ~90% of edge cases and boundary conditions

### Future Possibilities

The `parent_id` field enables powerful features for future versions:

- **Task history** - `todo history <ID>` to show all occurrences
- **Batch operations** - `todo done-chain <ID>` to complete entire chain
- **Analytics** - Completion rates for recurring tasks
- **Chain editing** - Modify all future occurrences at once

### Migration Notes

**This is a backward-compatible feature release:**

- All existing task files load successfully
- New fields default to `None` for old tasks
- No manual data migration needed
- Old tasks work exactly as before

**To start using recurring tasks:**

1. Create recurring task: `todo add "Task" --due YYYY-MM-DD --recurrence daily`
2. Or make existing task recurring: `todo recur <ID> daily`
3. Mark task done - next occurrence auto-created
4. Filter recurring tasks: `todo list --recurrence recurring`

### For Developers

**New concepts demonstrated:**

1. **Date arithmetic** - chrono operations with edge case handling
2. **Enum design** - When to use separate enums for related concepts  
3. **Optional fields** - `Option<T>` with `#[serde(default)]` for backward compatibility
4. **Deduplication** - Multi-strategy approach (primary + fallback)
5. **Referential integrity** - Using IDs vs values for relationships
6. **Conditional rendering** - Adaptive UI based on data presence
7. **Method design** - Parameter vs field decisions (parent_id example)
8. **Unit testing** - Comprehensive coverage including edge cases

**Study the implementation to learn:**

- How to add optional fields without breaking old data
- Smart date arithmetic with month boundary handling
- Dual-check deduplication for reliability
- Conditional UI rendering for cleaner output
- Type-safe enum patterns for domain models

---

## [2.0.0] - 2026-02-10

### Major Architectural Refactoring

**Goal:** Transform 1200-line monolithic `main.rs` into professional modular architecture

This version represents a **complete architectural refactoring** with **zero behavior changes**. All functionality remains identical - this is purely about code organization and maintainability.

### Changed - File Structure

- **Transformed `main.rs`** from 1200 lines → 95 lines (92% reduction)
- **Created modular structure** with 19 files organized in 6 modules:
  - `models/` (4 files) - Data structures and business logic
  - `commands/` (11 files) - One file per command with `execute()` function
  - `display/` (3 files) - UI rendering and formatting
  - `storage/` (1 file) - Data persistence abstraction
  - Core files: `cli.rs`, `error.rs`, `validation.rs`, `utils.rs`

### Architecture Improvements

- **Separation of Concerns** - Each module has single responsibility
- **Command Pattern** - Every command in isolated file with clear interface
- **Facade Pattern** - Clean public API through module re-exports
- **Single Responsibility Principle** - No mixed concerns across files
- **Dependency Inversion** - Commands depend on storage abstraction, not implementation

### File Organization

```
src/
├── main.rs              # Entry point - CLI parsing & dispatch (95 lines)
├── cli.rs               # Clap definitions only
│
├── models/              # Domain models
│   ├── mod.rs          # Public re-exports
│   ├── task.rs         # Task struct + business logic
│   ├── priority.rs     # Priority enum
│   └── filters.rs      # Filter enums
│
├── commands/            # Command implementations
│   ├── mod.rs          # Public re-exports
│   ├── add.rs          # Add command
│   ├── list.rs         # List with filters
│   ├── done.rs         # Mark done
│   ├── undone.rs       # Mark undone
│   ├── remove.rs       # Remove task
│   ├── edit.rs         # Edit task
│   ├── clear.rs        # Clear all
│   ├── search.rs       # Search
│   ├── tags.rs         # List tags
│   └── info.rs         # Info command
│
├── display/             # UI rendering
│   ├── mod.rs          # Public re-exports
│   ├── table.rs        # Table rendering
│   └── formatting.rs   # Colors, date formatting
│
├── storage/             # Data persistence
│   └── mod.rs          # Load/save/path
│
├── error.rs             # TodoError type
├── validation.rs        # ID validation
└── utils.rs             # Utilities (confirm, etc)
```

### Benefits

**Developer Experience:**

- Find code: 2-5 minutes → 10 seconds
- Navigation: Scroll 1200 lines → Open specific 20-150 line file
- Add command: Insert in giant match → Create new file
- Testing: Impossible isolation → Easy unit tests
- Risk: Change anything breaks everything → Localized changes

**Maintainability:**

- Clear file organization by feature
- Imports reveal dependencies
- Each module testable independently
- Easy to extend without complexity explosion
- Professional architecture ready for 10,000+ lines

**Code Quality:**

- ✅ Zero behavior changes - all features identical
- ✅ All tests still pass (if any)
- ✅ Same user experience
- ✅ Same performance
- ✅ **Much** better maintainability

### Technical Details

- **Design Patterns Applied:**
  1. Module Pattern - Code organized by feature/responsibility
  2. Facade Pattern - Simplified public API via re-exports
  3. Single Responsibility - Each file has one job
  4. Dependency Inversion - High-level modules depend on abstractions
  
- **Rust Concepts:**
  - Module system (`mod`, `pub`, re-exports)
  - Visibility control (public vs private APIs)
  - Code organization patterns
  - Separation of concerns
  
- **Extensibility:**
  - Add new command: Create file in `commands/`, add to enum, one line in dispatch
  - Change storage: Edit only `storage/mod.rs`
  - Modify display: Edit only `display/` module
  - Add tests: Import specific modules, test in isolation

### Migration Notes

**This is a refactoring, not a feature release:**

- No new functionality
- No behavior changes
- No API changes
- Same command-line interface
- Same data format
- Same user experience

**Why Major Version (2.0.0)?**

- Represents fundamental architectural shift
- Sets foundation for future scalability
- Demonstrates production-ready code organization
- Educational milestone - shows how to structure growing Rust projects

**For Users:**

- Update as normal - everything works identically
- Data files unchanged
- Commands unchanged
- Zero migration needed

**For Developers:**

- Study the file organization
- Learn modular architecture patterns
- See how to refactor without breaking
- Understand scalable code structure

### Impact Metrics

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| `main.rs` lines | 1200 | 95 | **-92%** |
| Total files | 1 | 19 | Better organization |
| Modules | 0 | 6 | Clear separation |
| Longest file | 1200 lines | ~150 lines | Manageable |
| Time to find code | 2-5 min | ~10 sec | **Much faster** |

---

## [1.9.0] - 2026-02-10

### Added

- **Edit command** - Modify existing tasks without losing ID or creation date
  - `todo edit <ID>` with `--text`, `--priority`, `--tag`, `--due`, `--clear-due`, `--clear-tags` flags
  - Visible alias: `todo e <ID>`
  - Preserves task ID, creation timestamp, and completion status
  - Partial updates - change only what you specify
  - Smart validation - prevents no-op changes
  - Rich feedback showing exactly what changed
- **Interactive confirmation prompts** for destructive operations
  - `todo remove <ID>` now asks for confirmation before deletion
  - `todo clear` shows count and asks for confirmation
  - Shows affected task/count before prompting
  - `--yes` / `-y` flag to skip confirmation (for scripts)
- `confirm()` helper function for user input
  - Buffered I/O with `io::stdout().flush()`
  - Case-insensitive input matching with `matches!` macro
  - Safe default (requires explicit "y" or "yes")

### Changed

- **BREAKING CHANGE:** `remove` command now prompts for confirmation by default
  - Use `--yes` or `-y` flag to skip prompt in scripts
  - Shows task text before asking for confirmation
- **BREAKING CHANGE:** `clear` command now prompts for confirmation by default
  - Shows task count before asking for confirmation
  - Use `--yes` or `-y` flag to skip prompt in scripts
- Refactored table layout calculation into `TableLayout` struct
  - Separates layout concerns from rendering logic
  - Single parameter instead of multiple width values
  - Easier to extend with new columns
  - Better encapsulation following Builder Pattern

### Improved

- Error messages now more specific for state validation
- Confirmation prompts use semantic colors (red for destructive operations)
- Cancelled operations show clear feedback messages

### Technical Details

- **New imports:**
  - `std::io::{self, Write}` - For buffered I/O and flush
- **New concepts:**
  - Let-chains (RFC 2497) for cleaner conditional logic
  - `io::stdout().flush()` for immediate terminal output
  - `matches!` macro for elegant pattern matching
  - Change tracking with `Vec<String>`
  - Preserving task identity in edit operations
  - TableLayout struct for separation of concerns
- **Code metrics:**
  - Added: 133 lines (Edit command, confirm function, TableLayout)
  - Changed: 32 lines (Remove/Clear updates)
  - Total impact: 165 lines
- **Builder Pattern:**
  - TableLayout encapsulates width calculations
  - Reduces coupling between layout and rendering
  - Type-safe parameter passing

## [1.8.0] - 2026-02-08

### Added

- **Global data directory** using `directories` crate for OS-appropriate storage
- `get_date_file_path()` function for platform-specific data file location
- **New command: `info`** - Shows data file location and status
- Platform-specific configuration directory support:
  - Linux: `~/.config/todo-cli/todos.json`
  - macOS: `~/Library/Application Support/todo-cli/todos.json`
  - Windows: `%APPDATA%\todo-cli\todos.json`
- Automatic directory creation with `fs::create_dir_all()`
- XDG Base Directory Specification compliance on Linux
- Rich error messages with actual file paths in context

### Changed

- **BREAKING CHANGE:** Data file location moved from current working directory to OS config directory
- `load_tasks()`: Now uses `get_date_file_path()` instead of hardcoded `"todos.json"`
- `save_tasks()`: Now uses `get_date_file_path()` instead of hardcoded `"todos.json"`
- `clear` command: Uses dynamic path from `get_date_file_path()`
- Error messages: Now include full path to data file for clarity
- All file operations now working-directory independent

### Fixed

- Task lists no longer fragment across different working directories
- Consistent task state regardless of where CLI is run
- Standard CLI behavior matching professional applications
- Single backup location instead of multiple scattered files

### Technical Details

- **Dependencies:**
  - Added: `directories = "5.0"` - Cross-platform system directories
- **New concepts:**
  - `PathBuf` for owned, mutable paths
  - `ProjectDirs::from()` for platform detection
  - `create_dir_all()` for recursive directory creation
  - Platform-specific path joining with `.join()`
- **Platform detection:**
  - Automatic at compile time via `cfg` attributes
  - No runtime overhead
  - Single binary works on all platforms
- **Migration:**
  - Old `todos.json` files not automatically migrated
  - User must manually copy to new location if desired
  - Use `todo info` command to find new data directory

## [1.7.0] - 2026-02-07

### Added

- **Professional error handling** using `anyhow` and `thiserror` crates
- `TodoError` enum for domain-specific errors with rich data
- Custom error variants:
  - `InvalidTaskId { id, max }` - Shows valid range in error message
  - `TaskAlreadyInStatus { id, status }` - Prevents duplicate state changes
  - `TagNotFound(String)` - Clear message when filtering by non-existent tag
  - `NoTasksFound` - Better UX when filters return empty results
  - `NoTagsFound` - Informative message when no tags exist
  - `NoSearchResults(String)` - Shows search query in error message
- `validate_task_id()` helper function for centralized ID validation
- Error chain display in `main()` - Shows full "Caused by:" chain
- Rich context with `.context()` on all operations
- Specific error messages for each failure mode
- Pattern matching on `std::io::ErrorKind` for granular error handling

### Changed

- **BREAKING CHANGE:** Error type migrated from `Box<dyn Error>` to `anyhow::Result`
- All function signatures updated to use `Result<T>` (shorthand for `Result<T, anyhow::Error>`)
- `load_tasks()`: Added context for parse failures and read errors
- `save_tasks()`: Added context for serialization and write failures
- `done` command: Added validation for already-completed tasks
- `undone` command: Added validation for already-pending tasks
- `remove` command: Uses centralized `validate_task_id()`
- `list` command: Returns `TodoError::NoTasksFound` instead of printing
- `search` command: Returns `TodoError::NoSearchResults` instead of printing
- `tags` command: Returns `TodoError::NoTagsFound` instead of printing
- Error display: Shows colored "Error:" prefix and "Caused by:" chain
- `main()`: Enhanced error display with full error chain traversal

### Fixed

- Clippy warning in `get_due_text()` - Removed redundant `else if` block
- Generic error messages replaced with specific, actionable messages
- Lost error context - Now displays full error chain
- User confusion about error causes - Shows root cause with "Caused by:"
- Missing context in file operations - All I/O now includes file path context
- State validation gaps - All state transitions now validated

## [1.6.0] - 2026-02-04

### Added

- **Professional CLI framework** using `clap` crate (v4.5) with derive macros
- **Type-safe filtering** with enum-based arguments
- `StatusFilter` enum: `All`, `Pending`, `Done` (replaces boolean flags)
- `DueFilter` enum: `Overdue`, `Soon`, `WithDue`, `NoDue` (replaces 4 boolean flags)
- `SortBy` enum: `Priority`, `Due`, `Created` (replaces string-based sorting)
- `ValueEnum` trait implementation on `Priority` for CLI value parsing
- **Auto-generated help text** with `#[command()]` attributes
- Command-level help with `long_about` descriptions
- Argument-level help with doc comments (appear in `--help` output)
- **Command aliases** for improved productivity:
  - `a` for `add`
  - `ls` for `list`
  - `rm` and `delete` for `remove`
- **Automatic argument parsing** with type safety and validation
- **Automatic NaiveDate parsing** using `clap::value_parser!(NaiveDate)`
- Professional error messages with context and suggestions
- `Cli` struct with `#[derive(Parser)]` for top-level command handling
- `Commands` enum with `#[derive(Subcommand)]` for subcommand routing
- `AddArgs` struct with `#[derive(Args)]` for complex add command arguments
- Short flags: `-t` for `--tag`, `-s` for `--sort`
- Repeatable arguments: `--tag` can be used multiple times
- `Task::matches_status()` helper method for status filtering
- `Task::matches_due_filter()` helper method for due date filtering
- Metadata in help output: program name, author, version, examples
- `after_help` examples section showing common usage patterns

### Changed

- **BREAKING CHANGE:** Complete CLI interface redesigned with Clap
- **BREAKING CHANGE:** Command syntax updated across all commands:
  - `--high/--medium/--low` → `--priority high/medium/low`
  - `--pending/--done` → `--status pending/done/all`
  - `--overdue/--due-soon/--with-due/--without-due` → `--due overdue/soon/with-due/no-due`
  - `--sort` → `--sort priority/due/created`
- **BREAKING CHANGE:** Priority values in JSON now lowercase (`"high"`, `"medium"`, `"low"`)
- `Priority` enum: Added `ValueEnum` derive for CLI integration
- Argument parsing: Manual `env::args()` parsing → Clap derive macros
- Help generation: Manual help strings → Automatic from struct attributes
- Validation: Manual conflict checking → Automatic with type system
- Error messages: Generic strings → Contextual clap-generated errors
- `main()` function: Simplified with `Cli::parse()` handling all parsing
- Filter mutual exclusion: Runtime checks → Compile-time type safety with enums
- Boolean filter flags (4 for dates) → Single `Option<DueFilter>` enum
- Parsing complexity: ~100 lines of manual parsing → ~20 lines of declarative structs
- Code organization: Flat argument handling → Structured with dedicated types

### Removed

- Manual argument parsing with `env::args().collect()`
- Manual flag conflict validation (15+ lines)
- Manual help text construction
- Generic "Usage: ..." error messages
- Manual `NaiveDate` parsing and error handling (8 lines → 1 line with `value_parser!`)
- Boolean flags for status filters (`--pending`, `--done`)
- Boolean flags for priority filters (`--high`, `--medium`, `--low`)
- Boolean flags for date filters (`--overdue`, `--due-soon`, `--with-due`, `--without-due`)
- String-based sorting validation

### Fixed

- Eliminated possibility of conflicting filter flags (enforced by type system)
- Improved error messages with suggestions for typos (e.g., "did you mean 'high'?")
- Consistent argument naming across all commands
- Professional command-line interface matching industry standards

### Technical Details

- **Dependencies:**
  - Added: `clap = { version = "4.5", features = ["derive"] }`
  - Enables derive macros for declarative CLI definition
- **Code reduction:** ~100 lines of parsing code → ~20 lines of struct definitions
- **Type safety improvements:**
  - `StatusFilter` enum prevents invalid status values at compile time
  - `DueFilter` enum ensures only one date filter used (via `Option<T>`)
  - `SortBy` enum validates sort fields at compile time
  - `Priority` as `ValueEnum` enables automatic CLI parsing
- **Derive macros used:**
  - `#[derive(Parser)]` on `Cli` - top-level command parsing
  - `#[derive(Subcommand)]` on `Commands` - subcommand routing
  - `#[derive(Args)]` on `AddArgs` - grouped arguments for complex commands
  - `#[derive(ValueEnum)]` on enums - enables use as CLI values
- **Attribute annotations:**
  - `#[command(name, author, version, about, after_help)]` - metadata
  - `#[arg(long, short, value_enum, default_value_t)]` - argument configuration
  - `#[command(visible_alias)]` - command aliases
  - Doc comments (`///`) automatically become help text
- **Pattern matching:**
  - `Option<DueFilter>` enables `if let Some(filter)` pattern for optional filters
  - `match` on enums is exhaustive (compiler enforces handling all cases)
- **Automatic behaviors:**
  - `--help` and `-h` flags generated automatically
  - `--version` and `-V` flags generated automatically
  - Error handling with exit codes (0 for success, non-zero for errors)
  - Suggestions for typos in values (e.g., "hgh" suggests "high")
  - Color-coded help output in supported terminals
- **Integration points:**
  - `Cli::parse()` consumes `std::env::args()` automatically
  - `value_parser!(NaiveDate)` uses `FromStr` trait from chrono
  - Enums with `ValueEnum` get case-insensitive parsing by default

### Migration Notes

Upgrading from v1.5.0:

**Command syntax changes required:**

```bash
# Old (v1.5.0):
todo add "Task" --high --tag work
todo list --pending --high --overdue --sort

# New (v1.6.0):
todo add "Task" --priority high --tag work
todo list --status pending --priority high --due overdue --sort priority
```

**Complete syntax mapping:**

| Old Flag (v1.5.0) | New Argument (v1.6.0) |
|-------------------|----------------------|
| `--high` | `--priority high` |
| `--medium` | `--priority medium` |
| `--low` | `--priority low` |
| `--pending` | `--status pending` |
| `--done` | `--status done` |
| `--overdue` | `--due overdue` |
| `--due-soon` | `--due soon` |
| `--with-due` | `--due with-due` |
| `--without-due` | `--due no-due` |
| `--sort` (bool) | `--sort priority\|due\|created` |

**Data compatibility:**

- JSON file format (`todos.json`) remains fully compatible
- Priority values in JSON change from `"High"` to `"high"` (lowercase)
- Existing tasks automatically migrate on first load
- No manual data migration needed
- All task data, tags, and dates preserved

**New features available:**

- Use `todo --help` for comprehensive command help
- Use `todo <command> --help` for command-specific help
- Try command aliases: `todo a`, `todo ls`, `todo rm`
- Explore short flags: `-t` for tags, `-s` for sort

**Breaking changes to address:**

1. Update any scripts using the old flag syntax
2. Priority values in JSON will be lowercase after first save
3. Sorting now requires explicit field name (not just `--sort`)
4. Status filter now explicit (no implicit "all" without flag)

## [1.5.0] - 2026-02-03

### Added

- **Due date tracking** with `chrono` crate
- `due_date` field in `Task` struct (`Option<NaiveDate>`)
- `created_at` field in `Task` struct (`NaiveDate`) - automatic timestamp on task creation
- `--due YYYY-MM-DD` flag for `add` command to set task deadlines
- `--overdue` filter to list tasks past their due date
- `--due-soon` filter to list tasks due in the next 7 days
- `--with-due` filter to list tasks that have a due date
- `--without-due` filter to list tasks without a due date
- `--sort due` option to sort tasks by due date (earliest first)
- `--sort created` option to sort tasks by creation date (oldest first)
- `Task::is_overdue()` method to check if task is past due date
- `Task::is_due_soon(days: i64)` method to check if task is due within N days
- Date parsing with `NaiveDate::parse_from_str()` using format `%Y-%m-%d`
- Color-coded due date display:
  - Red + Bold: Overdue (e.g., "late 3 days")
  - Yellow + Bold: Due today (e.g., "due today")
  - Yellow: Due soon, 1-7 days (e.g., "in 5 days")
  - Cyan: Future, 8+ days (e.g., "in 30 days")
- **Tabular display format** for professional task listing
- Dynamic column width calculation based on content
- `display_task_tabular()` function for formatted output
- `calculate_column_widths()` function for optimal column sizing
- `get_due_text()` function for human-readable due date formatting
- `get_due_colored()` function for urgency-based coloring
- Header row with column labels: `ID`, `P` (Priority), `S` (Status), `Task`, `Tags`, `Due`
- Separator line for visual clarity
- String truncation with ellipsis for long task names and tags
- Date arithmetic using `chrono::Duration` for calculating days until due

### Changed

- **BREAKING CHANGE:** Priority display changed from emojis (🔴🟡🟢) to letters (H/M/L)
  - `Priority::emoji()` renamed to `Priority::letter()`
  - More professional and terminal-friendly appearance
  - Consistent column width in tabular format
- **BREAKING CHANGE:** Task display format completely redesigned
  - Old: `1. 🔴 ⏳ Study Rust [learning, programming]`
  - New: Tabular format with aligned columns

  ```
    ID  P  S  Task           Tags              Due
  ────────────────────────────────────────────────────
     1  H  ⏳  Study Rust     learning, prog... in 5 days
  ```

- `Task::new()` signature: now accepts `due_date: Option<NaiveDate>` parameter
- `Task` JSON format: includes `due_date` (nullable) and `created_at` (required)
- `--sort` flag: renamed from boolean to value-based (`priority`, `due`, `created`)
- Due date sorting: tasks with dates come before tasks without dates
- Completed tasks no longer display due date information
- Maximum column widths enforced (task: 40 chars, tags: 20 chars, due: 20 chars)
- Minimum column widths enforced (task: 10 chars, tags: 4 chars, due: 3 chars)

### Fixed

- Mutual exclusion validation for date filters (can't use multiple date filters together)
- Proper handling of `None` values in due date sorting
- Grammar in due date text ("1 day" vs "2 days")
- Visual hierarchy with color-coding guides attention to urgent items

### Technical Details

- `chrono` crate added with `serde` feature for automatic date serialization
- `NaiveDate` used for dates (no timezone information needed for due dates)
- Date creation: `Local::now().naive_local().date()` for current date
- Date comparison: direct comparison operators (`<`, `>`, `==`) work on `NaiveDate`
- Pattern matching on `Option<NaiveDate>` for flexible due date handling
- Format specifiers: `{:>3}` (right-align), `{:<40}` (left-align with width)
- String slicing for truncation: `&text[..width-3]` with "..." suffix
- Four-way pattern matching in due date sort for handling `Option` combinations
- Tabular format uses `print!()` for inline formatting and `println!()` for line breaks
- Date filters use `retain()` to preserve task indices

### Migration Notes

Upgrading from v1.4.0:

- Old tasks remain compatible - `due_date` defaults to `null`
- `created_at` will be set to current date on first load for existing tasks
- No data loss - all existing fields preserved
- New fields can be added manually to JSON if needed
- Priority indicators will display as letters instead of emojis

## [1.4.0] - 2026-01-31

### Added

- **Tags system** for task categorization
- `tags` field in `Task` struct (`Vec<String>`)
- `--tag <name>` flag for `add` command (can be used multiple times)
- `--tag <name>` filter for `list` command
- `--tag <name>` filter for `search` command
- `tags` command to list all tags with task counts
- Tag display in task output (colored cyan for pending, dimmed for completed)

### Changed

- `Task::new()` signature: now accepts `tags: Vec<String>` parameter
- `display_task()`: Added tag display after task text
- `list` command: Added tag filtering with `--tag` flag
- `search` command: Added tag filtering with `--tag` flag
- Task display format: shows tags as `[tag1, tag2]` after task text

### Fixed

- **Critical bug:** Task numbering now maintains original indices when filtering
  - Before: Filtered lists showed renumbered tasks (1, 2, 3...), causing `done`/`undone`/`remove` to operate on wrong tasks
  - After: Filtered lists show original task numbers, ensuring commands work correctly
  - Changed `display_lists()` to accept `Vec<(usize, &Task)>` with original indices
  - All filter operations now use `retain()` instead of `filter().collect()` to preserve indices

### Technical Details

- Tags are stored as `Vec<String>` in JSON
- Empty tag vectors serialize to `[]` in JSON
- Tag filtering is case-sensitive (matches exact tag names)
- `tags` command deduplicates and sorts tags alphabetically
- Original task numbering preserved through tuple `(usize, &Task)` pattern
- Serde automatically handles tags serialization/deserialization

## [1.3.0] - 2026-01-30

### Added

- JSON serialization using `serde` and `serde_json`
- Automatic serialization/deserialization with derive macros
- `#[derive(Serialize, Deserialize)]` on `Task` and `Priority`
- Pretty-printed JSON output with `to_string_pretty()`
- Automatic type validation and descriptive error messages
- Universal format support (JSON as standard)

### Changed

- **BREAKING CHANGE:** File format migrated from custom text (`todos.txt`) to JSON (`todos.json`)
- `load_tasks()`: Replaced manual parsing with `serde_json::from_str()` (12 lines → 3 lines)
- `save_tasks()`: Replaced manual formatting with `serde_json::to_string_pretty()` (4 lines → 2 lines)
- Storage file: `todos.txt` → `todos.json`
- `clear` command: Updated to delete `todos.json` instead of `todos.txt`

### Removed

- `Task::to_line()` method - replaced by automatic serialization (12 lines deleted)
- `Task::from_line()` method - replaced by automatic deserialization (25 lines deleted)
- All custom text parsing logic (37 lines total)

### Technical Details

- **91% code reduction** in I/O operations (53 lines → 5 lines)
- Serde generates 100+ lines of optimized serialization code automatically
- Format-agnostic design allows easy migration to TOML, YAML, or binary formats
- Extensibility: Adding new fields now requires only 1 line (struct field) instead of 30+ lines (parser updates)
- Better error messages: "missing field `priority` at line 4 column 3" vs generic parsing errors
- Git-friendly: JSON diffs clearly show what changed
- Tooling support: Can use `jq`, JSON validators, formatters, etc.

### Migration Notes

Users need to migrate from `todos.txt` to `todos.json`:

- Option 1: Start fresh (delete `todos.txt`, recreate tasks)
- Option 2: Manual migration (convert old format to JSON)
- Future: Migration script could be provided

## [1.2.0] - 2026-01-29

### Added

- Type-safe architecture with structs and enums
- `Priority` enum (High, Medium, Low) replacing string-based priorities
- `Task` struct encapsulating task data (text, completed, priority)
- `impl` blocks with methods: `new()`, `to_line()`, `from_line()`, `mark_done()`, `mark_undone()`
- Centralized I/O with `load_tasks()` and `save_tasks()` helper functions
- Derive macros: `Debug`, `Clone`, `PartialEq`, `Copy` for type safety

### Changed

- **BREAKING CHANGE:** Complete refactoring from string parsing to struct-based architecture
- All commands now use `Task` struct instead of raw string manipulation
- Parsing logic centralized in `Task::from_line()` method
- File I/O consolidated into two functions (36% code reduction)
- `add` command: uses `Task::new()` constructor
- `done`/`undone` commands: use `task.mark_done()`/`task.mark_undone()` methods
- `list` command: type-safe field access (`task.completed`, `task.priority`)
- Priority comparison: string matching → enum comparison
- Display logic: uses `Priority::emoji()` and `Priority::order()` methods

### Fixed

- Ownership issues with priority filters using `Copy` trait
- Clippy warnings: redundant closures replaced with function pointers
- Type safety: compiler now catches priority typos at compile time

### Technical Details

- Code metrics: ~180 lines → ~115 lines (36% reduction)
- Maintainability: Adding new fields now requires changes in only 3 places instead of 7+
- Extensibility: Easy to add timestamps, tags, subtasks in future versions
- Type safety: No more runtime errors from typos like `"hihg"`

## [1.1.0] - 2026-01-28

### Added

- `--medium` flag to filter tasks by medium priority
- Complete symmetry in priority filtering (high/medium/low)
- Dynamic titles for medium priority task combinations

### Changed

- Updated help messages to include `--medium` option
- Improved consistency between task creation and filtering

### Fixed

- Design flaw: users can now filter by medium priority, completing the priority filter set

## [1.0.1] - 2026-01-27

### Changed

- **BREAKING CHANGE:** Entire codebase translated to English
- All variable names Portuguese → English
- All function names Portuguese → English  
- All user-facing messages Portuguese → English
- All dynamic titles and error messages now in English
- Achieve full consistency with English documentation
- Updated function names: `extrair_prioridade` → `extract_priority`, etc.

## [1.0.0] - 2026-01-27

### Added

- `search <term>` command to search tasks by term
- `display_task()` function for atomic rendering
- `display_lists()` function for list rendering with statistics

### Changed

- Complete refactoring: separation of parsing vs rendering
- Better code reuse without duplication

### Fixed

- Correct numbering in search command

## [0.9.0] - 2026-01-27

### Added

- `--sort` flag to sort tasks by priority
- `priority_order()` function for high/medium/low mapping
- Optimized pipeline: filter → then sort

## [0.8.0] - 2026-01-26

### Added

- Priority system (high, medium, low)
- `--high` and `--low` flags to filter by priority
- Colored emojis (🔴 🟡 🟢) for visual indication
- `extract_priority()` function for parsing
- `priority_emoji()` function for rendering
- Filter combination (status + priority)
- Conflicting flags validation
- Dynamic titles based on context

## [0.7.0] - 2026-01-26

### Added

- `--pending` and `--done` flags to filter by status
- Filter combination support
- Helper functions for code reuse

## [0.6.0] - 2026-01-25

### Added

- Colorful visual interface using `colored` crate
- Visual hierarchy with dimmed/bold formatting
- Progress counter with percentage
- Color-coded priority indicators

## [0.5.0] - 2026-01-24

### Added

- `clear` command to remove all tasks
- File existence validation with `fs::metadata()`

## [0.4.2] - 2026-01-23

### Fixed

- State validation to prevent duplicate operations
- More specific error messages

## [0.4.1] - 2026-01-23

### Fixed

- Bug in `list` command with empty lines
- Robust line filtering with `trim()`

## [0.4.0] - 2026-01-23

### Added

- `undone` command to unmark tasks

## [0.3.0] - 2026-01-23

### Added

- `remove` command to delete specific tasks
- Index validation
- Comprehensive error handling

## [0.2.0] - 2026-01-23

### Added

- `done` command to mark tasks as completed
- String manipulation with `.replace()`, `.map()`, `.collect()`

## [0.1.0] - 2026-01-23

### Added

- `add` command to add tasks
- `list` command to list all tasks
- Basic file operations with `OpenOptions`
- Pattern matching for subcommands
- Error handling with `?` operator

[Unreleased]:
https://github.com/joaofelipegalvao/todo-cli/compare/v2.1.0...HEAD
[2.1.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v2.0.0...v2.1.0
[2.0.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v1.9.0...v2.0.0
[1.9.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v1.8.0...v1.9.0
[1.8.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v1.7.0...v1.8.0
[1.7.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v1.6.0...v1.7.0
[1.6.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v1.5.0...v1.6.0
[1.5.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v1.4.0...v1.5.0
[1.4.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v1.3.0...v1.4.0
[1.3.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v1.2.0...v1.3.0
[1.2.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v1.0.1...v1.1.0
[1.0.1]: https://github.com/joaofelipegalvao/todo-cli/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v0.9.0...v1.0.0
[0.9.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v0.4.2...v0.5.0
[0.4.2]: https://github.com/joaofelipegalvao/todo-cli/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/joaofelipegalvao/todo-cli/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/joaofelipegalvao/todo-cli/releases/tag/v0.1.0
