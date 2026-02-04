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

### v1.4.0 - Tags System

**🎯 Goal:** Add categorization system with tags and fix critical numbering bug

**📦 What We're Adding:**

**Tags for task organization:**

```bash
# Before - no categorization:
todo add "Study Rust" --high
todo add "Fix bug" --high
# How to separate work from personal? No way!

# After - with tags:
todo add "Study Rust" --high --tag learning --tag programming
todo add "Fix bug" --high --tag work --tag urgent
todo list --tag work  # Show only work tasks
```

**Why tags matter:**

✅ **Categorization** - Group tasks by project, context, etc.  
✅ **Flexible filtering** - Multiple tags per task  
✅ **Easy to add** - Thanks to JSON/serde  
✅ **Visual feedback** - Tags shown in list  
✅ **Discovery** - `tags` command shows all tags  

**🧠 Key Concepts:**

#### Adding a field to existing struct

**This is where JSON/serde shines!**

```rust
// Before (v1.3.0):
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task {
    text: String,
    completed: bool,
    priority: Priority,
}

// After (v1.4.0):
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task {
    text: String,
    completed: bool,
    priority: Priority,
    tags: Vec<String>,  // ← Add ONE line
}
```

**That's it!** Serde handles everything automatically.

**What happens:**

1. **Serialization** - Tasks now include tags in JSON:

```json
{
  "text": "Study Rust",
  "completed": false,
  "priority": "High",
  "tags": ["learning", "programming"]
}
```

1. **Deserialization** - Serde parses tags automatically
2. **Backward compatibility** - Old tasks without tags? They fail to load!

**Handling backward compatibility:**

```rust
// Problem: Old JSON doesn't have "tags" field
{
  "text": "Old task",
  "completed": false,
  "priority": "High"
  // Missing "tags"!
}

// Serde error: "missing field `tags`"
```

**Solution: Use `#[serde(default)]`:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task {
    text: String,
    completed: bool,
    priority: Priority,
    #[serde(default)]  // ← Use Vec::default() if missing
    tags: Vec<String>,
}
```

**Now:**

- New tasks: `"tags": ["work"]` ✅
- Old tasks: Missing field → `tags: vec![]` ✅

**This is graceful degradation!**

#### `Vec<String>` for multiple values

**Why `Vec<String>` instead of single `String`?**

```rust
// ❌ Single tag - limited
tags: String  // Can only have one tag

// ✅ Multiple tags - flexible
tags: Vec<String>  // Can have many tags
```

**Usage:**

```rust
let task = Task {
    text: "Study Rust".to_string(),
    completed: false,
    priority: Priority::High,
    tags: vec!["learning".to_string(), "programming".to_string()],
};
```

**In JSON:**

```json
{
  "tags": ["learning", "programming"]
}
```

**Empty tags:**

```json
{
  "tags": []
}
```

**Serde handles all of this automatically!**

#### Updating `Task::new()` signature

**Before:**

```rust
fn new(text: String, priority: Priority) -> Self {
    Task {
        text,
        completed: false,
        priority,
    }
}
```

**After:**

```rust
fn new(text: String, priority: Priority, tags: Vec<String>) -> Self {
    Task {
        text,
        completed: false,
        priority,
        tags,
    }
}
```

**Impact:**

```rust
// All calls to Task::new() must be updated:
let task = Task::new(text, priority);  // ❌ Compile error

let task = Task::new(text, priority, tags);  // ✅ Works
```

**This is why type safety is great:**

- Compiler catches every place that needs updating
- Can't forget to handle tags
- No runtime surprises

#### Parsing multiple flags of same type

**Challenge:** Allow `--tag` multiple times:

```bash
todo add "Study Rust" --tag learning --tag programming --tag rust
```

**Solution: Loop and collect:**

```rust
let mut tags: Vec<String> = Vec::new();
let mut i = 3;  // Start after command and task text

while i < args.len() {
    match args[i].as_str() {
        "--high" => priority = Priority::High,
        "--medium" => priority = Priority::Medium,
        "--low" => priority = Priority::Low,
        "--tag" => {
            if i + 1 >= args.len() {
                return Err("--tag requires a value".into());
            }
            tags.push(args[i + 1].clone());  // Add tag to vector
            i += 1;  // Skip the tag value
        }
        _ => return Err(format!("Invalid flag: {}", args[i]).into()),
    }
    i += 1;
}
```

**How it works:**

```bash
todo add "Task" --tag work --tag urgent
#                    ↓         ↓
#                  i=3       i=5
```

**Step by step:**

```rust
// i = 3: args[3] = "--tag"
"--tag" => {
    // i + 1 = 4: args[4] = "work"
    tags.push(args[4].clone());  // tags = ["work"]
    i += 1;  // i = 4
}
i += 1;  // i = 5

// i = 5: args[5] = "--tag"
"--tag" => {
    // i + 1 = 6: args[6] = "urgent"
    tags.push(args[6].clone());  // tags = ["work", "urgent"]
    i += 1;  // i = 6
}
i += 1;  // i = 7 (end of args)
```

**Result:** `tags = ["work", "urgent"]`

**Why manual loop instead of iterator?**

```rust
// ❌ Hard with iterator (need lookahead for values)
args.iter().for_each(|arg| { /* how to get next arg? */ })

// ✅ Easy with manual loop (can increment i)
while i < args.len() {
    match args[i] {
        "--tag" => {
            tags.push(args[i + 1]);  // Easy access to next
            i += 1;  // Skip it
        }
    }
    i += 1;
}
```

**This is a common pattern for flag parsing!**

#### Filtering with `retain()`

**Before (v1.3.0):** Creating new vectors

```rust
let mut valid_lines: Vec<&str> = all_lines
    .iter()
    .filter(|line| !line.is_empty())
    .copied()
    .collect();  // Creates NEW vector
```

**After (v1.4.0):** Modifying in-place

```rust
let mut indexed_tasks: Vec<(usize, &Task)> = /* ... */;

// Filter in-place
indexed_tasks.retain(|(_, t)| !t.completed);  // Removes items
```

**What's `.retain()`?**

```rust
vec.retain(|item| predicate);
// Keeps only items where predicate returns true
// Modifies vector IN PLACE
```

**Example:**

```rust
let mut numbers = vec![1, 2, 3, 4, 5];
numbers.retain(|n| n % 2 == 0);  // Keep only even numbers
// Result: vec![2, 4]
```

**Comparison:**

```rust
// filter() - creates NEW vector
let evens: Vec<_> = numbers.iter().filter(|n| n % 2 == 0).copied().collect();

// retain() - modifies EXISTING vector
numbers.retain(|n| n % 2 == 0);
```

**Why use `retain()` here?**

```rust
// We need to apply MULTIPLE filters:
indexed_tasks.retain(|(_, t)| !t.completed);  // Filter 1
indexed_tasks.retain(|(_, t)| t.priority == pri);  // Filter 2
indexed_tasks.retain(|(_, t)| t.tags.contains(tag));  // Filter 3

// Each filter modifies the SAME vector
// More efficient than creating new vectors each time
```

**Pattern in our code:**

```rust
let mut indexed_tasks: Vec<(usize, &Task)> = all_tasks
    .iter()
    .enumerate()
    .map(|(i, task)| (i + 1, task))
    .collect();

// Apply filters sequentially
match status_filter {
    "pending" => indexed_tasks.retain(|(_, t)| !t.completed),
    "done" => indexed_tasks.retain(|(_, t)| t.completed),
    _ => {}
}

if let Some(pri) = priority_filter {
    indexed_tasks.retain(|(_, t)| t.priority == pri);
}

if let Some(tag) = &tag_filter {
    indexed_tasks.retain(|(_, t)| t.tags.contains(tag));
}
```

**Benefits:**

- ✅ Clear and readable
- ✅ Memory efficient (no intermediate vectors)
- ✅ Easy to add more filters

#### The critical numbering bug

**The Problem:**

**Before fix:**

```rust
// Load all tasks
let tasks = load_tasks()?;  // [Task1, Task2, Task3, Task4, Task5]

// Filter to pending only
let pending: Vec<&Task> = tasks
    .iter()
    .filter(|t| !t.completed)
    .collect();
// Result: [Task1, Task3, Task5]

