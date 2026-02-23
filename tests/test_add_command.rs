//! Integration tests for add command

mod helpers;

use helpers::{TestEnv, days_from_now};
use rustodo::commands::add;
use rustodo::models::{Priority, Recurrence};

#[test]
fn test_add_simple_task() {
    let env = TestEnv::new();

    let result = add::execute(
        env.storage(),
        "Buy milk".to_string(),
        Priority::Medium,
        vec![],
        None,
        None,
        None,
        vec![],
    );

    assert!(result.is_ok());
    assert_eq!(env.task_count(), 1);

    let tasks = env.load_tasks();
    assert_eq!(tasks[0].text, "Buy milk");
    assert_eq!(tasks[0].priority, Priority::Medium);
    assert!(!tasks[0].completed);
    assert!(tasks[0].tags.is_empty());
    assert!(tasks[0].due_date.is_none());
    assert!(tasks[0].recurrence.is_none());
}

#[test]
fn test_add_task_with_all_metadata() {
    let env = TestEnv::new();

    let due_date = days_from_now(7);

    let result = add::execute(
        env.storage(),
        "Complete project".to_string(),
        Priority::High,
        vec!["work".to_string(), "urgent".to_string()],
        None,
        Some(due_date),
        Some(Recurrence::Weekly),
        vec![],
    );

    assert!(result.is_ok());

    let tasks = env.load_tasks();
    assert_eq!(tasks.len(), 1);

    let task = &tasks[0];
    assert_eq!(task.text, "Complete project");
    assert_eq!(task.priority, Priority::High);
    assert_eq!(task.tags, vec!["work", "urgent"]);
    assert_eq!(task.due_date, Some(due_date));
    assert_eq!(task.recurrence, Some(Recurrence::Weekly));
}

#[test]
fn test_add_multiple_tasks_preserves_order() {
    let env = TestEnv::new();

    add::execute(
        env.storage(),
        "Task 1".to_string(),
        Priority::Low,
        vec![],
        None,
        None,
        None,
        vec![],
    )
    .unwrap();
    add::execute(
        env.storage(),
        "Task 2".to_string(),
        Priority::Medium,
        vec![],
        None,
        None,
        None,
        vec![],
    )
    .unwrap();
    add::execute(
        env.storage(),
        "Task 3".to_string(),
        Priority::High,
        vec![],
        None,
        None,
        None,
        vec![],
    )
    .unwrap();

    let tasks = env.load_tasks();
    assert_eq!(tasks.len(), 3);
    assert_eq!(tasks[0].text, "Task 1");
    assert_eq!(tasks[1].text, "Task 2");
    assert_eq!(tasks[2].text, "Task 3");
}

#[test]
fn test_add_recurring_task_requires_due_date() {
    let env = TestEnv::new();

    let result = add::execute(
        env.storage(),
        "Daily standup".to_string(),
        Priority::Medium,
        vec![],
        None,
        None, // No due date
        Some(Recurrence::Daily),
        vec![],
    );

    // Should fail validation
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("must have a due date"));
}

#[test]
fn test_add_empty_text_fails() {
    let env = TestEnv::new();

    let result = add::execute(
        env.storage(),
        "".to_string(),
        Priority::Medium,
        vec![],
        None,
        None,
        None,
        vec![],
    );

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("cannot be empty"));
}

#[test]
fn test_add_whitespace_only_text_fails() {
    let env = TestEnv::new();

    let result = add::execute(
        env.storage(),
        "   \t\n  ".to_string(),
        Priority::Medium,
        vec![],
        None,
        None,
        None,
        vec![],
    );

    assert!(result.is_err());
}

#[test]
fn test_add_with_invalid_tags_fails() {
    let env = TestEnv::new();

    // Tag with spaces
    let result = add::execute(
        env.storage(),
        "Task".to_string(),
        Priority::Medium,
        vec!["invalid tag".to_string()],
        None,
        None,
        None,
        vec![],
    );

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Invalid tag format")
    );
}

#[test]
fn test_add_with_duplicate_tags_fails() {
    let env = TestEnv::new();

    let result = add::execute(
        env.storage(),
        "Task".to_string(),
        Priority::Medium,
        vec!["work".to_string(), "Work".to_string()], // Case-insensitive duplicate
        None,
        None,
        None,
        vec![],
    );

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Duplicate tag"));
}
