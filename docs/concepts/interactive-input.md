# Interactive Input

**📚 Overview:**

This page covers interactive input patterns in Rust CLIs, focusing on user confirmations, prompts, and safe data modification workflows.

**🔗 Related Versions:**

- [v0.1.0](../getting-started/v0.1.0-basic-cli.md) - Basic CLI input
- [v0.3.0](../getting-started/v0.3.0-remove-command.md) - Argument parsing
- [v1.6.0](../advanced/v1.6.0-professional-cli-clap.md) - Professional CLI framework
- [v1.9.0](../advanced/v1.9.0-edit-command-confirmation.md) - **Interactive confirmations**

---

## User Confirmations

### The Need for Confirmations

**Problem: Destructive operations without safeguards**

```bash
# Before confirmations
$ todo remove 5
✓ Task removed: Fix critical production bug

# "Wait, I meant task 4!" 😱
# No undo, no confirmation, data lost forever
```

**Solution: Interactive confirmation**

```bash
# With confirmations
$ todo remove 5

Remove task: Fix critical production bug
Are you sure? [y/N]: n
Removal cancelled.

# User has time to think and can cancel
```

### Basic Confirmation Pattern

```rust
use std::io::{self, Write};

fn confirm(message: &str) -> Result<bool> {
    print!("{} ", message.yellow());
    io::stdout().flush()?;  // Critical!
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    let response = input.trim().to_lowercase();
    Ok(matches!(response.as_str(), "y" | "yes"))
}
```

**Usage:**

```rust
if !confirm("Are you sure? [y/N]:")? {
    println!("{}", "Operation cancelled.".yellow());
    return Ok(());
}

// Continue with destructive operation
```

---

## Buffered I/O and Flushing

### The Terminal Buffer Problem

**Without flush:**

```rust
print!("Enter name: ");  // ← Stays in buffer!
let mut input = String::new();
io::stdin().read_line(&mut input)?;  // ← Prompt not visible yet!
```

**What happens:**

1. `print!()` writes to buffer (not screen)
2. Buffer waits for newline or full before displaying
3. Cursor blinks but no prompt visible
4. User is confused

**With flush:**

```rust
print!("Enter name: ");
io::stdout().flush()?;  // ← Force immediate display
let mut input = String::new();
io::stdin().read_line(&mut input)?;
```

**Now:**

1. `print!()` writes to buffer
2. `flush()` forces immediate display
3. Prompt appears before user input
4. Clear, professional UX

### Why `flush()` is Needed

**Buffering behavior:**

- `println!()` - auto-flushes (has newline `\n`)
- `print!()` - stays in buffer (no newline)
- Buffer flushes when:
  - Newline encountered
  - Buffer is full
  - Manual `flush()` called
  - Program exits

**For prompts, we need immediate display:**

```rust
// ❌ Doesn't work - prompt stays in buffer
print!("Continue? [y/N]: ");

// ✅ Works - prompt appears immediately
print!("Continue? [y/N]: ");
io::stdout().flush()?;
```

### Error Handling with Flush

```rust
io::stdout().flush()?;
//                  ↑ Returns Result<()>
```

**Can fail if:**

- stdout is closed
- stdout is redirected to broken pipe
- Permission issues

**Always use `?` operator:**

```rust
fn confirm(message: &str) -> Result<bool> {
    print!("{} ", message);
    io::stdout().flush()?;  // Propagate error
    // ...
}
```

---

## Pattern Matching for Input

### The `matches!` Macro

**Elegant input validation:**

```rust
let response = input.trim().to_lowercase();
let confirmed = matches!(response.as_str(), "y" | "yes");
```

**Why this is better than alternatives:**

```rust
// ❌ Verbose
let confirmed = response == "y" || response == "yes";

// ❌ Hard to extend
let confirmed = response == "y" || response == "yes" || response == "Y";

// ✅ Clean and extensible
let confirmed = matches!(response.as_str(), "y" | "yes");
```

### Case-Insensitive Matching

```rust
let response = input.trim().to_lowercase();
//                         ↑ normalize to lowercase
matches!(response.as_str(), "y" | "yes")
//                          ↑ only need lowercase patterns
```

