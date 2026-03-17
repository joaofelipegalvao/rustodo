//! Integration tests for note commands
//!
//! Covers:
//! - note add (body inline, empty body fails)
//! - note list (all, filter by project, tag, language)
//! - note show
//! - note edit (body, title, language, tags, project, task link, resource links)
//! - note remove
//! - note clear
//! - soft-delete: removed notes don't appear in list
//! - task_id is cleared when linked task is removed
//! - resource_ids are cleaned when resource is removed

mod helpers;

use helpers::TestEnv;
use rustodo::cli::{AddArgs, NoteAddArgs, NoteEditArgs, NoteListArgs, ResourceAddArgs};
use rustodo::commands::{note, resource, task};
use rustodo::models::Priority;
use rustodo::storage::Storage;

// ─── helpers ─────────────────────────────────────────────────────────────────

fn add_task(env: &TestEnv, text: &str) {
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

fn add_note(env: &TestEnv, body: &str) {
    note::add::execute(
        env.storage(),
        NoteAddArgs {
            body: Some(body.to_string()),
            editor: false,
            file: None,
            title: None,
            tag: vec![],
            language: None,
            project: None,
            task: None,
        },
    )
    .unwrap();
}

fn add_note_full(
    env: &TestEnv,
    body: &str,
    title: Option<&str>,
    tags: Vec<&str>,
    language: Option<&str>,
    project: Option<&str>,
    task_num: Option<usize>,
) {
    note::add::execute(
        env.storage(),
        NoteAddArgs {
            body: Some(body.to_string()),
            editor: false,
            file: None,
            title: title.map(|s| s.to_string()),
            tag: tags.into_iter().map(|s| s.to_string()).collect(),
            language: language.map(|s| s.to_string()),
            project: project.map(|s| s.to_string()),
            task: task_num,
        },
    )
    .unwrap();
}

fn add_resource(env: &TestEnv, title: &str) {
    resource::add::execute(
        env.storage(),
        ResourceAddArgs {
            title: title.to_string(),
            r#type: None,
            url: None,
            description: None,
            tag: vec![],
        },
    )
    .unwrap();
}

fn note_count(env: &TestEnv) -> usize {
    env.storage()
        .load_notes()
        .unwrap()
        .into_iter()
        .filter(|n| !n.is_deleted())
        .count()
}

// ─── add ─────────────────────────────────────────────────────────────────────

#[test]
fn test_note_add_simple() {
    let env = TestEnv::new();
    add_note(&env, "My first note");
    assert_eq!(note_count(&env), 1);

    let notes = env.storage().load_notes().unwrap();
    assert_eq!(notes[0].body, "My first note");
}

#[test]
fn test_note_add_with_all_metadata() {
    let env = TestEnv::new();
    add_task(&env, "Some task");

    add_note_full(
        &env,
        "Detailed note body",
        Some("My Title"),
        vec!["rust", "docs"],
        Some("rust"),
        Some("Backend"),
        Some(1),
    );

    let notes = env.storage().load_notes().unwrap();
    assert_eq!(notes.len(), 1);
    let n = &notes[0];
    assert_eq!(n.body, "Detailed note body");
    assert_eq!(n.title.as_deref(), Some("My Title"));
    assert!(n.tags.contains(&"rust".to_string()));
    assert!(n.tags.contains(&"docs".to_string()));
    assert_eq!(n.language.as_deref(), Some("rust"));
    assert!(n.project_id.is_some());
    assert!(n.task_id.is_some());
}

#[test]
fn test_note_add_no_body_fails() {
    let env = TestEnv::new();
    let result = note::add::execute(
        env.storage(),
        NoteAddArgs {
            body: None,
            editor: false,
            file: None,
            title: None,
            tag: vec![],
            language: None,
            project: None,
            task: None,
        },
    );
    assert!(result.is_err());
}

#[test]
fn test_note_add_multiple() {
    let env = TestEnv::new();
    add_note(&env, "Note A");
    add_note(&env, "Note B");
    add_note(&env, "Note C");
    assert_eq!(note_count(&env), 3);
}

// ─── list ─────────────────────────────────────────────────────────────────────

#[test]
fn test_note_list_all() {
    let env = TestEnv::new();
    add_note(&env, "Note A");
    add_note(&env, "Note B");

    let result = note::list::execute(
        env.storage(),
        NoteListArgs {
            project: None,
            tag: None,
            language: None,
        },
    );
    assert!(result.is_ok());
}

#[test]
fn test_note_list_empty_is_ok() {
    let env = TestEnv::new();
    // note::list imprime "No notes found." e retorna Ok quando não há notas
    let result = note::list::execute(
        env.storage(),
        NoteListArgs {
            project: None,
            tag: None,
            language: None,
        },
    );
    assert!(result.is_ok());
}

#[test]
fn test_note_list_filter_by_project() {
    let env = TestEnv::new();
    add_note_full(
        &env,
        "Backend note",
        None,
        vec![],
        None,
        Some("Backend"),
        None,
    );
    add_note_full(
        &env,
        "Frontend note",
        None,
        vec![],
        None,
        Some("Frontend"),
        None,
    );

    let result = note::list::execute(
        env.storage(),
        NoteListArgs {
            project: Some("Backend".to_string()),
            tag: None,
            language: None,
        },
    );
    assert!(result.is_ok());
}

#[test]
fn test_note_list_filter_by_tag() {
    let env = TestEnv::new();
    add_note_full(&env, "Rust note", None, vec!["rust"], None, None, None);
    add_note_full(&env, "Python note", None, vec!["python"], None, None, None);

    let result = note::list::execute(
        env.storage(),
        NoteListArgs {
            project: None,
            tag: Some("rust".to_string()),
            language: None,
        },
    );
    assert!(result.is_ok());
}

#[test]
fn test_note_list_filter_by_language() {
    let env = TestEnv::new();
    add_note_full(&env, "Rust snippet", None, vec![], Some("rust"), None, None);
    add_note_full(
        &env,
        "Python snippet",
        None,
        vec![],
        Some("python"),
        None,
        None,
    );

    let result = note::list::execute(
        env.storage(),
        NoteListArgs {
            project: None,
            tag: None,
            language: Some("rust".to_string()),
        },
    );
    assert!(result.is_ok());
}

// ─── show ─────────────────────────────────────────────────────────────────────

#[test]
fn test_note_show_valid_id() {
    let env = TestEnv::new();
    add_note(&env, "Show me");

    let result = note::show::execute(env.storage(), 1);
    assert!(result.is_ok());
}

#[test]
fn test_note_show_invalid_id_fails() {
    let env = TestEnv::new();
    let result = note::show::execute(env.storage(), 1);
    assert!(result.is_err());
}

#[test]
fn test_note_show_out_of_range_fails() {
    let env = TestEnv::new();
    add_note(&env, "Only note");

    let result = note::show::execute(env.storage(), 99);
    assert!(result.is_err());
}

// ─── edit ─────────────────────────────────────────────────────────────────────

fn blank_edit(id: usize) -> NoteEditArgs {
    NoteEditArgs {
        id,
        body: None,
        editor: false,
        title: None,
        clear_title: false,
        language: None,
        clear_language: false,
        add_tag: vec![],
        remove_tag: vec![],
        clear_tags: false,
        project: None,
        clear_project: false,
        task: None,
        clear_task: false,
        add_resource: vec![],
        remove_resource: vec![],
        clear_resources: false,
    }
}

#[test]
fn test_note_edit_body() {
    let env = TestEnv::new();
    add_note(&env, "Old body");

    let result = note::edit::execute(
        env.storage(),
        NoteEditArgs {
            body: Some("New body".to_string()),
            ..blank_edit(1)
        },
    );
    assert!(result.is_ok());

    let notes = env.storage().load_notes().unwrap();
    assert_eq!(notes[0].body, "New body");
}

#[test]
fn test_note_edit_title() {
    let env = TestEnv::new();
    add_note(&env, "Body");

    note::edit::execute(
        env.storage(),
        NoteEditArgs {
            title: Some("New Title".to_string()),
            ..blank_edit(1)
        },
    )
    .unwrap();

    let notes = env.storage().load_notes().unwrap();
    assert_eq!(notes[0].title.as_deref(), Some("New Title"));
}

#[test]
fn test_note_edit_clear_title() {
    let env = TestEnv::new();
    add_note_full(&env, "Body", Some("Title"), vec![], None, None, None);

    note::edit::execute(
        env.storage(),
        NoteEditArgs {
            clear_title: true,
            ..blank_edit(1)
        },
    )
    .unwrap();

    let notes = env.storage().load_notes().unwrap();
    assert!(notes[0].title.is_none());
}

#[test]
fn test_note_edit_language() {
    let env = TestEnv::new();
    add_note(&env, "Code snippet");

    note::edit::execute(
        env.storage(),
        NoteEditArgs {
            language: Some("rust".to_string()),
            ..blank_edit(1)
        },
    )
    .unwrap();

    let notes = env.storage().load_notes().unwrap();
    assert_eq!(notes[0].language.as_deref(), Some("rust"));
}

#[test]
fn test_note_edit_clear_language() {
    let env = TestEnv::new();
    add_note_full(&env, "Code", None, vec![], Some("rust"), None, None);

    note::edit::execute(
        env.storage(),
        NoteEditArgs {
            clear_language: true,
            ..blank_edit(1)
        },
    )
    .unwrap();

    let notes = env.storage().load_notes().unwrap();
    assert!(notes[0].language.is_none());
}

#[test]
fn test_note_edit_add_tags() {
    let env = TestEnv::new();
    add_note_full(&env, "Body", None, vec!["rust"], None, None, None);

    note::edit::execute(
        env.storage(),
        NoteEditArgs {
            add_tag: vec!["docs".to_string()],
            ..blank_edit(1)
        },
    )
    .unwrap();

    let notes = env.storage().load_notes().unwrap();
    assert!(notes[0].tags.contains(&"rust".to_string()));
    assert!(notes[0].tags.contains(&"docs".to_string()));
}

#[test]
fn test_note_edit_remove_tags() {
    let env = TestEnv::new();
    add_note_full(&env, "Body", None, vec!["rust", "docs"], None, None, None);

    note::edit::execute(
        env.storage(),
        NoteEditArgs {
            remove_tag: vec!["docs".to_string()],
            ..blank_edit(1)
        },
    )
    .unwrap();

    let notes = env.storage().load_notes().unwrap();
    assert!(notes[0].tags.contains(&"rust".to_string()));
    assert!(!notes[0].tags.contains(&"docs".to_string()));
}

#[test]
fn test_note_edit_clear_tags() {
    let env = TestEnv::new();
    add_note_full(&env, "Body", None, vec!["rust", "docs"], None, None, None);

    note::edit::execute(
        env.storage(),
        NoteEditArgs {
            clear_tags: true,
            ..blank_edit(1)
        },
    )
    .unwrap();

    let notes = env.storage().load_notes().unwrap();
    assert!(notes[0].tags.is_empty());
}

#[test]
fn test_note_edit_assign_project() {
    let env = TestEnv::new();
    add_note(&env, "Body");

    note::edit::execute(
        env.storage(),
        NoteEditArgs {
            project: Some("Backend".to_string()),
            ..blank_edit(1)
        },
    )
    .unwrap();

    let notes = env.storage().load_notes().unwrap();
    assert!(notes[0].project_id.is_some());
}

#[test]
fn test_note_edit_clear_project() {
    let env = TestEnv::new();
    add_note_full(&env, "Body", None, vec![], None, Some("Backend"), None);

    note::edit::execute(
        env.storage(),
        NoteEditArgs {
            clear_project: true,
            ..blank_edit(1)
        },
    )
    .unwrap();

    let notes = env.storage().load_notes().unwrap();
    assert!(notes[0].project_id.is_none());
}

#[test]
fn test_note_edit_link_task() {
    let env = TestEnv::new();
    add_task(&env, "Some task");
    add_note(&env, "Body");

    note::edit::execute(
        env.storage(),
        NoteEditArgs {
            task: Some(1),
            ..blank_edit(1)
        },
    )
    .unwrap();

    let notes = env.storage().load_notes().unwrap();
    assert!(notes[0].task_id.is_some());
}

#[test]
fn test_note_edit_clear_task() {
    let env = TestEnv::new();
    add_task(&env, "Some task");
    add_note_full(&env, "Body", None, vec![], None, None, Some(1));

    note::edit::execute(
        env.storage(),
        NoteEditArgs {
            clear_task: true,
            ..blank_edit(1)
        },
    )
    .unwrap();

    let notes = env.storage().load_notes().unwrap();
    assert!(notes[0].task_id.is_none());
}

#[test]
fn test_note_edit_invalid_id_fails() {
    let env = TestEnv::new();
    let result = note::edit::execute(
        env.storage(),
        NoteEditArgs {
            body: Some("Body".to_string()),
            ..blank_edit(99)
        },
    );
    assert!(result.is_err());
}

#[test]
fn test_note_edit_no_changes_is_ok() {
    let env = TestEnv::new();
    add_note(&env, "Body");

    // nenhuma mudança — deve retornar Ok com mensagem "no changes"
    let result = note::edit::execute(env.storage(), blank_edit(1));
    assert!(result.is_ok());
}

// ─── remove ───────────────────────────────────────────────────────────────────

#[test]
fn test_note_remove_valid() {
    let env = TestEnv::new();
    add_note(&env, "Remove me");

    note::remove::execute(env.storage(), 1, true).unwrap();
    assert_eq!(note_count(&env), 0);
}

#[test]
fn test_note_remove_soft_deletes() {
    let env = TestEnv::new();
    add_note(&env, "Note");

    note::remove::execute(env.storage(), 1, true).unwrap();

    // tombstone deve existir
    let all = env.storage().load_notes().unwrap();
    assert_eq!(all.len(), 1);
    assert!(all[0].is_deleted());
}

#[test]
fn test_note_remove_invalid_id_fails() {
    let env = TestEnv::new();
    let result = note::remove::execute(env.storage(), 99, true);
    assert!(result.is_err());
}

#[test]
fn test_note_remove_does_not_affect_others() {
    let env = TestEnv::new();
    add_note(&env, "Keep");
    add_note(&env, "Remove");
    add_note(&env, "Keep too");

    note::remove::execute(env.storage(), 2, true).unwrap();

    assert_eq!(note_count(&env), 2);
    let notes: Vec<_> = env
        .storage()
        .load_notes()
        .unwrap()
        .into_iter()
        .filter(|n| !n.is_deleted())
        .collect();
    assert!(notes.iter().any(|n| n.body == "Keep"));
    assert!(notes.iter().any(|n| n.body == "Keep too"));
}

// ─── clear ────────────────────────────────────────────────────────────────────

#[test]
fn test_note_clear_removes_all() {
    let env = TestEnv::new();
    add_note(&env, "A");
    add_note(&env, "B");
    add_note(&env, "C");

    note::clear::execute(env.storage(), true).unwrap();
    assert_eq!(note_count(&env), 0);
}

#[test]
fn test_note_clear_empty_is_ok() {
    let env = TestEnv::new();
    let result = note::clear::execute(env.storage(), true);
    assert!(result.is_ok());
}

// ─── cascades ────────────────────────────────────────────────────────────────

#[test]
fn test_remove_task_clears_note_task_id() {
    let env = TestEnv::new();
    add_task(&env, "Linked task");
    add_note_full(&env, "Linked note", None, vec![], None, None, Some(1));

    // Verificar que o link foi criado
    let notes = env.storage().load_notes().unwrap();
    assert!(notes[0].task_id.is_some());

    task::remove::execute(env.storage(), 1, true).unwrap();

    let notes = env.storage().load_notes().unwrap();
    assert!(
        notes[0].task_id.is_none(),
        "task_id deve ser limpo quando a task é removida"
    );
}

#[test]
fn test_remove_resource_clears_note_resource_ids() {
    let env = TestEnv::new();
    add_resource(&env, "My resource");

    // Adicionar nota e linkar o resource via edit
    add_note(&env, "Note with resource");
    note::edit::execute(
        env.storage(),
        NoteEditArgs {
            add_resource: vec![1],
            ..blank_edit(1)
        },
    )
    .unwrap();

    let notes = env.storage().load_notes().unwrap();
    assert_eq!(notes[0].resource_ids.len(), 1);

    resource::remove::execute(env.storage(), 1, true).unwrap();

    let notes = env.storage().load_notes().unwrap();
    assert!(
        notes[0].resource_ids.is_empty(),
        "resource_id deve ser removido quando o resource é deletado"
    );
}
