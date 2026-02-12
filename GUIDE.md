# Complete User Guide

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/joaofelipegalvao/todo-cli
cd todo-cli

# Build release version
cargo build --release

# Install globally (optional)
sudo cp target/release/todo-cli /usr/local/bin/todo
```

### Requirements

- Rust 1.70 or higher
- Cargo

## Data Storage

**Where are my tasks stored?**

Tasks are automatically saved to a platform-specific location:

- **Linux:** `~/.local/share/todo-cli/todos.json`
- **macOS:** `~/Library/Application Support/todo-cli/todos.json`
- **Windows:** `%APPDATA%\todo-cli\todos.json`

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

**When to use:**

- Find where your tasks are stored
- Verify the CLI is working correctly
- Get file path for backup scripts
- Debug file permission issues

### Adding Tasks

```bash
# Add task with default priority (medium)
todo add "Learn Rust"

# Add with specific priority
todo add "Important meeting" --priority high
todo add "Organize desk" --priority low

# Add task with tags (repeatable flag)
todo add "Study Rust" --tag programming --tag learning
todo add "Study Rust" -t programming -t learning  # short form

# Add task with due date
todo add "Submit report" --due 2026-02-15

# Combine all features
todo add "Fix critical bug" --priority high --tag work --tag urgent --due 2026-02-05
todo add "Project deadline" --priority high -t work -t client --due 2026-02-10

# Using alias
todo a "Quick task"  # 'a' is alias for 'add'
```

**Notes:**

- Default priority is `medium`
- Tags can be repeated: `-t work -t urgent -t critical`
- Due date format: `YYYY-MM-DD` (example: `2026-12-31`)
- All flags are optional

### Recurring Tasks

#### Recur Command

```bash
# Make a task recurring
todo recur <id> <frequency>

Frequencies:
  daily      Repeat every day
  weekly     Repeat every 7 days
  monthly    Repeat on the same day of month
```

**Examples:**

```bash
# Daily tasks - things you do every day
todo add "Morning standup" --tag work --due 2026-02-13
todo recur 1 daily

todo add "Check email" --priority high --tag work
todo recur 2 daily

# Weekly tasks - things you do every week
todo add "Team meeting" --tag work --due 2026-02-14
todo recur 3 weekly

todo add "Grocery shopping" --tag personal --due 2026-02-15
todo recur 4 weekly

# Monthly tasks - things you do every month
todo add "Pay rent" --priority high --due 2026-03-01
todo recur 5 monthly

todo add "Review budget" --tag finance --due 2026-03-01
todo recur 6 monthly
```

**How it works:**

When you mark a recurring task as done with `todo done <id>`:

1. The task is marked as completed
2. A new instance is automatically created with:
   - Same text, priority, tags, and recurrence
   - New due date calculated based on frequency:
     - **Daily:** original_date + 1 day
     - **Weekly:** original_date + 7 days
     - **Monthly:** same day next month (Jan 31 → Feb 28 for boundary cases)
   - `parent_id` linking it to the original task
3. Intelligent deduplication prevents duplicate tasks
4. The new task appears in your pending list

**Example workflow:**

```bash
# Create a daily recurring task
$ todo add "Daily standup" --tag work --due 2026-02-13
✓ Task added: Daily standup (ID: 1)

$ todo recur 1 daily
✓ Task #1 is now recurring: daily

# View it in the list
$ todo list
  ID  P  R   S  Task               Tags  Due
──────────────────────────────────────────────
   1  M  🔁  [ ]  Daily standup      work  due today

# Complete it
$ todo done 1
✓ Task marked as done: Daily standup
✓ Next occurrence created: Daily standup (due 2026-02-14)

# New instance automatically created
$ todo list
  ID  P  R   S  Task               Tags  Due
──────────────────────────────────────────────
   1  M  🔁  [x]  Daily standup      work
   2  M  🔁  [ ]  Daily standup      work  in 1 day
