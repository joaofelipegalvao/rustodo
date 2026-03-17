//! Integration tests for project commands
//!
//! Complementa test_projects_command.rs cobrindo:
//! - project add (simples, duplicado, validação)
//! - project show
//! - project edit (name, difficulty, tech, due, done/undone)
//! - project done / undone
//! - project remove (soft delete, cascata em tasks e notes)
//! - project clear

mod helpers;

use helpers::TestEnv;
use rustodo::cli::{AddArgs, NoteAddArgs, ProjectAddArgs, ProjectEditArgs};
use rustodo::commands::{note, project, task};
use rustodo::models::{Difficulty, Priority};
use rustodo::storage::Storage;

// ─── helpers ─────────────────────────────────────────────────────────────────

fn add_proj(env: &TestEnv, name: &str) {
    project::add::execute(
        env.storage(),
        ProjectAddArgs {
            name: name.to_string(),
            difficulty: None,
            tech: vec![],
            due: None,
        },
    )
    .unwrap();
}

fn add_task_in_project(env: &TestEnv, text: &str, proj: &str) {
    task::add::execute(
        env.storage(),
        AddArgs {
            text: text.to_string(),
            priority: Priority::Medium,
            tag: vec![],
            project: Some(proj.to_string()),
            due: None,
            recurrence: None,
            depends_on: vec![],
        },
    )
    .unwrap();
}

fn add_note_in_project(env: &TestEnv, body: &str, proj: &str) {
    note::add::execute(
        env.storage(),
        NoteAddArgs {
            body: Some(body.to_string()),
            editor: false,
            file: None,
            title: None,
            tag: vec![],
            language: None,
            project: Some(proj.to_string()),
            task: None,
        },
    )
    .unwrap();
}

fn project_count(env: &TestEnv) -> usize {
    env.storage()
        .load_projects()
        .unwrap()
        .into_iter()
        .filter(|p| !p.is_deleted())
        .count()
}

fn blank_edit(id: usize) -> ProjectEditArgs {
    ProjectEditArgs {
        id,
        name: None,
        difficulty: None,
        done: false,
        undone: false,
        add_tech: vec![],
        remove_tech: vec![],
        clear_tech: false,
        due: None,
        clear_due: false,
    }
}

// ─── add ─────────────────────────────────────────────────────────────────────

#[test]
fn test_project_add_simple() {
    let env = TestEnv::new();
    add_proj(&env, "Backend");
    assert_eq!(project_count(&env), 1);

    let projects = env.storage().load_projects().unwrap();
    assert_eq!(projects[0].name, "Backend");
}

#[test]
fn test_project_add_multiple() {
    let env = TestEnv::new();
    add_proj(&env, "Backend");
    add_proj(&env, "Frontend");
    add_proj(&env, "Docs");
    assert_eq!(project_count(&env), 3);
}

#[test]
fn test_project_add_duplicate_fails() {
    let env = TestEnv::new();
    add_proj(&env, "Backend");

    let result = project::add::execute(
        env.storage(),
        ProjectAddArgs {
            name: "Backend".to_string(),
            difficulty: None,
            tech: vec![],
            due: None,
        },
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));
}

#[test]
fn test_project_add_with_metadata() {
    let env = TestEnv::new();
    project::add::execute(
        env.storage(),
        ProjectAddArgs {
            name: "API".to_string(),
            difficulty: Some(Difficulty::Hard),
            tech: vec!["rust".to_string(), "axum".to_string()],
            due: None,
        },
    )
    .unwrap();

    let projects = env.storage().load_projects().unwrap();
    assert_eq!(projects[0].difficulty, Difficulty::Hard);
    assert!(projects[0].tech.contains(&"rust".to_string()));
    assert!(projects[0].tech.contains(&"axum".to_string()));
}

// ─── show ─────────────────────────────────────────────────────────────────────

#[test]
fn test_project_show_valid_id() {
    let env = TestEnv::new();
    add_proj(&env, "Backend");

    let result = project::show::execute(env.storage(), 1);
    assert!(result.is_ok());
}

#[test]
fn test_project_show_invalid_id_fails() {
    let env = TestEnv::new();
    let result = project::show::execute(env.storage(), 1);
    assert!(result.is_err());
}

