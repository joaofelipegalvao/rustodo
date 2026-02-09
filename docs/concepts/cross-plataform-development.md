# Cross-Platform Development

**📚 Overview:**

This page covers cross-platform development patterns in Rust, focusing on handling platform-specific requirements like configuration directories, file paths, and OS conventions.

**🔗 Related Versions:**

- [v0.1.0](../getting-started/v0.1.0-basic-cli.md) - Basic file operations
- [v0.5.0](../getting-started/v0.5.0-clear-command.md) - File existence checks
- [v1.3.0](../advanced/v1.3.0-json-serialization.md) - JSON storage
- [v1.8.0](../advanced/v1.8.0-global-data-directory.md) - **Platform-specific directories**

---

## Platform Differences in File Storage

### The Problem: Hardcoded Paths

**Naive approach (doesn't work cross-platform):**

```rust
// ❌ Linux-specific path
let config_path = PathBuf::from("~/.config/myapp/config.json");

// ❌ Windows-specific path
let config_path = PathBuf::from("C:\\Users\\user\\AppData\\myapp\\config.json");

// ❌ Assumes Unix-style home directory
let config_path = PathBuf::from("/home/user/.myapp/config.json");
```

**Problems:**

- Hardcoded paths break on other platforms
- Doesn't follow platform conventions
- Can't expand `~` automatically
- Assumes specific filesystem layout

### The Solution: Platform Abstraction

**Using the `directories` crate:**

```rust
use directories::ProjectDirs;

if let Some(proj_dirs) = ProjectDirs::from("", "", "myapp") {
    let config_dir = proj_dirs.config_dir();
    // Linux:   /home/user/.config/myapp
    // macOS:   /Users/user/Library/Application Support/myapp
    // Windows: C:\Users\user\AppData\Roaming\myapp
}
```

**Benefits:**

- ✅ Works on Linux, macOS, Windows
- ✅ Follows platform conventions
- ✅ Automatically resolves correct paths
- ✅ Single codebase for all platforms

---

## Platform Conventions

### Linux: XDG Base Directory Specification

**Standard locations:**

```rust
// Config directory
~/.config/myapp/           // XDG_CONFIG_HOME or default

// Data directory
~/.local/share/myapp/      // XDG_DATA_HOME or default

// Cache directory
~/.cache/myapp/            // XDG_CACHE_HOME or default
```

**Environment variables:**

- `XDG_CONFIG_HOME` - User configuration files
- `XDG_DATA_HOME` - User data files
- `XDG_CACHE_HOME` - User cache files

**Example:**

```rust
use directories::ProjectDirs;

let proj_dirs = ProjectDirs::from("", "", "myapp").unwrap();

// Config: ~/.config/myapp
let config_dir = proj_dirs.config_dir();

// Data: ~/.local/share/myapp
let data_dir = proj_dirs.data_dir();

// Cache: ~/.cache/myapp
let cache_dir = proj_dirs.cache_dir();
```

### macOS: Application Support

**Standard locations:**

```rust
// Application Support
~/Library/Application Support/myapp/

// Caches
~/Library/Caches/myapp/

// Preferences (plist files)
~/Library/Preferences/
```

**Characteristics:**

- Uses `~/Library` for all app data
- Separate `Application Support` and `Caches`
- GUI apps often use property lists (`.plist`)

### Windows: AppData

**Standard locations:**

```rust
// Roaming profile (syncs across machines)
%APPDATA%\myapp\
// Example: C:\Users\Name\AppData\Roaming\myapp

// Local profile (machine-specific)
%LOCALAPPDATA%\myapp\
// Example: C:\Users\Name\AppData\Local\myapp
```

**Roaming vs Local:**

- **Roaming** (`APPDATA`) - Synced across domain computers
- **Local** (`LOCALAPPDATA`) - Machine-specific (caches, temp files)

---

## The `directories` Crate

### Installation

```toml
[dependencies]
directories = "5.0"
```

### Basic Usage

```rust
use directories::{ProjectDirs, BaseDirs};

// Project-specific directories
if let Some(proj_dirs) = ProjectDirs::from("com", "example", "myapp") {
    let config = proj_dirs.config_dir();    // Platform-specific config
    let data = proj_dirs.data_dir();        // Platform-specific data
    let cache = proj_dirs.cache_dir();      // Platform-specific cache
}

// User base directories
if let Some(base_dirs) = BaseDirs::new() {
    let home = base_dirs.home_dir();        // User home directory
    let desktop = base_dirs.desktop_dir();  // Desktop directory (if exists)
    let documents = base_dirs.document_dir(); // Documents directory
}
```

### `ProjectDirs::from()` Parameters

```rust
ProjectDirs::from(qualifier, organization, application)
```

**Parameters explained:**

| Parameter | Purpose | Example | Linux | macOS | Windows |
|-----------|---------|---------|-------|-------|---------|
| `qualifier` | Domain (reverse) | `"com"` | Ignored | Used in path | Used in path |
| `organization` | Company/org | `"Example"` | Ignored | Used in path | Used in path |
| `application` | App name | `"MyApp"` | Used | Used | Used |

**Examples:**

```rust
// Minimal (just app name)
ProjectDirs::from("", "", "myapp")
// Linux:   ~/.config/myapp
// macOS:   ~/Library/Application Support/myapp
// Windows: %APPDATA%\myapp

// Full qualification
ProjectDirs::from("com", "Example", "MyApp")
// Linux:   ~/.config/myapp (qualifier/org ignored)
// macOS:   ~/Library/Application Support/com.Example.MyApp
// Windows: %APPDATA%\Example\MyApp
```

**Which to use?**

- **Simple CLI tools**: Use minimal form `("", "", "appname")`
- **Company apps**: Use full form `("com", "CompanyName", "AppName")`

---

## Path Manipulation

### `PathBuf` vs `Path`

**Two types for working with paths:**

```rust
use std::path::{Path, PathBuf};

// PathBuf - owned, mutable (like String)
let mut path_buf = PathBuf::from("/home/user");
path_buf.push("documents");  // Can modify
path_buf.push("file.txt");
// Result: /home/user/documents/file.txt

// Path - borrowed, immutable (like &str)
let path: &Path = &path_buf;
let parent = path.parent();  // Can read, not modify
```

**When to use each:**

| Type | When to Use | Example |
|------|-------------|---------|
| `PathBuf` | Building paths, returning owned paths | `fn get_config_path() -> PathBuf` |
| `&Path` | Reading paths, passing parameters | `fn read_file(path: &Path)` |

### Common Path Operations

**Joining paths:**

```rust
let base = PathBuf::from("/home/user");
let full = base.join("documents").join("file.txt");
// Result: /home/user/documents/file.txt

// Alternative method (used in actual todo-cli implementation):
let mut path = base_dir.to_path_buf();
path.push("documents");
path.push("file.txt");
// Result: /home/user/documents/file.txt

// Both work, push() modifies in-place, join() creates new PathBuf
```

**Works with different separators on each platform:**

```rust
// Linux/macOS: /home/user/documents/file.txt
// Windows: C:\Users\user\documents\file.txt
```

**Difference between `.join()` and `.push()`:**

```rust
// .join() - creates new PathBuf
let path1 = base.join("file.txt");  // base unchanged

// .push() - modifies existing PathBuf (used in todo-cli)
let mut path2 = base.clone();
path2.push("file.txt");  // path2 modified

// Both work, choose based on whether you need the original
```

**Path components:**

```rust
let path = PathBuf::from("/home/user/file.txt");

path.file_name()    // Some("file.txt")
path.parent()       // Some("/home/user")
path.extension()    // Some("txt")
path.file_stem()    // Some("file")
```

**Display paths:**

```rust
let path = PathBuf::from("/home/user/file.txt");

// For user display
println!("Path: {}", path.display());

// For debugging
println!("Path: {:?}", path);
```

---

## Directory Creation

### `create_dir()` vs `create_dir_all()`

**Single directory:**

```rust
use std::fs;

// Creates only the last directory
fs::create_dir("/home/user/newdir")?;
// Success if /home/user exists
// Error if parent doesn't exist
```

**Recursive creation:**

```rust
// Creates all missing parent directories
fs::create_dir_all("/home/user/deeply/nested/path")?;
// Creates: /home/user/deeply
//          /home/user/deeply/nested
//          /home/user/deeply/nested/path

// Like 'mkdir -p' on Unix
```

**Which to use?**

- `create_dir()` - When you know parent exists
- `create_dir_all()` - When parent might not exist (safer)

### Idempotent Directory Creation

```rust
fs::create_dir_all(data_dir)?;
```

**What it does:**

```bash
# Creates all parent directories if they don't exist
# Like 'mkdir -p' on Unix

# If this doesn't exist:
/home/user/.config/todo-cli/

# create_dir_all() creates:
/home/user/.config/          # If needed
/home/user/.config/todo-cli/ # Always (or does nothing if exists)
```

**Error handling:**

- Returns `io::Result<()>`
- Fails if:
  - Path exists but is a file, not directory
  - Insufficient permissions
  - Disk full
  - Path contains null bytes (invalid)

**Idempotency:**

```rust
// Safe to call multiple times - the actual implementation
fs::create_dir_all(data_dir)?;  // Creates directory
fs::create_dir_all(data_dir)?;  // No error - directory already exists

// Note: Our implementation calls this EVERY time, not conditionally
// This is actually more robust than checking first
```

**Why unconditional is better:**

```rust
// ❌ Race condition possible
if !data_dir.exists() {
    fs::create_dir_all(data_dir)?;  // Another process might delete between check and create
}

// ✅ No race condition - actual implementation
fs::create_dir_all(data_dir)?;  // Atomic - always safe
```

---

## Platform Detection

### Compile-Time Detection

**Rust's `cfg` attributes:**

```rust
#[cfg(target_os = "linux")]
fn get_platform() -> &'static str {
    "Linux"
}

#[cfg(target_os = "macos")]
fn get_platform() -> &'static str {
    "macOS"
}

#[cfg(target_os = "windows")]
fn get_platform() -> &'static str {
    "Windows"
}
```

**Available cfg attributes:**

```rust
#[cfg(target_os = "linux")]      // Linux
#[cfg(target_os = "macos")]      // macOS
#[cfg(target_os = "windows")]    // Windows
#[cfg(target_os = "freebsd")]    // FreeBSD
#[cfg(target_os = "openbsd")]    // OpenBSD

#[cfg(unix)]                     // Any Unix (Linux, macOS, BSD)
#[cfg(windows)]                  // Windows
```

**Conditional compilation:**

```rust
fn get_separator() -> char {
    #[cfg(unix)]
    {
        '/'  // Unix path separator
    }
    
    #[cfg(windows)]
    {
        '\\'  // Windows path separator
    }
}
```

### Runtime vs Compile-Time

**Compile-time (preferred):**

```rust
#[cfg(target_os = "linux")]
const DEFAULT_SHELL: &str = "/bin/bash";

#[cfg(target_os = "windows")]
const DEFAULT_SHELL: &str = "cmd.exe";

// Compiler includes only one constant
// No runtime overhead
// Type-safe
```

**Runtime (when needed):**

```rust
use std::env;

fn get_platform() -> &'static str {
    if cfg!(target_os = "linux") {
        "Linux"
    } else if cfg!(target_os = "macos") {
        "macOS"
    } else if cfg!(target_os = "windows") {
        "Windows"
    } else {
        "Other"
    }
}

// Still evaluated at compile time
// But allows runtime-like logic
```

---

## Best Practices

### 1. Use Standard Library Paths

```rust
// ✅ Good - platform-aware
use std::path::PathBuf;

let path = PathBuf::from(dir);
let full_path = path.join(file);

// ❌ Bad - assumes Unix
let full_path = format!("{}/{}", dir, file);
```

### 2. Use `directories` for Standard Locations

```rust
// ✅ Good - platform conventions
use directories::ProjectDirs;

let proj_dirs = ProjectDirs::from("", "", "myapp").unwrap();
let config = proj_dirs.config_dir();

// ❌ Bad - hardcoded Linux path
let config = PathBuf::from("~/.config/myapp");
```

### 3. Handle Path Display Correctly

```rust
let path = config_dir.join("file.txt");

// ✅ Good - for user display
println!("Config at: {}", path.display());

// ✅ Good - for error messages
.context(format!("Failed to read {}", path.display()))

// ❌ Bad - Debug format for users
println!("Config at: {:?}", path);  // Shows quotes, escapes
```

### 4. Check Directory Existence

```rust
// ✅ Good - ensure directory exists
let config_dir = proj_dirs.config_dir();
if !config_dir.exists() {
    fs::create_dir_all(config_dir)?;
}

// ❌ Bad - assume directory exists
fs::write(config_dir.join("file.txt"), data)?;  // May fail
```

### 5. Use Appropriate Directory Types

```rust
// Configuration files
let config = proj_dirs.config_dir();

// Application data (databases, user files)
let data = proj_dirs.data_dir();

// Temporary/cache files
let cache = proj_dirs.cache_dir();

// Don't mix purposes!
```

---

## Testing Cross-Platform Code

### Manual Testing

```bash
# Linux
cargo build --release
./target/release/myapp

# macOS (if you have access)
cargo build --release
./target/release/myapp

# Windows (via WSL or VM)
cargo build --release
./target/release/myapp.exe
```

### Automated Testing with CI

**GitHub Actions example:**

```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    
    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Build
        run: cargo build --verbose
      
      - name: Test
        run: cargo test --verbose
```

### Platform-Specific Tests

```rust
#[test]
#[cfg(target_os = "linux")]
fn test_linux_paths() {
    let proj_dirs = ProjectDirs::from("", "", "test").unwrap();
    let config = proj_dirs.config_dir();
    
    assert!(config.to_str().unwrap().contains(".config"));
}

#[test]
#[cfg(target_os = "macos")]
fn test_macos_paths() {
    let proj_dirs = ProjectDirs::from("", "", "test").unwrap();
    let config = proj_dirs.config_dir();
    
    assert!(config.to_str().unwrap().contains("Application Support"));
}

#[test]
#[cfg(target_os = "windows")]
fn test_windows_paths() {
    let proj_dirs = ProjectDirs::from("", "", "test").unwrap();
    let config = proj_dirs.config_dir();
    
    assert!(config.to_str().unwrap().contains("AppData"));
}
```

---

## Common Pitfalls

### 1. Hardcoded Path Separators

```rust
// ❌ Bad - assumes Unix separator
let path = format!("{}/{}", dir, file);

// ✅ Good - platform-aware
let path = PathBuf::from(dir).join(file);
```

### 2. Not Handling `None` from `ProjectDirs`

```rust
// ❌ Bad - panics on unsupported platform
let proj_dirs = ProjectDirs::from("", "", "myapp").unwrap();

// ✅ Good - proper error handling
let proj_dirs = ProjectDirs::from("", "", "myapp")
    .context("Unsupported platform or unable to determine home directory")?;
```

### 3. Assuming Home Directory Expansion

```rust
// ❌ Bad - ~ not expanded
let path = PathBuf::from("~/.config/myapp");

// ✅ Good - use directories crate
let home = BaseDirs::new().unwrap().home_dir().to_path_buf();
let path = home.join(".config").join("myapp");

// ✅ Better - use standard config dir
let proj_dirs = ProjectDirs::from("", "", "myapp").unwrap();
let path = proj_dirs.config_dir();
```

### 4. Not Creating Parent Directories

```rust
// ❌ Bad - fails if parent doesn't exist
fs::create_dir("/home/user/deep/nested/dir")?;

// ✅ Good - creates all parents
fs::create_dir_all("/home/user/deep/nested/dir")?;
```

---

## Resources

- [directories crate documentation](https://docs.rs/directories/)
- [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)
- [Rust std::path documentation](https://doc.rust-lang.org/std/path/)
- [Rust std::fs documentation](https://doc.rust-lang.org/std/fs/)

---

**📚 See Also:**

- [File Operations](file-operations.md) - Basic file I/O patterns
- [Global Data Directory](../advanced/v1.8.0-global-data-directory.md) - Complete implementation
- [Type Safety](type-safety.md) - Using PathBuf correctly