```

**Smart deduplication:**

The system prevents duplicates even if you:

- Mark done and undone multiple times
- Edit the task text
- Have multiple recurring tasks with similar names

Deduplication checks both:

1. **parent_id** (primary) - tracks recurring chains
2. **Text matching** (fallback) - for backwards compatibility

#### Norecur Command

```bash
# Remove recurrence from a task
todo norecur <id>

# Example
$ todo list
  1  M  🔁  [ ]  Daily standup  work  due today

$ todo norecur 1
✓ Recurrence removed from task #1

$ todo list
  1  M      [ ]  Daily standup  work  due today
```

**When to use:**

- You no longer need the task to repeat
- You want to convert a recurring task to a one-time task
- You need to stop auto-generation of new instances

**What happens:**

- The task becomes a regular one-time task
- Existing future instances remain unchanged
- No more instances will be auto-generated when marked done
- Can be made recurring again with `todo recur`

#### Clear-Recur Command

```bash
# Remove all recurring tasks
todo clear-recur [--yes]

# With confirmation (default)
$ todo clear-recur
WARNING: 5 recurring tasks will be permanently deleted!
This will remove:
  • 2 daily tasks
  • 2 weekly tasks
  • 1 monthly task
Are you sure? [y/N]: y
✓ All recurring tasks have been removed

# Skip confirmation (for scripts)
$ todo clear-recur --yes
$ todo clear-recur -y
```

**When to use:**

- Clean slate - starting over with task planning
- Remove all automation during a workflow change
- Clear out old recurring tasks that are no longer relevant

**What it does:**

- Removes ALL tasks with any recurrence (daily, weekly, monthly)
- Both pending and completed recurring tasks are removed
- Non-recurring tasks are preserved
- Requires confirmation unless `--yes` is used

### Edit Command

```bash
# Modify existing tasks
todo edit <ID> [options]
todo e <ID> [options]  # alias

# Change task text
todo edit 5 --text "New description"

# Change priority
todo edit 3 --priority high

# Update tags (replaces all tags)
todo edit 1 --tag work --tag urgent

# Set due date
todo edit 2 --due 2026-03-15

# Clear due date
todo edit 2 --clear-due

# Clear all tags
todo edit 1 --clear-tags

# Multiple changes at once
todo edit 3 --text "Updated task" --priority low --due 2026-04-01
```

**When to use:**

- Fix typos in task descriptions
- Adjust priority as tasks become more/less urgent
- Update deadlines when they change
- Reorganize tags
- Correct mistakes without losing task history

**What's preserved:**

- ✅ Task ID (stays the same number)
- ✅ Creation date (`created_at` timestamp)
- ✅ Completion status (done/undone)
- ✅ Recurrence settings (if task is recurring)
- ✅ Parent ID (maintains recurring chain tracking)

**Smart validation:**

```bash
$ todo edit 5 --priority medium
✓ Task #5 updated:
  • priority → M

$ todo edit 5 --priority medium  # Try again with same value
No changes made (values are already set to the specified values).
```

**Examples:**

```bash
# Fix a typo
$ todo list
  5  M  ⏳  Fix bug in lgoin system  work  in 2 days

$ todo edit 5 --text "Fix bug in login system"
✓ Task #5 updated:
  • text → Fix bug in login system

# Increase urgency
$ todo edit 5 --priority high
✓ Task #5 updated:
  • priority → H

# Add deadline
$ todo edit 5 --due 2026-02-15
✓ Task #5 updated:
  • due date → 2026-02-15

# Make multiple changes
$ todo edit 5 --text "URGENT: Fix login" --priority high --tag critical
✓ Task #5 updated:
  • text → URGENT: Fix login
  • priority → H
  • tags → [critical]
```

**Error handling:**

```bash
# Empty text rejected
$ todo edit 5 --text ""
Error: Task text cannot be empty

