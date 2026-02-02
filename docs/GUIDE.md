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

# Add high priority task with multiple tags
todo add "Fix critical bug" --high --tag work --tag urgent
```

**Notes:**

- Default priority is `medium` (🟡)
- Tasks are stored in `todos.json` in JSON format
- Multiple tags can be added with multiple `--tag` flags
- Tags help categorize and filter tasks

### Listing Tasks

```bash
# List all tasks (creation order)
todo list

# Sort by priority (high → medium → low)
todo list --sort

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

# Combine filters
todo list --pending --high              # Pending high priority
todo list --pending --medium            # Pending medium priority
todo list --done --low                  # Completed low priority
todo list --pending --high --sort       # Pending high priority, sorted
todo list --pending --tag work          # Pending work tasks
todo list --high --tag urgent --sort    # High priority urgent tasks, sorted
```

**Visual indicators:**

- 🔴 High priority (red)
- 🟡 Medium priority (yellow)
- 🟢 Low priority (green)
- ✅ Completed (green, strikethrough)
- ⏳ Pending (yellow)
- [tag1, tag2] Tags in cyan (pending) or dimmed (completed)

**Important:** Task numbers in filtered views preserve the original numbering, ensuring `done`, `undone`, and `remove` commands work correctly on the intended task.

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
- Shows priority indicators and tags
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

## Examples

### Basic Workflow

```bash
# Start your day
todo add "Review pull requests" --high --tag work
todo add "Write documentation" --tag work --tag documentation
todo add "Team meeting at 3pm" --high --tag work
todo add "Refactor old code" --low --tag programming

# Check what's urgent
todo list --pending --high --sort
# Output:
# 📋 High priority pending tasks:
# 1. 🔴 ⏳ Review pull requests [work]
# 3. 🔴 ⏳ Team meeting at 3pm [work]

# Complete tasks
todo done 1
todo done 3

# See progress
todo list
# Shows completed tasks with ✅ and strikethrough
```

### Using Filters

```bash
# Focus on what matters
todo list --pending --high    # Today's priorities

# Focus on work tasks
todo list --tag work          # All work-related tasks
todo list --pending --tag work --sort  # Pending work tasks, sorted

# Review achievements
todo list --done              # What you've completed
todo list --done --tag programming  # Completed programming tasks

# Clean up low priority items
todo list --low
todo remove 5
todo remove 7
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
# 5. 🟡 ⏳ Fix login bug [work, urgent]

# Update status (using original number)
todo done 5  # Marks the correct task even in filtered view
```

### Organizing with Tags

```bash
# Create tasks with meaningful tags
todo add "Learn Rust macros" --tag learning --tag rust
todo add "Team standup" --tag work --tag meetings
todo add "Fix navbar bug" --high --tag work --tag frontend

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
todo list --tag learning       # All learning tasks
todo list --tag work --pending # Pending work tasks
todo list --tag frontend --high # High priority frontend work
```

## Tips and Best Practices

### Priority Guidelines

**🔴 High Priority (--high):**

- Urgent and important
- Deadlines today
- Blocking other work
- Critical bugs

**🟡 Medium Priority (default):**

- Important but not urgent
- This week's tasks
- Regular work items
- Most tasks should be here

**🟢 Low Priority (--low):**

- Nice to have
- No deadline
- Future improvements
- Can be postponed

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

### Workflow Suggestions

1. **Morning routine:**

   ```bash
   todo list --pending --high --sort
   todo list --tag work --pending
   ```

   Focus on urgent tasks and work items

2. **Quick capture:**

   ```bash
   todo add "Quick thought"
   todo add "Research topic" --tag learning
   ```

   Don't overthink priority for quick adds, tags help organize later

3. **End of day:**

   ```bash
   todo list --done
   todo list --done --tag work
   ```

   Review what you accomplished

4. **Weekly review:**

   ```bash
   todo tags  # See all categories
   todo list --low  # Review low priority items
   todo list --tag project-x  # Check project progress
   ```

5. **Weekly cleanup:**

   ```bash
   todo list --low
   todo clear  # Or selectively remove
   ```

### Combining Filters Effectively

```bash
# See only what needs attention
todo list --pending --high

# Focus on medium priority work (where most tasks are)
todo list --pending --medium

# Work-related high priority items
todo list --pending --high --tag work

# Review completed work by category
todo list --done --tag programming
todo list --done --tag documentation

# Find everything related to a project
todo list --tag project-name

# Organize by seeing everything sorted
todo list --sort

# Find specific tasks quickly
todo search "keyword"
todo search "keyword" --tag work
```

## File Format

Tasks are stored in `todos.json` as JSON:

```json
[
  {
    "text": "Study Rust",
    "completed": false,
    "priority": "High",
    "tags": ["programming", "learning"]
  },
  {
    "text": "Fix bug",
    "completed": true,
    "priority": "Medium",
    "tags": ["work", "urgent"]
  },
  {
    "text": "Buy coffee",
    "completed": false,
    "priority": "Low",
    "tags": []
  }
]
```

**Format breakdown:**

- `text`: Task description (string)
- `completed`: Status (boolean - `false` = pending, `true` = completed)
- `priority`: Priority level (string - "High", "Medium", or "Low")
- `tags`: List of tags (array of strings, can be empty)

**Notes:**

- JSON format enables automatic serialization with `serde`
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
3. Restore from backup if available
4. Use `todo clear` to start fresh (deletes all tasks)

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

## Development Usage

If running from source with Cargo:

```bash
# All commands work with cargo run --
cargo run -- add "Task"
cargo run -- add "Task" --tag work
cargo run -- list --pending
cargo run -- list --tag work
cargo run -- tags
cargo run -- done 1
```

## Advanced Usage

### Batch Operations

```bash
# Add multiple related tasks
todo add "Setup project" --high --tag project-x
todo add "Design database" --tag project-x --tag backend
todo add "Create API endpoints" --tag project-x --tag backend
todo add "Build frontend" --tag project-x --tag frontend

# Review all project tasks
todo list --tag project-x

# Focus on backend work
todo list --tag backend --pending --sort
```

### Context Switching

```bash
# Switch to work mode
alias work="todo list --tag work --pending --sort"
work

# Switch to learning mode
alias learn="todo list --tag learning --pending"
learn

# Review personal tasks
alias personal="todo list --tag personal"
personal
```

### Migration from Old Format

If you have tasks in the old `todos.txt` format:

1. Backup your `todos.txt` file
2. The new version uses `todos.json`
3. Manually recreate important tasks with tags
4. Or write a migration script (see LEARNING.md for examples)

## Next Steps

- Check [LEARNING.md](LEARNING.md) to understand how it was built
- Read [CHANGELOG.md](../CHANGELOG.md) for version history
- Contribute improvements on GitHub
- Share your tag organization strategies!
