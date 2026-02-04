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

## Commands Reference

### Adding Tasks

```bash
# Add task with default priority (medium)
todo add "Learn Rust"

# Add high priority task
todo add "Important meeting" --high

# Add low priority task
todo add "Organize desk" --low

# Add task with tags
todo add "Study Rust" --tag programming --tag learning

# Add task with due date
todo add "Submit report" --due 2026-02-15

# Add high priority task with tags and due date
todo add "Fix critical bug" --high --tag work --tag urgent --due 2026-02-05

# Combine all features
todo add "Project deadline" --high --tag work --tag client --due 2026-02-10
```

**Notes:**

- Default priority is `medium` (M)
- Tasks are stored in `todos.json` in JSON format
- Multiple tags can be added with multiple `--tag` flags
- Due dates use format `YYYY-MM-DD` (e.g., `2026-02-15`)
- Tasks automatically get a `created_at` timestamp
- Tags help categorize and filter tasks

**Due date format:**

- ✅ Valid: `2026-02-15`, `2026-12-31`, `2025-01-01`
- ❌ Invalid: `02/15/2026`, `15-02-2026`, `tomorrow`, `next week`

### Listing Tasks

```bash
# List all tasks (tabular format)
todo list

# Sort by priority (high → medium → low)
todo list --sort priority

# Sort by due date (earliest first)
todo list --sort due

# Sort by creation date (oldest first)
todo list --sort created

# Filter by status
todo list --pending        # Only incomplete tasks
todo list --done          # Only completed tasks

# Filter by priority
todo list --high          # Only high priority
todo list --medium        # Only medium priority
todo list --low           # Only low priority

# Filter by tag
todo list --tag work      # Only tasks with "work" tag
todo list --tag urgent    # Only tasks with "urgent" tag

# Filter by due date
todo list --overdue       # Tasks past their due date
todo list --due-soon      # Tasks due in next 7 days
todo list --with-due      # Tasks that have a due date
todo list --without-due   # Tasks without a due date

# Combine filters
todo list --pending --high                    # Pending high priority
todo list --pending --tag work --sort due     # Pending work tasks, sorted by due date
todo list --high --overdue                    # High priority overdue tasks
todo list --due-soon --tag urgent --sort due  # Urgent tasks due soon
```

**Visual indicators:**

**Priority:**

- H (red) - High priority
- M (yellow) - Medium priority
- L (green) - Low priority

**Status:**

- ✅ Completed (green, strikethrough)
- ⏳ Pending (white)

**Due dates (color-coded by urgency):**

- 🚨 **Red + Bold**: Overdue (e.g., "late 3 days")
- ⚠️ **Yellow + Bold**: Due today (e.g., "due today")
- 📅 **Yellow**: Due soon (1-7 days, e.g., "in 5 days")
- 🗓️ **Cyan**: Future (>7 days, e.g., "in 30 days")

**Example output:**

```
Tasks:

  ID  P  S  Task                              Tags              Due
──────────────────────────────────────────────────────────────────────
   1  H  ⏳  Submit quarterly report           work, urgent      in 2 days
   2  M  ⏳  Team standup meeting              work, meetings    due today
   3  M  ⏳  Review pull request #42           work, code        late 4 days
   4  L  ⏳  Plan summer vacation              personal          in 40 days
   5  M  ✅  Fix login bug                     work, frontend
   6  H  ⏳  Client presentation               work, client      in 1 day
```

**Important notes:**