# Invalid ID
$ todo edit 999 --text "New"
Error: Task ID 999 is invalid (valid range: 1-10)

# No changes detected
$ todo edit 5 --priority high  # Already high
No changes made (values are already set to the specified values).
```

### Listing Tasks

```bash
# List all tasks
todo list
todo ls  # alias

# Filter by status
todo list --status pending
todo list --status done
todo list --status all  # default

# Filter by priority
todo list --priority high
todo list --priority medium
todo list --priority low

# Filter by due date
todo list --due overdue     # Past due date
todo list --due soon        # Due in next 7 days
todo list --due with-due    # Tasks with any due date
todo list --due no-due      # Tasks without due date

# Filter by tag
todo list --tag work
todo list --tag urgent

# Filter by recurrence
todo list --recurrence daily        # Only daily recurring tasks
todo list --recurrence weekly       # Only weekly recurring tasks
todo list --recurrence monthly      # Only monthly recurring tasks
todo list --recurrence recurring    # All recurring tasks (any frequency)
todo list --recurrence non-recurring # Only non-recurring tasks

# Sort results
todo list --sort priority   # High → Low
todo list --sort due        # Earliest → Latest
todo list --sort created    # Oldest → Newest

# Combine filters
todo list --status pending --priority high --sort due
todo list --status pending --tag work --due overdue
todo list --due soon --sort priority
todo list --recurrence recurring --status pending --sort due
todo list --recurrence daily --priority high --tag work
```

**Output format:**

```
  ID  P  R   S  Task                      Tags              Due
────────────────────────────────────────────────────────────────
   1  H  🔁  [ ]  Daily standup             work              due today
   2  M  📅  [ ]  Weekly report             work              in 5 days
   3  L  📆  [x]  Monthly review            management
   4  M      [ ]  One-time task             personal          in 3 days
```

**Legend:**

- **ID:** Task number (use with `done`, `remove`, `edit`, `recur`, `norecur`)
- **P:** Priority (H=High, M=Medium, L=Low)
- **R:** Recurrence indicator
  - **🔁** Daily recurring task
  - **📅** Weekly recurring task
  - **📆** Monthly recurring task
  - (blank) Non-recurring task
- **S:** Status ([ ]=Pending, [x]=Completed)
- **Task:** Description (truncated if too long)
- **Tags:** Comma-separated tags (truncated if many)
- **Due:** Human-readable due date with color coding:
  - **Red bold:** Overdue
  - **Yellow bold:** Due today
  - **Yellow:** Due soon (1-7 days)
  - **Cyan:** Future (8+ days)

### Managing Tasks

```bash
# Mark task as completed
todo done 1

# Unmark task (mark as pending again)
todo undone 1

# Remove task permanently (with confirmation)
todo remove 1
todo rm 1      # alias
todo delete 1  # also an alias

# Skip confirmation (for scripts)
todo remove 1 --yes
todo remove 1 -y

# Remove all tasks (with confirmation)
todo clear
todo clear --yes  # skip confirmation

# Remove all recurring tasks (with confirmation)
todo clear-recur
todo clear-recur --yes  # skip confirmation
```

**Interactive confirmation:**

Both `remove` and `clear` commands now ask for confirmation to prevent accidents:

```bash
# Remove prompts for confirmation
$ todo remove 3
Remove task 'Buy groceries'? [y/N]: y
✓ Task removed: Buy groceries

$ todo remove 3
Remove task 'Buy groceries'? [y/N]: n
Removal cancelled.

# Clear shows warning
$ todo clear
WARNING: 25 tasks will be permanently deleted!
Are you sure? [y/N]: y
✓ All tasks have been removed

# Clear-recur shows detailed warning
$ todo clear-recur
WARNING: 5 recurring tasks will be permanently deleted!
This will remove:
  • 2 daily tasks
  • 2 weekly tasks
  • 1 monthly task
Are you sure? [y/N]: y
✓ All recurring tasks have been removed

