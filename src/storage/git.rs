//! Git-based storage implementation with automatic commits.
//!
//! Tasks are stored in a Git repository as `todos.json`, with each save
//! creating a new commit. This enables:
//!
//! - Full version history of all task changes
//! - Sync across devices via push/pull
//! - 3-way merge for conflict resolution
//!
//! The repository structure:
//! ```text
//! repo/
//!   ├── .git/
//!   └── todos.json  ← Task list (JSON)
//! ```

use anyhow::{Context, Result};
use git2::{Repository, Signature};
use std::path::{Path, PathBuf};
use std::{fs, io::Write};

use super::Storage;
use crate::models::Task;
use uuid::Uuid;

/// Git-based storage with automatic commits.
///
/// Each `save()` creates a new commit with the message "Update tasks".
/// This provides full version history and enables sync workflows.
///
/// # Examples
///
/// ```no_run
/// use rustodo::storage::GitStorage;
///
/// // Open existing repo
/// let storage = GitStorage::new("/path/to/repo")?;
///
/// // Or initialize a new one
/// let storage = GitStorage::init("/path/to/new/repo")?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub struct GitStorage {
    repo: Repository,
    json_path: PathBuf,
}

impl GitStorage {
    /// Opens an existing Git repository.
    ///
    /// # Arguments
    ///
    /// * `repo_path` - Path to the repository root (contains `.git/`)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path doesn't exist
    /// - The path is not a valid Git repository
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rustodo::storage::GitStorage;
    ///
    /// let storage = GitStorage::new("/home/user/todos")?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn new<P: AsRef<Path>>(repo_path: P) -> Result<Self> {
        let repo_path = repo_path.as_ref();
        let repo = Repository::open(repo_path).context(format!(
            "Failed to open Git repository at {}",
            repo_path.display()
        ))?;

        let json_path = repo_path.join("todos.json");

        Ok(Self { repo, json_path })
    }

    /// Initializes a new Git repository and creates initial commit.
    ///
    /// # Arguments
    ///
    /// * `repo_path` - Path where the repository will be created
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The directory cannot be created
    /// - Git initialization fails
    /// - Initial commit fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rustodo::storage::GitStorage;
    ///
    /// let storage = GitStorage::init("/home/user/todos")?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn init<P: AsRef<Path>>(repo_path: P) -> Result<Self> {
        let repo_path = repo_path.as_ref();

        // Create directory if it doesn't exist
        fs::create_dir_all(repo_path).context(format!(
            "Failed to create directory at {}",
            repo_path.display()
        ))?;

        // Initialize Git repository
        let repo = Repository::init(repo_path).context(format!(
            "Failed to initialize Git repository at {}",
            repo_path.display()
        ))?;

        let json_path = repo_path.join("todos.json");

        // Create initial empty todos.json
        fs::write(&json_path, "[]").context("Failed to create initial todos.json")?;

        // Create initial commit
        let storage = Self { repo, json_path };
        storage.commit("Initial commit")?;

        Ok(storage)
    }

    /// Creates a commit with the given message.
    ///
    /// Stages `todos.json` and commits it with the current timestamp.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Git operations fail
    /// - No changes to commit (returns Ok silently)
    fn commit(&self, message: &str) -> Result<()> {
        let mut index = self.repo.index()?;

        // Add todos.json to index
        index.add_path(Path::new("todos.json"))?;
        index.write()?;

        // Check if there are changes to commit
        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;

        let head = match self.repo.head() {
            Ok(head) => Some(head),
            Err(e) if e.code() == git2::ErrorCode::UnbornBranch => None,
            Err(e) => return Err(e.into()),
        };

        // Get parent commit if exists
        let parent = match &head {
            Some(h) => {
                let commit = h.peel_to_commit()?;
                Some(commit)
            }
            None => None,
        };

        // Create signature
        let signature = Signature::now("rustodo", "rustodo@local")?;

        // Create commit
        let parent_refs: Vec<&git2::Commit> = parent.as_ref().map(|p| vec![p]).unwrap_or_default();

        self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parent_refs,
        )?;

        Ok(())
    }

    /// Returns the path to the repository root.
    pub fn repo_path(&self) -> &Path {
        self.repo.path().parent().unwrap()
    }
}

impl Storage for GitStorage {
    fn load(&self) -> Result<Vec<Task>> {
        // Load from todos.json (same as JsonStorage)
        let mut tasks: Vec<Task> = match fs::read_to_string(&self.json_path) {
            Ok(content) => serde_json::from_str(&content)
                .context("Failed to parse todos.json - file may be corrupted")?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // File doesn't exist yet — create empty list
                Vec::new()
            }
            Err(e) => {
                return Err(e).context(format!(
                    "Failed to read todos.json from: {}",
                    self.json_path.display()
                ));
            }
        };

        // Migrate UUIDs if needed (same as JsonStorage)
        let mut modified = false;
        for task in &mut tasks {
            if task.uuid.is_nil() {
                task.uuid = Uuid::new_v4();
                modified = true;
            }
        }

