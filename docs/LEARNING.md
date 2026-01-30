# 📚 Learning Journey

> How I learned Rust by building this CLI, version by version

## 🎯 Project Goal

Learn Rust in practice by building something useful, documenting each decision and concept learned along the way.

## 📖 Navigation

- [Version History](#version-history)
- [Concepts Learned](#concepts-learned)
- [Design Decisions](#design-decisions)

---

## Version History

### v0.1.0 - Basic CLI

**🎯 Goal:** Create the foundation of a CLI with add/list functionality

**📦 Implementation:**

```rust
"add" => {
    let task = &args[2];
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("todos.txt")?;
    writeln!(file, "[ ] {}", task)?;
}
```

**🧠 Key Concepts:**

#### Why `OpenOptions` instead of `File::create()`?

```rust
// ❌ This OVERWRITES the entire file!
let file = File::create("todos.txt")?;

// ✅ OpenOptions gives granular control
let file = OpenOptions::new()
    .create(true)    // Create if doesn't exist
    .append(true)    // Don't overwrite, add to end
    .open("todos.txt")?;
```

#### The `?` operator

```rust
let file = open_file()?;  // Automatically propagates error
```

Equivalent verbose version:

```rust
let file = match open_file() {
    Ok(f) => f,
    Err(e) => return Err(e.into()),
};
```

**Why `?` is better:**

- Less code
- Clear intention
- Idiomatic Rust

#### Pattern matching for subcommands

```rust
match args[1].as_str() {
    "add" => { /* ... */ }
    "list" => { /* ... */ }
    _ => { eprintln!("Unknown command"); }
}
```

**🔗 Resources:**

- [Code v0.1.0](https://github.com/joaofelipegalvao/todo-cli/tree/v0.1.0)
- [Full diff](https://github.com/joaofelipegalvao/todo-cli/compare/v0.0.0...v0.1.0)

---

### v0.2.0 - Done Command

**🎯 Goal:** Mark tasks as completed

**📦 Implementation:**

```rust
"done" => {
    if args.len() < 3 {
        return Err("Usage: todo done <number>".into());
    }
    
    let number: usize = args[2].parse()?;
    let content = fs::read_to_string("todos.txt")?;
    
    let mut lines: Vec<String> = content
        .lines()
        .map(|l| l.to_string())
        .collect();
    
    let index = number - 1;
    lines[index] = lines[index].replace("[ ]", "[x]");
    
    fs::write("todos.txt", lines.join("\n") + "\n")?;
    println!("✓ Task marked as completed");
}
```

**🧠 Key Concepts:**

#### `.map().collect()` pattern

```rust
let mut lines: Vec<String> = content
    .lines()           // Iterator of &str
    .map(|l| l.to_string())  // Transform each &str → String
    .collect();        // Gather into Vec<String>
```

This is the **iterator pattern** - lazy evaluation until `collect()`.

**Why this pattern?**

- `.lines()` returns an iterator (doesn't allocate yet)
- `.map()` transforms each item (still lazy)
- `.collect()` forces evaluation and builds the Vec

#### Why `.to_string()` is needed

```rust
content.lines()  // Returns Iterator<Item = &str>
```

Each line is a **borrowed reference** (`&str`) to data in `content`. We need **owned** `String` values so we can:

1. Modify them (`.replace()`)
2. Store them in a Vec that outlives `content`

```rust
.map(|l| l.to_string())  // Converts &str → String (owned)
```

#### Parsing user input

```rust
let number: usize = args[2].parse()?;
```

**What's happening:**

- `args[2]` is a `String` (e.g., `"3"`)
- `.parse()` tries to convert it to `usize`
- `:usize` type annotation tells `parse()` what to produce
- `?` propagates error if user typed invalid number

**Error handling:**

```rust
// If user types "abc"
args[2].parse()?  // Returns Err → propagated to main
// User sees: "Error: invalid digit found in string"
```

#### Index conversion: 1-based → 0-based

```rust
let number: usize = args[2].parse()?;  // User sees: 1, 2, 3...
let index = number - 1;                 // Vec uses: 0, 1, 2...
lines[index] = lines[index].replace("[ ]", "[x]");
```

**Why this matters:**

- Users think in 1-based numbers (1st task, 2nd task)
- Rust Vecs are 0-indexed
- Must convert to avoid off-by-one errors

#### Writing back to file

```rust
fs::write("todos.txt", lines.join("\n") + "\n")?;
```

**Breaking it down:**

- `lines.join("\n")` - combines Vec elements with newlines between
- `+ "\n"` - adds final newline at end of file
- `fs::write()` - overwrites entire file with new content

**Why overwrite instead of append?**

- Can't "edit middle" of file efficiently
- Must read entire file, modify in memory, write back
- This is fine for small files (hundreds of tasks)

#### The `.replace()` method

```rust
lines[index] = lines[index].replace("[ ]", "[x]");
```

**What it does:**

- Finds first occurrence of `"[ ]"`
- Replaces it with `"[x]"`
- Returns a **new String** (doesn't modify original)

**Example:**

```rust
let task = "[ ] Buy milk";
let done = task.replace("[ ]", "[x]");
// done = "[x] Buy milk"
// task is unchanged (because String is immutable)
```

**Limitations not handled yet:**

- ❌ No validation if index is out of bounds
- ❌ No check if task is already done
- ❌ Empty file would panic

**These will be fixed in v0.3.0 and v0.4.2**

**🔗 Resources:**

- [Code v0.2.0](https://github.com/joaofelipegalvao/todo-cli/commit/a5626567103c1faa69c96caea1cab27ad6f89b14)
- [Full diff](https://github.com/joaofelipegalvao/todo-cli/compare/v0.1.0...v0.2.0)

---

### v0.3.0 - Remove Command

**🎯 Goal:** Delete specific tasks

**📦 Implementation:**

```rust
"remove" => {
    if args.len() < 3 {
        return Err("Usage: todo remove <number>".into());
    }
    
    let number: usize = args[2].parse()?;
    let content = fs::read_to_string("todos.txt")?;
    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    
    if number == 0 || number > lines.len() {
        return Err("Invalid task number".into());
    }
    
    let index = number - 1;
    lines.remove(index);
    
    fs::write("todos.txt", lines.join("\n") + "\n")?;
    println!("✓ Task removed");
}
```

**🧠 Key Concepts:**

#### Index validation

```rust
if number == 0 || number > lines.len() {
    return Err("Invalid task number".into());
}
```

**Why this validation is critical:**

**Case 1: Zero index**

```rust
// User types: todo remove 0
number == 0  // Invalid! Users see tasks numbered 1, 2, 3...
```

**Case 2: Out of bounds**

```rust
// File has 5 tasks, user types: todo remove 10
number > lines.len()  // Would panic at lines.remove(9)
```

**Without validation:**

```rust
lines.remove(9)  // panics: "index out of bounds"
// User sees: "thread 'main' panicked at..." ❌ Bad UX
```

**With validation:**

```rust
return Err("Invalid task number".into());
// User sees: "Error: Invalid task number" ✅ Clear message
```

#### `Vec::remove()` method

```rust
lines.remove(index);
```

**What it does:**

- Removes element at index `index`
- Shifts all following elements left
- Reduces Vec length by 1

**Example:**

```rust
let mut tasks = vec!["Task 1", "Task 2", "Task 3", "Task 4"];
tasks.remove(1);  // Remove "Task 2"
// Result: ["Task 1", "Task 3", "Task 4"]
//                     ↑ shifted left
```

**Performance note:**

- O(n) operation - must shift all elements after removed index
- For small lists (hundreds of tasks), this is fine
- For large lists (thousands), might need different data structure

#### Comparison: `remove()` vs `replace()`

```rust
// done command (v0.2.0) - modifies content
lines[index] = lines[index].replace("[ ]", "[x]");

// remove command (v0.3.0) - deletes entire element
lines.remove(index);
```

**Key difference:**

- `replace()` - task still exists, just changed
- `remove()` - task is gone, indices shift

**This affects user experience:**

```
Before remove(1):        After remove(1):
1. Task A                1. Task B  ← was 2, now 1
2. Task B                2. Task C  ← was 3, now 2
3. Task C
```

Users need to `list` again to see new numbers.

#### Validation added to `done` command

**Notice the improvement in v0.3.0:**

```rust
"done" => {
    // ... parsing ...
    
    // ✅ NEW: Validation added!
    if number == 0 || number > lines.len() {
        return Err("Invalid task number".into());
    }
    
    let index = number - 1;
    lines[index] = lines[index].replace("[ ]", "[x]");
    // ...
}
```

**Before v0.3.0:** `done` command could panic on invalid index  
**After v0.3.0:** Both `done` and `remove` have proper validation

**This is iterative improvement** - recognizing a pattern (validation) and applying it consistently.

**🔗 Resources:**

- [Code v0.3.0](https://github.com/joaofelipegalvao/todo-cli/tree/v0.3.0)
- [Full diff](https://github.com/joaofelipegalvao/todo-cli/compare/v0.2.0...v0.3.0)

---

### v0.4.0 - Undone Command

**🎯 Goal:** Unmark completed tasks (reverse of `done`)

**📦 Implementation:**

```rust
"undone" => {
    if args.len() < 3 {
        return Err("Usage: todo undone <number>".into());
    }
    
    let number: usize = args[2].parse()?;
    let content = fs::read_to_string("todos.txt")?;
    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    
    if number == 0 || number > lines.len() {
        return Err("Invalid task number".into());
    }
    
    let index = number - 1;
    lines[index] = lines[index].replace("[x]", "[ ]");  // ← Reverse!
    
    fs::write("todos.txt", lines.join("\n") + "\n")?;
    println!("✓ Task unmarked");
}
```

**🧠 Key Concepts:**

#### Inverse operations

```rust
// done: marks as completed
lines[index].replace("[ ]", "[x]")

// undone: marks as pending  
lines[index].replace("[x]", "[ ]")
```

**This is the power of simple data representation:**

- State is just text: `"[ ]"` or `"[x]"`
- Changing state is just string replacement
- Inverse operation is trivial

**Alternative design (that would be more complex):**

```rust
// If we had stored status separately:
struct Task {
    text: String,
    completed: bool,  // Now we need struct, serialization, etc.
}
```

**Our choice:** Keep it simple - plain text format enables easy state changes.

#### Code duplication notice

**Look at the pattern:**

```rust
// done, undone, and remove all have:
if args.len() < 3 { return Err(...); }
let number: usize = args[2].parse()?;
let content = fs::read_to_string("todos.txt")?;
let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
if number == 0 || number > lines.len() { return Err(...); }
let index = number - 1;
// ... specific operation ...
fs::write("todos.txt", lines.join("\n") + "\n")?;
```

**This duplication is intentional at this stage:**

- ✅ Learning step-by-step
- ✅ Each command is self-contained
- ✅ Easy to understand

**Later (v1.0.0+), this will be refactored** into helper functions. For now, repetition helps learning.

#### Boolean logic in action

The command structure now forms a **state machine:**

```
      ┌─────────┐
      │ Pending │
      │  [ ]    │
      └────┬────┘
           │
    done   │   undone
    ───────┼───────
           │
      ┌────▼────┐
      │Completed│
      │  [x]    │
      └─────────┘
```

Tasks can toggle between states, and `remove` deletes from any state.

**🔗 Resources:**

- [Code v0.4.0](https://github.com/joaofelipegalvao/todo-cli/tree/v0.4.0)
- [Full diff](https://github.com/joaofelipegalvao/todo-cli/compare/v0.3.0...v0.4.0)

---

### v0.4.1 - List Bug Fix

**🎯 Goal:** Handle empty lines properly in the list command

**📦 The Bug:**

**Before v0.4.1:**

```rust
"list" => match fs::read_to_string("todos.txt") {
    Ok(content) => {
        for (i, line) in content.lines().enumerate() {
            println!("{}. {}", i + 1, line);  // Shows empty lines!
        }
    }
    // ...
}
```

**Problem:**

```
File content:          Display output:
[ ] Task 1             1. [ ] Task 1
                       2.              ← Empty line!
[ ] Task 2             3. [ ] Task 2
```

**After v0.4.1:**

```rust
"list" => match fs::read_to_string("todos.txt") {
    Ok(content) => {
        let valid_lines: Vec<&str> = content
            .lines()
            .filter(|l| !l.trim().is_empty())  // ← The fix!
            .collect();
        
        if valid_lines.is_empty() {
            println!("No tasks");
        } else {
            for (i, line) in valid_lines.iter().enumerate() {
                println!("{}. {}", i + 1, line);
            }
        }
    }
    Err(_) => {
        println!("No tasks");
    }
}
```

**🧠 Key Concepts:**

#### Why `.trim()` is necessary

**Where do empty lines come from?**

```rust
// When we write file:
fs::write("todos.txt", lines.join("\n") + "\n")?;
//                                          ↑ adds final newline

// File content:
"[ ] Task 1\n[ ] Task 2\n"
//                      ↑ this creates an empty line when splitting
```

**Without trim:**

```rust
"[ ] Task 1\n[ ] Task 2\n".lines()
// Results in: ["[ ] Task 1", "[ ] Task 2", ""]
//                                            ↑ empty string!
```

**With trim:**

```rust
.filter(|l| !l.trim().is_empty())
// "".trim() = "" (still empty) → filtered out
// "   ".trim() = "" → also filtered out (whitespace-only lines)
```

#### The `.filter()` method

```rust
content.lines()
    .filter(|l| !l.trim().is_empty())
    .collect()
```

**How it works:**

- Takes each line from iterator
- Applies predicate: `!l.trim().is_empty()`
- Keeps only lines where predicate is `true`

**Example:**

```rust
let lines = vec!["Task 1", "", "  ", "Task 2"];
let valid: Vec<_> = lines.iter()
    .filter(|l| !l.trim().is_empty())
    .collect();
// Result: ["Task 1", "Task 2"]
```

#### Improved empty file handling

**Also added check for empty list:**

```rust
if valid_lines.is_empty() {
    println!("No tasks");
} else {
    // display tasks
}
```

**Why this matters:**

- File exists but has only empty lines
- Better UX than showing nothing

#### Edge cases now handled

✅ Empty file  
✅ File with only whitespace  
✅ File with trailing newlines  
✅ File with blank lines between tasks  

**Bug lesson:** Always test with "weird" input (empty files, extra newlines, whitespace).

**🔗 Resources:**

- [Code v0.4.1](https://github.com/joaofelipegalvao/todo-cli/tree/v0.4.1)
- [Full diff](https://github.com/joaofelipegalvao/todo-cli/compare/v0.4.0...v0.4.1)

---

### v0.4.2 - State Validations

**🎯 Goal:** Prevent invalid state transitions with specific error messages

**📦 Implementation:**

**Validation in `done` command:**

```rust
"done" => {
    // ... parsing and validation ...
    
    let index = number - 1;
    
    // ✅ NEW: Check if already completed
    if lines[index].contains("[x]") {
        return Err("Task is already marked as completed".into());
    }
    
    lines[index] = lines[index].replace("[ ]", "[x]");
    // ...
}
```

**Validation in `undone` command:**

```rust
"undone" => {
    // ... parsing and validation ...
    
    let index = number - 1;
    
    // ✅ NEW: Check if already pending
    if lines[index].contains("[ ]") {
        return Err("Task is already unmarked".into());
    }
    
    lines[index] = lines[index].replace("[x]", "[ ]");
    // ...
}
```

**Improved error message in `remove`:**

```rust
"remove" => {
    // ...
    if number == 0 || number > lines.len() {
        // ✅ More specific message
        return Err("This task doesn't exist or was already removed".into());
    }
    // ...
}
```

**Consistent filtering in all commands:**

```rust
// done, undone, and remove now all filter empty lines
let mut lines: Vec<String> = content
    .lines()
    .filter(|l| !l.trim().is_empty())  // ← Applied everywhere
    .map(|l| l.to_string())
    .collect();
```

**🧠 Key Concepts:**

#### Precondition validation

**What are preconditions?**

- Conditions that must be true before an operation
- Checked before performing the action
- Prevent invalid state transitions

**Example:**

```rust
// Precondition: task must be pending ([ ])
if lines[index].contains("[x]") {
    return Err("Task is already marked as completed".into());
}
// If we reach here, precondition is satisfied
lines[index] = lines[index].replace("[ ]", "[x]");
```

#### Specific vs generic error messages

**Before v0.4.2:**

```rust
// Generic - doesn't tell user what's wrong
if number > lines.len() {
    return Err("Invalid task number".into());
}
```

**After v0.4.2:**

```rust
// Specific - explains the actual problem
if lines[index].contains("[x]") {
    return Err("Task is already marked as completed".into());
}
```

**Why this matters:**

**User experience comparison:**

```bash
# Generic error
$ todo done 1
Error: Invalid task number
# User thinks: "But 1 is valid! What's wrong?"

# Specific error  
$ todo done 1
Error: Task is already marked as completed
# User thinks: "Oh, I already did this one!"
```

**Good error messages:**

1. ✅ Tell user what went wrong
2. ✅ Explain why it's wrong
3. ✅ (Ideally) Suggest how to fix it

#### State machine enforcement

**Valid transitions:**

```
[ ] ──done──> [x]
[x] ──undone──> [ ]
```

**Invalid transitions (now prevented):**

```
[x] ──done──> [x]  ❌ "Task is already marked as completed"
[ ] ──undone──> [ ]  ❌ "Task is already unmarked"
```

**This is defensive programming:**

- Assume user will make mistakes
- Validate before acting
- Provide helpful feedback

#### Consistency across commands

**Pattern established:**

All mutation commands now follow:

1. Parse arguments
2. Validate arguments (bounds)
3. Read file with empty line filtering
4. Validate preconditions (state)
5. Perform operation
6. Write file
7. Confirm to user

**This consistency:**

- ✅ Makes code predictable
- ✅ Easier to maintain
- ✅ Easier to add new commands

**🔗 Resources:**

- [Code v0.4.2](https://github.com/joaofelipegalvao/todo-cli/tree/v0.4.2)
- [Full diff](https://github.com/joaofelipegalvao/todo-cli/compare/v0.4.1...v0.4.2)

---

### v0.5.0 - Clear Command

**🎯 Goal:** Remove all tasks at once

**📦 Implementation:**

```rust
"clear" => {
    if fs::metadata("todos.txt").is_ok() {
        fs::remove_file("todos.txt")?;
        println!("✓ All tasks have been removed");
    } else {
        println!("No tasks to remove");
    }
}
```

**🧠 Key Concepts:**

#### `fs::metadata()` for file existence check

```rust
if fs::metadata("todos.txt").is_ok() {
    // File exists
} else {
    // File doesn't exist
}
```

**Why not just try to delete?**

```rust
// Without check
fs::remove_file("todos.txt")?;  
// If file doesn't exist → Error propagated to user
// User sees: "Error: No such file or directory" ❌
```

**With check:**

```rust
// With check
if fs::metadata("todos.txt").is_ok() {
    fs::remove_file("todos.txt")?;
    println!("✓ All tasks have been removed");
} else {
    println!("No tasks to remove");  // ✅ Friendly message
}
```

**User experience:**

```bash
# First time - file exists
$ todo clear
✓ All tasks have been removed

# Second time - already cleared
$ todo clear
No tasks to remove  # Not an error!
```

#### Why `.is_ok()` instead of `.unwrap()`?

```rust
fs::metadata("todos.txt").is_ok()  // Returns bool: true or false
```

**We don't care about the metadata**, only if the file exists.

**Alternatives:**

```rust
// ❌ Overkill
let metadata = fs::metadata("todos.txt")?;
// Why get metadata if we're just going to delete it?

// ✅ Simple
fs::metadata("todos.txt").is_ok()
```

#### `fs::remove_file()` behavior

```rust
fs::remove_file("todos.txt")?;
```

**What it does:**

- Deletes the file from filesystem
- **Permanent** - no undo!
- Returns `Result` - can fail if no permissions

**Different from clearing contents:**

```rust
// This would clear contents but keep file
fs::write("todos.txt", "")?;

// This deletes the file entirely
fs::remove_file("todos.txt")?;
```

**Why delete instead of clear?**

- Cleaner - file truly gone
- `list` command already handles missing file
- Consistent with "no tasks" state

#### Idempotent operations

**What's idempotency?**
> An operation that can be called multiple times with the same result

**clear is idempotent:**

```bash
todo clear  # Deletes file
todo clear  # No file to delete, but still succeeds
todo clear  # Still succeeds
```

**Why this matters:**

- ✅ No confusing errors
- ✅ Scripts can call it safely
- ✅ User doesn't have to check first

**🔗 Resources:**

- [Code v0.5.0](https://github.com/joaofelipegalvao/todo-cli/tree/v0.5.0)
- [Full diff](https://github.com/joaofelipegalvao/todo-cli/compare/v0.4.2...v0.5.0)

---

### v0.6.0 - Visual Interface with Colors

**🎯 Goal:** Add colorful visual hierarchy and progress statistics

**📦 Implementation:**

**Import colored crate:**

```rust
use colored::Colorize;
```

**Enhanced list display:**

```rust
"list" => match fs::read_to_string("todos.txt") {
    Ok(content) => {
        let valid_lines: Vec<&str> = content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .collect();

        if valid_lines.is_empty() {
            println!("No tasks");
        } else {
            println!("\n📋 Tasks:\n");

            let mut completed = 0;
            let total = valid_lines.len();

            for (i, line) in valid_lines.iter().enumerate() {
                let number = format!("{}.", i + 1);

                if line.contains("[x]") {
                    let text = line.replace("[x]", "").trim().to_string();
                    
                    println!(
                        "{} {} {}",
                        number.dimmed(),
                        "✅".green(),
                        text.green().strikethrough()
                    );
                    completed += 1;
                } else {
                    let text = line.replace("[ ]", "").trim().to_string();
                    println!(
                        "{} {} {}",
                        number.dimmed(),
                        "⏳".yellow(),
                        text.bright_white()
                    );
                }
            }

            println!("\n{}", "─".repeat(30).dimmed());

            let percentage = (completed as f32 / total as f32 * 100.0) as u32;
            let stats = format!("{} of {} completed ({}%)", completed, total, percentage);

            if percentage == 100 {
                println!("{}", stats.green().bold());
            } else if percentage >= 50 {
                println!("{}", stats.yellow());
            } else {
                println!("{}", stats.red());
            }

            println!();
        }
    }
    Err(_) => {
        println!("No tasks");
    }
}
```

**🧠 Key Concepts:**

#### The `colored` crate

```rust
use colored::Colorize;

"text".red()            // Red text
"text".green()          // Green text
"text".yellow()         // Yellow text
"text".dimmed()         // Dim/gray text
"text".bold()           // Bold text
"text".strikethrough()  // Strike̶t̶h̶r̶o̶u̶g̶h̶
```

**These are chainable:**

```rust
"text".green().bold()           // Bold green
"text".red().strikethrough()    // Red with strikethrough
```

#### Visual hierarchy

```rust
number.dimmed()          // De-emphasize task numbers
"✅".green()              // Completed indicator
text.green().strikethrough()  // Completed task
"⏳".yellow()             // Pending indicator
text.bright_white()     // Pending task (prominent)
```

**Design principle:**

1. **Numbers are dimmed** - helper info, not main content
2. **Icons are colored** - quick visual scan
3. **Completed tasks strikethrough** - clearly done
4. **Pending tasks bright** - what needs attention

#### Progress statistics with percentage

```rust
let completed = 0;
let total = valid_lines.len();

// Count completed during loop
if line.contains("[x]") {
    completed += 1;
}

// Calculate percentage
let percentage = (completed as f32 / total as f32 * 100.0) as u32;
```

**Type conversions explained:**

```rust
completed as f32   // usize → f32 (for division)
total as f32       // usize → f32
* 100.0            // f32 result
as u32             // f32 → u32 (truncate decimals)
```

**Why this chain?**

```rust
// Without conversion
let percentage = completed / total * 100;
// 2 / 5 * 100 = 0 * 100 = 0  ❌ Integer division!

// With f32
let percentage = (2 as f32 / 5 as f32 * 100.0) as u32;
// 2.0 / 5.0 * 100.0 = 40.0 → 40  ✅ Correct!
```

#### Dynamic color based on progress

```rust
if percentage == 100 {
    println!("{}", stats.green().bold());     // 🎉 All done!
} else if percentage >= 50 {
    println!("{}", stats.yellow());            // 📊 Making progress
} else {
    println!("{}", stats.red());               // 🔴 Just started
}
```

**Psychology:**

- **Green** = Achievement, success, positive reinforcement
- **Yellow** = In progress, keep going
- **Red** = Attention needed, urgency

#### Separator line

```rust
println!("\n{}", "─".repeat(30).dimmed());
```

**Creates visual separation:**

```
1. ⏳ Task 1
2. ✅ Task 2
──────────────────────────────  ← separator
2 of 2 completed (100%)
```

**Why dimmed?**

- Visual break without being loud
- Guides eye but doesn't distract

#### Colored feedback messages

```rust
println!("{}", "✓ Task added".green());                 // Success
println!("{}", "✓ Task marked as completed".green());   // Success
println!("{}", "✓ Task unmarked".yellow());             // Neutral
println!("{}", "✓ Task removed".red());                  // Destructive
println!("{}", "✓ All tasks have been removed".red().bold());  // Very destructive
```

**Semantic coloring:**

- **Green** - Creating, completing (positive actions)
- **Yellow** - Undoing (neutral/reversible)
- **Red** - Deleting (destructive/permanent)
- **Red + Bold** - Very destructive (clear all)

**🔗 Resources:**

- [Code v0.6.0](https://github.com/joaofelipegalvao/todo-cli/tree/v0.6.0)
- [Full diff](https://github.com/joaofelipegalvao/todo-cli/compare/v0.5.0...v0.6.0)
- [colored crate docs](https://docs.rs/colored/)

---

### v0.7.0 - Advanced Filters (--pending, --done)

**🎯 Goal:** Filter tasks by status with helper function for display

**📦 Implementation:**

**Helper function extracted:**

```rust
fn display_tasks(tasks: &[&str], title: &str) {
    println!("\n📋 {}:\n", title);

    let mut completed = 0;
    let total = tasks.len();

    for (i, line) in tasks.iter().enumerate() {
        // ... display logic (same as v0.6.0) ...
    }

    // ... statistics ...
}
```

**Filter logic in list command:**

```rust
"list" => {
    let filter = if args.len() > 2 {
        args[2].as_str()
    } else {
        "all"
    };

    match fs::read_to_string("todos.txt") {
        Ok(content) => {
            let valid_lines: Vec<&str> = content
                .lines()
                .filter(|l| !l.trim().is_empty())
                .collect();

            if valid_lines.is_empty() {
                println!("No tasks");
                return Ok(());
            }

            match filter {
                "--pending" => {
                    let pending: Vec<&str> = valid_lines
                        .iter()
                        .filter(|line| line.contains("[ ]"))
                        .copied()
                        .collect();

                    if pending.is_empty() {
                        println!("No pending tasks");
                    } else {
                        display_tasks(&pending, "Pending tasks");
                    }
                }

                "--done" => {
                    let completed: Vec<&str> = valid_lines
                        .iter()
                        .filter(|line| line.contains("[x]"))
                        .copied()
                        .collect();

                    if completed.is_empty() {
                        println!("No completed tasks");
                    } else {
                        display_tasks(&completed, "Completed tasks");
                    }
                }

                "all" => {
                    display_tasks(&valid_lines, "Tasks");
                }

                _ => {
                    return Err(format!(
                        "Invalid filter: {}. Use --pending or --done",
                        filter
                    ).into());
                }
            }
        }
        Err(_) => {
            println!("No tasks");
        }
    }
}
```

*See LEARNING.md for complete detailed explanation of DRY principle, slices, and filter patterns.*

**🔗 Resources:**

- [Code v0.7.0](https://github.com/joaofelipegalvao/todo-cli/tree/v0.7.0)
- [Full diff](https://github.com/joaofelipegalvao/todo-cli/compare/v0.6.0...v0.7.0)

---

### v0.8.0 - Priority System + Priority Filters

**🎯 Goal:** Add three-level priority system with visual indicators and filtering

**📦 Key Implementations:**

**Priority extraction function:**

```rust
fn extract_priority(line: &str) -> (&str, String) {
    let without_checkbox = line
        .replace("[ ]", "")
        .replace("[x]", "")
        .trim()
        .to_string();

    if without_checkbox.contains("(high)") {
        let text = without_checkbox.replace("(high)", "").trim().to_string();
        ("high", text)
    } else if without_checkbox.contains("(low)") {
        let text = without_checkbox.replace("(low)", "").trim().to_string();
        ("low", text)
    } else {
        ("medium", without_checkbox)
    }
}
```

**Priority emoji function:**

```rust
fn priority_emoji(priority: &str) -> String {
    match priority {
        "high" => "🔴".red().to_string(),
        "low" => "🟢".green().to_string(),
        _ => "🟡".yellow().to_string(),
    }
}
```

**Adding tasks with priority:**

```rust
"add" => {
    // ... validation ...
    
    let line = match args.len() {
        3 => format!("[ ] {}", task),  // No flag = medium
        
        4 => {
            let flag = args[3].as_str();
            match flag {
                "--high" => format!("[ ] (high) {}", task),
                "--low" => format!("[ ] (low) {}", task),
                _ => return Err(format!("Invalid flag '{}'. Use --high or --low", flag).into()),
            }
        }
        
        _ => return Err("Usage: todo add <task> [--high | --low]. Only one flag allowed".into()),
    };
    
    // Write to file...
}
```

**Multi-flag parsing in list:**

```rust
"list" => {
    let mut status_filter = "all";
    let mut priority_filter: Option<&str> = None;

    for arg in &args[2..] {
        match arg.as_str() {
            "--pending" => {
                if status_filter != "all" {
                    return Err("Use only one status filter (--pending or --done)".into());
                }
                status_filter = "pending";
            }
            "--done" => {
                if status_filter != "all" {
                    return Err("Use only one status filter".into());
                }
                status_filter = "done";
            }
            "--high" => {
                if priority_filter.is_some() {
                    return Err("Use only one priority filter".into());
                }
                priority_filter = Some("high");
            }
            "--low" => {
                if priority_filter.is_some() {
                    return Err("Use only one priority filter".into());
                }
                priority_filter = Some("low");
            }
            _ => return Err(format!("Invalid filter: {}", arg).into()),
        }
    }
    
    // Apply filters sequentially...
    
    // First filter by status
    valid_lines = match status_filter {
        "pending" => valid_lines.iter().filter(|l| l.contains("[ ]")).copied().collect(),
        "done" => valid_lines.iter().filter(|l| l.contains("[x]")).copied().collect(),
        _ => valid_lines,
    };

    // Then filter by priority (if specified)
    if let Some(pri) = priority_filter {
        valid_lines = valid_lines
            .iter()
            .filter(|line| {
                let (priority, _) = extract_priority(line);
                priority == pri
            })
            .copied()
            .collect();
    }
    
    // Dynamic title based on filters
    let title = match (status_filter, priority_filter) {
        ("pending", Some("high")) => "High priority pending tasks",
        ("pending", Some("low")) => "Low priority pending tasks",
        ("pending", None) => "Pending tasks",
        ("done", Some("high")) => "High priority completed tasks",
        ("done", Some("low")) => "Low priority completed tasks",
        ("done", None) => "Completed tasks",
        (_, Some("high")) => "High priority",
        (_, Some("low")) => "Low priority",
        _ => "Tasks",
    };
}
```

*This version added 200+ lines implementing the complete priority system with parsing, filtering, validation, and display logic. See original explanation for detailed breakdown of Option<T>, pattern matching with tuples, and design decisions.*

**🔗 Resources:**

- [Code v0.8.0](https://github.com/joaofelipegalvao/todo-cli/tree/v0.8.0)
- [Full diff](https://github.com/joaofelipegalvao/todo-cli/compare/v0.7.0...v0.8.0)

---

### v0.9.0 - Priority Sorting

**🎯 Goal:** Sort tasks by priority (high → medium → low)

**📦 Key Implementation:**

**Priority mapping function:**

```rust
fn priority_order(pri: &str) -> u8 {
    match pri {
        "high" => 0,      // First
        "medium" => 1,    // Middle
        "low" => 2,       // Last
        _ => 3,           // Unknown (end)
    }
}
```

**Sort flag handling:**

```rust
"list" => {
    let mut status_filter = "all";
    let mut priority_filter: Option<&str> = None;
    let mut sort = false;  // ← NEW

    for arg in &args[2..] {
        match arg.as_str() {
            // ... other flags ...
            "--sort" => {
                if sort {
                    return Err("Use --sort only once.".into());
                }
                sort = true;
            }
            _ => return Err(format!("Invalid filter: {}", arg).into()),
        }
    }

    // ... filter tasks ...

    // Sort AFTER filtering (optimization!)
    if sort {
        valid_lines.sort_by(|a, b| {
            let (pri_a, _) = extract_priority(a);
            let (pri_b, _) = extract_priority(b);
            priority_order(pri_a).cmp(&priority_order(pri_b))
        });
    }
    
    // ... display ...
}
```

*Key insight: Sort happens AFTER filtering for performance (O(5 log 5) vs O(50 log 50)). See original explanation for detailed breakdown of .sort_by(), Ordering enum, and optimization rationale.*

**🔗 Resources:**

- [Code v0.9.0](https://github.com/joaofelipegalvao/todo-cli/tree/v0.9.0)
- [Full diff](https://github.com/joaofelipegalvao/todo-cli/compare/v0.8.0...v0.9.0)

---

### v1.0.0 - Search + Display Refactoring

**🎯 Goal:** Add search command and refactor display into atomic/orchestrated functions

**📦 Key Implementations:**

**Atomic rendering function:**

```rust
fn display_task(number: usize, line: &str) {
    let number_fmt = format!("{}.", number);
    let completed = line.contains("[x]");

    let (priority, text) = extract_priority(line);
    let emoji = priority_emoji(priority);

    if completed {
        println!(
            "{} {} {} {}",
            number_fmt.dimmed(),
            emoji,
            "✅".green(),
            text.green().strikethrough()
        );
    } else {
        println!(
            "{} {} {} {}",
            number_fmt.dimmed(),
            emoji,
            "⏳".yellow(),
            text.bright_white()
        );
    }
}
```

**Orchestrated rendering function (renamed):**

```rust
fn display_lists(tasks: &[&str], title: &str) {
    println!("\n📋 {}:\n", title);

    let mut completed = 0;
    let total = tasks.len();

    for (i, line) in tasks.iter().enumerate() {
        display_task(i + 1, line);  // ← Uses atomic function

        if line.contains("[x]") {
            completed += 1;
        }
    }

    // ... statistics ...
}
```

**Search command:**

```rust
"search" => {
    if args.len() < 3 {
        return Err("Usage: todo search <term>".into());
    }

    let term = &args[2];

    match fs::read_to_string("todos.txt") {
        Ok(content) => {
            let valid_lines: Vec<&str> = content
                .lines()
                .filter(|l| !l.trim().is_empty())
                .collect();

            if valid_lines.is_empty() {
                println!("No tasks");
                return Ok(());
            }

            let mut results: Vec<(usize, &str)> = Vec::new();

            for (i, line) in valid_lines.iter().enumerate() {
                if line.to_lowercase().contains(&term.to_lowercase()) {
                    results.push((i + 1, line));  // Keep original number!
                }
            }

            if results.is_empty() {
                println!("No results for '{}'", term);
            } else {
                println!("\n📋 Results for \"{}\":\n", term);

                for (number, line) in &results {
                    display_task(*number, line);  // ← Uses atomic function
                }

                println!("\n{} result(s) found\n", results.len());
            }
        }
        Err(_) => {
            println!("No tasks");
        }
    }
}
```

**🧠 Key Design:**

**Atomic vs Orchestrated:**

- `display_task()` - Renders ONE task, maintains original numbering
- `display_lists()` - Renders LIST with statistics, renumbers sequentially

**Why search uses atomic function:**

- Must keep original task numbers (so user can run `todo done 5`)
- No statistics needed (partial results)
- Different semantic meaning than full list

This is **mature CLI architecture** - not duplication, but proper separation of concerns.

**🔗 Resources:**

- [Code v1.0.0](https://github.com/joaofelipegalvao/todo-cli/tree/v1.0.0)
- [Full diff](https://github.com/joaofelipegalvao/todo-cli/compare/v0.9.0...v1.0.0)

---

### v1.1.0 - Medium Priority Filter

**🎯 Goal:** Add `--medium` filter to complete priority filtering system

**📦 Problem Identified:**

Users could create medium priority tasks but couldn't filter by them:

```bash
# ✅ Can create medium priority
todo add "Normal task"              # Creates as medium (default)
todo add "Normal task" --medium     # Future: explicit medium

# ❌ Can't filter medium priority
todo list --high    # Works
todo list --low     # Works
todo list --medium  # Error: Invalid filter ❌
```

**This was a design flaw** - asymmetry between creation and querying.

**📦 Implementation:**

```rust
"list" => {
    // ... existing code ...
    
    for arg in &args[2..] {
        match arg.as_str() {
            "--pending" => { /* ... */ }
            "--done" => { /* ... */ }
            "--high" => {
                if priority_filter.is_some() {
                    return Err("Use only one priority filter (--high, --medium or --low)".into());
                }
                priority_filter = Some("high");
            }
            "--medium" => {  // ← NEW
                if priority_filter.is_some() {
                    return Err("Use only one priority filter (--high, --medium or --low)".into());
                }
                priority_filter = Some("medium");
            }
            "--low" => {
                if priority_filter.is_some() {
                    return Err("Use only one priority filter (--high, --medium or --low)".into());
                }
                priority_filter = Some("low");
            }
            // ... rest of filters ...
        }
    }
    
    // Update dynamic titles to include medium
    let title = match (status_filter, priority_filter) {
        ("pending", Some("high")) => "High priority pending tasks",
        ("pending", Some("medium")) => "Medium priority pending tasks",  // ← NEW
        ("pending", Some("low")) => "Low priority pending tasks",
        ("pending", None) => "Pending tasks",
        ("done", Some("high")) => "High priority completed tasks",
        ("done", Some("medium")) => "Medium priority completed tasks",  // ← NEW
        ("done", Some("low")) => "Low priority completed tasks",
        ("done", None) => "Completed tasks",
        (_, Some("high")) => "High priority",
        (_, Some("medium")) => "Medium priority",  // ← NEW
        (_, Some("low")) => "Low priority",
        _ => "Tasks",
    };
}
```

**🧠 Key Concepts:**

#### Why this fix was necessary

**Original flawed assumption:**
> "Medium is the default, so users don't need to filter by it"

**Reality:**
> "Medium is the default, so MOST tasks will be medium, making filtering essential"

**Analogy:**

```
E-commerce without "medium price" filter:
- Filter by cheap ✅
- Filter by expensive ✅  
- Filter by medium ❌  ← where most products are!
```

Doesn't make sense!

#### The problem in practice

```bash
# Scenario: 50 total tasks
# 5 high
# 40 medium  ← MAJORITY
# 5 low

# Without --medium:
todo list              # Shows all 50 (overwhelming)
todo list --pending    # Still shows high + medium + low mixed

# Can't focus on the 40 medium tasks!
```

**With --medium:**

```bash
todo list --medium              # Shows 40 medium tasks
todo list --pending --medium    # Focus on pending medium work
```

#### Symmetry in design

**Before (asymmetric):**

```
Creation:
--high   ✅
--medium (implicit) ✅
--low    ✅

Filtering:
--high   ✅
--medium ❌  ← Missing!
--low    ✅
```

**After (symmetric):**

```
Creation:
--high   ✅
--medium ✅
--low    ✅

Filtering:
--high   ✅
--medium ✅  ← Complete!
--low    ✅
```

**Symmetry = predictability = better UX**

#### Design principle learned

> "Defaults are for INPUT convenience, not QUERY limitation"

**Input (creation):**

- Default to medium → users don't always need to specify
- Makes quick task creation easy

**Query (filtering):**

- Provide ALL options → users need full control
- Never assume they don't need to filter by default value

#### Real-world impact

**Scenario:** Developer with 100 tasks

```bash
# Distribution:
# 10 high priority (urgent work)
# 75 medium priority (regular work)  ← BULK OF WORK
# 15 low priority (nice to have)

# Without --medium:
todo list --high       # See 10 tasks (good)
todo list --low        # See 15 tasks (good)
# How to see the 75 medium tasks? 🤔
todo list              # Shows all 100 (too much!)

# With --medium:
todo list --medium     # See 75 tasks (perfect!)
todo list --pending --medium  # Even more focused
```

**This is essential for real-world usage with many tasks.**

#### Consistency with user expectations

**User mental model:**

```bash
# "If I can create with --medium, I should be able to filter by --medium"
todo add "Task" --medium   # (future feature)
todo list --medium         # Should work!
```

**Breaking this expectation creates confusion:**

- "Why can't I filter by the priority I created?"
- "Did I do something wrong?"
- "Is medium not a real priority?"

#### Code changes minimal, impact maximal

**What changed:**

- ✅ Added one `match` arm (`"--medium" => { ... }`)
- ✅ Updated 3 title strings
- ✅ Updated error messages to include "medium"

**Impact:**

- ✅ Complete feature symmetry
- ✅ Solves real use case (filter majority of tasks)
- ✅ Meets user expectations
- ✅ Professional, complete API

#### Testing the fix

```bash
# Setup
rm todos.txt
todo add "Urgent work" --high
todo add "Regular task 1"
todo add "Regular task 2"
todo add "Regular task 3"
todo add "Regular task 4"
todo add "Regular task 5"
todo add "Low priority task" --low

# Test 1: Filter by medium (NEW!)
todo list --medium
# Output:
# 📋 Medium priority:
# 2. 🟡 ⏳ Regular task 1
# 3. 🟡 ⏳ Regular task 2
# 4. 🟡 ⏳ Regular task 3
# 5. 🟡 ⏳ Regular task 4
# 6. 🟡 ⏳ Regular task 5

# Test 2: Combine with status filter
todo list --pending --medium
# Shows only pending medium tasks

# Test 3: All three priorities work
todo list --high    # ✅ 1 task
todo list --medium  # ✅ 5 tasks (NEW!)
todo list --low     # ✅ 1 task

# Test 4: Sorting still works
todo list --medium --sort
# Medium tasks already same priority, but shows consistency
```

#### Why this is a "design fix" not just a "feature"

**Not a new feature because:**

- Doesn't add new capability
- Completes existing priority system
- Fixes asymmetry/incompleteness

**It's a design fix because:**

- Original design was flawed (incomplete)
- Creates consistency users expect
- Solves problem users encounter in practice

**Lesson:** Even with careful planning, design flaws can slip through. **Critical thinking and real usage** reveal them. This is why:

- ✅ Testing with realistic data matters
- ✅ Questioning design decisions is valuable
- ✅ User perspective differs from developer perspective

**🔗 Resources:**

- [Code v1.1.0](https://github.com/joaofelipegalvao/todo-cli/tree/v1.1.0)
- [Full diff](https://github.com/joaofelipegalvao/todo-cli/compare/v1.0.1...v1.1.0)

---

### v1.2.0 - Struct Refactoring (Type-Safe Architecture)

**🎯 Goal:** Replace string parsing with type-safe structs and enums for maintainability and extensibility

**📦 The Problem We're Solving:**

**Before v1.2.0:**

Every command parsed strings manually:

```rust
// Parsing priority from strings (repeated everywhere)
let without_checkbox = line.replace("[ ]", "").replace("[x]", "").trim();
if without_checkbox.contains("(high)") {
    let text = without_checkbox.replace("(high)", "").trim();
    // ... handle high priority
} else if without_checkbox.contains("(low)") {
    // ... handle low priority
}

// Checking completion status (string matching)
if line.contains("[x]") {
    // completed
} else {
    // pending
}
```

**Problems:**

❌ String parsing repeated in every command  
❌ No type safety - typos like `"hihg"` compile fine  
❌ Hard to add new fields (timestamps, tags)  
❌ Can't leverage Rust's type system  
❌ Prone to bugs (what if format changes?)  

**Example of the mess:**

```rust
// In 'add' command
format!("[ ] (high) {}", task)

// In 'list' command  
.filter(|line| line.contains("(high)"))

// In 'display' command
if line.contains("(high)") { "🔴" } else { "🟡" }

// All coupled to string format "[ ] (high) Task"
// Change format → must update EVERYWHERE
```

---

**📦 The Solution: Structs + Enums**

**Core data structures:**

```rust
#[derive(Debug, Clone, PartialEq, Copy)]
enum Priority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq)]
struct Task {
    text: String,
    completed: bool,
    priority: Priority,
}
```

**Why this is revolutionary:**

✅ **Type-safe** - compiler catches mistakes  
✅ **Self-documenting** - clear what a Task contains  
✅ **Extensible** - add fields easily  
✅ **DRY** - parsing logic in ONE place  
✅ **Testable** - can unit test Task methods  

**🧠 Key Concepts:**

#### What is a `struct`?

Think of it as a **custom data type** that groups related data:

```rust
// Instead of passing around 3 separate values:
fn display_task(text: String, completed: bool, priority: String) { }

// We bundle them into one logical unit:
fn display_task(task: &Task) { }
```

**Real-world analogy:**

```rust
// Like a form with labeled fields:
struct Person {
    name: String,      // "John Doe"
    age: u32,          // 25
    email: String,     // "john@example.com"
}

// Instead of passing 3 separate strings/numbers around
```

**Our Task struct:**

```rust
struct Task {
    text: String,       // "Study Rust"
    completed: bool,    // false
    priority: Priority, // High
}
```

#### What is an `enum`?

An enum represents **one of several possible values**:

```rust
enum Priority {
    High,    // ← can ONLY be one of these
    Medium,  // ← mutually exclusive
    Low,     // ← not multiple at once
}
```

**Why enum vs strings?**

```rust
// ❌ String approach - error-prone
let priority = "hihg";  // Typo! Compiles fine, breaks at runtime
if priority == "high" { }  // Won't match, silent bug

// ✅ Enum approach - compile-time safety
let priority = Priority::Hihg;  // ERROR: no variant `Hihg`
// Won't compile! Catches typo immediately
```

**Enums are exhaustive:**

```rust
match priority {
    Priority::High => "🔴",
    Priority::Medium => "🟡",
    // Forgot Low? ❌ Compiler error: "non-exhaustive patterns"
}
```

Compiler **forces** you to handle all cases!

#### The `#[derive(...)]` attributes

```rust
#[derive(Debug, Clone, PartialEq, Copy)]
enum Priority {
    High,
    Medium,
    Low,
}
```

**What do these mean?**

**`Debug`** - Enables printing for debugging:

```rust
let pri = Priority::High;
println!("{:?}", pri);  // Output: High
```

Without `Debug`, this would fail!

**`Clone`** - Enables creating copies:

```rust
let pri1 = Priority::High;
let pri2 = pri1.clone();  // Explicit copy
```

**`PartialEq`** - Enables comparison:

```rust
if task.priority == Priority::High {
    // This works because of PartialEq
}
```

**`Copy`** - Enables implicit copying:

```rust
let pri1 = Priority::High;
let pri2 = pri1;  // Automatically copied (not moved!)
// pri1 still valid here ✅
```

**Why Copy is important:**

```rust
// Without Copy:
let pri = Priority::High;
some_function(pri);  // pri moved into function
// pri no longer accessible here! ❌

// With Copy:
let pri = Priority::High;
some_function(pri);  // pri copied into function
// pri still valid here! ✅
```

**When can you use Copy?**

Only for types that are "cheap to copy" (no heap allocation):

✅ Numbers: `i32`, `u8`, `f64`  
✅ Booleans: `bool`  
✅ Simple enums: `Priority`  
❌ Strings: `String` (heap-allocated)  
❌ Vectors: `Vec<T>` (heap-allocated)  

**Our Priority enum is 1 byte - perfect for Copy!**

#### `impl` blocks - Adding methods to types

```rust
impl Priority {
    fn order(&self) -> u8 {
        match self {
            Priority::High => 0,
            Priority::Medium => 1,
            Priority::Low => 2,
        }
    }

    fn emoji(&self) -> ColoredString {
        match self {
            Priority::High => "🔴".red(),
            Priority::Medium => "🟡".yellow(),
            Priority::Low => "🟢".green(),
        }
    }
}
```

**What's `&self`?**

```rust
fn emoji(&self) -> ColoredString
//       ↑ "self" = the Priority value we're calling this on
```

**Usage:**

```rust
let pri = Priority::High;
let icon = pri.emoji();  // "self" is `pri`
//         ↑ calling method ON the value
```

**Why this is powerful:**

```rust
// Before (global function):
fn priority_emoji(priority: &str) -> String {
    match priority {
        "high" => "🔴",
        // ...
    }
}
let icon = priority_emoji("high");  // Must pass string

// After (method):
impl Priority {
    fn emoji(&self) -> ColoredString {
        match self {
            Priority::High => "🔴".red(),
            // ...
        }
    }
}
let icon = Priority::High.emoji();  // Called ON the type
```

**Methods are "attached" to the type** - cleaner and more discoverable.

#### Implementing Task methods

```rust
impl Task {
    // Constructor - creates new Task
    fn new(text: String, priority: Priority) -> Self {
        Self {
            text,
            completed: false,  // Always starts incomplete
            priority,
        }
    }

    // Convert Task → string format for file
    fn to_line(&self) -> String {
        let checkbox = if self.completed { "[x]" } else { "[ ]" };
        let pri_str = match self.priority {
            Priority::High => " (high)",
            Priority::Low => " (low)",
            Priority::Medium => "",
        };
        format!("{}{} {}", checkbox, pri_str, self.text)
    }

    // Parse string → Task (returns Option because can fail)
    fn from_line(line: &str) -> Option<Self> {
        // ... parsing logic ...
    }

    // Mark as done
    fn mark_done(&mut self) {
        self.completed = true;
    }

    // Mark as pending
    fn mark_undone(&mut self) {
        self.completed = false;
    }
}
```

**Key patterns:**

**`Self` vs `self`:**

```rust
fn new(text: String) -> Self {  // "Self" = Task type
    Self { text, ... }           // Constructs Task instance
}

fn mark_done(&mut self) {       // "self" = this instance
    self.completed = true;       // Modifies this instance
}
```

**`&mut self` - Mutable reference:**

```rust
fn mark_done(&mut self) {
//           ↑ need mut to modify
    self.completed = true;
}

// Usage:
let mut task = Task::new("Study".into(), Priority::High);
task.mark_done();  // Modifies task
```

**Why `Option<Self>` for parsing?**

```rust
fn from_line(line: &str) -> Option<Self> {
    // Parsing can fail if line is invalid
    if line.is_empty() {
        return None;  // Invalid - return nothing
    }
    
    Some(Task {  // Valid - return Task wrapped in Some
        text: "...".to_string(),
        completed: false,
        priority: Priority::Medium,
    })
}
```

**Using it:**

```rust
match Task::from_line("[ ] Task") {
    Some(task) => println!("Parsed: {:?}", task),
    None => println!("Invalid line"),
}

// Or with if let:
if let Some(task) = Task::from_line("[ ] Task") {
    println!("Got: {:?}", task);
}
```

#### Centralized I/O with `load_tasks` and `save_tasks`

**Before - I/O scattered everywhere:**

```rust
// In 'add' command
let mut file = OpenOptions::new().append(true).open("todos.txt")?;
writeln!(file, "[ ] (high) {}", task)?;

// In 'done' command
let content = fs::read_to_string("todos.txt")?;
let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
// ... modify ...
fs::write("todos.txt", lines.join("\n"))?;

// In 'remove' command
let content = fs::read_to_string("todos.txt")?;
// ... same parsing again ...
```

**After - Centralized:**

```rust
fn load_tasks() -> Result<Vec<Task>, Box<dyn Error>> {
    match fs::read_to_string("todos.txt") {
        Ok(content) => {
            let tasks: Vec<Task> = content
                .lines()
                .filter(|l| !l.trim().is_empty())
                .filter_map(Task::from_line)  // Parsing in ONE place
                .collect();
            Ok(tasks)
        }
        Err(_) => Ok(Vec::new()),  // Missing file = empty list
    }
}

fn save_tasks(tasks: &[Task]) -> Result<(), Box<dyn Error>> {
    let lines: Vec<String> = tasks.iter().map(|t| t.to_line()).collect();
    fs::write("todos.txt", lines.join("\n") + "\n")?;
    Ok(())
}
```

**Now every command:**

```rust
"done" => {
    let mut tasks = load_tasks()?;  // Load
    tasks[index].mark_done();       // Modify
    save_tasks(&tasks)?;            // Save
}
```

**Benefits:**

✅ **Single source of truth** for parsing  
✅ **Consistent error handling**  
✅ **Easy to change format** (only edit 2 functions)  
✅ **Ready for JSON/database** later  

#### Understanding `.filter_map()`

```rust
content
    .lines()                           // Iterator<Item = &str>
    .filter(|l| !l.trim().is_empty())  // Remove empty lines
    .filter_map(Task::from_line)       // Parse + filter in one step
    .collect()
```

**What's `.filter_map()`?**

It combines `.map()` and `.filter()`:

```rust
// Long way:
.map(|line| Task::from_line(line))     // Vec<Option<Task>>
.filter(|opt| opt.is_some())           // Remove None values
.map(|opt| opt.unwrap())               // Unwrap Some(Task)

// Short way:
.filter_map(Task::from_line)           // Vec<Task> directly!
```

**How it works:**

```rust
fn filter_map<B, F>(self, f: F) -> FilterMap<Self, F>
where
    F: FnMut(Self::Item) -> Option<B>,
//                          ↑ Function returns Option
```

- If function returns `Some(value)` → keep value
- If function returns `None` → skip

**Example:**

```rust
let lines = vec!["[ ] Task 1", "", "invalid", "[ ] Task 2"];

let tasks: Vec<Task> = lines
    .iter()
    .filter_map(|line| Task::from_line(line))
    .collect();

// Result: vec![Task("Task 1"), Task("Task 2")]
// Empty and invalid lines silently filtered out ✅
```

#### Function pointers vs closures

```rust
// ❌ Clippy warning:
.filter_map(|line| Task::from_line(line))

// ✅ Idiomatic:
.filter_map(Task::from_line)
```

**Why is the second better?**

**Closures have overhead:**

```rust
|line| Task::from_line(line)
// Creates closure that:
// 1. Captures environment (even if nothing to capture)
// 2. Calls function
// 3. Returns result
```

**Function pointers are direct:**

```rust
Task::from_line
// Direct reference to function
// No closure creation
// Compiler can inline better
```

**When can you do this?**

When signatures match exactly:

```rust
// Function signature:
fn from_line(line: &str) -> Option<Task>

// .filter_map expects:
fn(Item) -> Option<B>

// Perfect match! ✅
```

**When you CAN'T:**

```rust
// Need to transform:
.map(|x| x + 1)  // Can't use function pointer

// Need to capture:
let prefix = "Task: ";
.map(|x| format!("{}{}", prefix, x))  // Closure needed
```

#### Refactoring Strategy

**Step-by-step approach:**

1. ✅ Add structs/enums (doesn't break existing code)
2. ✅ Add helper functions (`load_tasks`, `save_tasks`)
3. ✅ Refactor one command at a time
4. ✅ Test after each change
5. ✅ Delete old code only when fully replaced

**This incremental approach:**

- Reduces risk of breaking everything
- Allows testing at each step
- Easy to roll back if needed
- Teaches one concept at a time

#### Command Refactoring Examples

**Before/After: `add` command**

```rust
// Before (string manipulation):
"add" => {
    let task = &args[2];
    
    let line = match args.len() {
        3 => format!("[ ] {}", task),
        4 => {
            let flag = args[3].as_str();
            match flag {
                "--high" => format!("[ ] (high) {}", task),
                "--low" => format!("[ ] (low) {}", task),
                _ => return Err("Invalid flag".into()),
            }
        }
        _ => return Err("Usage: todo add <task> [--high | --low]".into()),
    };
    
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("todos.txt")?;
    
    writeln!(file, "{}", line)?;
    println!("{}", "✓ Task added".green());
}

// After (using Task):
"add" => {
    if args.len() < 3 {
        return Err("Usage: todo add <task> [--high | --low]".into());
    }

    let text = args[2].clone();
    
    let priority = if args.len() >= 4 {
        match args[3].as_str() {
            "--high" => Priority::High,
            "--medium" => Priority::Medium,
            "--low" => Priority::Low,
            _ => return Err(format!("Invalid flag '{}'", args[3]).into()),
        }
    } else {
        Priority::Medium
    };

    let task = Task::new(text, priority);
    
    let mut tasks = load_tasks()?;
    tasks.push(task);
    save_tasks(&tasks)?;

    println!("{}", "✓ Task added".green());
}
```

**Improvements:**

✅ **Type-safe priority** - `Priority::High` not `"high"`  
✅ **No string formatting** - `Task::new()` handles it  
✅ **Centralized I/O** - `load_tasks/save_tasks`  
✅ **Clearer logic** - no nested string matching  

**Before/After: `done` command**

```rust
// Before (string replacement):
"done" => {
    let number: usize = args[2].parse()?;
    let content = fs::read_to_string("todos.txt")?;
    
    let mut lines: Vec<String> = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.to_string())
        .collect();
    
    if number == 0 || number > lines.len() {
        return Err("Invalid task number".into());
    }
    
    let index = number - 1;
    
    if lines[index].contains("[x]") {
        return Err("Task is already marked as completed".into());
    }
    
    lines[index] = lines[index].replace("[ ]", "[x]");
    
    fs::write("todos.txt", lines.join("\n") + "\n")?;
    println!("{}", "✓ Task marked as completed".green());
}

// After (using Task):
"done" => {
    if args.len() < 3 {
        return Err("Usage: todo done <number>".into());
    }

    let number: usize = args[2].parse()?;
    let mut tasks = load_tasks()?;

    if number == 0 || number > tasks.len() {
        return Err("Invalid task number".into());
    }

    let index = number - 1;

    if tasks[index].completed {
        return Err("Task is already marked as completed".into());
    }

    tasks[index].mark_done();
    save_tasks(&tasks)?;

    println!("{}", "✓ Task marked as completed".green());
}
```

**Improvements:**

✅ **15 lines → 10 lines** (33% reduction)  
✅ **Direct field check** - `tasks[index].completed`  
✅ **Dedicated method** - `mark_done()`  
✅ **No string manipulation**  
✅ **Clearer intent**  

#### Problems Encountered

**Issue 1: Ownership error with `priority_filter`**

**The bug:**

```rust
if let Some(pri) = priority_filter {
//          ↑ MOVE happens here
    tasks = tasks.into_iter().filter(|t| t.priority == pri).collect();
}

let title = match (status_filter, priority_filter) {
//                                ↑ error: value moved
}
```

**What happened:**

1. `if let Some(pri)` **moves** the `Priority` out of the `Option`
2. `priority_filter` becomes partially moved
3. Can't use `priority_filter` in the `match` later

**Solution: Derive `Copy`:**

```rust
#[derive(Debug, Clone, PartialEq, Copy)]  // ← Add Copy
enum Priority {
    High,
    Medium,
    Low,
}
```

**Why Copy solves it:**

```rust
// Without Copy:
if let Some(pri) = priority_filter {
    // pri = moved Priority
    // priority_filter = None
}

// With Copy:
if let Some(pri) = priority_filter {
    // pri = copied Priority
    // priority_filter = still Some(Priority) ✅
}
```

**Why Copy is safe here:**

- `Priority` is 3 variants = 1 byte
- No heap allocation
- Trivial to copy
- Idiomatically correct for small enums

**Issue 2: Clippy warning on redundant closure**

**The warning:**

```rust
.filter_map(|line| Task::from_line(line))
//          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//          warning: redundant closure
```

**The fix:**

```rust
// Before:
.filter_map(|line| Task::from_line(line))

// After:
.filter_map(Task::from_line)
```

**When can you do this?**

When closure signature matches function signature:

```rust
// Function:
fn from_line(line: &str) -> Option<Task>

// Closure in filter_map:
|line: &str| -> Option<Task>

// Perfect match! Use function directly ✅
```

#### Impact of Refactoring

**Code metrics:**

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| Total lines | ~180 | ~115 | **-36%** |
| `add` command | 30 lines | 20 lines | -33% |
| `list` command | 90 lines | 60 lines | -33% |
| `done` command | 25 lines | 15 lines | -40% |
| `search` command | 35 lines | 20 lines | -43% |

**36% less code with MORE features!**

**Maintainability improvements:**

```rust
// Before - To add a "tags" field:
// 1. Update add command (string formatting)
// 2. Update display_task (parsing)
// 3. Update extract_priority (parsing)
// 4. Update list filters (parsing)
// 5. Update search (parsing)
// 6. Update done/undone (preserve tags)
// 7. Update remove (preserve tags)
// = TOUCH 7+ PLACES

// After - To add a "tags" field:
struct Task {
    text: String,
    completed: bool,
    priority: Priority,
    tags: Vec<String>,  // ← Add here
}

// Update:
// 1. Task::new() - add parameter
// 2. Task::to_line() - serialize tags
// 3. Task::from_line() - parse tags
// = TOUCH 3 FUNCTIONS
// All commands automatically work! ✅
```

**Extensibility unlocked:**

Now trivial to add:

✅ Timestamps:

```rust
struct Task {
    // ...
    created_at: DateTime,
    completed_at: Option<DateTime>,
}
```

✅ Tags:

```rust
struct Task {
    // ...
    tags: Vec<String>,
}
```

✅ Subtasks:

```rust
struct Task {
    // ...
    subtasks: Vec<Task>,
}
```

✅ JSON serialization (next version):

```rust
#[derive(Serialize, Deserialize)]  // One line!
struct Task { /* ... */ }
```

**🔗 Resources:**

- [Code v1.2.0](https://github.com/joaofelipegalvao/todo-cli/tree/v1.2.0)
- [Full diff](https://github.com/joaofelipegalvao/todo-cli/compare/v1.1.0...v1.2.0)
- [Rust Book - Structs](https://doc.rust-lang.org/book/ch05-00-structs.html)
- [Rust Book - Enums](https://doc.rust-lang.org/book/ch06-00-enums.html)
- [Rust Book - Methods](https://doc.rust-lang.org/book/ch05-03-method-syntax.html)

---

### v1.3.0 - JSON Serialization with Serde

**🎯 Goal:** Replace custom text format with JSON using automatic serialization

**📦 The Problem We're Solving:**

**Before v1.3.0:**

Even with structs, we still had custom text parsing:

```rust
// Custom format:
"[ ] (high) Study Rust"
"[x] (low) Buy coffee"
"[ ] Read docs"

// Manual serialization in to_line():
fn to_line(&self) -> String {
    let checkbox = if self.completed { "[x]" } else { "[ ]" };
    let pri_str = match self.priority {
        Priority::High => " (high)",
        Priority::Low => " (low)",
        Priority::Medium => "",
    };
    format!("{}{} {}", checkbox, pri_str, self.text)
}

// Manual parsing in from_line():
fn from_line(line: &str) -> Option<Self> {
    // 25+ lines of parsing logic...
}
```

**Problems:**

❌ Custom format is fragile  
❌ Parsing logic is complex  
❌ Hard to add new fields (need to update parsing)  
❌ Not a standard format  
❌ Can't integrate with other tools  
❌ Manual maintenance of serialization  

**📦 The Solution: JSON with Serde**

**After v1.3.0:**

```json
[
  {
    "text": "Study Rust",
    "completed": false,
    "priority": "High"
  },
  {
    "text": "Buy coffee",
    "completed": true,
    "priority": "Low"
  },
  {
    "text": "Read docs",
    "completed": false,
    "priority": "Medium"
  }
]
```

**Why this is revolutionary:**

✅ **Standard format** - universal, tools everywhere  
✅ **Automatic serialization** - serde does everything  
✅ **2 lines of code** for complete I/O  
✅ **Trivial to extend** - add field = one line  
✅ **Export/Import ready** - JSON is portable  
✅ **Git-friendly** - readable diffs  

**🧠 Key Concepts:**

#### What is serialization?

**Serialization** = Converting data structures → storage format  
**Deserialization** = Converting storage format → data structures

```rust
// Serialization:
Task { text: "Study", completed: false, priority: High }
    ↓
"{\"text\":\"Study\",\"completed\":false,\"priority\":\"High\"}"

// Deserialization:
"{\"text\":\"Study\",\"completed\":false,\"priority\":\"High\"}"
    ↓
Task { text: "Study", completed: false, priority: High }
```

**Manual vs Automatic:**

```rust
// ❌ Manual (what we had):
fn to_line(&self) -> String {
    // 12 lines of string formatting
}

fn from_line(line: &str) -> Option<Self> {
    // 25 lines of parsing
}

// ✅ Automatic (with serde):
#[derive(Serialize, Deserialize)]
struct Task { /* ... */ }
// That's it! Serde generates everything
```

#### Adding serde dependencies

**In `Cargo.toml`:**

```toml
[dependencies]
colored = "2.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**What each does:**

| Crate | Purpose |
|-------|---------|
| `serde` | Core serialization framework |
| `features = ["derive"]` | Enables `#[derive(Serialize, Deserialize)]` macros |
| `serde_json` | JSON-specific implementation |

**Why two crates?**

- `serde` is **format-agnostic** (works with JSON, YAML, TOML, etc.)
- `serde_json` is the **JSON implementation**

**This separation means:**

```rust
// Same derives work for ANY format:
#[derive(Serialize, Deserialize)]
struct Task { /* ... */ }

// Just swap the implementation:
serde_json::to_string(task)  // JSON
toml::to_string(task)        // TOML
bincode::serialize(task)     // Binary
```

#### The `#[derive(Serialize, Deserialize)]` attribute

**Add to imports:**

```rust
use serde::{Deserialize, Serialize};
```

**Add to types:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]  // ← Add these
struct Task {
    text: String,
    completed: bool,
    priority: Priority,
}

#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize)]  // ← Add these
enum Priority {
    High,
    Medium,
    Low,
}
```

**What these derives do:**

**`Serialize`** - Generates code to convert type → format:

```rust
// Auto-generated by derive:
impl Serialize for Task {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Complex serialization logic here
        // We don't write this - macro does!
    }
}
```

**`Deserialize`** - Generates code to convert format → type:

```rust
// Auto-generated by derive:
impl Deserialize for Task {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer,
    {
        // Complex deserialization logic here
        // We don't write this - macro does!
    }
}
```

**The magic:**

```rust
// Before (manual - 37 lines):
impl Task {
    fn to_line(&self) -> String { /* 12 lines */ }
    fn from_line(line: &str) -> Option<Self> { /* 25 lines */ }
}

// After (automatic - 1 line):
#[derive(Serialize, Deserialize)]
// Generates 100+ lines of perfect serialization code!
```

#### Refactored `load_tasks()` with JSON

**Before (custom text format):**

```rust
fn load_tasks() -> Result<Vec<Task>, Box<dyn Error>> {
    match fs::read_to_string("todos.txt") {
        Ok(content) => {
            let tasks: Vec<Task> = content
                .lines()
                .filter(|l| !l.trim().is_empty())
                .filter_map(Task::from_line)  // Manual parsing
                .collect();
            Ok(tasks)
        }
        Err(_) => Ok(Vec::new()),
    }
}
```

**After (JSON):**

```rust
fn load_tasks() -> Result<Vec<Task>, Box<dyn Error>> {
    match fs::read_to_string("todos.json") {  // .txt → .json
        Ok(content) => {
            let tasks: Vec<Task> = serde_json::from_str(&content)?;  // ← ONE LINE!
            Ok(tasks)
        }
        Err(_) => Ok(Vec::new()),
    }
}
```

**Breakdown:**

```rust
serde_json::from_str(&content)?
//         ↑ Function that deserializes from JSON string
//                      ↑ JSON string to parse
//                                   ↑ Propagate errors
```

**Type inference:**

```rust
let tasks: Vec<Task> = serde_json::from_str(&content)?;
//         ↑ Type annotation tells serde what to deserialize INTO
```

**Error handling:**

```rust
serde_json::from_str(&content)?
// Returns Result<Vec<Task>, serde_json::Error>
// ? converts to Box<dyn Error> automatically
```

**What serde does automatically:**

1. Parse JSON string
2. Validate structure matches `Vec<Task>`
3. For each object:
   - Validate fields match `Task` struct
   - Parse `text` as String
   - Parse `completed` as bool
   - Parse `priority` as Priority enum
4. Return `Vec<Task>` or descriptive error

#### Refactored `save_tasks()` with JSON

**Before (custom text format):**

```rust
fn save_tasks(tasks: &[Task]) -> Result<(), Box<dyn Error>> {
    let lines: Vec<String> = tasks.iter().map(|t| t.to_line()).collect();
    fs::write("todos.txt", lines.join("\n") + "\n")?;
    Ok(())
}
```

**After (JSON):**

```rust
fn save_tasks(tasks: &[Task]) -> Result<(), Box<dyn Error>> {
    let json = serde_json::to_string_pretty(tasks)?;  // ← ONE LINE!
    fs::write("todos.json", json)?;
    Ok(())
}
```

**Breakdown:**

```rust
serde_json::to_string_pretty(tasks)?
//         ↑ Function that serializes to formatted JSON
//                           ↑ Data to serialize
//                                        ↑ Propagate errors
```

**Why `to_string_pretty` instead of `to_string`?**

```rust
// to_string - compact, one line:
serde_json::to_string(tasks)
// Output: [{"text":"Study","completed":false,"priority":"High"}]

// to_string_pretty - formatted, readable:
serde_json::to_string_pretty(tasks)
// Output:
// [
//   {
//     "text": "Study",
//     "completed": false,
//     "priority": "High"
//   }
// ]
```

**For a file we might edit or inspect, pretty is better!**

#### What we deleted

**Deleted methods - no longer needed:**

```rust
impl Task {
    // ❌ DELETED - serde does this now
    // fn to_line(&self) -> String { ... }
    
    // ❌ DELETED - serde does this now
    // fn from_line(line: &str) -> Option<Self> { ... }
    
    // ✅ KEPT - business logic
    fn new(text: String, priority: Priority) -> Self { ... }
    fn mark_done(&mut self) { ... }
    fn mark_undone(&mut self) { ... }
}
```

**Key insight:**

- **Serialization** (to_line/from_line) → automated by serde
- **Business logic** (new/mark_done/mark_undone) → stays in impl

**This is separation of concerns:**

- Serde handles: How to store data
- Our impl handles: What the data means

#### File format migration

**Updated `clear` command:**

```rust
"clear" => {
    if fs::metadata("todos.json").is_ok() {  // .txt → .json
        fs::remove_file("todos.json")?;      // .txt → .json
        println!("{}", "✓ All tasks have been removed".red().bold());
    } else {
        println!("No tasks to remove");
    }
}
```

**Migration for users:**

```bash
# Option 1: Fresh start
rm todos.txt
todo add "First task"

# Option 2: Manual migration
# (Could write a migration script if needed)
```

#### Testing the JSON format

**Create tasks:**

```bash
cargo run -- add "Study Rust" --high
cargo run -- add "Buy coffee" --low
cargo run -- add "Read docs"
```

**Inspect the JSON:**

```bash
cat todos.json
```

**Output:**

```json
[
  {
    "text": "Study Rust",
    "completed": false,
    "priority": "High"
  },
  {
    "text": "Buy coffee",
    "completed": false,
    "priority": "Low"
  },
  {
    "text": "Read docs",
    "completed": false,
    "priority": "Medium"
  }
]
```

**Mark task as done:**

```bash
$ cargo run -- done 1
✓ Task marked as completed

$ cat todos.json
```

**Updated JSON:**

```json
[
  {
    "text": "Study Rust",
    "completed": true,  // ← Changed!
    "priority": "High"
  },
  {
    "text": "Buy coffee",
    "completed": false,
    "priority": "Low"
  },
  {
    "text": "Read docs",
    "completed": false,
    "priority": "Medium"
  }
]
```

**Everything still works:**

```bash
$ cargo run -- list
📋 Tasks:

1. 🔴 ✅ Study Rust
2. 🟢 ⏳ Buy coffee
3. 🟡 ⏳ Read docs

$ cargo run -- search "rust"
📋 Results for "rust":

1. 🔴 ✅ Study Rust

$ cargo run -- list --high --pending
No high priority pending tasks  # (first one is done)
```

#### Impact of JSON migration

**Code reduction:**

| Component | Before | After | Reduction |
|-----------|--------|-------|-----------|
| `load_tasks()` | 12 lines | **3 lines** | -75% |
| `save_tasks()` | 4 lines | **2 lines** | -50% |
| `Task::from_line()` | 25 lines | **DELETED** | -100% |
| `Task::to_line()` | 12 lines | **DELETED** | -100% |
| **Total** | 53 lines | **5 lines** | **-91%** |

**91% reduction in I/O code!** 🎯

**Maintainability improvements:**

```rust
// Want to add a "created_at" field?

// Before (custom format):
// 1. Update to_line() to write field
// 2. Update from_line() to parse field
// 3. Update all tests
// 4. Handle backward compatibility
// = 30+ lines changed

// After (JSON):
#[derive(Serialize, Deserialize)]
struct Task {
    text: String,
    completed: bool,
    priority: Priority,
    created_at: String,  // ← Add ONE line
}
// That's it! Serde handles everything ✅
```

#### Understanding serde's magic

**What happens when we call:**

```rust
serde_json::from_str::<Vec<Task>>(&content)?
```

**Step by step:**

1. **Parse JSON structure** (validates syntax)
2. **Check it's an array** (matches `Vec<_>`)
3. **For each object in array:**
   - Check it has `text`, `completed`, `priority` fields
   - Validate `text` is string → convert to `String`
   - Validate `completed` is boolean → convert to `bool`
   - Validate `priority` is string → match against enum variants
   - Construct `Task` instance
4. **Collect all Task instances** into `Vec<Task>`
5. **Return result** or descriptive error

**All of this from one line of code!**

**Error examples:**

```json
// Invalid JSON syntax
{broken

// Error: EOF while parsing an object at line 1 column 7
```

```json
// Missing required field
[
  {
    "text": "Study",
    "completed": false
  }
]

// Error: missing field `priority` at line 4 column 3
```

```json
// Wrong type
[
  {
    "text": "Study",
    "completed": "yes",
    "priority": "High"
  }
]

// Error: invalid type: string "yes", expected a boolean at line 3
```

**Serde gives clear, helpful errors automatically!**

#### Why JSON is better than custom format

**Comparison:**

| Aspect | Custom Text | JSON |
|--------|-------------|------|
| Format | `[ ] (high) Task` | Standard JSON |
| Parsing | 25 lines manual | 1 line automatic |
| Formatting | 12 lines manual | 1 line automatic |
| Validation | Manual checks | Automatic |
| Errors | Generic | Descriptive |
| Extensibility | Modify parser | Add field |
| Tooling | None | Universal |
| Integration | Custom code | Native support |
| Debugging | `println!` | Any JSON viewer |
| Backup/Diff | Hard to read | Git-friendly |

**JSON wins in every category!**

**Real-world benefits:**

```bash
# View with any JSON tool
$ cat todos.json | jq '.[] | select(.priority == "High")'

# Format/validate
$ cat todos.json | jq '.'

# Export to other tools
$ curl -X POST api.example.com/todos -d @todos.json

# Version control
$ git diff todos.json
# Shows clear field changes, not gibberish
```

#### Extensibility demonstration

**Adding tags support:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task {
    text: String,
    completed: bool,
    priority: Priority,
    #[serde(default)]  // ← Use empty vec if field missing
    tags: Vec<String>,  // ← Add ONE line
}
```

**That's it!** Serde now serializes/deserializes tags automatically.

**Example JSON:**

```json
[
  {
    "text": "Study Rust",
    "completed": false,
    "priority": "High",
    "tags": ["learning", "programming"]
  }
]
```

**No changes needed in:**

- ✅ `load_tasks()` - still 3 lines
- ✅ `save_tasks()` - still 2 lines
- ✅ All commands - work automatically

**Only changes needed:**

- Add field to struct (1 line)
- Update `Task::new()` if desired
- Add commands to manage tags

**This is the power of declarative serialization!**

**🔗 Resources:**

- [Code v1.3.0](https://github.com/joaofelipegalvao/todo-cli/tree/v1.3.0)
- [Full diff](https://github.com/joaofelipegalvao/todo-cli/compare/v1.2.0...v1.3.0)
- [Serde documentation](https://serde.rs/)
- [Serde JSON docs](https://docs.serde.rs/serde_json/)

---

## Concepts Learned

### File Manipulation

- `OpenOptions` with `.create()` and `.append()` to add without overwriting
- `writeln!` macro for formatted writing
- `fs::read_to_string()` for complete file reading
- `fs::write()` to overwrite entire file
- `fs::remove_file()` to delete files
- `fs::metadata()` to check existence without opening

### Strings and Collections

- `enumerate()` to get indices + values in loops
- `parse()` for string → number conversion with validation
- `.map().collect()` to transform iterators
- `.replace()` for text substitution
- `.contains()` for string searching
- `.trim()` to remove whitespace
- `.to_string()` to solve lifetimes (`&str` → `String`)
- `.join()` to concatenate with separator
- `.filter()` to select elements
- `.copied()` to convert `&&str` → `&str` in iterators
- `Vec::remove()` to delete by index
- `.repeat()` for repeated strings
- Slices `&[&str]` to pass data slices without copying
- Tuples `(T, U)` to return multiple values from functions
- `.sort_by()` for custom sorting with comparator
- `Ordering` enum for type-safe comparisons (Less, Equal, Greater)
- `.cmp()` to compare orderable values

### Control Flow and Errors

- Pattern matching with `match` for subcommands
- Nested match for multi-level decisions
- Pattern matching with tuples `(a, b)` to combine conditions
- Error handling with `?` operator (automatic propagation)
- `Result<T, E>` for functions that can fail
- `Box<dyn Error>` for generic errors
- `if let` for simplified pattern matching
- Input validation and preconditions
- Conflicting flags validation (fail fast)
- Specific, educational error messages
- `Option<T>` for optional values (avoids "magic values")

### CLI and UX

- `env::args()` to capture arguments
- Subcommands with pattern matching
- Optional flags (`--pending`, `--done`, `--high`, `--low`)
- Parsing arguments with multiple flags
- Filter combination (status + priority)
- Argument validation (quantity, type, state)
- `println!` vs `eprintln!` (stdout vs stderr)
- `process::exit()` for exit codes
- Visual hierarchy with colors and formatting
- Immediate feedback with semantic colors
- Visual breathing room (whitespace matters)
- Smart defaults (medium as default)
- Dynamic titles based on context
- User error prevention

### Design and Colors

- `colored` crate for cross-platform colors
- `.dimmed()`, `.bold()`, `.strikethrough()` for formatting
- Semantic colors (green = success, red = attention)
- Color psychology (🔴 red = urgent, 🟡 yellow = normal, 🟢 green = low)
- Visual priority system with emojis
- Visual hierarchy (dimmed numbers, highlighted content)
- Multiple signals (color + icon + strikethrough) for accessibility
- `as f32` conversion for percentage calculations
- `as u32` to truncate decimals
- Positive reinforcement (always green for completed)
- Reduced visual pollution (priority only on pending)

### Functions and Organization

- Helper functions to avoid code duplication (DRY)
- Parameters with slices (`&[&str]`) for efficiency
- Code reuse with specialized functions
- Separation of responsibilities (parsing vs display)
- Parsing functions (`extract_priority`)
- Tuple returns for multiple values
- Transformation pipeline (filters in sequence)
- Visual logic modularization (`priority_emoji`)
- Mapping functions (`priority_order`) for concept → number conversion
- Appropriate type choices (`u8` vs `i32`) based on semantics
- Atomic functions (`display_task`) for unit rendering
- Orchestrated functions (`display_tasks`) for complete display with statistics
- Clear separation: data parsing vs visual rendering
- Mature CLI design without code duplication

### Structs and Type Safety (v1.2.0+)

- **Structs** - Custom data types grouping related data
- **Enums** - Type-safe enumeration with exhaustive matching
- **`impl` blocks** - Methods attached to types
- **Derive macros** - Auto-implementing traits (`Debug`, `Clone`, `PartialEq`, `Copy`)
- **`&self` vs `&mut self`** - Immutable vs mutable methods
- **`Self` type** - Referring to the implementing type in methods
- **`Option<T>`** - Representing optional/nullable values safely
- **`.filter_map()`** - Combining map and filter operations
- **Function pointers** - Using functions as values directly
- **`Copy` trait** - When types can be copied implicitly
- **Type-safe state** - Compiler-enforced correctness
- **Centralized parsing** - Single source of truth for data format
- **Method encapsulation** - Business logic on types

### Serialization with Serde (v1.3.0+)

- **Serialization** - Converting data structures to storage format
- **Deserialization** - Converting storage format to data structures
- **`serde` crate** - Format-agnostic serialization framework
- **`serde_json` crate** - JSON-specific implementation
- **`#[derive(Serialize, Deserialize)]`** - Automatic serialization code generation
- **`to_string_pretty()`** - Formatted JSON output
- **`from_str()`** - Parse JSON into typed structures
- **Type inference with serde** - Compiler determines target type from context
- **Automatic validation** - Serde validates structure and types
- **Error propagation** - `serde_json::Error` converts to `Box<dyn Error>`
- **Separation of concerns** - Serialization vs business logic
- **Declarative data format** - Derive macros define format automatically
- **Format agnostic** - Same derives work for JSON, TOML, YAML, etc.
- **`#[serde(default)]`** - Handle missing fields with defaults
- **Descriptive errors** - Serde provides clear error messages with line/column info

### Debug and Quality

- Finding bugs through manual testing
- Using `eprintln!` for debug prints
- File investigation with `cat` and `od`
- Precondition validation (avoid invalid states)
- Edge case thinking (empty file, invalid indices)
- Iterative refactoring without breaking functionality
- Consistency across commands (filter in all)
- Pipeline optimization (filter before sorting)
- Complexity analysis (Big-O) for performance decisions
- YAGNI principle (You Aren't Gonna Need It) - don't add unnecessary complexity
- Opt-in complexity - complex features are optional
- Incremental migration - refactor step by step
- Clippy lints - Follow community best practices

### Lifetimes and Ownership

- Lifetime problem with `.trim()` returning `&str`
- Solution with `.to_string()` to create owned `String`
- Difference between temporary reference and owned value
- Compiler detecting invalid reference usage
- `.copied()` to work with double references (`&&str`)
- Move vs Copy semantics
- Ownership transfer with `into_iter()`
- Borrowing with `&` and `&mut`

---

## Design Decisions

### Why 3 Priority Levels?

**Alternatives considered:**

- **2 levels** (high/normal)
  - ❌ Doesn't capture nuances
  - ❌ "What about important but not urgent?"

- **5+ levels** (critical/high/medium/low/trivial)
  - ❌ Decision paralysis
  - ❌ "Is this high or critical?"
  - ❌ User spends time categorizing instead of doing

**Chosen: 3 levels (high/medium/low)**

- ✅ Traffic light rule (universal convention)
- ✅ Cognitively simple
- ✅ Forces real prioritization

**Psychology behind it:**
> "If everything is a priority, nothing is a priority"

With 3 levels, you **must choose** what really matters.

---

### Why Sort AFTER Filtering?

**Wrong order:**

```rust
sort(50 tasks)     // O(50 log 50) ≈ 280 operations
filter to 5 tasks  // Wasted work!
```

**Correct order:**

```rust
filter 50 → 5      // O(50)
sort 5             // O(5 log 5) ≈ 12 operations  
// 23x faster!
```

**Performance matters**, even for small datasets. Good habits scale.

---

### Why `(high)` Format?

**Considered:**

```
[high][ ] Task       (prefix)
[ ]:high: Task       (inline)
high|[ ]|Task        (separator)
```

**Chosen: `[ ] (high) Task`**

- ✅ Human-readable (open `todos.txt` in any editor)
- ✅ Easy to parse (`.contains("(high)")`)
- ✅ Doesn't pollute visually (parentheses are subtle)
- ✅ Extensible (could add `(urgent)`, `(someday)`)

**Trade-off:**

- ❌ More verbose than `[h]`
- ✅ But clarity > brevity for user data

---

### Why Separate `display_task()` and `display_tasks()`?

**Could have done:**

```rust
// One function for everything
fn display_everything(tasks: &[&str], mode: &str) {
    if mode == "search" { /* ... */ }
    else if mode == "list" { /* ... */ }
}
```

**Why separation is better:**

- ✅ **Single Responsibility** - each function does one thing
- ✅ **Reusability** - atomic function works anywhere
- ✅ **Testability** - easier to test small functions
- ✅ **Maintainability** - changes don't cascade

**This is mature software design.**

---

### Why Add `--medium` Filter? (Design Evolution)

**Original flawed assumption:**
> "Medium is the default, so users don't need to filter by it"

**Reality discovered through usage:**
> "Medium is the default, so MOST tasks will be medium, making filtering essential"

**The fix:**

- ✅ Added `--medium` flag for completeness
- ✅ Created symmetry: if you can create with it, you can filter by it
- ✅ Solved real problem: filtering the majority of tasks

**Design principle learned:**
> "Defaults are for INPUT convenience, not QUERY limitation"

**Input (creation):**

- Default to medium → users don't always need to specify
- Makes quick task creation easy

**Query (filtering):**

- Provide ALL options → users need full control
- Never assume they don't need to filter by default value

**Impact:**

```bash
# With 100 tasks: 10 high, 75 medium, 15 low
# Before: Could only filter high or low
# After: Can filter medium (the majority!)
todo list --medium  # Shows 75 tasks ✅
```

---

### Why Refactor to Structs? (v1.2.0)

**String-based approach problems:**

- ❌ String parsing repeated everywhere
- ❌ No compile-time validation (typos like `"hihg"`)
- ❌ Hard to extend (adding fields requires touching many places)
- ❌ Prone to bugs (format changes break everything)

**Struct-based approach benefits:**

- ✅ Type-safe - compiler catches mistakes
- ✅ Self-documenting - clear what data contains
- ✅ Extensible - add fields in one place
- ✅ DRY - parsing logic centralized
- ✅ Testable - can unit test methods

**The transformation:**

```rust
// Before: Parsing everywhere
if line.contains("(high)") { /* ... */ }

// After: Type-safe field access
if task.priority == Priority::High { /* ... */ }
```

**Lesson:** Start simple (strings), refactor to robust (structs) once you understand the domain.

---

### When to Use Enums vs Strings?

**Use Enums when:**

- ✅ Fixed set of values (priorities: High/Medium/Low)
- ✅ Need compile-time validation
- ✅ Want exhaustive matching
- ✅ Values are fundamental to domain

**Use Strings when:**

- ✅ User-provided data (task text)
- ✅ Open-ended values
- ✅ Display purposes only
- ✅ Simpler prototype phase

**Our evolution:**

- **v0.1-v1.1:** Strings for everything (learning)
- **v1.2+:** Enums for priority, strings for task text (mature)

---

### Why JSON over Custom Text Format? (v1.3.0)

**What we had (custom text):**

```
[ ] (high) Study Rust
[x] (low) Buy coffee
```

**What we moved to (JSON):**

```json
{
  "text": "Study Rust",
  "completed": false,
  "priority": "High"
}
```

**Why make the switch?**

**Custom text pros:**

- ✅ Simple to start with
- ✅ Human-readable for manual editing
- ✅ No dependencies needed

**Custom text cons:**

- ❌ Manual parsing (25+ lines)
- ❌ Manual serialization (12+ lines)
- ❌ Fragile (format changes break everything)
- ❌ Hard to extend (add field = rewrite parser)
- ❌ No tooling support
- ❌ Poor error messages

**JSON pros:**

- ✅ **Automatic** - serde does everything (91% code reduction)
- ✅ **Standard** - universal format
- ✅ **Extensible** - add field = one line
- ✅ **Tooling** - jq, formatters, validators
- ✅ **Integration** - APIs, exports, imports
- ✅ **Errors** - descriptive with line numbers
- ✅ **Validation** - automatic type checking
- ✅ **Git-friendly** - clear diffs

**JSON cons:**

- ❌ Not human-editable (but we don't need to)
- ❌ Slightly larger files (irrelevant for our use case)
- ❌ Adds dependencies (but they're tiny and standard)

**The decision:**

For a tool where:

- Users interact via CLI (not file editing)
- We want to add features easily (tags, dates, etc.)
- We want standard integration capabilities

**JSON is the clear winner!**

**When custom format IS better:**

- Config files users edit manually (use TOML instead)
- Ultra-simple formats (single line per item, no structure)
- No-dependency requirement (embedded systems)

**But for structured data with commands, JSON is superior.**

**Code impact:**

```rust
// Custom format:
fn load_tasks() -> Result<Vec<Task>, Box<dyn Error>> {
    // 12 lines of parsing
}
fn save_tasks(tasks: &[Task]) -> Result<(), Box<dyn Error>> {
    // 4 lines of formatting
}
// Plus: 37 lines in Task::from_line() and Task::to_line()
// Total: ~53 lines

// JSON:
fn load_tasks() -> Result<Vec<Task>, Box<dyn Error>> {
    match fs::read_to_string("todos.json") {
        Ok(content) => Ok(serde_json::from_str(&content)?),
        Err(_) => Ok(Vec::new()),
    }
}
fn save_tasks(tasks: &[Task]) -> Result<(), Box<dyn Error>> {
    fs::write("todos.json", serde_json::to_string_pretty(tasks)?)?;
    Ok(())
}
// Total: 5 lines
```

**91% reduction while gaining:**

- Type safety
- Validation
- Better errors
- Extensibility
- Tooling support

**No-brainer decision!** 🎯

---

## Conclusion

This project taught me:

1. **Rust fundamentals** through practical application
2. **CLI design patterns** that feel natural to users
3. **Incremental development** - each version builds on previous
4. **Error handling** that's helpful, not frustrating
5. **Visual design** that reduces cognitive load
6. **Code organization** that scales without duplication
7. **Type safety** - using the compiler to prevent bugs
8. **Refactoring strategy** - evolve code without breaking it
9. **Serialization** - leveraging ecosystem tools (serde)
10. **Declarative programming** - derive macros doing the work

**Most importantly:** Learning is a process. Each version had bugs, inefficiencies, and room for improvement. That's expected and valuable.

**Version milestones:**

- **v0.1-v0.5:** Basic functionality (make it work)
- **v0.6-v0.9:** Polish and features (make it nice)
- **v1.0-v1.1:** Professional quality (make it complete)
- **v1.2:** Type safety and architecture (make it right)
- **v1.3:** Automatic serialization (make it maintainable)

**Evolution of the codebase:**

```
v0.1: String matching everywhere
  ↓
v1.2: Type-safe structs and enums (36% reduction)
  ↓
v1.3: Automatic JSON serialization (91% I/O reduction)
  ↓
Next: Even more powerful features with minimal code
```

**The journey is the lesson.** 🦀

---

## Next Steps

**Potential future versions:**

- **v1.4:** Tags system with filtering
- **v1.5:** Due dates and timestamps (using `chrono`)
- **v1.6:** Recurring tasks
- **v1.7:** Subtasks/nested tasks
- **v1.8:** Multiple projects/contexts
- **v1.9:** Export/import commands
- **v2.0:** TUI (Terminal User Interface) with `ratatui`
- **v2.1:** Sync with cloud storage
- **v2.2:** Web API for mobile apps

**Each version would teach:**

| Version | Feature | Rust Concepts |
|---------|---------|---------------|
| v1.4 | Tags | `HashSet`, `Vec<String>`, filtering |
| v1.5 | Due dates | `chrono` crate, `DateTime<Utc>` |
| v1.6 | Recurring | Pattern matching, date arithmetic |
| v1.7 | Subtasks | Recursive data structures, `Box<T>` |
| v1.8 | Projects | Multiple files, better organization |
| v1.9 | Export | CLI arguments, file I/O variations |
| v2.0 | TUI | Event loops, rendering, `ratatui` |
| v2.1 | Sync | HTTP clients, `tokio`, async/await |
| v2.2 | Web API | `axum`, REST APIs, JSON responses |

**The beauty of JSON:**

All these features are **trivial** to add now:

```rust
#[derive(Serialize, Deserialize)]
struct Task {
    text: String,
    completed: bool,
    priority: Priority,
    tags: Vec<String>,        // ← v1.4: Add ONE line
    due_date: Option<String>, // ← v1.5: Add ONE line
    recurring: Option<Recurrence>, // ← v1.6: Add ONE line
    subtasks: Vec<Task>,      // ← v1.7: Add ONE line
    project: String,          // ← v1.8: Add ONE line
}
```

**No parser changes needed!** Serde handles everything automatically.

**This is why we refactored to structs + JSON:**

Not just for now, but to **enable future growth** with minimal friction.

**Stay curious, keep building!** 🚀

---

**Built with ❤️ as a learning journey - Each version teaches something new**
