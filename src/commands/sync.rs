//! Handler for `todo sync`.
//!
//! Performs a full sync workflow with Git repository:
//! 1. Pull latest changes from remote (if configured)
//! 2. Merge changes using 3-way merge algorithm
//! 3. Resolve conflicts automatically (last-write-wins via `updated_at`)
//! 4. Push merged result to remote (if configured)
//!
//! Requires [`GitStorage`](crate::storage::GitStorage) backend.

use anyhow::{Result, bail};
use colored::Colorize;

use crate::storage::Storage;

/// Execute sync command.
///
/// # Phase 1 (Current): Basic validation
///
/// Validates that the storage backend supports sync operations.
/// Currently only [`GitStorage`](crate::storage::GitStorage) is supported.
///
/// # Phase 2 (Future): Full sync workflow
///
/// Will implement:
/// - Pull from remote
/// - 3-way merge with UUID-based conflict resolution
/// - Push to remote
/// - Detailed sync summary
///
/// # Errors
///
/// Returns an error if:
/// - Storage backend does not support sync (not GitStorage)
/// - Git operations fail (no remote, authentication issues, etc.)
/// - Merge conflicts cannot be resolved automatically
pub fn execute(_storage: &impl Storage) -> Result<()> {
    // Phase 1: Basic structure
    // This will be expanded in Phase 2 with actual sync logic

    println!("{}", "Sync functionality is under development".yellow());
    println!();
    println!("{}", "Planned workflow:".dimmed());
    println!("  {} Fetch latest changes from remote", "1.".dimmed());
    println!("  {} Perform 3-way merge using UUIDs", "2.".dimmed());
    println!("  {} Resolve conflicts (last-write-wins)", "3.".dimmed());
    println!("  {} Push merged result to remote", "4.".dimmed());
    println!();

    bail!("Sync is not yet implemented. Stay tuned for v3.0.0!")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::memory::InMemoryStorage;

    #[test]
    fn test_sync_not_yet_implemented() {
        let storage = InMemoryStorage::default();
        let result = execute(&storage);

        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("not yet implemented"));
    }
}
