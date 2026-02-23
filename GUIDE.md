# Complete User Guide

## Installation

### Via Cargo

```bash
cargo install rustodo
```

### From Source

```bash
git clone https://github.com/joaofelipegalvao/rustodo
cd rustodo
cargo build --release
cargo install --path .
```

### Requirements

- Rust 1.70 or higher
- Cargo

## Data Storage

Tasks are automatically saved to a platform-specific location:

- **Linux:** `~/.local/share/rustodo/todos.json`
- **macOS:** `~/Library/Application Support/rustodo/todos.json`
- **Windows:** `%APPDATA%\rustodo\todos.json`

The CLI creates this directory automatically on first use. You can view your data location with:

```bash
todo info
```

**Key benefits:**

- ✅ Same task list from any directory
- ✅ Follows OS conventions (XDG on Linux)
- ✅ Easy to find for backups
- ✅ Portable across machines

## Commands Reference

### Info Command

```bash
todo info
```

**Example output:**

```
Todo-List Information

Data file: /home/user/.local/share/rustodo/todos.json
Status: exists ✓
Size: 1245 bytes
```

### Adding Tasks

```bash
todo add <DESCRIPTION> [OPTIONS]
todo a <DESCRIPTION> [OPTIONS]   # alias

Options:
  --priority <high|medium|low>        Default: medium
  -t, --tag <TAG>                     Add tags (repeatable)
  -p, --project <PROJECT>             Assign to a project
  --due <DATE|EXPRESSION>             Due date
  --recurrence <daily|weekly|monthly>
  --depends-on <ID>                   Add dependency (repeatable)
```

**Examples:**

```bash
# Basic
todo add "Learn Rust"
todo add "Important meeting" --priority high
todo add "Organize desk" --priority low

# With tags
todo add "Fix bug" --tag work --tag urgent
todo add "Fix bug" -t work -t urgent    # short form

# With project
todo add "Setup database" --project "Backend"
todo add "Write migrations" --project "Backend" --priority high

# With due date (natural language)
todo add "Submit report" --due tomorrow
todo add "Team meeting" --due "next friday"
todo add "Project deadline" --due "in 3 days"
todo add "Review" --due "in 2 weeks"
todo add "Appointment" --due "jan 15"
todo add "Release" --due 2026-03-15     # YYYY-MM-DD also works

# With dependencies
todo add "Deploy to production" --depends-on 1 --depends-on 2

# Combine everything
todo add "Fix critical bug" --priority high -t work -t urgent --due tomorrow --project "Backend"
```

**Due date formats:**

| Input | Interpreted as |
|-------|----------------|
| `tomorrow` | Next day |
| `"next friday"` | Next Friday |
| `"in 3 days"` | 3 days from today |
| `"in 2 weeks"` | 2 weeks from today |
| `"in 1 month"` | 1 month from today |
| `monday` | Next Monday |
| `"jan 15"` | January 15 |
| `2026-03-15` | Strict YYYY-MM-DD |

**Tag normalization:**

The CLI automatically corrects tag typos and case variations:

```bash
$ todo add "Fix bug" --tag Rust
  ~ Tag normalized: 'Rust' → 'rust'
✓ Added task #8

$ todo add "Deploy" --tag fronteend
  ~ Tag normalized: 'fronteend' → 'frontend'
✓ Added task #9
```

Normalization runs in three steps:

1. Exact match — tag exists as-is, no change
2. Case-insensitive match — `Rust` → `rust`
3. Fuzzy match (Levenshtein distance) — `fronteend` → `frontend`

Normalization only applies to tags that already exist in your data. New tags are always accepted as-is.

### Editing Tasks

```bash
todo edit <ID> [OPTIONS]
todo e <ID> [OPTIONS]   # alias

Options:
  --text <TEXT>            New description
  --priority <PRIORITY>    New priority
  --due <DATE>             New due date (natural language or YYYY-MM-DD)
  --clear-due              Remove due date
  --add-tag <TAG>          Add tags (repeatable)
  --remove-tag <TAG>       Remove specific tags (repeatable)
  --clear-tags             Remove all tags
  -p, --project <PROJECT>  Assign to project
  --clear-project          Remove from project
  --add-dep <ID>           Add dependency (repeatable)
  --remove-dep <ID>        Remove dependency (repeatable)
  --clear-deps             Remove all dependencies
```