**User experience:**

```bash
# All these work:
Are you sure? [y/N]: y     ✅
Are you sure? [y/N]: Y     ✅
Are you sure? [y/N]: yes   ✅
Are you sure? [y/N]: YES   ✅
Are you sure? [y/N]: YeS   ✅
```

### Default Behavior Design

**Why NOT accept empty Enter:**

```rust
// ❌ Dangerous for destructive operations
matches!(response.as_str(), "" | "y" | "yes")
// Accidental Enter = deletion!
```

**Our design choice:**

```rust
// ✅ Safer - requires explicit confirmation
matches!(response.as_str(), "y" | "yes")
// Empty Enter = no = safe default
```

**The `[y/N]` convention:**

- `[y/N]` - N is uppercase, default is No
- `[Y/n]` - Y is uppercase, default is Yes
- Our prompts use `[y/N]` for safety on destructive ops

---

## Confirmation Workflow Patterns

### Destructive Operation Pattern

```rust
Commands::Remove { id, yes } => {
    let mut tasks = load_tasks()?;
    validate_task_id(id, tasks.len())?;
    
    let index = id - 1;
    let task_text = &tasks[index].text;
    
    // Skip prompt if --yes flag provided
    if !yes {
        println!(
            "\n{} {}",
            "Remove task:".red().bold(),
            task_text.bright_white()
        );
        
        if !confirm("Are you sure? [y/N]:")? {
            println!("{}", "Removal cancelled.".yellow());
            return Ok(());
        }
    }
    
    // Proceed with deletion
    let removed_task = tasks.remove(index);
    save_tasks(&tasks)?;
    println!("{} {}", "✓ Task removed:".red(), removed_task.text.dimmed());
}
```

**Key elements:**

1. **Show what will be affected** - Display task before asking
2. **Ask for confirmation** - Clear prompt with [y/N]
3. **Handle cancellation** - Early return with message
4. **Bypass option** - `--yes` flag for scripts

### Batch Operation Pattern

```rust
Commands::Clear { yes } => {
    let path = get_date_file_path()?;
    
    if !path.exists() {
        println!("No tasks to remove");
        return Ok(());
    }
    
    let tasks = load_tasks()?;
    let count = tasks.len();
    
    if !yes {
        println!(
            "\n{} {} tasks will be permanently deleted!",
            "WARNING:".red().bold(),
            count
        );
        
        if !confirm("Are you sure? [y/N]:")? {
            println!("{}", "Clear cancelled.".yellow());
            return Ok(());
        }
    }
    
    fs::remove_file(&path)?;
    println!("{}", "✓ All tasks have been removed".red().bold());
}
```

**Differences from single operation:**

- **Quantify impact** - Show number of affected items
- **Stronger warning** - "WARNING:" prefix
- **Batch message** - "X tasks will be deleted"

---

## The `--yes` Flag Pattern

### Why Automation Needs `--yes`

**Problem: Scripts hang waiting for input**

```bash
#!/bin/bash
# Daily cleanup script

# This hangs waiting for confirmation
for id in $(todo list --status done | awk '{print $1}'); do
    todo remove $id  # ← Blocks waiting for y/n
done
```

**Solution: Skip confirmation with flag**

```bash
#!/bin/bash
# Daily cleanup script

for id in $(todo list --status done | awk '{print $1}'); do
    todo remove $id --yes  # ← Non-interactive, auto-confirm
done
```

### Implementation Pattern

```rust
#[derive(Subcommand)]
enum Commands {
    Remove {
        id: usize,
        
        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
    },
}
```

**Usage in command:**

```rust
if !yes {
    // Interactive mode - ask user
    if !confirm("Are you sure? [y/N]:")? {
        return Ok(());
    }
}
// Proceed with operation
```

**Both forms work:**

```bash
todo remove 5 --yes  # Long form
todo remove 5 -y     # Short form
```

---

## Input Trimming and Normalization

### Why `.trim()` is Essential

**Input includes newline:**

```rust
let mut input = String::new();
io::stdin().read_line(&mut input)?;
// User types: "yes" + Enter
// input = "yes\n"  ← includes newline!
```

