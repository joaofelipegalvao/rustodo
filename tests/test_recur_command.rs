//! Integration tests for `recur`, `clear_recur`, and recurrence flows in `done`
//!
//! Covers:
//! - Set recurrence on task with due date
//! - Set recurrence without due date (fails)
//! - Change recurrence pattern
//! - Remove recurrence via clear_recur
//! - Clear recurrence on task with no recurrence (no-op)
//! - done on recurring task creates next occurrence
//! - done on recurring task does not duplicate if next already exists
//! - Next occurrence preserves text, priority, tags, project
//! - Next occurrence has correct due date (daily/weekly/monthly)
//! - parent_id is set correctly on next occurrence

mod helpers;

use helpers::{TestEnv, days_from_now};
use rustodo::cli::AddArgs;
use rustodo::commands::{add, clear_recur, done, recur};
use rustodo::models::{Priority, Recurrence};

// ─── helpers ─────────────────────────────────────────────────────────────────

fn add_with_due(env: &TestEnv, text: &str, days: i64) -> usize {
    add::execute(
        env.storage(),
        AddArgs {
            text: text.to_string(),
            priority: Priority::Medium,
            tag: vec![],
            project: None,
            due: Some(days_from_now(days).to_string()),
            recurrence: None,
            depends_on: vec![],
        },
    )
    .unwrap();
    env.task_count()
}

fn add_recurring(env: &TestEnv, text: &str, days: i64, pattern: Recurrence) -> usize {
    add::execute(
        env.storage(),
        AddArgs {
            text: text.to_string(),
            priority: Priority::Medium,
            tag: vec![],
            project: None,
            due: Some(days_from_now(days).to_string()),
            recurrence: Some(pattern),
            depends_on: vec![],
        },
    )
    .unwrap();
    env.task_count()
}

fn add_simple(env: &TestEnv, text: &str) -> usize {
    add::execute(
        env.storage(),
        AddArgs {
            text: text.to_string(),
            priority: Priority::Medium,
            tag: vec![],
            project: None,
            due: None,
            recurrence: None,
            depends_on: vec![],
        },
    )
    .unwrap();
    env.task_count()
}

// ─── recur: set recurrence ────────────────────────────────────────────────────

#[test]
fn test_recur_set_daily_on_task_with_due_date() {
    let env = TestEnv::new();
    add_with_due(&env, "Daily standup", 1);

    let result = recur::execute(env.storage(), 1, Recurrence::Daily);
    assert!(result.is_ok());

    let tasks = env.load_tasks();
    assert_eq!(tasks[0].recurrence, Some(Recurrence::Daily));
}

#[test]
fn test_recur_set_weekly() {
    let env = TestEnv::new();
    add_with_due(&env, "Weekly review", 3);

    recur::execute(env.storage(), 1, Recurrence::Weekly).unwrap();

    let tasks = env.load_tasks();
    assert_eq!(tasks[0].recurrence, Some(Recurrence::Weekly));
}

#[test]
fn test_recur_set_monthly() {
    let env = TestEnv::new();
    add_with_due(&env, "Monthly report", 7);

    recur::execute(env.storage(), 1, Recurrence::Monthly).unwrap();

    let tasks = env.load_tasks();
    assert_eq!(tasks[0].recurrence, Some(Recurrence::Monthly));
}

#[test]
fn test_recur_change_pattern() {
    let env = TestEnv::new();
    add_with_due(&env, "Task", 1);
    recur::execute(env.storage(), 1, Recurrence::Daily).unwrap();

    // Change from daily to weekly
    let result = recur::execute(env.storage(), 1, Recurrence::Weekly);
    assert!(result.is_ok());

    let tasks = env.load_tasks();
    assert_eq!(tasks[0].recurrence, Some(Recurrence::Weekly));
}