**Examples:**

```bash
# Fix a typo
todo edit 5 --text "Fix bug in login system"

# Change priority
todo edit 3 --priority high

# Update due date (natural language works here too)
todo edit 2 --due "next monday"
todo edit 2 --due "in 5 days"
todo edit 2 --clear-due

# Manage tags
todo edit 1 --add-tag urgent
todo edit 1 --remove-tag low-priority
todo edit 1 --clear-tags

# Manage project
todo edit 4 --project "Frontend"
todo edit 4 --clear-project

# Manage dependencies
todo edit 5 --add-dep 3
todo edit 5 --remove-dep 2
todo edit 5 --clear-deps

# Multiple changes at once
todo edit 3 --text "Updated task" --priority high --due "next friday" --project "Backend"
```

**What's preserved on edit:**

- ✅ Task ID (stays the same)
- ✅ Creation date (`created_at`)
- ✅ Completion status
- ✅ Recurrence settings
- ✅ `parent_id` (recurring chain)

### Listing Tasks

```bash
todo list [OPTIONS]
todo ls [OPTIONS]   # alias

Options:
  --status <all|pending|done>
  --priority <high|medium|low>
  --due <overdue|soon|with-due|no-due>
  -t, --tag <TAG>
  -p, --project <PROJECT>
  --recurrence <daily|weekly|monthly|recurring|non-recurring>
  -s, --sort <priority|due|created>
```

**Examples:**

```bash
# Basic
todo list
todo list --status pending
todo list --status done

# By priority
todo list --priority high

# By due date
todo list --due overdue
todo list --due soon
todo list --due with-due
todo list --due no-due

# By tag or project
todo list --tag work
todo list --project "Backend"

# By recurrence
todo list --recurrence recurring
todo list --recurrence daily
todo list --recurrence non-recurring

# Sorting
todo list --sort priority
todo list --sort due
todo list --sort created

# Combine filters
todo list --status pending --priority high --sort due
todo list --project "Backend" --status pending --sort due
todo list --tag work --due soon --sort priority
```

**Output format:**

```
 ID  P  R  S    Task                   Project   Tags          Due
─────────────────────────────────────────────────────────────────────
  1  H  D  [ ]  Daily standup          Backend   work          due today
  2  M  W  [ ]  Weekly report          Backend   work          in 5 days
  3  L     [x]  Write docs             Frontend  docs
  4  H     [~]  Deploy to production   Backend   devops        in 2 days
─────────────────────────────────────────────────────────────────────
```

**Legend:**

- **P:** Priority (H=High, M=Medium, L=Low)
- **R:** Recurrence (D=daily, W=weekly, M=monthly, blank=none)
- **S:** Status ([ ] pending, [x] done, [~] blocked by dependency)
- **Due colors:** red=overdue, yellow=today/soon, cyan=future

Columns are **contextual** — Project, Tags, Due and R only appear if at least one task in the current view has that field set.

### Managing Tasks

```bash
# Complete / reopen
todo done <ID>
todo undone <ID>

# Remove
todo remove <ID>           # with confirmation prompt
todo remove <ID> --yes     # skip confirmation
todo rm <ID>               # alias
todo delete <ID>           # alias

# Clear
todo clear                 # remove all tasks (with confirmation)
todo clear --yes           # skip confirmation
todo clear-recur           # remove all recurring tasks (with confirmation)
todo clear-recur --yes
```

**Interactive confirmation:**

```bash
$ todo remove 3
Remove task 'Buy groceries'? [y/N]: y
✓ Task removed: Buy groceries

$ todo clear
WARNING: 25 tasks will be permanently deleted!
Are you sure? [y/N]: y
✓ All tasks have been removed
```

### Task Dependencies

Use dependencies to model blocking relationships between tasks. A task is **blocked** when any of its dependencies are still pending.

