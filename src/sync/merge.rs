//! Semantic 3-way merge for task lists.
//!
//! Merges two task lists using UUID as the stable identity key and
//! `updated_at` as the conflict-resolution tiebreaker (last-write-wins).
//!
//! # Algorithm (Phase 2)
//!
//! Given a `local` list and a `remote` list:
//!
//! 1. Build a map `uuid → task` for each side.
//! 2. For each UUID present in either side:
//!    - Only in local  → keep (task was added locally or deleted remotely; we keep it)
//!    - Only in remote → add to local (task was added remotely)
//!    - In both        → pick the version with the most recent `updated_at`
//! 3. Return the merged list.
//!
//! Deletion is **not** handled in Phase 1 — a task deleted on one side but
//! present on the other will be re-added. Soft-delete support is deferred.
//!
//! # Current status
//!
//! This module is a placeholder. The merge logic will be implemented in
//! Phase 2 alongside `todo sync pull`.

use crate::models::Task;

/// Result of a merge operation.
#[derive(Debug, Default)]
pub struct MergeResult {
    /// Final merged task list.
    pub tasks: Vec<Task>,
    /// Number of tasks added from remote.
    pub added: usize,
    /// Number of tasks updated (remote version was newer).
    pub updated: usize,
    /// Number of tasks kept unchanged (local version was newer or equal).
    pub kept: usize,
}

/// Merges `remote` into `local` using UUID + `updated_at` (last-write-wins).
///
/// # Current behaviour (Phase 1)
///
/// Returns `local` unchanged as a placeholder. Full implementation in Phase 2.
pub fn merge(local: Vec<Task>, _remote: Vec<Task>) -> MergeResult {
    let kept = local.len();
    MergeResult {
        tasks: local,
        added: 0,
        updated: 0,
        kept,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Priority, Task};

    fn task(text: &str) -> Task {
        Task::new(text.to_string(), Priority::Medium, vec![], None, None, None)
    }

    #[test]
    fn test_merge_placeholder_returns_local() {
        let local = vec![task("A"), task("B")];
        let remote = vec![task("C")];

        let result = merge(local, remote);

        assert_eq!(result.tasks.len(), 2);
        assert_eq!(result.tasks[0].text, "A");
        assert_eq!(result.kept, 2);
        assert_eq!(result.added, 0);
        assert_eq!(result.updated, 0);
    }

    #[test]
    fn test_merge_empty_local() {
        let result = merge(vec![], vec![task("remote-only")]);
        assert_eq!(result.tasks.len(), 0);
        assert_eq!(result.kept, 0);
    }
}