#[test]
fn test_project_show_out_of_range_fails() {
    let env = TestEnv::new();
    add_proj(&env, "Backend");

    let result = project::show::execute(env.storage(), 99);
    assert!(result.is_err());
}

// ─── edit ─────────────────────────────────────────────────────────────────────

#[test]
fn test_project_edit_name() {
    let env = TestEnv::new();
    add_proj(&env, "OldName");

    project::edit::execute(
        env.storage(),
        ProjectEditArgs {
            name: Some("NewName".to_string()),
            ..blank_edit(1)
        },
    )
    .unwrap();

    let projects = env.storage().load_projects().unwrap();
    assert_eq!(projects[0].name, "NewName");
}

#[test]
fn test_project_edit_difficulty() {
    let env = TestEnv::new();
    add_proj(&env, "Project");

    project::edit::execute(
        env.storage(),
        ProjectEditArgs {
            difficulty: Some(Difficulty::Hard),
            ..blank_edit(1)
        },
    )
    .unwrap();

    let projects = env.storage().load_projects().unwrap();
    assert_eq!(projects[0].difficulty, Difficulty::Hard);
}

#[test]
fn test_project_edit_add_tech() {
    let env = TestEnv::new();
    add_proj(&env, "Project");

    project::edit::execute(
        env.storage(),
        ProjectEditArgs {
            add_tech: vec!["rust".to_string(), "postgres".to_string()],
            ..blank_edit(1)
        },
    )
    .unwrap();

    let projects = env.storage().load_projects().unwrap();
    assert!(projects[0].tech.contains(&"rust".to_string()));
    assert!(projects[0].tech.contains(&"postgres".to_string()));
}

#[test]
fn test_project_edit_remove_tech() {
    let env = TestEnv::new();
    project::add::execute(
        env.storage(),
        ProjectAddArgs {
            name: "Project".to_string(),
            difficulty: None,
            tech: vec!["rust".to_string(), "postgres".to_string()],
            due: None,
        },
    )
    .unwrap();

    project::edit::execute(
        env.storage(),
        ProjectEditArgs {
            remove_tech: vec!["postgres".to_string()],
            ..blank_edit(1)
        },
    )
    .unwrap();

    let projects = env.storage().load_projects().unwrap();
    assert!(projects[0].tech.contains(&"rust".to_string()));
    assert!(!projects[0].tech.contains(&"postgres".to_string()));
}

#[test]
fn test_project_edit_clear_tech() {
    let env = TestEnv::new();
    project::add::execute(
        env.storage(),
        ProjectAddArgs {
            name: "Project".to_string(),
            difficulty: None,
            tech: vec!["rust".to_string(), "postgres".to_string()],
            due: None,
        },
    )
    .unwrap();

    project::edit::execute(
        env.storage(),
        ProjectEditArgs {
            clear_tech: true,
            ..blank_edit(1)
        },
    )
    .unwrap();

    let projects = env.storage().load_projects().unwrap();
    assert!(projects[0].tech.is_empty());
}

#[test]
fn test_project_edit_invalid_id_fails() {
    let env = TestEnv::new();
    let result = project::edit::execute(
        env.storage(),
        ProjectEditArgs {
            name: Some("New".to_string()),
            ..blank_edit(99)
        },
    );
    assert!(result.is_err());
}

#[test]
fn test_project_edit_no_changes_is_ok() {
    let env = TestEnv::new();
    add_proj(&env, "Project");

    let result = project::edit::execute(env.storage(), blank_edit(1));
    assert!(result.is_ok());
}

// ─── done / undone ────────────────────────────────────────────────────────────

#[test]
fn test_project_done() {
    let env = TestEnv::new();
    add_proj(&env, "Project");

    project::done::execute(env.storage(), 1).unwrap();

    let projects = env.storage().load_projects().unwrap();
    assert!(projects[0].completed);
    assert!(projects[0].completed_at.is_some());
}

#[test]
fn test_project_done_already_done_is_ok() {
    let env = TestEnv::new();
    add_proj(&env, "Project");

    project::done::execute(env.storage(), 1).unwrap();
    // Segunda chamada não deve falhar
    let result = project::done::execute(env.storage(), 1);
    assert!(result.is_ok());
}

#[test]
fn test_project_undone() {
    let env = TestEnv::new();
    add_proj(&env, "Project");
    project::done::execute(env.storage(), 1).unwrap();

    project::undone::execute(env.storage(), 1).unwrap();

    let projects = env.storage().load_projects().unwrap();
    assert!(!projects[0].completed);
    assert!(projects[0].completed_at.is_none());
}

