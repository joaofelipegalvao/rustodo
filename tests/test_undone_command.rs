//! Integration tests for the `undone` command
//!
//! Covers:
//! - Revert completed task to pending
//! - Cannot undone an already pending task
//! - Invalid ID
//! - completed_at is cleared on undone
//! - Multiple tasks: only target is reverted

mod helpers;

use helpers::TestEnv;
use rustodo::commands::{add, done, undone};
use rustodo::models::Priority;

// ─── helpers ─────────────────────────────────────────────────────────────────

fn add_simple(env: &TestEnv, text: &str) -> usize {
    add::execute(
        env.storage(),
        text.to_string(),
        Priority::Medium,
        vec![],
        None,
        None,
        None,
        vec![],
    )
    .unwrap();
    env.task_count()
}

// ─── happy path ───────────────────────────────────────────────────────────────

#[test]
fn test_undone_reverts_completed_task() {
    let env = TestEnv::new();
    add_simple(&env, "Task");

    done::execute(env.storage(), 1).unwrap();

    let result = undone::execute(env.storage(), 1);
    assert!(result.is_ok());

    let tasks = env.load_tasks();
    assert!(!tasks[0].completed);
}

#[test]
fn test_undone_clears_completed_at() {
    let env = TestEnv::new();
    add_simple(&env, "Task");

    done::execute(env.storage(), 1).unwrap();

    // Verify completed_at was set
    let tasks = env.load_tasks();
    assert!(tasks[0].completed_at.is_some());

    undone::execute(env.storage(), 1).unwrap();

    // Verify completed_at was cleared
    let tasks = env.load_tasks();
    assert!(tasks[0].completed_at.is_none());
}

#[test]
fn test_undone_allows_redone_after() {
    let env = TestEnv::new();
    add_simple(&env, "Task");

    done::execute(env.storage(), 1).unwrap();
    undone::execute(env.storage(), 1).unwrap();

    // Should be able to complete again
    let result = done::execute(env.storage(), 1);
    assert!(result.is_ok());

    let tasks = env.load_tasks();
    assert!(tasks[0].completed);
}

#[test]
fn test_undone_only_affects_target_task() {
    let env = TestEnv::new();
    add_simple(&env, "Task A");
    add_simple(&env, "Task B");
    add_simple(&env, "Task C");

    done::execute(env.storage(), 1).unwrap();
    done::execute(env.storage(), 2).unwrap();
    done::execute(env.storage(), 3).unwrap();

    undone::execute(env.storage(), 2).unwrap();

    let tasks = env.load_tasks();
    assert!(tasks[0].completed, "Task A should still be done");
    assert!(!tasks[1].completed, "Task B should be pending");
    assert!(tasks[2].completed, "Task C should still be done");
}

#[test]
fn test_undone_then_done_cycle() {
    let env = TestEnv::new();
    add_simple(&env, "Task");

    for _ in 0..3 {
        done::execute(env.storage(), 1).unwrap();
        undone::execute(env.storage(), 1).unwrap();
    }

    let tasks = env.load_tasks();
    assert!(!tasks[0].completed);
}

// ─── error cases ─────────────────────────────────────────────────────────────

#[test]
fn test_undone_already_pending_fails() {
    let env = TestEnv::new();
    add_simple(&env, "Task");

    // Never completed — should fail
    let result = undone::execute(env.storage(), 1);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already"));
}

#[test]
fn test_undone_id_zero_fails() {
    let env = TestEnv::new();
    add_simple(&env, "Task");

    let result = undone::execute(env.storage(), 0);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("invalid"));
}

#[test]
fn test_undone_id_out_of_range_fails() {
    let env = TestEnv::new();
    add_simple(&env, "Task");

    let result = undone::execute(env.storage(), 99);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("invalid"));
}

#[test]
fn test_undone_empty_storage_fails() {
    let env = TestEnv::new();

    let result = undone::execute(env.storage(), 1);
    assert!(result.is_err());
}