        if modified {
            self.save(&tasks)?;
        }

        Ok(tasks)
    }

    fn save(&self, tasks: &[Task]) -> Result<()> {
        let json =
            serde_json::to_string_pretty(tasks).context("Failed to serialize tasks to JSON")?;

        // Write to file
        let mut file = fs::File::create(&self.json_path).context(format!(
            "Failed to create {} - check file permissions",
            self.json_path.display()
        ))?;

        file.write_all(json.as_bytes())
            .context(format!("Failed to write to {}", self.json_path.display()))?;

        // Commit changes
        self.commit("Update tasks")?;

        Ok(())
    }

    fn location(&self) -> String {
        format!("git:{}", self.repo_path().display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Priority;
    use tempfile::TempDir;

    #[test]
    fn test_git_storage_init() {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path();

        let storage = GitStorage::init(repo_path).unwrap();

        // Verify repo was created
        assert!(repo_path.join(".git").exists());
        assert!(repo_path.join("todos.json").exists());

        // Verify we can load (should be empty)
        let tasks = storage.load().unwrap();
        assert_eq!(tasks.len(), 0);
    }

    #[test]
    fn test_git_storage_new_opens_existing() {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path();

        // Init first
        GitStorage::init(repo_path).unwrap();

        // Open existing
        let storage = GitStorage::new(repo_path).unwrap();
        let tasks = storage.load().unwrap();
        assert_eq!(tasks.len(), 0);
    }

    #[test]
    fn test_git_storage_save_creates_commit() {
        let temp = TempDir::new().unwrap();
        let storage = GitStorage::init(temp.path()).unwrap();

        // Initial commit exists (from init)
        let repo = &storage.repo;
        let mut rw = repo.revwalk().unwrap();
        rw.push_head().unwrap();
        let initial_commits = rw.count();

        // Save task
        let tasks = vec![Task::new(
            "Test task".to_string(),
            Priority::Medium,
            vec![],
            None,
            None,
            None,
        )];
        storage.save(&tasks).unwrap();

        // Should have created one more commit
        let mut rw = repo.revwalk().unwrap();
        rw.push_head().unwrap();
        let final_commits = rw.count();
        assert_eq!(final_commits, initial_commits + 1);
    }

    #[test]
    fn test_git_storage_multiple_saves_create_history() {
        let temp = TempDir::new().unwrap();
        let storage = GitStorage::init(temp.path()).unwrap();

        // Save 3 times
        for i in 1..=3 {
            let tasks = vec![Task::new(
                format!("Task {}", i),
                Priority::Medium,
                vec![],
                None,
                None,
                None,
            )];
            storage.save(&tasks).unwrap();
        }

        // Should have 4 commits total (initial + 3 saves)
        let repo = &storage.repo;
        let mut rw = repo.revwalk().unwrap();
        rw.push_head().unwrap();
        let commits = rw.count();
        assert_eq!(commits, 4);
    }

    #[test]
    fn test_git_storage_uuid_migration() {
        let temp = TempDir::new().unwrap();
        let storage = GitStorage::init(temp.path()).unwrap();

        // Simulate old JSON without UUIDs
        let old_json = r#"[
            {
                "text": "Old task",
                "completed": false,
                "priority": "medium",
                "tags": [],
                "created_at": "2025-01-01",
                "depends_on": []
            }
        ]"#;
        fs::write(&storage.json_path, old_json).unwrap();

        // Load should migrate
        let tasks = storage.load().unwrap();
        assert_eq!(tasks.len(), 1);
        assert!(!tasks[0].uuid.is_nil());

        // UUID should be stable
        let first_uuid = tasks[0].uuid;
        let tasks2 = storage.load().unwrap();
        assert_eq!(tasks2[0].uuid, first_uuid);
    }

    #[test]
    fn test_git_storage_location() {
        let temp = TempDir::new().unwrap();
        let storage = GitStorage::init(temp.path()).unwrap();

        let location = storage.location();
        assert!(location.starts_with("git:"));
        assert!(location.contains("tmp")); // TempDir is in /tmp
    }

    #[test]
    fn test_git_storage_save_and_load_preserves_data() {
        let temp = TempDir::new().unwrap();
        let storage = GitStorage::init(temp.path()).unwrap();

        // Save complex task
        let tasks = vec![Task::new(
            "Complex task".to_string(),
            Priority::High,
            vec!["work".to_string(), "urgent".to_string()],
            Some("Backend".to_string()),
            None,
            None,
        )];
        storage.save(&tasks).unwrap();

        // Load and verify
        let loaded = storage.load().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].text, "Complex task");
        assert_eq!(loaded[0].priority, Priority::High);
        assert_eq!(loaded[0].tags, vec!["work", "urgent"]);
        assert_eq!(loaded[0].project.as_deref(), Some("Backend"));
    }
}