# Skip confirmation for automation
$ todo remove 3 --yes
$ todo clear -y
$ todo clear-recur -y
```

**Notes:**

- Task numbers are shown in `list` and `search` commands
- Numbers remain consistent even in filtered views
- `done`/`undone` only change status, don't delete
- `remove` and `clear` are permanent (no undo)
- Completed tasks don't show due date information
- Use `--yes` or `-y` to skip confirmation prompts (useful in scripts)
- `done` on a recurring task creates the next instance automatically

### Searching Tasks

```bash
# Search by text (case-insensitive)
todo search "rust"
todo find "meeting"  # alias

# Search and filter by tag
todo search "documentation" --tag work
todo search "bug" -t urgent
```

**How it works:**

- Case-insensitive text matching
- Searches entire task description
- Returns all matching tasks with original IDs
- Can be combined with tag filter

### Managing Tags

```bash
# List all tags with counts
todo tags

# Example output:
# Tags:
#
#   learning (5 tasks)
#   programming (8 tasks)
#   urgent (2 tasks)
#   work (12 tasks)
```

**Notes:**

- Automatically sorted alphabetically
- Shows task count for each tag
- Only displays tags currently in use
- Tags are case-sensitive

## Examples

### Basic Workflow

```bash
# Start your day
todo add "Review pull requests" --priority high --tag work --due 2026-02-03
todo add "Write documentation" --tag work --tag documentation --due 2026-02-05
todo add "Team meeting at 3pm" --priority high --tag work --due 2026-02-03
todo add "Refactor old code" --priority low --tag programming

# Check what's urgent
todo list --status pending --priority high --sort due
# Output:
#   ID  P   S  Task                      Tags  Due
# ────────────────────────────────────────────────
#    1  H  [ ]  Review pull requests      work  due today
#    3  H  [ ]  Team meeting at 3pm       work  due today

# See what's overdue
todo list --due overdue
# Shows tasks past their due date in red

# See what's coming up
todo list --due soon
# Shows tasks due in the next 7 days

# Complete tasks
todo done 1
todo done 3

# See progress
todo list
# Shows completed tasks with ✅ and no due date
```

### Working with Recurring Tasks

```bash
# Set up daily routines
todo add "Morning standup" --tag work --due 2026-02-13
todo recur 1 daily

todo add "Check email" --priority high --tag work --due 2026-02-13
todo recur 2 daily

todo add "Evening review" --tag personal --due 2026-02-13
todo recur 3 daily

# Set up weekly tasks
todo add "Team meeting" --tag work --due 2026-02-14
todo recur 4 weekly

todo add "Grocery shopping" --tag personal --due 2026-02-15
todo recur 5 weekly

todo add "Expense report" --tag finance --due 2026-02-16
todo recur 6 weekly

# Set up monthly tasks
todo add "Pay rent" --priority high --tag finance --due 2026-03-01
todo recur 7 monthly

todo add "Review budget" --tag finance --due 2026-03-01
todo recur 8 monthly

todo add "Backup data" --priority medium --tag tech --due 2026-03-01
todo recur 9 monthly

# View all recurring tasks
todo list --recurrence recurring

# View daily tasks only
todo list --recurrence daily --sort due

# Complete a recurring task (automatically creates next instance)
$ todo done 1
✓ Task marked as done: Morning standup
✓ Next occurrence created: Morning standup (due 2026-02-14)

# View upcoming recurring tasks
todo list --recurrence recurring --status pending --sort due

# Remove recurrence from a task (keep task, stop automation)
todo norecur 5

# Remove all recurring tasks
todo clear-recur
```

**Recurring task workflow:**

```bash
# Day 1: Setup
$ todo add "Daily standup" --tag work --due 2026-02-13
$ todo recur 1 daily

# Day 1: Complete it
$ todo done 1
# New instance automatically created for 2026-02-14

# Day 2: Complete it again
$ todo done 2
# New instance automatically created for 2026-02-15