**Without trim:**

```rust
matches!(input.as_str(), "y" | "yes")  // false! "yes\n" != "yes"
```

**With trim:**

```rust
let response = input.trim();  // "yes\n" → "yes"
matches!(response, "y" | "yes")  // true!
```

### Full Normalization Pipeline

```rust
let mut input = String::new();
io::stdin().read_line(&mut input)?;

let response = input
    .trim()           // Remove whitespace and newline
    .to_lowercase();  // Normalize case

// Now matches all variants:
// "y", "Y", "yes", "YES", "Yes", " y ", " YES "
matches!(response.as_str(), "y" | "yes")
```

---

## Error Handling Patterns

### Graceful Degradation

```rust
fn confirm(message: &str) -> Result<bool> {
    print!("{} ", message.yellow());
    
    // Handle flush errors gracefully
    if let Err(e) = io::stdout().flush() {
        eprintln!("Warning: failed to flush stdout: {}", e);
        // Continue anyway - best effort
    }
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    let response = input.trim().to_lowercase();
    Ok(matches!(response.as_str(), "y" | "yes"))
}
```

### Input Read Errors

```rust
let mut input = String::new();
match io::stdin().read_line(&mut input) {
    Ok(_) => {
        let response = input.trim().to_lowercase();
        Ok(matches!(response.as_str(), "y" | "yes"))
    }
    Err(e) => {
        eprintln!("Error reading input: {}", e);
        // Default to "no" for safety
        Ok(false)
    }
}
```

---

## UX Best Practices

### 1. Show Context Before Asking

```rust
// ❌ Bad - user doesn't know what they're confirming
if !confirm("Are you sure? [y/N]:")? {
    return Ok(());
}
remove_task(id)?;

// ✅ Good - clear what will happen
println!("Remove task: {}", task.text);
if !confirm("Are you sure? [y/N]:")? {
    return Ok(());
}
remove_task(id)?;
```

### 2. Use Semantic Colors

```rust
// Destructive operations
println!("{} {}", "Remove task:".red().bold(), task.text);

// Warnings
println!("{} {} tasks will be deleted!", "WARNING:".red().bold(), count);

// Cancellation messages
println!("{}", "Operation cancelled.".yellow());
```

### 3. Provide Clear Feedback

```rust
// After cancellation
println!("{}", "Removal cancelled.".yellow());

// After confirmation
println!("{} {}", "✓ Task removed:".red(), task.text.dimmed());
```

### 4. Consistent Prompt Format

```rust
// Always use same format
confirm("Are you sure? [y/N]:")
confirm("Continue? [y/N]:")
confirm("Delete all? [y/N]:")

// NOT mixed formats
confirm("Are you sure (y/n)?")  // ❌
confirm("Continue [Y/n]:")      // ❌ (different default)
```

---

## Testing Interactive Features

### Manual Testing

```bash
# Test confirmation acceptance
$ todo remove 1
Remove task: Test task
Are you sure? [y/N]: y
✓ Task removed

# Test confirmation rejection
$ todo remove 1
Remove task: Test task
Are you sure? [y/N]: n
Removal cancelled.

# Test case insensitivity
$ todo remove 1
Are you sure? [y/N]: YES  # Should work

# Test --yes flag
$ todo remove 1 --yes
✓ Task removed  # No prompt
```

### Edge Cases

```bash
# Empty input (should default to No)
Are you sure? [y/N]: [Enter]
Removal cancelled.

# Whitespace (should handle gracefully)
Are you sure? [y/N]:    y   
✓ Task removed

# Invalid input (should default to No)
Are you sure? [y/N]: maybe
Removal cancelled.
```

---

## Resources

- [std::io documentation](https://doc.rust-lang.org/std/io/)
- [matches! macro docs](https://doc.rust-lang.org/std/macro.matches.html)
- [Clap arguments](https://docs.rs/clap/latest/clap/)

---

**📚 See Also:**

- [CLI Design](cli-design.md) - Overall CLI patterns
- [Error Handling](error-handling.md) - Error management
- [Edit Command](../advanced/v1.9.0-edit-command-confirmation.md) - Complete implementation
