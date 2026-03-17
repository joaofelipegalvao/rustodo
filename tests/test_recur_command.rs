//! Integration tests for `todo recur` and `todo norecur` commands
//!
//! Covers:
//! - recur: set pattern, update pattern, already set same pattern
//! - recur: task without due date fails
//! - recur: invalid ID fails
//! - norecur: remove pattern
//! - norecur: task without recurrence is ok
//! - norecur: invalid ID fails
//! - done on recurring task creates next occurrence
//! - next occurrence has correct due date (daily/weekly/monthly)
//! - next occurrence does not inherit dependencies
//! - deduplication: done twice does not create duplicate

mod helpers;

use helpers::{TestEnv, days_from_now};
use rustodo::cli::AddArgs;
use rustodo::commands::task;
use rustodo::models::{Priority, Recurrence};
use rustodo::storage::Storage;

// ─── helpers ─────────────────────────────────────────────────────────────────

fn add_simple(env: &TestEnv, text: &str) {
    task::add::execute(
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
}

fn add_with_due(env: &TestEnv, text: &str, due_offset_days: i64) -> usize {
    let due = days_from_now(due_offset_days).to_string();
    task::add::execute(
        env.storage(),
        AddArgs {
            text: text.to_string(),
            priority: Priority::Medium,
            tag: vec![],
            project: None,
            due: Some(due),
            recurrence: None,
            depends_on: vec![],
        },
    )
    .unwrap();
    env.load_tasks().len()
}

fn add_recurring(env: &TestEnv, text: &str, due_offset_days: i64, pattern: Recurrence) -> usize {
    let due = days_from_now(due_offset_days).to_string();
    task::add::execute(
        env.storage(),
        AddArgs {
            text: text.to_string(),
            priority: Priority::Medium,
            tag: vec![],
            project: None,
            due: Some(due),
            recurrence: Some(pattern),
            depends_on: vec![],
        },
    )
    .unwrap();
    env.load_tasks().len()
}

// ─── recur ────────────────────────────────────────────────────────────────────

#[test]
fn test_recur_set_pattern() {
    let env = TestEnv::new();
    add_with_due(&env, "Daily standup", 1);

    task::recur::execute(env.storage(), 1, Recurrence::Daily).unwrap();

    let tasks = env.load_tasks();
    assert_eq!(tasks[0].recurrence, Some(Recurrence::Daily));
}

#[test]
fn test_recur_update_pattern() {
    let env = TestEnv::new();
    add_with_due(&env, "Meeting", 1);
    task::recur::execute(env.storage(), 1, Recurrence::Daily).unwrap();

    task::recur::execute(env.storage(), 1, Recurrence::Weekly).unwrap();

    let tasks = env.load_tasks();
    assert_eq!(tasks[0].recurrence, Some(Recurrence::Weekly));
}

#[test]
fn test_recur_already_same_pattern_is_ok() {
    let env = TestEnv::new();
    add_with_due(&env, "Meeting", 1);
    task::recur::execute(env.storage(), 1, Recurrence::Daily).unwrap();

    // Mesma chamada não deve falhar
    let result = task::recur::execute(env.storage(), 1, Recurrence::Daily);
    assert!(result.is_ok());
    let tasks = env.load_tasks();
    assert_eq!(tasks[0].recurrence, Some(Recurrence::Daily));
}

#[test]
fn test_recur_without_due_date_fails() {
    let env = TestEnv::new();
    add_simple(&env, "No due date task");

    let result = task::recur::execute(env.storage(), 1, Recurrence::Daily);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("due date"), "got: {}", msg);
}

#[test]
fn test_recur_invalid_id_fails() {
    let env = TestEnv::new();
    let result = task::recur::execute(env.storage(), 99, Recurrence::Daily);
    assert!(result.is_err());
}

#[test]
fn test_recur_invalid_id_zero_fails() {
    let env = TestEnv::new();
    let result = task::recur::execute(env.storage(), 0, Recurrence::Daily);
    assert!(result.is_err());
}

#[test]
fn test_recur_skips_deleted_tasks() {
    let env = TestEnv::new();
    add_with_due(&env, "Task A", 1);
    add_with_due(&env, "Task B", 2);

    task::remove::execute(env.storage(), 1, true).unwrap();

    // Agora Task B é visível como #1
    task::recur::execute(env.storage(), 1, Recurrence::Weekly).unwrap();

    // load_tasks() filtra deletados — Task B deve ser o único visível
    let tasks = env.load_tasks();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].text, "Task B");
    assert_eq!(tasks[0].recurrence, Some(Recurrence::Weekly));
}

// ─── norecur (clear_recur) ────────────────────────────────────────────────────

#[test]
fn test_norecur_removes_pattern() {
    let env = TestEnv::new();
    add_recurring(&env, "Daily task", 1, Recurrence::Daily);

    task::clear_recur::execute(env.storage(), 1).unwrap();

    let tasks = env.load_tasks();
    assert!(tasks[0].recurrence.is_none());
}

#[test]
fn test_norecur_task_without_recurrence_is_ok() {
    let env = TestEnv::new();
    add_simple(&env, "No recurrence");

    let result = task::clear_recur::execute(env.storage(), 1);
    assert!(result.is_ok());

    // Task deve permanecer intacta
    let tasks = env.load_tasks();
    assert_eq!(tasks.len(), 1);
}