# Day 3: Miss a day (leave it pending)
# The task stays pending with due date 2026-02-15

# Day 4: Complete the late task
$ todo done 2
# New instance created for 2026-02-16 (not 2026-02-17)
# Due date is based on original date + frequency

# Check the chain
$ todo list
  2  M  🔁  [x]  Daily standup  work  (completed)
  3  M  🔁  [ ]  Daily standup  work  in 3 days
```

### Editing Tasks

```bash
# Fix a typo
todo list
#  5  M  [ ]  Fix bug in lgoin system  work  in 2 days

todo edit 5 --text "Fix bug in login system"
✓ Task #5 updated:
  • text → Fix bug in login system

# Increase urgency
todo edit 5 --priority high
✓ Task #5 updated:
  • priority → H

# Add tags
todo edit 5 --tag urgent --tag critical
✓ Task #5 updated:
  • tags → [urgent, critical]

# Extend deadline
todo edit 5 --due 2026-02-15
✓ Task #5 updated:
  • due date → 2026-02-15

# Multiple changes
todo edit 5 --text "URGENT: Fix login" --priority high --due 2026-02-11
✓ Task #5 updated:
  • text → URGENT: Fix login
  • priority → H
  • due date → 2026-02-11
```

### Working with Due Dates

```bash
# Add tasks with different due dates
todo add "Dentist appointment" --due 2026-02-10
todo add "Project deadline" --priority high --due 2026-02-08
todo add "Birthday party" --due 2026-03-15

# Check overdue tasks
todo list --due overdue

# Plan for the week
todo list --due soon --sort due

# View all dated tasks
todo list --due with-due --sort due

# Find tasks without dates
todo list --due no-due

# Update a deadline
todo edit 2 --due 2026-02-12

# Remove a deadline
todo edit 3 --clear-due
```

### Using Tags Effectively

```bash
# Create tasks with tags
todo add "Deploy to staging" --tag work --tag devops
todo add "Review design mockups" --tag work --tag design
todo add "Buy milk" --tag personal --tag shopping
todo add "File taxes" --tag personal --tag finance --tag urgent

# View tasks by tag
todo list --tag work
todo list --tag urgent

# See all tags and their usage
todo tags

# Update task tags
todo edit 4 --tag personal --tag finance  # removes 'urgent'
todo edit 4 --clear-tags  # remove all tags

# Combine tag with other filters
todo list --tag work --status pending --sort priority
todo search "review" --tag work
```

### Automation with Scripting

```bash
#!/bin/bash
# Daily standup helper

# Show today's high-priority tasks
echo "High priority tasks:"
todo list --status pending --priority high --sort due

# Show overdue items
echo "
Overdue tasks:"
todo list --due overdue

# Show recurring tasks due today
echo "
Recurring tasks for today:"
todo list --recurrence recurring --due soon

# Auto-cleanup completed tasks older than 30 days (example logic)
# Note: todo-cli doesn't have date-based cleanup yet
```

**Script-friendly features:**

```bash
# Skip confirmations with --yes flag
todo remove 5 --yes
todo clear --yes
todo clear-recur --yes

# Exit codes for automation
if todo list --status pending --tag urgent > /dev/null 2>&1; then
    echo "You have urgent tasks!"
fi

# Check for overdue recurring tasks
if todo list --recurrence recurring --due overdue > /dev/null 2>&1; then
    echo "You have overdue recurring tasks!"
    todo list --recurrence recurring --due overdue
fi
```

## Tips and Best Practices

### Task Organization

**Use meaningful tags:**

```bash
# Good: specific, actionable tags
todo add "Deploy v2.0" --tag deployment --tag production

# Avoid: vague or overlapping tags
todo add "Deploy v2.0" --tag stuff --tag things
```

**Set realistic due dates:**

```bash
# Review due dates regularly
todo list --due soon