// Display with NEW numbering
for (i, task) in pending.iter().enumerate() {
    println!("{}. {}", i + 1, task.text);  // ❌ 1, 2, 3
}
// Output:
// 1. Task1  (was index 0)
// 2. Task3  (was index 2)
// 3. Task5  (was index 4)
```

**User sees:**

```bash
$ todo list --pending
1. Study Rust
2. Fix bug
3. Write docs

$ todo done 2  # User wants to mark "Fix bug" as done
```

**What actually happens:**

```rust
// "2" maps to index 1 in the FULL array
tasks[1].mark_done();  // Marks Task2, not Task3!
```

**Bug:** User marked the WRONG task! 🐛

**The Solution: Preserve original indices**

```rust
// Create tuples with ORIGINAL indices
let mut indexed_tasks: Vec<(usize, &Task)> = tasks
    .iter()
    .enumerate()
    .map(|(i, task)| (i + 1, task))  // Store (number, task)
    .collect();
// Result: [(1, Task1), (2, Task2), (3, Task3), (4, Task4), (5, Task5)]

// Filter while KEEPING original numbers
indexed_tasks.retain(|(_, t)| !t.completed);
// Result: [(1, Task1), (3, Task3), (5, Task5)]

// Display with ORIGINAL numbers
for (number, task) in &indexed_tasks {
    println!("{}. {}", number, task.text);  // ✅ 1, 3, 5
}
```

**User sees:**

```bash
$ todo list --pending
1. Study Rust
3. Fix bug      ← Original number preserved!
5. Write docs

$ todo done 3   # Correctly marks Task3
```

**Now it works!** ✅

**Implementation details:**

```rust
// Updated display_lists signature:
fn display_lists(tasks: &[(usize, &Task)], title: &str) {
    //                    ↑ Tuple with (number, task)
    
    for (number, task) in tasks {
        display_task(*number, task);  // Use original number
    }
}

// Updated display_task signature:
fn display_task(number: usize, task: &Task) {
    //              ↑ Accept the original number
    let number_fmt = format!("{}.", number);
    // Display with original numbering
}
```

**Why this is critical:**

- ✅ Commands work on correct tasks
- ✅ No user confusion
- ✅ Filtering becomes safe
- ✅ Professional UX

**This bug would have made the app unusable with filters!**

#### New command: `tags`

**Purpose:** Show all tags in use with counts

```rust
"tags" => {
    let tasks = load_tasks()?;

    if tasks.is_empty() {
        println!("No tasks");
        return Ok(());
    }

    // Collect all unique tags
    let mut all_tags: Vec<String> = Vec::new();
    for task in &tasks {
        for tag in &task.tags {
            if !all_tags.contains(tag) {
                all_tags.push(tag.clone());
            }
        }
    }

    if all_tags.is_empty() {
        println!("No tags found");
        return Ok(());
    }

    all_tags.sort();  // Alphabetical order

    println!("\n Tags:\n");
    for tag in &all_tags {
        let count = tasks.iter().filter(|t| t.tags.contains(tag)).count();
        println!(
            "  {} ({} task{})",
            tag.cyan(),
            count,
            if count == 1 { "" } else { "s" }  // Grammar!
        );
    }

    println!()
}
```

**How it works:**

1. **Collect unique tags:**

```rust
let mut all_tags: Vec<String> = Vec::new();
for task in &tasks {
    for tag in &task.tags {
        if !all_tags.contains(tag) {  // Deduplicate
            all_tags.push(tag.clone());
        }
    }
}
```

**Example:**

```rust
// Tasks:
// Task1: tags = ["work", "urgent"]
// Task2: tags = ["work", "learning"]
// Task3: tags = ["urgent"]

// Result: all_tags = ["work", "urgent", "learning"]
```

1. **Sort alphabetically:**

```rust
all_tags.sort();
// Result: ["learning", "urgent", "work"]
```

1. **Count occurrences:**

```rust
for tag in &all_tags {
    let count = tasks.iter().filter(|t| t.tags.contains(tag)).count();
    // For "work": count = 2 (Task1, Task2)
}
```

1. **Display with grammar:**

```rust
if count == 1 { "" } else { "s" }
// "1 task" vs "2 tasks"
```

**Output:**

```bash
$ todo tags

 Tags:

  learning (1 task)
  urgent (2 tasks)
  work (2 tasks)
```

**Why this is useful:**

- ✅ See all available tags
- ✅ Find popular tags
- ✅ Discover categorization patterns
- ✅ Clean up unused tags

#### Visual feedback for tags

**Updated `display_task()`:**

```rust
fn display_task(number: usize, task: &Task) {
    let number_fmt = format!("{}.", number);
    let emoji = task.priority.emoji();

    // Format tags
    let tags_str = if task.tags.is_empty() {
        String::new()  // No tags = empty string
    } else {
        format!(" [{}]", task.tags.join(", "))  // [tag1, tag2]
    };

    if task.completed {
        println!(
            "{} {} {} {}{}",
            number_fmt.dimmed(),
            emoji,
            "✅".green(),
            task.text.green().strikethrough(),
            tags_str.dimmed()  // ← Dimmed for completed
        );
    } else {
        println!(
            "{} {} {} {}{}",
            number_fmt.dimmed(),
            emoji,
            "⏳".yellow(),
            task.text.bright_white(),
            tags_str.cyan()  // ← Cyan for pending
        );
    }
}
```

**Output examples:**

```bash
# Completed task with tags
1. 🔴 ✅ Study Rust [learning, programming]
                   ↑ dimmed

# Pending task with tags
2. 🟡 ⏳ Fix bug [work, urgent]
                ↑ cyan (stands out)

# Task without tags
3. 🟢 ⏳ Buy coffee
```

**Design rationale:**

- **Completed tasks:** Tags dimmed (less important)
- **Pending tasks:** Tags cyan (visible, but not distracting)
- **No tags:** Nothing shown (clean)

**Why `join(", ")`?**

```rust
let tags = vec!["work", "urgent", "learning"];
tags.join(", ")
// Output: "work, urgent, learning"
```

**Alternative approaches:**

```rust
// ❌ Manual concatenation
let mut result = String::new();
for (i, tag) in tags.iter().enumerate() {
    result.push_str(tag);
    if i < tags.len() - 1 {
        result.push_str(", ");
    }
}

// ✅ Idiomatic
tags.join(", ")
```

**Much cleaner!**

#### Testing the complete feature

```bash
# Add tasks with tags
$ cargo run -- add "Study Rust" --high --tag learning --tag programming
✓ Task added

$ cargo run -- add "Fix bug #123" --tag work --tag urgent
✓ Task added

$ cargo run -- add "Buy coffee" --low
✓ Task added

$ cargo run -- add "Write docs" --tag work --tag documentation
✓ Task added

# List all tasks
$ cargo run -- list

📋 Tasks:

1. 🔴 ⏳ Study Rust [learning, programming]
2. 🟡 ⏳ Fix bug #123 [work, urgent]
3. 🟢 ⏳ Buy coffee
4. 🟡 ⏳ Write docs [work, documentation]

# Filter by tag
$ cargo run -- list --tag work

📋 Tasks:

2. 🟡 ⏳ Fix bug #123 [work, urgent]
4. 🟡 ⏳ Write docs [work, documentation]

# Mark task as done (with correct numbering!)
$ cargo run -- done 2
✓ Task marked as completed

# Search with tag filter
$ cargo run -- search "bug" --tag work

📋 Results for "bug":

2. 🟡 ✅ Fix bug #123 [work, urgent]
                      ↑ dimmed now

# View all tags
$ cargo run -- tags

 Tags:

  documentation (1 task)
  learning (1 task)
  programming (1 task)
  urgent (1 task)
  work (2 tasks)