#[test]
fn test_recur_same_pattern_is_ok() {
    let env = TestEnv::new();
    add_with_due(&env, "Task", 1);
    recur::execute(env.storage(), 1, Recurrence::Daily).unwrap();

    // Setting same pattern should not error
    let result = recur::execute(env.storage(), 1, Recurrence::Daily);
    assert!(result.is_ok());

    let tasks = env.load_tasks();
    assert_eq!(tasks[0].recurrence, Some(Recurrence::Daily));
}

#[test]
fn test_recur_without_due_date_fails() {
    let env = TestEnv::new();
    add_simple(&env, "Task without due date");

    let result = recur::execute(env.storage(), 1, Recurrence::Daily);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("due date"), "got: {}", msg);
}

#[test]
fn test_recur_invalid_id_fails() {
    let env = TestEnv::new();
    add_with_due(&env, "Task", 1);

    assert!(recur::execute(env.storage(), 0, Recurrence::Daily).is_err());
    assert!(recur::execute(env.storage(), 99, Recurrence::Daily).is_err());
}

// ─── clear_recur ─────────────────────────────────────────────────────────────

#[test]
fn test_clear_recur_removes_recurrence() {
    let env = TestEnv::new();
    add_recurring(&env, "Daily task", 1, Recurrence::Daily);

    let result = clear_recur::execute(env.storage(), 1);
    assert!(result.is_ok());

    let tasks = env.load_tasks();
    assert!(tasks[0].recurrence.is_none());
}

#[test]
fn test_clear_recur_on_non_recurring_task_is_ok() {
    let env = TestEnv::new();
    add_with_due(&env, "Non-recurring task", 1);

    // Should not error — just a no-op
    let result = clear_recur::execute(env.storage(), 1);
    assert!(result.is_ok());

    let tasks = env.load_tasks();
    assert!(tasks[0].recurrence.is_none());
}

#[test]
fn test_clear_recur_invalid_id_fails() {
    let env = TestEnv::new();
    add_with_due(&env, "Task", 1);

    assert!(clear_recur::execute(env.storage(), 0).is_err());
    assert!(clear_recur::execute(env.storage(), 99).is_err());
}

#[test]
fn test_clear_recur_preserves_due_date() {
    let env = TestEnv::new();
    let due = days_from_now(5);
    add::execute(
        env.storage(),
        AddArgs {
            text: "Task".to_string(),
            priority: Priority::Medium,
            tag: vec![],
            project: None,
            due: Some(due.to_string()),
            recurrence: Some(Recurrence::Weekly),
            depends_on: vec![],
        },
    )
    .unwrap();

    clear_recur::execute(env.storage(), 1).unwrap();

    let tasks = env.load_tasks();
    assert!(tasks[0].recurrence.is_none());
    assert_eq!(tasks[0].due_date, Some(due));
}

// ─── done with recurrence: creates next occurrence ────────────────────────────

#[test]
fn test_done_daily_recurring_creates_next_occurrence() {
    let env = TestEnv::new();
    let due = days_from_now(1);
    add::execute(
        env.storage(),
        AddArgs {
            text: "Daily standup".to_string(),
            priority: Priority::Medium,
            tag: vec![],
            project: None,
            due: Some(due.to_string()),
            recurrence: Some(Recurrence::Daily),
            depends_on: vec![],
        },
    )
    .unwrap();

    done::execute(env.storage(), 1).unwrap();

    let tasks = env.load_tasks();
    assert_eq!(tasks.len(), 2, "should have created next occurrence");
    assert!(tasks[0].completed);
    assert!(!tasks[1].completed);
    assert_eq!(tasks[1].due_date, Some(due + chrono::Duration::days(1)));
}

#[test]
fn test_done_weekly_recurring_creates_next_occurrence() {
    let env = TestEnv::new();
    let due = days_from_now(3);
    add::execute(
        env.storage(),
        AddArgs {
            text: "Weekly review".to_string(),
            priority: Priority::Medium,
            tag: vec![],
            project: None,
            due: Some(due.to_string()),
            recurrence: Some(Recurrence::Weekly),
            depends_on: vec![],
        },
    )
    .unwrap();

    done::execute(env.storage(), 1).unwrap();

    let tasks = env.load_tasks();
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[1].due_date, Some(due + chrono::Duration::days(7)));
}