# Extend if needed
todo edit 5 --due 2026-03-01
```

**Prioritize ruthlessly:**

- **High:** Must be done today/tomorrow
- **Medium:** Important but can wait a week
- **Low:** Nice to have, no urgency

### Recurring Tasks Best Practices

**Use recurring tasks for:**

✅ **Daily routines:**

- Morning standup meetings
- Email checking
- Daily reports
- Exercise or meditation
- End-of-day reviews

✅ **Weekly tasks:**

- Team meetings
- Grocery shopping
- Expense reports
- Weekly planning sessions
- House cleaning

✅ **Monthly tasks:**

- Bill payments
- Budget reviews
- Monthly reports
- Subscription renewals
- Data backups

**Don't use recurring tasks for:**

❌ **Project milestones** - These are one-time events with specific dates
❌ **Irregular events** - Tasks with no clear pattern
❌ **Conditional tasks** - Tasks that depend on other factors

**Date handling tips:**

```bash
# Monthly tasks on the 31st
todo add "Monthly report" --due 2026-01-31
todo recur 1 monthly
# Jan 31 → Feb 28 (no Feb 31)
# Feb 28 → Mar 28 (not Mar 31)

# Solution: Use the 1st or last day manually
todo add "Monthly report" --due 2026-02-01
todo recur 1 monthly
# Always lands on the 1st of each month
```

**Managing recurring chains:**

```bash
# View all instances of a recurring task
todo list --status all | grep "Daily standup"

# Stop a recurring task
todo norecur 5

# Clean up all recurring tasks at once
todo clear-recur
```

### Edit vs Delete+Recreate

**Use `edit` when:**

- Correcting typos or mistakes
- Adjusting priority/due dates
- Want to preserve task history (created_at)
- Want to keep the same task ID
- Task is part of a recurring chain

**Delete+recreate when:**

- Task is completely wrong
- Starting over with different task
- Task ID doesn't matter

**Benefits of edit:**

- ✅ Preserves creation timestamp
- ✅ Keeps task ID (useful in scripts)
- ✅ Maintains recurrence settings
- ✅ Preserves parent_id for recurring chains
- ✅ Faster than delete+add
- ✅ Clear audit trail of changes

### Workflow Suggestions

**Morning routine:**

```bash
# See what's on your plate
todo list --status pending --sort priority

# Check deadlines
todo list --due overdue
todo list --due soon

# Check recurring tasks for today
todo list --recurrence recurring --due soon

# Reprioritize if needed
todo edit 3 --priority high
```

**End of day:**

```bash
# Mark completed tasks (recurring tasks auto-generate next instance)
todo done 1
todo done 5
todo done 8

# Update tomorrow's priorities
todo edit 2 --priority high

# Plan for tomorrow
todo list --status pending --priority high
todo list --recurrence daily --status pending
```

**Weekly review:**

```bash
# See all tags
todo tags

# Review by category
todo list --tag work
todo list --tag personal

# Check recurring task status
todo list --recurrence recurring --sort due

# Adjust recurring patterns if needed
todo norecur 7  # Stop a recurring task
todo recur 9 weekly  # Make a task recurring

# Clean up completed tasks
# (Note: Currently no auto-archive, manual cleanup with 'remove' or wait for future feature)
```

**Monthly planning:**

```bash
# Review all monthly recurring tasks
todo list --recurrence monthly --sort due

# Adjust dates for next month
todo edit 5 --due 2026-03-01

# Add new monthly tasks
todo add "Quarterly review prep" --due 2026-03-25 --tag work
todo recur 12 monthly
```

### Data Management

**Backup your tasks:**

```bash
# Find data file location
todo info

# Copy to backup location
cp ~/.config/todo-cli/todos.json ~/Backup/todos-$(date +%Y%m%d).json

# Or use version control
cd ~/.config/todo-cli/
git init
git add todos.json
git commit -m "Backup tasks"
```

**Share across machines:**

```bash
# Option 1: Sync data directory with cloud storage
ln -s ~/Dropbox/todo-cli ~/.config/todo-cli