# Inspect JSON
$ cat todos.json
[
  {
    "text": "Study Rust",
    "completed": false,
    "priority": "High",
    "tags": [
      "learning",
      "programming"
    ]
  },
  {
    "text": "Fix bug #123",
    "completed": true,
    "priority": "Medium",
    "tags": [
      "work",
      "urgent"
    ]
  },
  {
    "text": "Buy coffee",
    "completed": false,
    "priority": "Low",
    "tags": []
  },
  {
    "text": "Write docs",
    "completed": false,
    "priority": "Medium",
    "tags": [
      "work",
      "documentation"
    ]
  }
]
```

**Everything works perfectly!** ✨

**🔗 Resources:**

- [Code v1.4.0](https://github.com/joaofelipegalvao/todo-cli/tree/v1.4.0)
- [Full diff](https://github.com/joaofelipegalvao/todo-cli/compare/v1.3.0...v1.4.0)

---

### v1.5.0 - Due Dates and Tabular Display

**🎯 Goal:** Add due date tracking with `chrono` and improve visual display with tabular format

**📦 What We're Adding:**

**Due dates for deadline management:**

```bash
# Before - no deadline tracking:
todo add "Submit report" --high
# When is it due? No way to know!

# After - with due dates:
todo add "Submit report" --high --due 2026-02-15
todo list --overdue        # See what's late
todo list --due-soon       # See what's coming up
todo list --sort due       # Sort by deadline
```

**Why due dates matter:**

✅ **Deadline awareness** - Never miss important dates  
✅ **Smart filtering** - `--overdue`, `--due-soon`  
✅ **Flexible sorting** - By priority, due date, or creation  
✅ **Visual urgency** - Color-coded warnings  
✅ **Automatic timestamps** - `created_at` tracks when task was added  

**🧠 Key Concepts:**

#### Adding the `chrono` crate

**In `Cargo.toml`:**

```toml
[dependencies]
colored = "2.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }  # ← ADD
```

**Why chrono?**

- Rust's standard library has basic `std::time` but no date parsing
- `chrono` is the de-facto standard for dates/times in Rust
- Feature `serde` enables automatic serialization

**Import:**

```rust
use chrono::{Local, NaiveDate};
```

**What's `NaiveDate`?**

- "Naive" = no timezone information
- Just year-month-day (2026-02-15)
- Perfect for due dates (we care about the day, not the hour)

**Alternative: `DateTime`**

```rust
// If we wanted time + timezone:
use chrono::{DateTime, Utc};
let timestamp: DateTime<Utc> = Utc::now();

// But for due dates, we just need the day:
use chrono::NaiveDate;
let due_date = NaiveDate::from_ymd(2026, 2, 15);
```

#### Adding date fields to `Task`

**Updated struct:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task {
    text: String,
    completed: bool,
    priority: Priority,
    tags: Vec<String>,
    due_date: Option<NaiveDate>,  // ← NEW: Optional due date
    created_at: NaiveDate,         // ← NEW: Creation timestamp
}
```

**Why `Option<NaiveDate>`?**

```rust
// Not all tasks have deadlines:
due_date: Option<NaiveDate>

// Some(...) = has due date
// None = no due date
```

**Why `NaiveDate` for `created_at` (not `Option`)?**

```rust
created_at: NaiveDate  // Always has a value
```

Every task has a creation time, so no `Option` needed.

**Updated constructor:**

```rust
fn new(
    text: String,
    priority: Priority,
    tags: Vec<String>,
    due_date: Option<NaiveDate>,  // ← NEW parameter
) -> Self {
    Task {
        text,
        completed: false,
        priority,
        tags,
        due_date,
        created_at: Local::now().naive_local().date(),  // ← AUTO
    }
}
```

**Breaking down `created_at`:**

```rust
Local::now()           // Get current local time (with timezone)
    .naive_local()     // Convert to naive (remove timezone)
    .date()            // Extract just the date (no time)
```

**Result:** `NaiveDate` like `2026-02-03`

#### Date methods on `Task`

**Check if overdue:**

```rust
fn is_overdue(&self) -> bool {
    if let Some(due) = self.due_date {
        let today = Local::now().naive_local().date();
        due < today && !self.completed
    } else {
        false  // No due date = can't be overdue
    }
}
```

**Breakdown:**

```rust
if let Some(due) = self.due_date  // Only if task has due date
```

Pattern matching on `Option`:

- If `Some(date)` → extract date, continue
- If `None` → skip to `else { false }`

```rust
let today = Local::now().naive_local().date();
due < today && !self.completed
```

Task is overdue if:

1. Due date is before today (`due < today`)
2. AND task is not completed (`!self.completed`)

**Example:**

```rust
// Today: 2026-02-03
// Due: 2026-02-01
// Completed: false
// Result: true (overdue!)

// Today: 2026-02-03
// Due: 2026-02-01
// Completed: true
// Result: false (was overdue, but done now)
```

**Check if due soon:**

```rust
fn is_due_soon(&self, days: i64) -> bool {
    if let Some(due) = self.due_date {
        let today = Local::now().naive_local().date();
        let days_until = (due - today).num_days();
        days_until >= 0 && days_until <= days && !self.completed
    } else {
        false
    }
}
```

**New concept: Date arithmetic**

```rust
let days_until = (due - today).num_days();
```

**What's happening:**

```rust
// Today: 2026-02-03
// Due: 2026-02-10
let days_until = (2026-02-10) - (2026-02-03);
// Result: Duration of 7 days
.num_days()  // Extract as i64: 7
```

**The condition:**

```rust
days_until >= 0 && days_until <= days && !self.completed
```

Task is "due soon" if:

1. Due date is not in the past (`days_until >= 0`)
2. AND due within N days (`days_until <= days`)
3. AND not completed

**Example:**

```rust
// Today: 2026-02-03
// Due: 2026-02-08
// days_until = 5
// is_due_soon(7) → true (within 7 days)
// is_due_soon(3) → false (more than 3 days away)
```

#### Parsing dates from command line

**In `add` command:**

```rust
"--due" => {
    if i + 1 >= args.len() {
        return Err("--due requires a date in format YYYY-MM-DD".into());
    }

    let date_str = &args[i + 1];
    match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        Ok(date) => due_date = Some(date),
        Err(_) => {
            return Err(format!(
                "Invalid date format: '{}'. Use YYYY-MM-DD",
                date_str
            )
            .into());
        }
    }

    i += 1;
}
```

**What's `parse_from_str`?**

```rust
NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
```

**Parameters:**

- `date_str`: String to parse ("2026-02-15")
- `"%Y-%m-%d"`: Format pattern
  - `%Y` = 4-digit year (2026)
  - `%m` = 2-digit month (02)
  - `%d` = 2-digit day (15)

**Returns:** `Result<NaiveDate, ParseError>`

**Error handling:**

```rust
match NaiveDate::parse_from_str(...) {
    Ok(date) => due_date = Some(date),  // Success
    Err(_) => return Err("Invalid date format...".into()),  // Failure
}
```

**Examples:**

```bash
# Valid:
todo add "Task" --due 2026-02-15  ✅

# Invalid:
todo add "Task" --due 02/15/2026  ❌ Wrong format
todo add "Task" --due 2026-13-01  ❌ Invalid month
todo add "Task" --due tomorrow    ❌ Not a date
```

#### Enhanced sorting

**New sort options:**

```rust
match sort_by {
    "priority" => {
        indexed_tasks.sort_by(|(_, a), (_, b)| 
            a.priority.order().cmp(&b.priority.order())
        );
    }
    "due" => {
        indexed_tasks.sort_by(|(_, a), (_, b)| 
            match (a.due_date, b.due_date) {
                (Some(date_a), Some(date_b)) => date_a.cmp(&date_b),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        );
    }
    "created" => {
        indexed_tasks.sort_by(|(_, a), (_, b)| 
            a.created_at.cmp(&b.created_at)
        );
    }
    _ => {}
}
```

**Understanding the `due` sort:**

```rust
match (a.due_date, b.due_date) {
    (Some(date_a), Some(date_b)) => date_a.cmp(&date_b),
    (Some(_), None) => std::cmp::Ordering::Less,
    (None, Some(_)) => std::cmp::Ordering::Greater,
    (None, None) => std::cmp::Ordering::Equal,
}
```

**Pattern matching on two `Option`s:**

| Case | a.due_date | b.due_date | Result | Why |
|------|------------|------------|--------|-----|
| 1 | `Some(date_a)` | `Some(date_b)` | Compare dates | Both have dates |
| 2 | `Some(_)` | `None` | a comes first (`Less`) | Tasks with dates come first |
| 3 | `None` | `Some(_)` | b comes first (`Greater`) | Tasks with dates come first |
| 4 | `None` | `None` | Equal | Neither has date |