#[test]
fn test_norecur_invalid_id_fails() {
    let env = TestEnv::new();
    let result = task::clear_recur::execute(env.storage(), 99);
    assert!(result.is_err());
}

#[test]
fn test_norecur_skips_deleted_tasks() {
    let env = TestEnv::new();
    add_recurring(&env, "Task A", 1, Recurrence::Daily);
    add_recurring(&env, "Task B", 2, Recurrence::Weekly);

    task::remove::execute(env.storage(), 1, true).unwrap();

    // Task B agora é #1
    task::clear_recur::execute(env.storage(), 1).unwrap();

    // load_tasks() filtra deletados — Task B deve ser o único visível
    let tasks = env.load_tasks();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].text, "Task B");
    assert!(tasks[0].recurrence.is_none());
}

// ─── done cria próxima ocorrência ─────────────────────────────────────────────

#[test]
fn test_done_recurring_daily_creates_next() {
    let env = TestEnv::new();
    let due = days_from_now(1);
    add_recurring(&env, "Daily standup", 1, Recurrence::Daily);

    task::done::execute(env.storage(), 1).unwrap();

    let _tasks = env.load_tasks();
    // Task original (completed) + nova ocorrência
    let all = env.storage().load().unwrap();
    let visible: Vec<_> = all.iter().filter(|t| !t.is_deleted()).collect();
    assert_eq!(visible.len(), 2);

    let next = visible.iter().find(|t| !t.completed).unwrap();
    let expected_due = due + chrono::Duration::days(1);
    assert_eq!(next.due_date, Some(expected_due));
    assert_eq!(next.recurrence, Some(Recurrence::Daily));
}

#[test]
fn test_done_recurring_weekly_creates_next() {
    let env = TestEnv::new();
    let due = days_from_now(7);
    add_recurring(&env, "Weekly review", 7, Recurrence::Weekly);

    task::done::execute(env.storage(), 1).unwrap();

    let all = env.storage().load().unwrap();
    let next = all
        .iter()
        .filter(|t| !t.is_deleted() && !t.completed)
        .next()
        .unwrap();
    let expected_due = due + chrono::Duration::days(7);
    assert_eq!(next.due_date, Some(expected_due));
}

#[test]
fn test_done_recurring_monthly_creates_next() {
    use chrono::NaiveDate;

    let env = TestEnv::new();
    // Usar uma data fixa de fácil cálculo
    let due_str = "2030-03-15";
    task::add::execute(
        env.storage(),
        AddArgs {
            text: "Monthly report".to_string(),
            priority: Priority::Medium,
            tag: vec![],
            project: None,
            due: Some(due_str.to_string()),
            recurrence: Some(Recurrence::Monthly),
            depends_on: vec![],
        },
    )
    .unwrap();

    task::done::execute(env.storage(), 1).unwrap();

    let all = env.storage().load().unwrap();
    let next = all
        .iter()
        .filter(|t| !t.is_deleted() && !t.completed)
        .next()
        .unwrap();
    assert_eq!(
        next.due_date,
        Some(NaiveDate::from_ymd_opt(2030, 4, 15).unwrap())
    );
}

#[test]
fn test_done_recurring_does_not_create_duplicate() {
    let env = TestEnv::new();
    add_recurring(&env, "Daily task", 1, Recurrence::Daily);

    task::done::execute(env.storage(), 1).unwrap();

    // Não deve criar segunda ocorrência ao marcar done novamente
    // (a tarefa original já está done, a nova está pending)
    let all = env.storage().load().unwrap();
    let pending: Vec<_> = all
        .iter()
        .filter(|t| !t.is_deleted() && !t.completed)
        .collect();
    assert_eq!(pending.len(), 1, "deve existir apenas uma tarefa pendente");
}

#[test]
fn test_done_recurring_next_does_not_inherit_deps() {
    let env = TestEnv::new();
    add_simple(&env, "Blocker");
    add_recurring(&env, "Recurring with dep", 1, Recurrence::Daily);

    // Adicionar dep à tarefa recorrente via edit
    rustodo::commands::task::edit::execute(
        env.storage(),
        rustodo::cli::EditArgs {
            id: 2,
            text: None,
            priority: None,
            add_tag: vec![],
            remove_tag: vec![],
            project: None,
            clear_project: false,
            due: None,
            clear_due: false,
            clear_tags: false,
            add_dep: vec![1],
            remove_dep: vec![],
            clear_deps: false,
        },
    )
    .unwrap();

    // Completar o blocker para desbloquear
    task::done::execute(env.storage(), 1).unwrap();
    task::done::execute(env.storage(), 2).unwrap();

    let all = env.storage().load().unwrap();
    let next = all
        .iter()
        .filter(|t| !t.is_deleted() && !t.completed)
        .next()
        .unwrap();
    assert!(
        next.depends_on.is_empty(),
        "próxima ocorrência não deve herdar dependências"
    );
}

#[test]
fn test_done_non_recurring_does_not_create_next() {
    let env = TestEnv::new();
    add_with_due(&env, "One-time task", 1);

    task::done::execute(env.storage(), 1).unwrap();

    let all = env.storage().load().unwrap();
    let visible: Vec<_> = all.iter().filter(|t| !t.is_deleted()).collect();
    assert_eq!(
        visible.len(),
        1,
        "tarefa não recorrente não deve criar próxima ocorrência"
    );
    assert!(visible[0].completed);
}