```bash
# Add dependency when creating
todo add "Deploy to production" --depends-on 1 --depends-on 2

# Add/remove dependency on existing task
todo edit 5 --add-dep 3
todo edit 5 --remove-dep 2
todo edit 5 --clear-deps

# View dependency graph
todo deps <ID>
```

**Example `todo deps` output:**

```
Task #5: Deploy to production

  Depends on:
    ✓ #1 — Setup database
    ◦ #3 — Write tests

  Required by:
    ◦ #7 — Update documentation

  [~] Blocked by: #3
```

**Blocked tasks** are shown with `[~]` in `todo list` and cannot be completed until all dependencies are done.

**Cycle detection:** The CLI prevents circular dependencies and will reject them with an error.

### Projects

Group related tasks into named projects.

```bash
# Assign project on add
todo add "Setup database" --project "Backend"
todo add "Write migrations" --project "Backend"
todo add "Build UI components" --project "Frontend"

# Assign/change project on edit
todo edit 4 --project "Frontend"
todo edit 4 --clear-project

# Filter by project
todo list --project "Backend"

# List all projects with task counts
todo projects
```

**Example `todo projects` output:**

```
Projects

  Backend    4 tasks  (3 pending, 1 done)
  Frontend   3 tasks  (2 pending, 1 done)
```

### Stats

```bash
todo stats
```

**Example output:**

```
Todo Statistics

Overview

  Total tasks      12
  Completed        7 (58%)
  Pending          5
  Overdue          2
  Due soon         3
  Blocked          1

By Priority

  High     3  (1 pending, 2 done)
  Medium   6  (2 pending, 4 done)
  Low      3  (2 pending, 1 done)

By Project

  Backend                  4 tasks  (75% done)
  Frontend                 3 tasks  (33% done)
  (no project)             5 tasks

Activity — last 7 days

  Feb 15  ░░░░░░░░░░    0 completed
  Feb 16  ██░░░░░░░░    1 completed
  Feb 17  ░░░░░░░░░░    0 completed
  Feb 18  ████░░░░░░    2 completed
  Feb 19  ██░░░░░░░░    1 completed
  Feb 20  ████░░░░░░    2 completed
  Feb 21  ██████████    5 completed
```

The activity chart tracks tasks by their `completed_at` date. Tasks completed before upgrading to v2.7.0 will not appear in the chart until re-completed.

### Recurring Tasks

```bash
# Set recurrence
todo recur <ID> <daily|weekly|monthly>

# Remove recurrence
todo norecur <ID>

# Remove all recurring tasks
todo clear-recur [--yes]

# Filter recurring tasks
todo list --recurrence recurring
todo list --recurrence daily
```

**How it works:**

When you mark a recurring task as done, a new instance is automatically created with the same text, priority, tags, and project. Due dates are calculated as:

- **Daily:** +1 day
- **Weekly:** +7 days
- **Monthly:** same day next month (Jan 31 → Feb 28 for boundary cases)

Tasks are linked via `parent_id` for deduplication — marking done and undone multiple times will never create duplicate instances.

**Example workflow:**

```bash
$ todo add "Daily standup" --tag work --due tomorrow
✓ Added task #1

$ todo recur 1 daily
✓ Task #1 is now recurring: daily

$ todo done 1
✓ Task marked as done: Daily standup
✓ Next occurrence created: Daily standup (due 2026-02-22)
```

### Searching Tasks

```bash
todo search <QUERY> [OPTIONS]
todo find <QUERY>   # alias

Options:
  --tag <TAG>                  Filter by tag
  -p, --project <PROJECT>      Filter by project
  --status <all|pending|done>  Filter by status (default: all)

# Examples
todo search "rust"
todo search "meeting" --tag work
todo search "bug" --project "Backend" --status pending
```

Search is case-insensitive and matches anywhere in the task description.

### Tags

```bash
# List all tags with counts
todo tags
```

**Example output:**

```
Tags

  docs          (3 tasks)
  programming   (8 tasks)
  urgent        (2 tasks)
  work          (12 tasks)
```