**Why tasks with dates come first?**

```rust
(Some(_), None) => std::cmp::Ordering::Less
```

**Logic:** Tasks with deadlines are more important than tasks without deadlines.

**Result:**

```
Sorted list:
1. Task due 2026-02-05
2. Task due 2026-02-10
3. Task due 2026-02-20
4. Task with no due date
5. Task with no due date
```

**Created sort is simpler:**

```rust
"created" => {
    indexed_tasks.sort_by(|(_, a), (_, b)| 
        a.created_at.cmp(&b.created_at)
    );
}
```

No `Option` to handle since `created_at` always has a value.

#### Date filters

**Four new filters:**

```rust
if overdue {
    indexed_tasks.retain(|(_, t)| t.is_overdue());
}
if due_soon {
    indexed_tasks.retain(|(_, t)| t.is_due_soon(7));
}
if with_due {
    indexed_tasks.retain(|(_, t)| t.due_date.is_some());
}
if without_due {
    indexed_tasks.retain(|(_, t)| t.due_date.is_none());
}
```

**Mutual exclusion:**

```rust
if overdue || due_soon || with_due || without_due {
    return Err("Use only one date filter".into());
}
```

Only one date filter allowed at a time.

**Examples:**

```bash
todo list --overdue        # Only late tasks
todo list --due-soon       # Due in next 7 days
todo list --with-due       # Any task with a due date
todo list --without-due    # Tasks with no deadline
```

#### Tabular display format

**Major visual change:**

**Before (v1.4.0):**

```
1. 🔴 ⏳ Study Rust [learning, programming]
2. 🟡 ✅ Fix bug [work, urgent]
```

**After (v1.5.0):**

```
  ID  P  S  Task                Tags            Due
─────────────────────────────────────────────────────
   1  H  ⏳  Study Rust          learning, ...   in 5 days
   2  M  ✅  Fix bug             work, urgent
```

**Why the change?**

- ✅ More professional
- ✅ Easier to scan
- ✅ Aligned columns
- ✅ Better for many tasks

**New display function:**

```rust
fn display_task_tabular(
    number: usize,
    task: &Task,
    task_width: usize,
    tags_width: usize
) {
    let number_str = format!("{:>3}", number);
    let letter = task.priority.letter();
    let checkbox = if task.completed { "✅".green() } else { "⏳".bright_white() };

    // Truncate if too long
    let task_text = if task.text.len() > task_width {
        format!("{}...", &task.text[..task_width - 3])
    } else {
        task.text.clone()
    };

    // Format tags
    let tags_str = if task.tags.is_empty() {
        String::new()
    } else {
        let joined = task.tags.join(", ");
        if joined.len() > tags_width {
            format!("{}...", &joined[..tags_width - 3])
        } else {
            joined
        }
    };

    // Get due date text and color
    let due_text = get_due_text(task);
    let due_colored = get_due_colored(task, &due_text);

    // Print with alignment
    if task.completed {
        print!("{:>4} ", number_str.dimmed());
        print!(" {} ", letter);
        print!(" {} ", checkbox);
        print!("{:<width$}", task_text.green(), width = task_width);
        print!("  {:<width$}", tags_str.dimmed(), width = tags_width);
        println!("  {}", due_colored);
    } else {
        print!("{:>4} ", number_str.dimmed());
        print!(" {} ", letter);
        print!(" {} ", checkbox);
        print!("{:<width$}", task_text.bright_white(), width = task_width);
        print!("  {:<width$}", tags_str.cyan(), width = tags_width);
        println!("  {}", due_colored);
    }
}
```

**Formatting tricks:**

```rust
format!("{:>3}", number)   // Right-align in 3 chars: "  1"
format!("{:<40}", text)    // Left-align in 40 chars: "Task text             "
```

**String slicing for truncation:**

```rust
if task.text.len() > task_width {
    format!("{}...", &task.text[..task_width - 3])
}
```

**Example:**

```rust
// task_width = 20
// task.text = "This is a very long task name"
// Length: 30 > 20
&task.text[..17]  // "This is a very lo"
format!("{}...", ...)  // "This is a very lo..."
```

#### Dynamic column widths

**Calculate optimal widths:**

```rust
fn calculate_column_widths(tasks: &[(usize, &Task)]) -> (usize, usize, usize) {
    let mut max_task_len = 10;   // Minimum width
    let mut max_tags_len = 4;
    let mut max_due_len = 3;

    for (_, task) in tasks {
        max_task_len = max_task_len.max(task.text.len());
        
        if !task.tags.is_empty() {
            let tags_str = task.tags.join(", ");
            max_tags_len = max_tags_len.max(tags_str.len());
        }
        
        let due_text = get_due_text(task);
        if !due_text.is_empty() {
            max_due_len = max_due_len.max(due_text.len());
        }
    }

    // Enforce maximum widths
    max_task_len = max_task_len.min(40);
    max_tags_len = max_tags_len.min(20);
    max_due_len = max_due_len.min(20);

    (max_task_len, max_tags_len, max_due_len)
}
```

**Why dynamic widths?**

```bash
# Few tasks, short names:
  ID  P  S  Task    Tags  Due
────────────────────────────────
   1  H  ⏳  Code    work  in 2 days

# Many tasks, long names:
  ID  P  S  Task                              Tags              Due
──────────────────────────────────────────────────────────────────────
   1  H  ⏳  Implement authentication sys...  backend, secur... in 5 days
```

Columns adjust to content!

#### Due date text and colors

**Format due date:**

```rust
fn get_due_text(task: &Task) -> String {
    if let Some(due) = task.due_date {
        let today = Local::now().naive_local().date();
        let days_until = (due - today).num_days();

        if task.completed {
            String::new()  // Don't show for completed tasks
        } else if days_until < 0 {
            format!("late {} day{}", -days_until, if -days_until == 1 { "" } else { "s" })
        } else if days_until == 0 {
            "due today".to_string()
        } else if days_until <= 7 {
            format!("in {} day{}", days_until, if days_until == 1 { "" } else { "s" })
        } else {
            format!("in {} day{}", days_until, if days_until == 1 { "" } else { "s" })
        }
    } else {
        String::new()  // No due date
    }
}
```

**Examples:**

```rust
// days_until = -3
"late 3 days"

// days_until = 0
"due today"

// days_until = 1
"in 1 day"

// days_until = 5
"in 5 days"

// days_until = 30
"in 30 days"
```

**Grammar handling:**

```rust
if -days_until == 1 { "" } else { "s" }
```

Result: "1 day" vs "2 days" (correct grammar!)

**Apply colors:**

```rust
fn get_due_colored(task: &Task, text: &str) -> ColoredString {
    if text.is_empty() {
        return "".normal();
    }

    if let Some(due) = task.due_date {
        let today = Local::now().naive_local().date();
        let days_until = (due - today).num_days();

        if days_until < 0 {
            text.red().bold()         // Overdue: RED + BOLD
        } else if days_until == 0 {
            text.yellow().bold()      // Due today: YELLOW + BOLD
        } else if days_until <= 7 {
            text.yellow()             // Due soon: YELLOW
        } else {
            text.cyan()               // Future: CYAN
        }
    } else {
        text.normal()
    }
}
```

**Color scheme:**

| Days Until | Color | Style | Urgency |
|------------|-------|-------|---------|
| < 0 | Red | Bold | 🚨 LATE! |
| 0 | Yellow | Bold | ⚠️ TODAY! |
| 1-7 | Yellow | Normal | 📅 Soon |
| > 7 | Cyan | Normal | 🗓️ Future |

**Visual hierarchy guides attention to urgent items!**

#### Priority display change

**Before (v1.4.0):**

```rust
fn emoji(&self) -> ColoredString {
    match self {
        Priority::High => "🔴".red(),
        Priority::Medium => "🟡".yellow(),
        Priority::Low => "🟢".green(),
    }
}
```

**After (v1.5.0):**

```rust
fn letter(&self) -> ColoredString {
    match self {
        Priority::High => "H".red(),
        Priority::Medium => "M".yellow(),
        Priority::Low => "L".green(),
    }
}
```

**Why letters instead of emojis?**

