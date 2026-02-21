//! Integration tests for edit command

mod helpers;

use helpers::{TestEnv, days_from_now};
use todo_cli::commands::{add, edit};
use todo_cli::models::Priority;

#[test]
fn test_edit_text() {
    let env = TestEnv::new();

    // Setup: Create task
    add::execute(
        env.storage(),
        "Old text".to_string(),
        Priority::Medium,
        vec![],
        None,
        None,
        None,
        vec![],
    )
    .unwrap();

    // Execute: Edit text
    let result = edit::execute(
        env.storage(),
        1, // ID
        Some("New text".to_string()),
        None,   // priority
        vec![], // add_tag
        vec![], // remove_tag
        None,   // due
        false,
        None,
        false, // clear_due
        false, // clear_tags
        vec![],
        vec![],
        false,
    );

    assert!(result.is_ok());

    // Verify
    let tasks = env.load_tasks();
    assert_eq!(tasks[0].text, "New text");
}

#[test]
fn test_edit_priority() {
    let env = TestEnv::new();

    add::execute(
        env.storage(),
        "Task".to_string(),
        Priority::Low,
        vec![],
        None,
        None,
        None,
        vec![],
    )
    .unwrap();

    let result = edit::execute(
        env.storage(),
        1,
        None,
        Some(Priority::High), // Change to High
        vec![],
        vec![],
        None,
        false,
        None,
        false,
        false,
        vec![],
        vec![],
        false,
    );

    assert!(result.is_ok());

    let tasks = env.load_tasks();
    assert_eq!(tasks[0].priority, Priority::High);
}