## Quick Reference

```bash
# Add
todo add "Task"                          # medium priority
todo add "Task" --priority high          # high priority
todo add "Task" -t tag1 -t tag2          # with tags
todo add "Task" -p "Project"             # with project
todo add "Task" --due tomorrow           # natural language date
todo add "Task" --due "next friday"
todo add "Task" --due "in 3 days"
todo add "Task" --due 2026-12-31         # YYYY-MM-DD
todo add "Task" --depends-on 1           # with dependency

# Edit
todo edit ID --text "New description"
todo edit ID --priority high
todo edit ID --due "in 5 days"
todo edit ID --clear-due
todo edit ID --add-tag urgent
todo edit ID --remove-tag old
todo edit ID --clear-tags
todo edit ID -p "Backend"
todo edit ID --clear-project
todo edit ID --add-dep 3
todo edit ID --remove-dep 2
todo edit ID --clear-deps

# List
todo list                                # all tasks
todo list --status pending
todo list --status done
todo list --priority high
todo list --due overdue
todo list --due soon
todo list --tag work
todo list -p "Backend"
todo list --recurrence recurring
todo list --recurrence daily
todo list --sort due

# Complete / reopen
todo done ID
todo undone ID

# Remove
todo remove ID                           # with confirmation
todo remove ID --yes                     # skip confirmation
todo clear                               # all tasks
todo clear-recur                         # all recurring tasks

# Recurring
todo recur ID daily
todo recur ID weekly
todo recur ID monthly
todo norecur ID

# Info
todo stats
todo deps ID
todo tags
todo projects
todo search "query"
todo info

# Aliases
todo a        = todo add
todo e        = todo edit
todo ls       = todo list
todo rm       = todo remove
todo delete   = todo remove
todo complete = todo done
todo undo     = todo undone
todo reset    = todo clear
todo find     = todo search
todo norecur  = todo clear-recur

# Help
todo --help
todo <command> --help
```

## Tips and Best Practices

### Priority Guidelines

- **High:** Must be done today or tomorrow
- **Medium:** Important but can wait a week (default)
- **Low:** Nice to have, no urgency

### Using Projects Effectively

Projects work best for grouping tasks that belong to the same context, repository, or goal. Keep project names short and consistent — the CLI is case-sensitive (`Backend` ≠ `backend`).

### Recurring Tasks

Use recurring tasks for predictable, time-based routines:

- ✅ Daily standups, email reviews, end-of-day checks
- ✅ Weekly meetings, expense reports, planning sessions
- ✅ Monthly bill payments, budget reviews, data backups
- ❌ Project milestones (one-time events)
- ❌ Tasks that depend on external conditions

For monthly tasks, avoid dates 29–31 to prevent boundary issues — the 1st is the safest choice.

### Task Dependencies

Use dependencies to model a real sequence of work. Keep dependency chains short and direct — deep chains (A → B → C → D) are hard to manage and usually a sign that a project needs better breakdown.

### Data Backup

```bash
# Find your data file
todo info

# Copy to backup
cp ~/.local/share/rustodo/todos.json ~/backup/todos-$(date +%Y%m%d).json
```

## Troubleshooting

**"No such file or directory"** — Run any command (e.g. `todo list`) to create the data directory automatically.

**`todo` command not found** — Make sure `~/.cargo/bin` is in your `$PATH`:

```bash
export PATH="$PATH:$HOME/.cargo/bin"
```

**"No changes made"** — The value you're setting is already the current one. Use `todo list` to check the current state.

**Recurring task created a duplicate** — This is a bug. Please [open an issue](https://github.com/joaofelipegalvao/rustodo/issues) with steps to reproduce.

**Activity chart shows 0 for old completions** — Tasks completed before v2.7.0 don't have a `completed_at` date. Re-complete them to start tracking.

## Bug Reports

Found a bug? Please [open an issue](https://github.com/joaofelipegalvao/rustodo/issues) with:

- Description of the problem
- Steps to reproduce
- Expected vs actual behavior
- Your OS and Rust version (`rustc --version`)
- Output of `todo info`