- ✅ More professional/terminal-friendly
- ✅ Consistent width (emojis can be double-width)
- ✅ Works better in table format
- ✅ Easier to grep/search in output

#### Testing the feature

```bash
# Add tasks with due dates
$ todo add "Submit report" --high --due 2026-02-05
✓ Task added

$ todo add "Team meeting" --due 2026-02-03  # Today!
✓ Task added

$ todo add "Review PR" --due 2026-01-30  # Overdue
✓ Task added

$ todo add "Plan vacation" --low --due 2026-03-15
✓ Task added

$ todo add "Read email"  # No due date
✓ Task added

# List all tasks
$ todo list

Tasks:

  ID  P  S  Task            Tags  Due
────────────────────────────────────────
   1  H  ⏳  Submit report         in 2 days
   2  M  ⏳  Team meeting          due today
   3  M  ⏳  Review PR             late 4 days
   4  L  ⏳  Plan vacation         in 40 days
   5  M  ⏳  Read email

# See overdue tasks
$ todo list --overdue

Overdue tasks:

  ID  P  S  Task        Tags  Due
──────────────────────────────────
   3  M  ⏳  Review PR         late 4 days

# See what's due soon
$ todo list --due-soon

Due soon (next 7 days):

  ID  P  S  Task            Tags  Due
────────────────────────────────────────
   1  H  ⏳  Submit report         in 2 days
   2  M  ⏳  Team meeting          due today

# Sort by due date
$ todo list --sort due

Tasks:

  ID  P  S  Task            Tags  Due
────────────────────────────────────────
   3  M  ⏳  Review PR             late 4 days
   2  M  ⏳  Team meeting          due today
   1  H  ⏳  Submit report         in 2 days
   4  L  ⏳  Plan vacation         in 40 days
   5  M  ⏳  Read email

# Mark task as done
$ todo done 2
✓ Task marked as completed

# Check JSON
$ cat todos.json
[
  {
    "text": "Submit report",
    "completed": false,
    "priority": "High",
    "tags": [],
    "due_date": "2026-02-05",
    "created_at": "2026-02-03"
  },
  {
    "text": "Team meeting",
    "completed": true,
    "priority": "Medium",
    "tags": [],
    "due_date": "2026-02-03",
    "created_at": "2026-02-03"
  }
]
```

**Everything works!** ✨

**🔗 Resources:**