#[test]
fn test_done_monthly_recurring_creates_next_occurrence() {
    let env = TestEnv::new();
    use chrono::NaiveDate;

    // Use a fixed date to avoid month boundary issues
    let due = NaiveDate::from_ymd_opt(2030, 6, 15).unwrap();
    add::execute(
        env.storage(),
        AddArgs {
            text: "Monthly report".to_string(),
            priority: Priority::Medium,
            tag: vec![],
            project: None,
            due: Some(due.to_string()),
            recurrence: Some(Recurrence::Monthly),
            depends_on: vec![],
        },
    )
    .unwrap();

    done::execute(env.storage(), 1).unwrap();

    let tasks = env.load_tasks();
    assert_eq!(tasks.len(), 2);
    assert_eq!(
        tasks[1].due_date,
        Some(NaiveDate::from_ymd_opt(2030, 7, 15).unwrap())
    );
}

#[test]
fn test_done_recurring_preserves_metadata() {
    let env = TestEnv::new();
    let due = days_from_now(1);
    add::execute(
        env.storage(),
        AddArgs {
            text: "Tagged task".to_string(),
            priority: Priority::High,
            tag: vec!["work".to_string(), "urgent".to_string()],
            project: Some("Backend".to_string()),
            due: Some(due.to_string()),
            recurrence: Some(Recurrence::Daily),
            depends_on: vec![],
        },
    )
    .unwrap();

    done::execute(env.storage(), 1).unwrap();

    let tasks = env.load_tasks();
    let next = &tasks[1];
    assert_eq!(next.text, "Tagged task");
    assert_eq!(next.priority, Priority::High);
    assert_eq!(next.tags, vec!["work", "urgent"]);
    assert_eq!(next.project.as_deref(), Some("Backend"));
    assert_eq!(next.recurrence, Some(Recurrence::Daily));
    assert!(!next.completed);
}

#[test]
fn test_done_recurring_sets_parent_id() {
    let env = TestEnv::new();
    add::execute(
        env.storage(),
        AddArgs {
            text: "Task".to_string(),
            priority: Priority::Medium,
            tag: vec![],
            project: None,
            due: Some(days_from_now(1).to_string()),
            recurrence: Some(Recurrence::Daily),
            depends_on: vec![],
        },
    )
    .unwrap();

    done::execute(env.storage(), 1).unwrap();

    let tasks = env.load_tasks();
    let parent_uuid = tasks[0].uuid;
    assert_eq!(tasks[1].parent_id, Some(parent_uuid));
}

#[test]
fn test_done_recurring_does_not_duplicate_if_next_exists() {
    let env = TestEnv::new();
    let due = days_from_now(1);
    add::execute(
        env.storage(),
        AddArgs {
            text: "Task".to_string(),
            priority: Priority::Medium,
            tag: vec![],
            project: None,
            due: Some(due.to_string()),
            recurrence: Some(Recurrence::Daily),
            depends_on: vec![],
        },
    )
    .unwrap();

    // First done creates next occurrence (task 2)
    done::execute(env.storage(), 1).unwrap();
    assert_eq!(env.task_count(), 2);

    // Undone and redo — should not create duplicate
    rustodo::commands::undone::execute(env.storage(), 1).unwrap();
    done::execute(env.storage(), 1).unwrap();

    // Still 2 tasks, not 3
    assert_eq!(env.task_count(), 2);
}

#[test]
fn test_done_non_recurring_does_not_create_next() {
    let env = TestEnv::new();
    add_simple(&env, "Non-recurring task");

    done::execute(env.storage(), 1).unwrap();

    assert_eq!(env.task_count(), 1, "should not create next occurrence");
}