- Task numbers preserve original numbering in filtered views
- Due dates are hidden for completed tasks
- Column widths adjust dynamically to content
- Only one date filter can be used at a time (e.g., can't use `--overdue` with `--due-soon`)

### Searching Tasks

```bash
# Search for term in task text
todo search "rust"
todo search "meeting"

# Search with tag filter
todo search "bug" --tag work
todo search "docs" --tag programming
```

**Notes:**

- Case-insensitive search
- Maintains original task numbering
- Shows priority indicators, tags, and due dates
- Tag filter narrows search results
- Useful for large task lists

### Managing Tags

```bash
# List all tags with counts
todo tags

# Example output:
# 📋 Tags:
#   learning (2 tasks)
#   programming (3 tasks)
#   urgent (1 task)
#   work (4 tasks)
```

**Notes:**

- Shows all unique tags across tasks
- Displays task count for each tag
- Sorted alphabetically
- Helps discover categorization patterns

### Managing Tasks

```bash
# Mark task as completed
todo done 1

# Unmark task (mark as pending again)
todo undone 1

# Remove task permanently
todo remove 1

# Remove all tasks
todo clear
```

**Notes:**

- Task numbers are shown in `list` and `search` commands
- Numbers remain consistent even in filtered views
- `done`/`undone` only change status, don't delete
- `remove` and `clear` are permanent (no undo)
- Completed tasks don't show due date information

## Examples

### Basic Workflow

```bash
# Start your day
todo add "Review pull requests" --high --tag work --due 2026-02-03
todo add "Write documentation" --tag work --tag documentation --due 2026-02-05
todo add "Team meeting at 3pm" --high --tag work --due 2026-02-03
todo add "Refactor old code" --low --tag programming

# Check what's urgent
todo list --pending --high --sort due
# Output:
#   ID  P  S  Task                      Tags  Due
# ────────────────────────────────────────────────
#    1  H  ⏳  Review pull requests      work  due today
#    3  H  ⏳  Team meeting at 3pm       work  due today

# See what's overdue
todo list --overdue
# Shows tasks past their due date in red

# See what's coming up
todo list --due-soon
# Shows tasks due in the next 7 days

# Complete tasks
todo done 1
todo done 3

# See progress
todo list
# Shows completed tasks with ✅ and no due date
```

### Working with Due Dates

```bash
# Add tasks with various due dates
todo add "Submit tax documents" --high --due 2026-04-15
todo add "Dentist appointment" --due 2026-02-10
todo add "Birthday gift for mom" --due 2026-03-05

# See everything sorted by deadline
todo list --sort due
# Output shows tasks in order: overdue → today → soon → future

# Focus on immediate deadlines
todo list --due-soon
# Shows only tasks due in next 7 days

# Find tasks that are late
todo list --overdue
# Shows tasks with due dates in the past (in red)

# Find tasks without deadlines
todo list --without-due
# Shows tasks you can schedule later

# Find tasks with deadlines
todo list --with-due
# Shows all tasks that have a due date set
```

### Using Filters

```bash
# Focus on what matters
todo list --pending --high              # Today's priorities
todo list --pending --high --sort due   # Today's priorities by deadline

# Focus on work tasks
todo list --tag work                    # All work-related tasks
todo list --pending --tag work --sort due  # Pending work tasks by deadline

# Review achievements
todo list --done                        # What you've completed
todo list --done --tag programming      # Completed programming tasks

# Clean up low priority items
todo list --low
todo remove 5
todo remove 7

# Critical: high priority + overdue
todo list --high --overdue
# Shows urgent tasks you're behind on

# Planning: medium priority + no due date
todo list --medium --without-due
# Shows tasks you need to schedule
```

### Search and Update

```bash
# Find tasks about a topic
todo search "documentation"
# Output shows task numbers with original numbering

# Find work-related bugs
todo search "bug" --tag work
# Output:
# 📋 Results for "bug":
#   ID  P  S  Task              Tags          Due
# ───────────────────────────────────────────────
#    5  M  ⏳  Fix login bug     work, urgent  in 2 days

# Update status (using original number)
todo done 5  # Marks the correct task even in filtered view
```

### Organizing with Tags

```bash
# Create tasks with meaningful tags
todo add "Learn Rust macros" --tag learning --tag rust --due 2026-02-20
todo add "Team standup" --tag work --tag meetings --due 2026-02-03
todo add "Fix navbar bug" --high --tag work --tag frontend --due 2026-02-05

# See all your tags
todo tags
# Output:
# 📋 Tags:
#   frontend (1 task)
#   learning (1 task)
#   meetings (1 task)
#   rust (1 task)
#   work (2 tasks)

# Focus on specific areas
todo list --tag learning               # All learning tasks
todo list --tag work --pending         # Pending work tasks
todo list --tag frontend --high        # High priority frontend work
todo list --tag work --due-soon        # Work tasks due soon
```

### Advanced Deadline Management

```bash
# Morning routine: what needs attention today?
todo list --overdue           # What's already late
todo list --due-soon          # What's coming up this week
todo list --high --sort due   # High priority sorted by deadline

# Weekly planning
todo list --without-due       # Tasks that need scheduling
todo list --sort due          # See entire timeline
todo list --tag project-x --sort due  # Project timeline

# End of day review
todo list --done              # What you accomplished
todo list --overdue           # What still needs attention

# Project deadline tracking
todo add "Design phase complete" --tag project-x --due 2026-02-10
todo add "Development complete" --tag project-x --due 2026-02-20
todo add "Testing complete" --tag project-x --due 2026-02-25
todo add "Deployment" --high --tag project-x --due 2026-03-01

# See project timeline
todo list --tag project-x --sort due
```

## Tips and Best Practices

### Priority Guidelines

**H - High Priority (--high):**

- Urgent and important
- Deadlines today or tomorrow
- Blocking other work
- Critical bugs
- Client deliverables

**M - Medium Priority (default):**

- Important but not urgent
- This week's tasks
- Regular work items
- Most tasks should be here

**L - Low Priority (--low):**

- Nice to have
- No deadline
- Future improvements
- Can be postponed

### Due Date Best Practices

**When to set due dates:**

- ✅ Hard deadlines (client deliverables, appointments)
- ✅ Time-sensitive tasks (event preparation)
- ✅ Personal goals with commitment (finish book by X)
- ❌ Flexible tasks (general learning, ideas)
- ❌ Ongoing work (code refactoring)

**Tips:**

- Use `--due-soon` daily to stay on top of upcoming deadlines
- Check `--overdue` regularly to catch slipping tasks
- Combine due dates with high priority for critical deadlines
- Use `--without-due` to find tasks that need scheduling
- Sort by due date (`--sort due`) for timeline view
- Don't over-schedule: leave some tasks flexible

**Due date workflow:**

1. **Daily check:**

   ```bash
   todo list --overdue        # Fix what's late
   todo list --due-soon       # Prepare for upcoming
   ```

2. **Weekly planning:**

   ```bash
   todo list --without-due    # Schedule flexible tasks
   todo list --sort due       # Review timeline
   ```

3. **Project tracking:**

   ```bash
   todo list --tag project --sort due  # See project timeline
   ```

### Tag Best Practices

**Recommended tag categories:**

1. **Context tags:** `work`, `personal`, `home`
2. **Project tags:** `project-name`, `client-name`
3. **Activity tags:** `coding`, `documentation`, `meetings`
4. **Status tags:** `urgent`, `blocked`, `waiting`
5. **Technology tags:** `rust`, `python`, `frontend`, `backend`

**Tips:**

- Use lowercase for consistency
- Keep tag names short and clear
- Don't over-tag (2-3 tags per task is usually enough)
- Use `todo tags` regularly to see your categorization
- Create tag conventions (e.g., `proj-` prefix for projects)
- Combine tags with due dates for powerful filtering

### Workflow Suggestions

1. **Morning routine:**

   ```bash
   todo list --overdue               # What's late
   todo list --due-soon              # What's coming up
   todo list --pending --high --sort due  # Today's priorities
   todo list --tag work --pending --sort due  # Work focus
   ```

2. **Quick capture:**

   ```bash
   todo add "Quick thought"
   todo add "Research topic" --tag learning
   todo add "Call dentist" --due 2026-02-10
   ```

   Don't overthink priority for quick adds

3. **End of day:**

   ```bash
   todo list --done              # Review accomplishments
   todo list --done --tag work   # Work achievements
   todo list --overdue           # What needs attention
   ```

4. **Weekly review:**

   ```bash
   todo tags                     # See all categories
   todo list --low               # Review low priority items
   todo list --without-due       # Schedule flexible tasks
   todo list --tag project-x --sort due  # Check project timeline
   ```

5. **Weekly cleanup:**

   ```bash
   todo list --done --tag learning  # Review completed learning
   todo list --low                  # Clean up low priority
   ```

### Combining Filters Effectively

```bash
# Critical work
todo list --high --overdue --tag work
# High priority work that's already late

# This week's focus
todo list --pending --due-soon --sort due
# What needs attention soon

# Project deadlines
todo list --tag project-x --with-due --sort due
# Project tasks with deadlines, in timeline order

# Flexible work
todo list --tag work --without-due
# Work tasks you can schedule when you have time

# What to do next
todo list --pending --high --sort due
# High priority tasks in deadline order

# Tag + deadline combination
todo list --tag frontend --due-soon
# Frontend work due soon

# Review by category
todo list --done --tag learning
# Completed learning tasks
```

### Sorting Strategies

**Priority sort (`--sort priority`):**

- Best for: Daily task selection
- Shows: High → Medium → Low
- Use when: Choosing what to work on next

**Due date sort (`--sort due`):**

- Best for: Timeline planning
- Shows: Overdue → Today → Soon → Future → No due date
- Use when: Managing deadlines and schedules

**Created sort (`--sort created`):**

- Best for: Seeing task history
- Shows: Oldest → Newest
- Use when: Finding long-standing tasks

## File Format

Tasks are stored in `todos.json` as JSON:

```json
[
  {
    "text": "Study Rust",
    "completed": false,
    "priority": "High",
    "tags": ["programming", "learning"],
    "due_date": "2026-02-15",
    "created_at": "2026-02-03"
  },
  {
    "text": "Fix bug",
    "completed": true,
    "priority": "Medium",
    "tags": ["work", "urgent"],
    "due_date": "2026-02-01",
    "created_at": "2026-01-28"
  },
  {
    "text": "Buy coffee",
    "completed": false,
    "priority": "Low",
    "tags": [],
    "due_date": null,
    "created_at": "2026-02-03"
  }
]
```

**Format breakdown:**

- `text`: Task description (string)
- `completed`: Status (boolean - `false` = pending, `true` = completed)
- `priority`: Priority level (string - "High", "Medium", or "Low")
- `tags`: List of tags (array of strings, can be empty)
- `due_date`: Due date in YYYY-MM-DD format (string or `null`)
- `created_at`: Creation date in YYYY-MM-DD format (string, always present)

**Notes:**

- JSON format enables automatic serialization with `serde`
- `due_date` can be `null` (no deadline)
- `created_at` is set automatically when task is added
- File can be edited manually if needed, but be careful with syntax
- Backup recommended before manual edits
- Invalid JSON will cause the app to fail to load tasks

## Troubleshooting

### Tasks not showing up

```bash
# Check if file exists
cat todos.json

# Verify JSON format (should be valid JSON array)
# Use a JSON validator if needed
```

### Invalid JSON error

If you get a JSON parsing error:

1. Check `todos.json` for syntax errors
2. Ensure proper JSON format (commas, quotes, brackets)
3. Verify date format is `YYYY-MM-DD` or `null`
4. Restore from backup if available
5. Use `todo clear` to start fresh (deletes all tasks)

### Invalid date format error

If you get a date parsing error:

```bash
# Wrong formats:
todo add "Task" --due 02/15/2026  # ❌
todo add "Task" --due 15-02-2026  # ❌
todo add "Task" --due tomorrow    # ❌

# Correct format:
todo add "Task" --due 2026-02-15  # ✅

# Format: YYYY-MM-DD (year-month-day)
```

### Wrong priority colors

- Ensure terminal supports colors
- Check `colored` crate is installed
- Try a different terminal emulator

### Can't find task number

```bash
# Use list to see all numbers
todo list

# Or search for the task
todo search "keyword"

# Remember: filtered views preserve original numbers
todo list --tag work  # Numbers won't be 1, 2, 3... if tasks are filtered
```

### Tags not showing

```bash
# Verify task has tags in JSON
cat todos.json

# Tags are case-sensitive when filtering
todo list --tag Work   # Won't match "work"
todo list --tag work   # Will match "work"
```

### Date filters not working

```bash
# Can't use multiple date filters together:
todo list --overdue --due-soon  # ❌ Error

# Use one at a time:
todo list --overdue             # ✅
todo list --due-soon            # ✅

# But you can combine date filter with other filters:
todo list --overdue --high      # ✅
todo list --due-soon --tag work # ✅
```

### Due date colors not showing

- Ensure terminal supports ANSI colors
- Colors are based on urgency:
  - Red + Bold = overdue
  - Yellow + Bold = due today
  - Yellow = due in 1-7 days
  - Cyan = due in 8+ days
- Completed tasks don't show due dates

## Development Usage

If running from source with Cargo:

```bash
# All commands work with cargo run --
cargo run -- add "Task"
cargo run -- add "Task" --tag work --due 2026-02-15
cargo run -- list --pending
cargo run -- list --overdue
cargo run -- list --sort due
cargo run -- list --tag work
cargo run -- tags
cargo run -- done 1
```

## Advanced Usage

### Batch Operations

```bash
# Add multiple related tasks with deadlines
todo add "Setup project" --high --tag project-x --due 2026-02-10
todo add "Design database" --tag project-x --tag backend --due 2026-02-12
todo add "Create API endpoints" --tag project-x --tag backend --due 2026-02-18
todo add "Build frontend" --tag project-x --tag frontend --due 2026-02-20
todo add "Testing" --tag project-x --due 2026-02-25
todo add "Deployment" --high --tag project-x --due 2026-03-01

# Review project timeline
todo list --tag project-x --sort due

# Focus on what's next
todo list --tag project-x --pending --sort due
```

### Context Switching

```bash
# Switch to work mode
alias work="todo list --tag work --pending --sort due"
work

# Switch to learning mode
alias learn="todo list --tag learning --pending"
learn

# Review personal tasks
alias personal="todo list --tag personal --sort due"
personal

# Check what's urgent across all contexts
alias urgent="todo list --overdue"
urgent

# See this week's deadlines
alias thisweek="todo list --due-soon --sort due"
thisweek
```

### Time-based Workflows

```bash
# Sprint planning (2-week cycle)
todo add "Feature A" --high --tag sprint-5 --due 2026-02-15
todo add "Feature B" --tag sprint-5 --due 2026-02-15
todo add "Bug fixes" --tag sprint-5 --due 2026-02-12

# Track sprint progress
todo list --tag sprint-5 --sort due
todo list --tag sprint-5 --pending

# Daily standup prep
todo list --done --tag sprint-5        # What I did
todo list --pending --tag sprint-5 --sort due  # What I'm doing
todo list --tag sprint-5 --overdue     # Blockers/issues
```

### Recurring Tasks Simulation

```bash
# Add weekly tasks with due dates
todo add "Weekly report" --tag work --due 2026-02-07
todo add "Team meeting prep" --tag work --due 2026-02-06

# When done, re-add for next week
todo done 1
todo add "Weekly report" --tag work --due 2026-02-14

# Monthly tasks
todo add "Monthly review" --tag personal --due 2026-03-01
todo add "Expense report" --tag work --due 2026-03-05
```

### Migration from v1.4.0

If upgrading from v1.4.0 (without due dates):

1. Backup your `todos.json` file
2. The new version adds `due_date` (optional) and `created_at` (required)
3. Old tasks are compatible - they'll get `due_date: null`
4. `created_at` will be set to current date on first load
5. No data loss - all tasks, tags, and priorities preserved

**Example migration:**

```json
// Old format (v1.4.0):
{
  "text": "Task",
  "completed": false,
  "priority": "High",
  "tags": ["work"]
}

// New format (v1.5.0):
{
  "text": "Task",
  "completed": false,
  "priority": "High",
  "tags": ["work"],
  "due_date": null,
  "created_at": "2026-02-03"
}
```

## Next Steps

- Check [LEARNING.md](LEARNING.md) to understand how it was built
- Read [CHANGELOG.md](../CHANGELOG.md) for version history
- Try the new due date features and sorting options
- Use the tabular display for better task management
- Contribute improvements on GitHub
- Share your productivity workflows!

## Quick Reference Card

```bash
# Essential commands
todo add "Task" [--high|--low] [--tag TAG] [--due YYYY-MM-DD]
todo list [--pending|--done] [--high|--medium|--low] [--tag TAG]
todo list [--overdue|--due-soon|--with-due|--without-due]
todo list [--sort priority|due|created]
todo done NUMBER
todo search "keyword" [--tag TAG]

# Powerful combinations
todo list --pending --high --sort due     # Today's priorities by deadline
todo list --overdue --tag work            # Late work tasks
todo list --due-soon --sort due           # This week's deadlines
todo list --tag project --sort due        # Project timeline
todo list --without-due --medium          # Tasks to schedule

# Daily routine
todo list --overdue                       # What's late
todo list --due-soon                      # What's coming
todo list --pending --high --sort due     # What to do now
todo list --done                          # What you accomplished
```