- [Code v1.5.0](https://github.com/joaofelipegalvao/todo-cli/tree/v1.5.0)
- [Full diff](https://github.com/joaofelipegalvao/todo-cli/compare/v1.4.0...v1.5.0)
- [chrono documentation](https://docs.rs/chrono/)

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

### Collections and Data Structures (v1.4.0+)

- **`Vec<String>`** - Dynamic array of owned strings
- **Multiple values** - Using vectors for repeatable data (tags)
- **`.retain()`** - In-place filtering of vectors
- **`.contains()`** - Check if vector contains element
- **`.join()`** - Concatenate vector elements with separator
- **Deduplication** - Removing duplicate elements from vector
- **Manual loop vs iterator** - When to use while loop for complex parsing
- **Index preservation** - Tuple pattern `(usize, &T)` to keep original indices
- **Sequential filtering** - Applying multiple filters to same vector
- **Backward compatibility** - Using `#[serde(default)]` for optional fields
- **Graceful degradation** - Handling missing fields in old data

### Date and Time (v1.5.0+)

- **`chrono` crate** - De-facto standard for dates/times in Rust
- **`NaiveDate`** - Date without timezone information
- **`Local::now()`** - Get current local time
- **`.naive_local()`** - Convert DateTime to naive (remove timezone)
- **`.date()`** - Extract date from DateTime
- **Date parsing** - `parse_from_str()` with format patterns (`%Y-%m-%d`)
- **Date arithmetic** - Subtracting dates to get Duration
- **`.num_days()`** - Extract days from Duration
- **`Option<NaiveDate>`** - Optional dates (not all tasks have due dates)
- **Date comparison** - Using `<`, `>`, `==` with dates
- **Timestamp fields** - `created_at` for automatic tracking
- **ISO 8601 format** - Standard YYYY-MM-DD date format

### Sorting and Comparison (v1.5.0+)

- **Multi-field sorting** - Sort by priority, due date, or created date
- **`std::cmp::Ordering`** - Less, Equal, Greater for comparisons
- **Sorting with `Option`** - Handling `Some`/`None` in sort comparisons
- **Custom sort order** - Tasks with dates come before tasks without
- **`.sort_by()`** - Custom comparison logic
- **Pattern matching in sort** - Match on `(Option, Option)` tuples
- **Comparison priorities** - Deciding sort order for None values

### String Formatting and Display (v1.5.0+)

- **Format alignment** - `{:>3}` (right), `{:<40}` (left)
- **String slicing** - `&str[..n]` to truncate
- **Truncation with ellipsis** - "Long text..." for readability
- **Dynamic column widths** - Calculate based on content
- **`.max()` and `.min()`** - Enforce width constraints
- **Tabular output** - Professional table formatting
- **Grammar in output** - Singular/plural handling ("1 day" vs "2 days")
- **Conditional text** - Different text based on logic
- **Color-coded urgency** - Visual hierarchy with colors

### CLI Frameworks and Parsing (v1.6.0+)

- **`clap` crate** - Industry-standard CLI framework
- **`#[derive(Parser)]`** - Derive macro for CLI parsing
- **`#[derive(Subcommand)]`** - Enum as subcommands
- **`#[derive(Args)]`** - Struct for command arguments
- **`ValueEnum` trait** - Enums as CLI values
- **`#[arg()]` attributes** - Configure argument behavior
- **`#[command()]` attributes** - Configure command metadata
- **Positional arguments** - Arguments without flags
- **Optional arguments** - `Option<T>` for optional flags
- **Repeatable arguments** - `Vec<T>` for multiple values
- **`value_parser!` macro** - Custom type parsing
- **`value_enum` attribute** - Enable ValueEnum
- **`default_value_t` attribute** - Type-safe defaults
- **`visible_alias` attribute** - Command shortcuts
- **Auto-generated help** - From doc comments and attributes
- **Automatic validation** - Type checking by clap

### Type-Safe Design Patterns (v1.6.0+)

- **Enums over booleans** - Mutually exclusive states
- **Compile-time validation** - Invalid states impossible
- **Exhaustive matching** - Compiler enforces all cases
- **Zero runtime conflicts** - Type system prevents issues
- **ValueEnum pattern** - Enums as CLI values
- **Option pattern for filters** - `None` = filter disabled
- **Separate Args structs** - Complex commands stay organized
- **Doc comments as help** - Documentation generates UI
- **Type-safe parsing** - `NaiveDate` from string automatically
- **Professional error messages** - Clap generates context

### CLI Patterns and UX (v1.4.0+)

- **Repeatable flags** - Parsing `--tag` multiple times
- **Flag with values** - `--tag <value>` pattern
- **Lookahead in parsing** - Accessing next argument (`args[i + 1]`)
- **Grammar in output** - "1 task" vs "2 tasks" (singular/plural)
- **Visual hierarchy** - Different colors for different states
- **Critical bug patterns** - Index mismatches in filtered views
- **Original numbering** - Preserving indices across filters
- **Discovery commands** - `tags` command for exploration

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
11. **Professional CLI** - clap framework and industry patterns

**Most importantly:** Learning is a process. Each version had bugs, inefficiencies, and room for improvement. That's expected and valuable.

**Version milestones:**

- **v0.1-v0.5:** Basic functionality (make it work)
- **v0.6-v0.9:** Polish and features (make it nice)
- **v1.0-v1.1:** Professional quality (make it complete)
- **v1.2:** Type safety and architecture (make it right)
- **v1.3:** Automatic serialization (make it maintainable)
- **v1.4:** Tags and bug fixes (make it reliable)
- **v1.5:** Due dates and professional display (make it practical)
- **v1.6:** Clap framework and enums (make it professional)

**Evolution of the codebase:**

```
v0.1: String matching everywhere
  ↓
v1.2: Type-safe structs and enums (36% reduction)
  ↓
v1.3: Automatic JSON serialization (91% I/O reduction)
  ↓
v1.4: Extensible with tags (1 line = new feature)
  ↓
v1.5: Due dates + tabular display (deadline tracking + professional UX)
  ↓
v1.6: Clap + ValueEnum (zero manual parsing, compile-time safety)
  ↓
Next: Ready for crates.io publication!
```

**The journey is the lesson.** 🦀

---

## Next Steps

**The CLI is now production-ready:**

✅ Type-safe from command line to storage  
✅ Professional help and error messages  
✅ Zero manual parsing  
✅ Industry-standard patterns  
✅ Extensible architecture  

**Potential future versions:**

- **v1.7:** Recurring tasks with chrono patterns
- **v1.8:** Subtasks/nested tasks with `Box<Task>`
- **v1.9:** Multiple projects/contexts
- **v2.0:** TUI with `ratatui`
- **v2.1:** Configuration file with `config` crate
- **v2.2:** Shell completions (bash, zsh, fish)
- **v2.3:** Export/import (CSV, JSON, Markdown)
- **v2.4:** Sync with cloud storage
- **v2.5:** Web API with `axum`
- **v3.0:** Plugin system

**Each version would teach:**

| Version | Feature | Rust Concepts |
|---------|---------|---------------|
| ~~v1.4~~ | ~~Tags~~ | ~~`Vec<String>`, `.retain()`, `#[serde(default)]`~~ ✅
| ~~v1.5~~ | ~~Due dates~~ | ~~`chrono`, `NaiveDate`, date arithmetic, sorting with `Option`~~ ✅
| ~~v1.6~~ | ~~Clap CLI~~ | ~~`clap`, derive macros, `ValueEnum`, type-safe enums~~ ✅
| v1.7 | Recurring | `chrono::Duration`, pattern matching, date intervals |
| v1.8 | Subtasks | Recursive data structures, `Box<T>`, tree traversal |
| v1.9 | Projects | Multiple files, workspace patterns, namespacing |
| v2.0 | TUI | Event loops, rendering, `ratatui`, state management |
| v2.1 | Config | TOML parsing, `config` crate, user preferences |
| v2.2 | Completions | Shell scripting, clap completions, cross-platform |
| v2.3 | Export | CSV, Markdown, format conversions |
| v2.4 | Sync | HTTP clients, `tokio`, async/await, `reqwest` |
| v2.5 | Web API | `axum`, REST APIs, JSON responses, routing |
| v3.0 | Plugins | Dynamic loading, FFI, trait objects |

**The beauty of this architecture:**

All new features benefit from:

```rust
#[derive(Serialize, Deserialize)]
struct Task {
    text: String,
    completed: bool,
    priority: Priority,
    tags: Vec<String>,        // ✅ v1.4: DONE
    due_date: Option<NaiveDate>, // ✅ v1.5: DONE
    created_at: NaiveDate,    // ✅ v1.5: DONE
    recurring: Option<Recurrence>, // ← v1.7: Add ONE line
    subtasks: Vec<Task>,      // ← v1.8: Add ONE line
    project: String,          // ← v1.9: Add ONE line
}

// CLI automatically gets new filters:
#[derive(ValueEnum)]
enum RecurrenceFilter {
    Daily,
    Weekly,
    Monthly,
}

// And clap generates everything!
```

**No parser changes needed!** Serde + Clap handle everything automatically.

**This is why we refactored:**

Not just for now, but to **enable future growth** with minimal friction.

**Ready for the real world!** 🚀

---

### v1.6.0 - Professional CLI with Clap

**🎯 Goal:** Replace manual argument parsing with `clap` and migrate boolean flags to type-safe enums

**📦 The Problem We're Solving:**

**Before v1.6.0 - Manual parsing with conflicts:**

```rust
// Manual parsing:
let args: Vec<String> = env::args().collect();
if args.len() < 2 { return Err("Usage: ...".into()); }

let command = &args[1];
match command.as_str() {
    "add" => { /* manual parsing of flags */ }
    "list" => {
        let mut status_filter = "all";
        let mut priority_filter: Option<Priority> = None;
        let mut overdue = false;
        let mut due_soon = false;
        let mut with_due = false;
        let mut without_due = false;
        
        // Manual conflict checking:
        if overdue || due_soon || with_due || without_due {
            return Err("Use only one date filter".into());
        }
    }
}
```

**Problems:**

❌ Manual parsing (100+ lines just for flags)  
❌ Boolean flags conflict manually (`conflicts_with_all`)  
❌ Help text written by hand  
❌ No type safety (strings everywhere)  
❌ Hard to add new flags (lots of boilerplate)  
❌ Error messages generic  

**After v1.6.0 - Clap with enums:**

```rust
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    List {
        #[arg(long, value_enum, default_value_t = StatusFilter::All)]
        status: StatusFilter,
        
        #[arg(long, value_enum)]
        due: Option<DueFilter>,
        
        // No conflicts - enums are mutually exclusive!
    }
}
```

**Benefits:**

✅ **Zero manual parsing** - clap does everything  
✅ **Zero conflicts** - enums are inherently exclusive  
✅ **Auto-generated help** - beautiful and complete  
✅ **Type-safe** - compiler validates everything  
✅ **Easy to extend** - add enum value = done  
✅ **Professional errors** - clap provides context  

**🧠 Key Concepts:**

#### What is Clap?

**Clap** = **C**ommand **L**ine **A**rgument **P**arser

The most popular CLI framework in Rust (used by cargo, ripgrep, bat, fd, etc.)

**Why clap?**

- ✅ Derive macros - write structs, get parser
- ✅ Auto-generated help/version
- ✅ Subcommands support
- ✅ Type-safe parsing
- ✅ Shell completions
- ✅ Industry standard

**Adding clap:**

```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
```

**Feature `derive`** enables `#[derive(Parser)]` macros.

#### The derive macro pattern

**Instead of manual parsing:**

```rust
// 50 lines of if/else/match
```

**Write declarative structs:**

```rust
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
```

**Clap generates:**

- Parsing logic
- Help text
- Error messages
- Validation
- Type conversions

**All automatically!**

#### Main CLI structure

```rust
#[derive(Parser)]
#[command(name = "todo-list")]
#[command(author = "github.com/joaofelipegalvao")]
#[command(version = "1.6.0")]
#[command(about = "A modern, powerful task manager built with Rust")]
#[command(after_help = "EXAMPLES:
    todo add \"Task\" --priority high
    todo list --status pending
")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
```

**Attributes explained:**

| Attribute | Purpose |
|-----------|---------|
| `#[command(name)]` | Program name in help |
| `#[command(author)]` | Shown in `--version` |
| `#[command(version)]` | Version string |
| `#[command(about)]` | Short description |
| `#[command(after_help)]` | Examples after help |

**Result:**

```bash
$ todo --help
A modern, powerful task manager built with Rust

Usage: todo-list <COMMAND>

Commands:
  add     Add a new task
  list    List and filter tasks
  ...

EXAMPLES:
    todo add "Task" --priority high
    todo list --status pending
```

**All of this from struct attributes!**

#### Subcommands with enum

```rust
#[derive(Subcommand)]
enum Commands {
    Add(AddArgs),
    List { ... },
    Done { id: usize },
    Remove { id: usize },
    Search { query: String },
    Tags,
    Clear,
}
```

**Pattern:**

- **Named fields** for simple args: `Done { id: usize }`
- **Separate struct** for complex args: `Add(AddArgs)`

**Why?**

```rust
// Simple command - inline:
Done { id: usize }

// Complex command - separate struct:
Add(AddArgs)

#[derive(Args)]
struct AddArgs {
    text: String,
    #[arg(long)]
    priority: Priority,
    #[arg(long, short = 't')]
    tag: Vec<String>,
    // ... more args
}
```

**Keeps `Commands` enum clean!**

#### StatusFilter enum

**The problem we had:**

```rust
// Boolean flags:
let mut status_filter = "all";  // "all", "pending", or "done"

// Manual conflict checking:
if args.contains("--pending") && args.contains("--done") {
    return Err("Can't use both --pending and --done".into());
}
```

**The solution:**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum StatusFilter {
    /// Show only pending tasks
    Pending,
    /// Show only completed tasks
    Done,
    /// Show all tasks (default)
    All,
}
```

**What's `ValueEnum`?**

A clap trait that makes enums work as CLI values:

```bash
todo list --status pending  # Parses to StatusFilter::Pending
todo list --status done     # Parses to StatusFilter::Done
todo list --status all      # Parses to StatusFilter::All
```

**Automatic:**

- Parsing from string
- Validation (rejects invalid values)
- Help text generation
- Case-insensitive matching

**Usage in command:**

```rust
List {
    #[arg(long, value_enum, default_value_t = StatusFilter::All)]
    status: StatusFilter,
}
```

**Attributes:**

- `value_enum` - Use ValueEnum trait
- `default_value_t` - Default if not provided

**Helper method:**

```rust
impl Task {
    fn matches_status(&self, status: StatusFilter) -> bool {
        match status {
            StatusFilter::Pending => !self.completed,
            StatusFilter::Done => self.completed,
            StatusFilter::All => true,
        }
    }
}
```

**Why this is better:**

```rust
// Before:
match status_filter {
    "pending" => !self.completed,  // Could typo "pendingg"
    "done" => self.completed,
    _ => true,  // Catch-all needed
}

// After:
match status {
    StatusFilter::Pending => !self.completed,  // Typos = compile error
    StatusFilter::Done => self.completed,
    StatusFilter::All => true,  // Exhaustive - can't forget case
}
```

**Compiler enforces correctness!**

#### DueFilter enum

**The old way - 4 boolean flags:**

```rust
let mut overdue = false;
let mut due_soon = false;
let mut with_due = false;
let mut without_due = false;

// Manual mutual exclusion:
if overdue || due_soon || with_due || without_due {
    return Err("Use only one date filter".into());
}

if overdue {
    indexed_tasks.retain(|(_, t)| t.is_overdue());
}
if due_soon {
    indexed_tasks.retain(|(_, t)| t.is_due_soon(7));
}
// ... etc
```

**The new way - 1 enum:**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum DueFilter {
    /// Tasks past their due date
    Overdue,
    /// Tasks due in the next 7 days
    Soon,
    /// Tasks with any due date set
    WithDue,
    /// Tasks without a due date
    NoDue,
}
```

**Usage:**

```rust
List {
    #[arg(long, value_enum)]
    due: Option<DueFilter>,  // None = no filter
}
```

**CLI:**

```bash
todo list --due overdue
todo list --due soon
todo list --due with-due
todo list --due no-due
```

**Helper method:**

```rust
impl Task {
    fn matches_due_filter(&self, filter: DueFilter) -> bool {
        match filter {
            DueFilter::Overdue => self.is_overdue(),
            DueFilter::Soon => self.is_due_soon(7),
            DueFilter::WithDue => self.due_date.is_some(),
            DueFilter::NoDue => self.due_date.is_none(),
        }
    }
}
```

**Filtering logic:**

```rust
if let Some(due_filter) = due {
    indexed_tasks.retain(|(_, t)| t.matches_due_filter(due_filter));
}
```

**Benefits:**

| Aspect | 4 Booleans | 1 Enum |
|--------|------------|--------|
| Conflicts | Manual checking | Automatic (Option) |
| Validation | Runtime | Compile-time |
| Adding filter | +1 bool + conflict check | +1 enum variant |
| Help text | 4 separate flags | 1 organized group |
| Type safety | Can set multiple true | Impossible |

**Code reduction: 15 lines → 3 lines**

#### SortBy enum

**Before - string matching:**

```rust
let mut sort_by = "none";

if args.contains("--sort") {
    let idx = args.iter().position(|a| a == "--sort").unwrap();
    if idx + 1 < args.len() {
        sort_by = match args[idx + 1].as_str() {
            "priority" => "priority",
            "due" => "due",
            "created" => "created",
            _ => return Err("Invalid sort field".into()),
        };
    }
}

match sort_by {
    "priority" => { /* sort */ }
    "due" => { /* sort */ }
    "created" => { /* sort */ }
    _ => {}
}
```

**After - enum:**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum SortBy {
    /// Sort by priority (High → Medium → Low)
    Priority,
    /// Sort by due date (earliest first)
    Due,
    /// Sort by creation date (oldest first)
    Created,
}
```

**Usage:**

```rust
List {
    #[arg(long, short = 's', value_enum)]
    sort: Option<SortBy>,
}
```

**CLI:**

```bash
todo list --sort priority
todo list --sort due
todo list --sort created
todo list -s priority  # short form
```

**Sorting logic:**

```rust
if let Some(sort_by) = sort {
    match sort_by {
        SortBy::Priority => {
            indexed_tasks.sort_by(|(_, a), (_, b)| 
                a.priority.order().cmp(&b.priority.order()));
        }
        SortBy::Due => {
            indexed_tasks.sort_by(|(_, a), (_, b)| 
                match (a.due_date, b.due_date) {
                    (Some(date_a), Some(date_b)) => date_a.cmp(&date_b),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                });
        }
        SortBy::Created => {
            indexed_tasks.sort_by(|(_, a), (_, b)| 
                a.created_at.cmp(&b.created_at));
        }
    }
}
```

**Clean and exhaustive!**

#### Priority with ValueEnum

**Updated Priority enum:**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
enum Priority {
    /// High priority - urgent and important tasks
    High,
    /// Medium priority - default for most tasks
    Medium,
    /// Low priority - nice to have, not urgent
    Low,
}
```

**New trait: `ValueEnum`**

Enables using Priority as a CLI value:

```bash
todo add "Task" --priority high
todo add "Task" --priority medium
todo add "Task" --priority low
```

**In JSON (lowercase thanks to serde):**

```json
{
  "priority": "high"
}
```

**In CLI help:**

```
--priority <PRIORITY>
    Task priority level
    
    Possible values:
    - high:   High priority - urgent and important tasks
    - medium: Medium priority - default for most tasks
    - low:    Low priority - nice to have, not urgent
```

**Doc comments become help text!**

#### AddArgs structure

**Dedicated struct for complex command:**

```rust
#[derive(Args)]
struct AddArgs {
    /// Task description
    #[arg(value_name = "DESCRIPTION")]
    text: String,

    /// Task priority level
    #[arg(long, value_enum, default_value_t = Priority::Medium)]
    priority: Priority,

    /// Add tags (can be repeated: -t work -t urgent)
    #[arg(long, short = 't', value_name = "TAG")]
    tag: Vec<String>,

    /// Due date in format YYYY-MM-DD
    #[arg(long, value_name = "DATE", value_parser = clap::value_parser!(NaiveDate))]
    due: Option<NaiveDate>,
}
```

**Attributes explained:**

**`text` field:**

```rust
#[arg(value_name = "DESCRIPTION")]
text: String,
```

- Positional argument (no `--` flag)
- Required (not `Option`)
- `value_name` for help: `todo add <DESCRIPTION>`

**`priority` field:**

```rust
#[arg(long, value_enum, default_value_t = Priority::Medium)]
priority: Priority,
```

- `long` - uses `--priority`
- `value_enum` - use ValueEnum trait
- `default_value_t` - default if not provided

**`tag` field:**

```rust
#[arg(long, short = 't', value_name = "TAG")]
tag: Vec<String>,
```

- `long` - `--tag`
- `short = 't'` - also accepts `-t`
- `Vec<String>` - can repeat: `-t work -t urgent`

**`due` field:**

```rust
#[arg(long, value_name = "DATE", value_parser = clap::value_parser!(NaiveDate))]
due: Option<NaiveDate>,
```

- `value_parser!(NaiveDate)` - **THIS IS MAGIC**
- Clap automatically parses string → NaiveDate
- Returns helpful error if invalid format

**Example usage:**

```bash
todo add "Deploy to production" \
    --priority high \
    --tag work \
    --tag deployment \
    --tag urgent \
    --due 2026-02-15
```

**Parsed into:**

```rust
AddArgs {
    text: "Deploy to production".to_string(),
    priority: Priority::High,
    tag: vec!["work".to_string(), "deployment".to_string(), "urgent".to_string()],
    due: Some(NaiveDate::from_ymd_opt(2026, 2, 15).unwrap()),
}
```

**All automatic!**

#### Automatic NaiveDate parsing

**Before v1.6.0 - manual:**

```rust
"--due" => {
    if i + 1 >= args.len() {
        return Err("--due requires a date in format YYYY-MM-DD".into());
    }
    
    let date_str = &args[i + 1];
    match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        Ok(date) => due_date = Some(date),
        Err(_) => {
            return Err(format!(
                "Invalid date format: '{}'. Use YYYY-MM-DD",
                date_str
            ).into());
        }
    }
    
    i += 1;
}
```

**8 lines of code!**

**After v1.6.0 - clap parser:**

```rust
#[arg(long, value_parser = clap::value_parser!(NaiveDate))]
due: Option<NaiveDate>,
```

**1 line!**

**How `value_parser!` works:**

```rust
value_parser!(NaiveDate)
```

Clap looks for `FromStr` implementation:

```rust
impl FromStr for NaiveDate {
    type Err = ParseError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // chrono implements this
    }
}
```

**Clap automatically:**

1. Takes CLI string
2. Calls `NaiveDate::from_str()`
3. Returns parsed value or error

**Error handling:**

```bash
$ todo add "Task" --due 2026-13-50
error: invalid value '2026-13-50' for '--due <DATE>': input is out of range

$ todo add "Task" --due tomorrow
error: invalid value 'tomorrow' for '--due <DATE>': premature end of input
```

**Professional errors for free!**

#### Command aliases

**Before - single command name:**

```rust
"add" => { /* ... */ }
```

**After - multiple names:**

```rust
#[command(visible_alias = "a")]
Add(AddArgs),

#[command(visible_alias = "ls")]
List { /* ... */ },

#[command(visible_aliases = ["rm", "delete"])]
Remove { id: usize },
```

**Usage:**

```bash
# All equivalent:
todo add "Task"
todo a "Task"

# All equivalent:
todo list --status pending
todo ls --status pending

# All equivalent:
todo remove 3
todo rm 3
todo delete 3
```

**Why `visible_alias`?**

Shows in help:

```
Commands:
  add      Add a new task [aliases: a]
  list     List tasks [aliases: ls]
  remove   Remove task [aliases: rm, delete]
```

**Improves UX without code duplication!**

#### Auto-generated help

**Command-level help:**

```rust
#[command(long_about = "Add a new task to your todo list\n\n\
    Creates a new task with the specified text and optional metadata like priority,\n\
    tags, and due date. Tasks are saved immediately to todos.json.")]
Add(AddArgs),
```

**Result:**

```bash
$ todo add --help
Add a new task to your todo list

Creates a new task with the specified text and optional metadata like priority,
tags, and due date. Tasks are saved immediately to todos.json.

Usage: todo add <DESCRIPTION> [OPTIONS]

Arguments:
  <DESCRIPTION>  Task description

Options:
      --priority <PRIORITY>  Task priority level [default: medium]
  -t, --tag <TAG>           Add tags (can be repeated)
      --due <DATE>          Due date in format YYYY-MM-DD
  -h, --help                Print help
```

**All from struct attributes!**

**Top-level examples:**

```rust
#[command(after_help = "EXAMPLES:
    # Add a high priority task
    todo add \"Task\" --priority high

    # List pending tasks
    todo list --status pending
")]
struct Cli { /* ... */ }
```

**Shows after main help:**

```bash
$ todo --help
...

EXAMPLES:
    # Add a high priority task
    todo add "Task" --priority high

    # List pending tasks
    todo list --status pending
```

**No manual help writing needed!**

#### Combining filters elegantly

**Before - manual validation:**

```rust
// Can't combine these:
if overdue && due_soon {
    return Err("Can't use both --overdue and --due-soon".into());
}
if overdue && with_due {
    return Err("Can't use both --overdue and --with-due".into());
}
// ... 6 more checks
```

**After - type system prevents it:**

```rust
List {
    #[arg(long, value_enum)]
    due: Option<DueFilter>,  // Can only be ONE value or None
}
```

**Impossible to have conflicts:**

```bash
# Before - possible (needed manual check):
todo list --overdue --due-soon  # ERROR (manual)

# After - impossible (clap rejects):
todo list --due overdue --due soon
error: the argument '--due <DUE_FILTER>' cannot be used multiple times
```

**Combining different filter types:**

```rust
List {
    status: StatusFilter,      // Always set (has default)
    priority: Option<Priority>, // Optional filter
    due: Option<DueFilter>,     // Optional filter
    tag: Option<String>,        // Optional filter
    sort: Option<SortBy>,       // Optional sorting
}
```

**All can be combined:**

```bash
todo list \
    --status pending \
    --priority high \
    --due soon \
    --tag work \
    --sort due
```

**Filtering logic:**

```rust
// 1. Status (always applied)
indexed_tasks.retain(|(_, t)| t.matches_status(status));

// 2. Priority (if Some)
if let Some(pri) = priority {
    indexed_tasks.retain(|(_, t)| t.priority == pri);
}

// 3. Due filter (if Some)
if let Some(due_filter) = due {
    indexed_tasks.retain(|(_, t)| t.matches_due_filter(due_filter));
}

// 4. Tag (if Some)
if let Some(tag_name) = &tag {
    indexed_tasks.retain(|(_, t)| t.tags.contains(tag_name));
}

// 5. Sort (if Some)
if let Some(sort_by) = sort {
    // Apply sorting
}
```

**Clean, sequential, type-safe!**

#### Main function with clap

**Before - manual:**

```rust
fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        return Err("Usage: ...".into());
    }
    
    let command = &args[1];
    
    match command.as_str() {
        // ... 100+ lines
    }
}
```

**After - clap:**

```rust
fn main() {
    let cli = Cli::parse();  // ← Clap does EVERYTHING
    
    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    match cli.command {
        Commands::Add(args) => { /* ... */ }
        Commands::List { status, priority, due, tag, sort } => { /* ... */ }
        Commands::Done { id } => { /* ... */ }
        // ...
    }
}
```

**What `Cli::parse()` does:**

1. Reads `std::env::args()`
2. Parses all arguments
3. Validates types
4. Checks requirements
5. Returns `Cli` struct OR exits with help/error

**Automatic behaviors:**

```bash
$ todo
error: 'todo' requires a subcommand but one was not provided

$ todo --help
# Shows full help

$ todo --version
todo-list 1.6.0

$ todo add
error: the following required arguments were not provided:
  <DESCRIPTION>

$ todo add "Task" --priority invalid
error: invalid value 'invalid' for '--priority <PRIORITY>'
  [possible values: high, medium, low]
```

**All automatic!**

#### Testing the migration

```bash
# 1. Add tasks with new syntax
$ todo add "Study Rust async" --priority high --tag learning -t rust --due 2026-02-20
✓ Task added

$ todo add "Team meeting" --priority medium --tag work --due 2026-02-10
✓ Task added

$ todo add "Buy groceries" --priority low --tag personal
✓ Task added

# 2. List with filters
$ todo list --status pending --priority high

High priority pending tasks:

  ID  P  S  Task                Tags            Due
────────────────────────────────────────────────────
   1  H  ⏳  Study Rust async    learning, rust  in 16 days

# 3. Date filters
$ todo list --due soon --sort due

Tasks due soon:

  ID  P  S  Task            Tags  Due
──────────────────────────────────────
   2  M  ⏳  Team meeting    work  in 6 days

# 4. Combine everything
$ todo list --status pending --priority high --tag learning --sort due

High priority pending tasks:

  ID  P  S  Task                Tags            Due
────────────────────────────────────────────────────
   1  H  ⏳  Study Rust async    learning, rust  in 16 days

# 5. Use aliases
$ todo a "Quick task"  # alias for 'add'
$ todo ls              # alias for 'list'
$ todo rm 3            # alias for 'remove'

# 6. Check help
$ todo add --help
Add a new task to your todo list
...

$ todo list --help
List and filter tasks with powerful filtering options
...
```

**Everything works with better UX!**

**🔗 Resources:**

- [Code v1.6.0](https://github.com/joaofelipegalvao/todo-cli/tree/v1.6.0)
- [Full diff](https://github.com/joaofelipegalvao/todo-cli/compare/v1.5.0...v1.6.0)
- [Clap documentation](https://docs.rs/clap/)
- [Clap derive tutorial](https://docs.rs/clap/latest/clap/_derive/index.html)

---

**Built with ❤️ to learn Rust - Each commit represents a step in the learning journey**