#[test]
fn test_edit_add_invalid_tag_fails() {
    let env = TestEnv::new();

    add::execute(
        env.storage(),
        "Task".to_string(),
        Priority::Medium,
        vec![],
        None,
        None,
        None,
        vec![],
    )
    .unwrap();

    let result = edit::execute(
        env.storage(),
        1,
        None,
        None,
        vec!["invalid tag".to_string()],
        vec![],
        None,
        false,
        None,
        false,
        false,
        vec![],
        vec![],
        false,
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
fn test_edit_add_tags_preserves_existing() {
    let env = TestEnv::new();

    // Setup: Task with one tag
    add::execute(
        env.storage(),
        "Task".to_string(),
        Priority::Medium,
        vec!["work".to_string()],
        None,
        None,
        None,
        vec![],
    )
    .unwrap();

    // Execute: Add another tag
    let result = edit::execute(
        env.storage(),
        1,
        None,
        None,
        vec!["urgent".to_string()], // Add tag
        vec![],
        None,
        false,
        None,
        false,
        false,
        vec![],
        vec![],
        false,
    );

    assert!(result.is_ok());

    // Verify: Should have BOTH tags
    let tasks = env.load_tasks();
    assert_eq!(tasks[0].tags.len(), 2);
    assert!(tasks[0].tags.contains(&"work".to_string()));
    assert!(tasks[0].tags.contains(&"urgent".to_string()));
}

#[test]
fn test_edit_remove_specific_tag() {
    let env = TestEnv::new();

    // Setup: Task with multiple tags
    add::execute(
        env.storage(),
        "Task".to_string(),
        Priority::Medium,
        vec![
            "work".to_string(),
            "urgent".to_string(),
            "frontend".to_string(),
        ],
        None,
        None,
        None,
        vec![],
    )
    .unwrap();

    // Execute: Remove one tag
    let result = edit::execute(
        env.storage(),
        1,
        None,
        None,
        vec![],
        vec!["urgent".to_string()], // Remove only this
        None,
        false,
        None,
        false,
        false,
        vec![],
        vec![],
        false,
    );

    assert!(result.is_ok());

    // Verify: Should have work and frontend, NOT urgent
    let tasks = env.load_tasks();
    assert_eq!(tasks[0].tags.len(), 2);
    assert!(tasks[0].tags.contains(&"work".to_string()));
    assert!(tasks[0].tags.contains(&"frontend".to_string()));
    assert!(!tasks[0].tags.contains(&"urgent".to_string()));
}

#[test]
fn test_edit_add_and_remove_tags_simultaneously() {
    let env = TestEnv::new();

    // Setup
    add::execute(
        env.storage(),
        "Task".to_string(),
        Priority::Medium,
        vec!["work".to_string(), "old".to_string()],
        None,
        None,
        None,
        vec![],
    )
    .unwrap();

    // Execute: Remove 'old', add 'new'
    let result = edit::execute(
        env.storage(),
        1,
        None,
        None,
        vec!["new".to_string()], // Add
        vec!["old".to_string()], // Remove
        None,
        false,
        None,
        false,
        false,
        vec![],
        vec![],
        false,
    );

    assert!(result.is_ok());

    // Verify
    let tasks = env.load_tasks();
    assert_eq!(tasks[0].tags.len(), 2);
    assert!(tasks[0].tags.contains(&"work".to_string()));
    assert!(tasks[0].tags.contains(&"new".to_string()));
    assert!(!tasks[0].tags.contains(&"old".to_string()));
}

#[test]
fn test_edit_clear_all_tags() {
    let env = TestEnv::new();

    // Setup
    add::execute(
        env.storage(),
        "Task".to_string(),
        Priority::Medium,
        vec!["work".to_string(), "urgent".to_string()],
        None,
        None,
        None,
        vec![],
    )
    .unwrap();

    // Execute: Clear all tags
    let result = edit::execute(
        env.storage(),
        1,
        None,
        None,
        vec![],
        vec![],
        None,
        false,
        None,
        false,
        true, // clear_tags
        vec![],
        vec![],
        false,
    );

    assert!(result.is_ok());

    // Verify
    let tasks = env.load_tasks();
    assert!(tasks[0].tags.is_empty());
}

#[test]
fn test_edit_remove_nonexistent_tag_fails() {
    let env = TestEnv::new();

    // Setup: Task with only 'work' tag
    add::execute(
        env.storage(),
        "Task".to_string(),
        Priority::Medium,
        vec!["work".to_string()],
        None,
        None,
        None,
        vec![],
    )
    .unwrap();

    // Execute: Try to remove tag that doesn't exist
    let result = edit::execute(
        env.storage(),
        1,
        None,
        None,
        vec![],
        vec!["nonexistent".to_string()],
        None,
        false,
        None,
        false,
        false,
        vec![],
        vec![],
        false,
    );

    // Should fail
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("None of the specified tags")
    );
}

#[test]
fn test_edit_invalid_id() {
    let env = TestEnv::new();

    // No tasks exist
    let result = edit::execute(
        env.storage(),
        99,
        None,
        None,
        vec![],
        vec![],
        None,
        false,
        None,
        false,
        false,
        vec![],
        vec![],
        false,
    );

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("invalid"));
}

#[test]
fn test_edit_due_date() {
    let env = TestEnv::new();

    add::execute(
        env.storage(),
        "Task".to_string(),
        Priority::Medium,
        vec![],
        None,
        None,
        None,
        vec![],
    )
    .unwrap();

    let due_date = days_from_now(7);

    let result = edit::execute(
        env.storage(),
        1,
        None,
        None,
        vec![],
        vec![],
        None,
        false,
        Some(due_date), // Set due date
        false,
        false,
        vec![],
        vec![],
        false,
    );

    assert!(result.is_ok());

    let tasks = env.load_tasks();
    assert_eq!(tasks[0].due_date, Some(due_date));
}

#[test]
fn test_edit_clear_due_date() {
    let env = TestEnv::new();

    let due_date = days_from_now(7);
    add::execute(
        env.storage(),
        "Task".to_string(),
        Priority::Medium,
        vec![],
        None,
        Some(due_date),
        None,
        vec![],
    )
    .unwrap();

    let result = edit::execute(
        env.storage(),
        1,
        None,
        None,
        vec![],
        vec![],
        None,
        false,
        None,
        true, // clear_due
        false,
        vec![],
        vec![],
        false,
    );

    assert!(result.is_ok());

    let tasks = env.load_tasks();
    assert!(tasks[0].due_date.is_none());
}