#[test]
fn test_project_undone_already_pending_is_ok() {
    let env = TestEnv::new();
    add_proj(&env, "Project");

    let result = project::undone::execute(env.storage(), 1);
    assert!(result.is_ok());
}

#[test]
fn test_project_done_invalid_id_fails() {
    let env = TestEnv::new();
    let result = project::done::execute(env.storage(), 99);
    assert!(result.is_err());
}

#[test]
fn test_project_undone_invalid_id_fails() {
    let env = TestEnv::new();
    let result = project::undone::execute(env.storage(), 99);
    assert!(result.is_err());
}

// ─── remove ───────────────────────────────────────────────────────────────────

#[test]
fn test_project_remove_valid() {
    let env = TestEnv::new();
    add_proj(&env, "Backend");

    project::remove::execute(env.storage(), 1, true).unwrap();
    assert_eq!(project_count(&env), 0);
}

#[test]
fn test_project_remove_soft_deletes() {
    let env = TestEnv::new();
    add_proj(&env, "Backend");

    project::remove::execute(env.storage(), 1, true).unwrap();

    let all = env.storage().load_projects().unwrap();
    assert_eq!(all.len(), 1);
    assert!(all[0].is_deleted());
}

#[test]
fn test_project_remove_invalid_id_fails() {
    let env = TestEnv::new();
    let result = project::remove::execute(env.storage(), 99, true);
    assert!(result.is_err());
}

#[test]
fn test_project_remove_clears_task_project_id() {
    let env = TestEnv::new();
    add_task_in_project(&env, "Task A", "Backend");
    add_task_in_project(&env, "Task B", "Backend");

    let tasks = env.load_tasks();
    assert!(tasks.iter().all(|t| t.project_id.is_some()));

    project::remove::execute(env.storage(), 1, true).unwrap();

    let tasks = env.load_tasks();
    assert!(
        tasks.iter().all(|t| t.project_id.is_none()),
        "project_id deve ser limpo quando o projeto é removido"
    );
}

#[test]
fn test_project_remove_clears_note_project_id() {
    let env = TestEnv::new();
    add_note_in_project(&env, "Note A", "Backend");
    add_note_in_project(&env, "Note B", "Backend");

    let notes = env.storage().load_notes().unwrap();
    assert!(notes.iter().all(|n| n.project_id.is_some()));

    project::remove::execute(env.storage(), 1, true).unwrap();

    let notes = env.storage().load_notes().unwrap();
    assert!(
        notes.iter().all(|n| n.project_id.is_none()),
        "project_id deve ser limpo nas notes quando o projeto é removido"
    );
}

#[test]
fn test_project_remove_does_not_affect_other_projects() {
    let env = TestEnv::new();
    add_proj(&env, "Backend");
    add_proj(&env, "Frontend");

    project::remove::execute(env.storage(), 1, true).unwrap();

    assert_eq!(project_count(&env), 1);
    let projects: Vec<_> = env
        .storage()
        .load_projects()
        .unwrap()
        .into_iter()
        .filter(|p| !p.is_deleted())
        .collect();
    assert_eq!(projects[0].name, "Frontend");
}

// ─── clear ────────────────────────────────────────────────────────────────────

#[test]
fn test_project_clear_removes_all() {
    let env = TestEnv::new();
    add_proj(&env, "A");
    add_proj(&env, "B");
    add_proj(&env, "C");

    project::clear::execute(env.storage(), true).unwrap();
    assert_eq!(project_count(&env), 0);
}

#[test]
fn test_project_clear_empty_is_ok() {
    let env = TestEnv::new();
    let result = project::clear::execute(env.storage(), true);
    assert!(result.is_ok());
}

#[test]
fn test_project_clear_cleans_task_and_note_links() {
    let env = TestEnv::new();
    add_task_in_project(&env, "Task", "Backend");
    add_note_in_project(&env, "Note", "Backend");

    project::clear::execute(env.storage(), true).unwrap();

    let tasks = env.load_tasks();
    assert!(tasks[0].project_id.is_none());

    let notes = env.storage().load_notes().unwrap();
    assert!(notes[0].project_id.is_none());
}
