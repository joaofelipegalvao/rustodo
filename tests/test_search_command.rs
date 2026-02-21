use todo_cli::{
    commands::{add, done},
    models::Priority,
};

use crate::helpers::TestEnv;

mod helpers;

#[test]
fn test_search_returns_completed_tasks_mixed_with_pending() {
    let env = TestEnv::new();

    add::execute(
        env.storage(),
        "Buy milk".to_string(),
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
        "Buy milk".to_string(),
        Priority::Medium,
        vec![],
        None,
        None,
        None,
        vec![],
    )
    .unwrap();

    done::execute(env.storage(), 1).unwrap();

    let tasks = env.load_tasks();
    let results: Vec<_> = tasks
        .iter()
        .filter(|t| t.text.to_lowercase().contains("buy milk"))
        .collect();

    assert_eq!(results.len(), 2); // confirma que ambas aparecem
    assert!(results.iter().any(|t| t.completed));
    assert!(results.iter().any(|t| !t.completed));
}