# Option 2: Copy JSON file manually
scp ~/.config/todo-cli/todos.json remote-machine:.config/todo-cli/

# Option 3: Version control (recommended)
cd ~/.config/todo-cli/
git remote add origin git@github.com:you/tasks-backup.git
git push
```

**Restore from backup:**

```bash
# Overwrite current tasks
cp ~/Backup/todos-20260201.json ~/.config/todo-cli/todos.json

# Verify
todo list
```

## Quick Reference Card

```bash
# Add tasks
todo add "Task description"               # Default priority (medium)
todo add "Task" --priority high          # With priority
todo add "Task" -t tag1 -t tag2          # With tags
todo add "Task" --due 2026-12-31         # With due date

# Recurring tasks
todo recur ID daily                      # Make task repeat daily
todo recur ID weekly                     # Make task repeat weekly
todo recur ID monthly                    # Make task repeat monthly
todo norecur ID                          # Remove recurrence
todo clear-recur                         # Remove all recurring tasks

# Edit tasks
todo edit ID --text "New"                # Change description
todo edit ID --priority high             # Change priority
todo edit ID --tag TAG                   # Replace tags
todo edit ID --due 2026-03-15           # Set due date
todo edit ID --clear-due                 # Remove due date
todo edit ID --clear-tags                # Remove tags

# List and filter
todo list                                # All tasks
todo list --status pending               # Only pending
todo list --priority high                # High priority
todo list --due overdue                  # Overdue
todo list --tag work                     # By tag
todo list --recurrence daily             # Daily recurring
todo list --recurrence recurring         # All recurring
todo list --sort due                     # Sorted by due date

# Manage tasks
todo done ID                             # Mark completed (creates next if recurring)
todo undone ID                           # Mark pending
todo remove ID                           # Delete (with confirmation)
todo remove ID --yes                     # Delete (skip confirmation)
todo clear                               # Delete all (with confirmation)
todo clear --yes                         # Delete all (skip confirmation)
todo clear-recur                         # Delete recurring (with confirmation)

# Search
todo search "query"                      # Search text
todo search "query" --tag work           # Search + tag filter

# Info
todo tags                                # List all tags
todo info                                # Show data location

# Aliases
todo a        = todo add
todo e        = todo edit
todo ls       = todo list
todo rm       = todo remove
todo delete   = todo remove
todo find     = todo search
todo reset    = todo clear

# Getting help
todo --help                              # All commands
todo COMMAND --help                      # Command-specific help
```

## Advanced Features

### Priority System

Priorities are represented as:

- **H** (High) - Urgent, do first
- **M** (Medium) - Normal priority (default)
- **L** (Low) - Can wait

**Sorting behavior:**

```bash
todo list --sort priority
# Shows: High → Medium → Low
```

### Due Date System

**Date format:** `YYYY-MM-DD`

**Filters:**

- `--due overdue` - Past due date
- `--due soon` - Due within 7 days
- `--due with-due` - Has any due date
- `--due no-due` - No due date set

**Color coding:**

- Red bold: Late (overdue)
- Yellow bold: Due today
- Yellow: Due soon (1-7 days)
- Cyan: Future (8+ days)

### Recurring Task System

**Frequencies:**

- **Daily** (🔁) - Repeats every day
- **Weekly** (📅) - Repeats every 7 days
- **Monthly** (📆) - Repeats on same day of month

**How auto-generation works:**

1. Mark recurring task as done
2. New instance created with:
   - Same text, priority, tags, recurrence
   - New due date: `original_date + frequency`
   - `parent_id` linking to original task
3. Deduplication check (prevents duplicates)
4. New task appears in pending list

**Date arithmetic:**

```
Daily:   2026-02-13 → 2026-02-14 (+1 day)
Weekly:  2026-02-13 → 2026-02-20 (+7 days)
Monthly: 2026-01-31 → 2026-02-28 (handles boundaries)
```

**Tracking chains:**

Each recurring task has a `parent_id` field:

- First instance: `parent_id = None`
- Generated instances: `parent_id = original_task_id`
- Enables deduplication and future features (history, batch operations)

### Filter Combinations

You can combine multiple filters:

```bash
# Pending high-priority work tasks due soon
todo list --status pending --priority high --tag work --due soon

# All recurring tasks sorted by due date
todo list --recurrence recurring --status pending --sort due

# Overdue daily recurring tasks
todo list --recurrence daily --due overdue

# Completed weekly tasks
todo list --recurrence weekly --status done
```

**Mutual exclusions:**

- Can't use multiple `--status` values (use one at a time)
- Can't use multiple `--due` values (use one at a time)
- Can't use multiple `--recurrence` values (use one at a time)
- `--clear-due` conflicts with `--due` in edit command
- Only one `--sort` option allowed

## Troubleshooting

### Common Issues

**Q: "No such file or directory" error**

```bash
# The data directory doesn't exist yet
# Just run any command to create it:
todo add "First task"

# Or check the expected location:
todo info
```

**Q: Tasks not saving**

```bash
# Check file permissions
todo info
ls -l ~/.config/todo-cli/

# Verify you have write access
touch ~/.config/todo-cli/test.txt
rm ~/.config/todo-cli/test.txt
```

**Q: Can't find the `todo` command**

```bash
# Add to PATH if installed locally
export PATH="$PATH:$HOME/.cargo/bin"

# Or use full path
~/.cargo/bin/todo list

# Or install globally
sudo cp target/release/todo-cli /usr/local/bin/todo
```

**Q: Edit says "No changes made" but I changed something**

This means the value you're setting is already the current value:

```bash
# Task 5 already has priority=high
$ todo edit 5 --priority high
No changes made (values are already set to the specified values).

# Check current state first
$ todo list
  5  H  ⏳  Important task  ...
```

**Q: Recurring task created duplicate**

The system should prevent this, but if it happens:

```bash
# Check for duplicates
$ todo list --status pending | grep "Daily standup"
  1  M  🔁  [ ]  Daily standup  work  due today
  2  M  🔁  [ ]  Daily standup  work  due today

# Remove the duplicate manually
$ todo remove 2 --yes

# This is a bug - please report it!
```

**Q: Monthly recurring task has wrong date**

```bash
# If a task is created on Jan 31
$ todo add "Monthly report" --due 2026-01-31
$ todo recur 1 monthly
$ todo done 1

# Next instance will be Feb 28 (not Mar 31)
# This is by design - Feb has no 31st day

# Solution: Use early dates (1st-28th) for monthly tasks
$ todo edit 2 --due 2026-02-01
$ todo done 2
# Now it will always be on the 1st
```

**Q: Confirmation prompt not showing**

If running in a non-interactive shell (like in scripts), use `--yes`:

```bash
# In scripts
todo remove 5 --yes
todo clear --yes
todo clear-recur --yes

# Interactive terminal
todo remove 5  # Will prompt for confirmation
```

### Getting Help

```bash
# General help
todo --help

# Command-specific help
todo add --help
todo edit --help
todo list --help
todo recur --help
todo norecur --help
todo clear-recur --help

# Check version
todo --version
```

### Bug Reports

Found a bug? Please report it:

1. Check existing issues: <https://github.com/joaofelipegalvao/todo-cli/issues>
2. Create a new issue with:
   - Description of the problem
   - Steps to reproduce
   - Expected vs actual behavior
   - Your OS and Rust version (`rustc --version`)
   - Output of `todo info`

---

**Need more help?** Check the [full documentation](https://joaofelipegalvao.github.io/todo-cli/) or [open an issue](https://github.com/joaofelipegalvao/todo-cli/issues).
