//! Integration tests for resource commands
//!
//! Covers:
//! - resource add (simple, with metadata, duplicate warning bypass)
//! - resource list (all, filter by tag, filter by type)
//! - resource show
//! - resource edit (title, type, url, description, tags)
//! - resource remove (soft delete, cascata em notes)
//! - resource clear

mod helpers;

use helpers::TestEnv;
use rustodo::cli::{
    NoteAddArgs, NoteEditArgs, ResourceAddArgs, ResourceEditArgs, ResourceListArgs,
};
use rustodo::commands::{note, resource};
use rustodo::models::ResourceType;
use rustodo::storage::Storage;

// ─── helpers ─────────────────────────────────────────────────────────────────

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

fn add_resource_full(
    env: &TestEnv,
    title: &str,
    r#type: Option<ResourceType>,
    url: Option<&str>,
    description: Option<&str>,
    tags: Vec<&str>,
) {
    resource::add::execute(
        env.storage(),
        ResourceAddArgs {
            title: title.to_string(),
            r#type,
            url: url.map(|s| s.to_string()),
            description: description.map(|s| s.to_string()),
            tag: tags.into_iter().map(|s| s.to_string()).collect(),
        },
    )
    .unwrap();
}

fn resource_count(env: &TestEnv) -> usize {
    env.storage()
        .load_resources()
        .unwrap()
        .into_iter()
        .filter(|r| !r.is_deleted())
        .count()
}

fn blank_edit(id: usize) -> ResourceEditArgs {
    ResourceEditArgs {
        id,
        title: None,
        r#type: None,
        clear_type: false,
        url: None,
        clear_url: false,
        description: None,
        clear_description: false,
        add_tag: vec![],
        remove_tag: vec![],
        clear_tags: false,
    }
}

// ─── add ─────────────────────────────────────────────────────────────────────

#[test]
fn test_resource_add_simple() {
    let env = TestEnv::new();
    add_resource(&env, "sqlx docs");
    assert_eq!(resource_count(&env), 1);

    let resources = env.storage().load_resources().unwrap();
    assert_eq!(resources[0].title, "sqlx docs");
}

#[test]
fn test_resource_add_with_all_metadata() {
    let env = TestEnv::new();
    add_resource_full(
        &env,
        "Tokio docs",
        Some(ResourceType::Docs),
        Some("https://docs.rs/tokio"),
        Some("Async runtime for Rust"),
        vec!["rust", "async"],
    );

    let resources = env.storage().load_resources().unwrap();
    let r = &resources[0];
    assert_eq!(r.title, "Tokio docs");
    assert_eq!(r.resource_type, Some(ResourceType::Docs));
    assert_eq!(r.url.as_deref(), Some("https://docs.rs/tokio"));
    assert_eq!(r.description.as_deref(), Some("Async runtime for Rust"));
    assert!(r.tags.contains(&"rust".to_string()));
    assert!(r.tags.contains(&"async".to_string()));
}

#[test]
fn test_resource_add_multiple() {
    let env = TestEnv::new();
    add_resource(&env, "Resource A");
    add_resource(&env, "Resource B");
    add_resource(&env, "Resource C");
    assert_eq!(resource_count(&env), 3);
}

// ─── list ─────────────────────────────────────────────────────────────────────

#[test]
fn test_resource_list_all() {
    let env = TestEnv::new();
    add_resource(&env, "R1");
    add_resource(&env, "R2");

    let result = resource::list::execute(
        env.storage(),
        ResourceListArgs {
            tag: None,
            r#type: None,
        },
    );
    assert!(result.is_ok());
}

#[test]
fn test_resource_list_empty_is_ok() {
    let env = TestEnv::new();
    // resource::list imprime "No resources found." e retorna Ok quando não há resources
    let result = resource::list::execute(
        env.storage(),
        ResourceListArgs {
            tag: None,
            r#type: None,
        },
    );
    assert!(result.is_ok());
}

#[test]
fn test_resource_list_filter_by_tag() {
    let env = TestEnv::new();
    add_resource_full(&env, "Rust resource", None, None, None, vec!["rust"]);
    add_resource_full(&env, "Python resource", None, None, None, vec!["python"]);

    let result = resource::list::execute(
        env.storage(),
        ResourceListArgs {
            tag: Some("rust".to_string()),
            r#type: None,
        },
    );
    assert!(result.is_ok());
}

#[test]
fn test_resource_list_filter_by_type() {
    let env = TestEnv::new();
    add_resource_full(
        &env,
        "Tokio docs",
        Some(ResourceType::Docs),
        None,
        None,
        vec![],
    );
    add_resource_full(
        &env,
        "Rust book",
        Some(ResourceType::Book),
        None,
        None,
        vec![],
    );

    let result = resource::list::execute(
        env.storage(),
        ResourceListArgs {
            tag: None,
            r#type: Some(ResourceType::Docs),
        },
    );
    assert!(result.is_ok());
}

// ─── show ─────────────────────────────────────────────────────────────────────

#[test]
fn test_resource_show_valid_id() {
    let env = TestEnv::new();
    add_resource(&env, "Show me");

    let result = resource::show::execute(env.storage(), 1);
    assert!(result.is_ok());
}

#[test]
fn test_resource_show_invalid_id_fails() {
    let env = TestEnv::new();
    let result = resource::show::execute(env.storage(), 1);
    assert!(result.is_err());
}

#[test]
fn test_resource_show_out_of_range_fails() {
    let env = TestEnv::new();
    add_resource(&env, "Only resource");

    let result = resource::show::execute(env.storage(), 99);
    assert!(result.is_err());
}

// ─── edit ─────────────────────────────────────────────────────────────────────

#[test]
fn test_resource_edit_title() {
    let env = TestEnv::new();
    add_resource(&env, "Old title");

    resource::edit::execute(
        env.storage(),
        ResourceEditArgs {
            title: Some("New title".to_string()),
            ..blank_edit(1)
        },
    )
    .unwrap();

    let resources = env.storage().load_resources().unwrap();
    assert_eq!(resources[0].title, "New title");
}

#[test]
fn test_resource_edit_type() {
    let env = TestEnv::new();
    add_resource(&env, "Resource");

    resource::edit::execute(
        env.storage(),
        ResourceEditArgs {
            r#type: Some(ResourceType::Repo),
            ..blank_edit(1)
        },
    )
    .unwrap();

    let resources = env.storage().load_resources().unwrap();
    assert_eq!(resources[0].resource_type, Some(ResourceType::Repo));
}

#[test]
fn test_resource_edit_clear_type() {
    let env = TestEnv::new();
    add_resource_full(
        &env,
        "Resource",
        Some(ResourceType::Docs),
        None,
        None,
        vec![],
    );

    resource::edit::execute(
        env.storage(),
        ResourceEditArgs {
            clear_type: true,
            ..blank_edit(1)
        },
    )
    .unwrap();

    let resources = env.storage().load_resources().unwrap();
    assert!(resources[0].resource_type.is_none());
}

#[test]
fn test_resource_edit_url() {
    let env = TestEnv::new();
    add_resource(&env, "Resource");

    resource::edit::execute(
        env.storage(),
        ResourceEditArgs {
            url: Some("https://example.com".to_string()),
            ..blank_edit(1)
        },
    )
    .unwrap();

    let resources = env.storage().load_resources().unwrap();
    assert_eq!(resources[0].url.as_deref(), Some("https://example.com"));
}

#[test]
fn test_resource_edit_clear_url() {
    let env = TestEnv::new();
    add_resource_full(
        &env,
        "Resource",
        None,
        Some("https://example.com"),
        None,
        vec![],
    );

    resource::edit::execute(
        env.storage(),
        ResourceEditArgs {
            clear_url: true,
            ..blank_edit(1)
        },
    )
    .unwrap();

    let resources = env.storage().load_resources().unwrap();
    assert!(resources[0].url.is_none());
}

#[test]
fn test_resource_edit_description() {
    let env = TestEnv::new();
    add_resource(&env, "Resource");

    resource::edit::execute(
        env.storage(),
        ResourceEditArgs {
            description: Some("A great resource".to_string()),
            ..blank_edit(1)
        },
    )
    .unwrap();

    let resources = env.storage().load_resources().unwrap();
    assert_eq!(
        resources[0].description.as_deref(),
        Some("A great resource")
    );
}

#[test]
fn test_resource_edit_clear_description() {
    let env = TestEnv::new();
    add_resource_full(&env, "Resource", None, None, Some("Old desc"), vec![]);

    resource::edit::execute(
        env.storage(),
        ResourceEditArgs {
            clear_description: true,
            ..blank_edit(1)
        },
    )
    .unwrap();

    let resources = env.storage().load_resources().unwrap();
    assert!(resources[0].description.is_none());
}

#[test]
fn test_resource_edit_add_tags() {
    let env = TestEnv::new();
    add_resource_full(&env, "Resource", None, None, None, vec!["rust"]);

    resource::edit::execute(
        env.storage(),
        ResourceEditArgs {
            add_tag: vec!["async".to_string()],
            ..blank_edit(1)
        },
    )
    .unwrap();

    let resources = env.storage().load_resources().unwrap();
    assert!(resources[0].tags.contains(&"rust".to_string()));
    assert!(resources[0].tags.contains(&"async".to_string()));
}

#[test]
fn test_resource_edit_remove_tags() {
    let env = TestEnv::new();
    add_resource_full(&env, "Resource", None, None, None, vec!["rust", "async"]);

    resource::edit::execute(
        env.storage(),
        ResourceEditArgs {
            remove_tag: vec!["async".to_string()],
            ..blank_edit(1)
        },
    )
    .unwrap();

    let resources = env.storage().load_resources().unwrap();
    assert!(resources[0].tags.contains(&"rust".to_string()));
    assert!(!resources[0].tags.contains(&"async".to_string()));
}

#[test]
fn test_resource_edit_clear_tags() {
    let env = TestEnv::new();
    add_resource_full(&env, "Resource", None, None, None, vec!["rust", "async"]);

    resource::edit::execute(
        env.storage(),
        ResourceEditArgs {
            clear_tags: true,
            ..blank_edit(1)
        },
    )
    .unwrap();

    let resources = env.storage().load_resources().unwrap();
    assert!(resources[0].tags.is_empty());
}

#[test]
fn test_resource_edit_invalid_id_fails() {
    let env = TestEnv::new();
    let result = resource::edit::execute(
        env.storage(),
        ResourceEditArgs {
            title: Some("New".to_string()),
            ..blank_edit(99)
        },
    );
    assert!(result.is_err());
}

#[test]
fn test_resource_edit_no_changes_is_ok() {
    let env = TestEnv::new();
    add_resource(&env, "Resource");

    let result = resource::edit::execute(env.storage(), blank_edit(1));
    assert!(result.is_ok());
}

// ─── remove ───────────────────────────────────────────────────────────────────

#[test]
fn test_resource_remove_valid() {
    let env = TestEnv::new();
    add_resource(&env, "Remove me");

    resource::remove::execute(env.storage(), 1, true).unwrap();
    assert_eq!(resource_count(&env), 0);
}

#[test]
fn test_resource_remove_soft_deletes() {
    let env = TestEnv::new();
    add_resource(&env, "Resource");

    resource::remove::execute(env.storage(), 1, true).unwrap();

    let all = env.storage().load_resources().unwrap();
    assert_eq!(all.len(), 1);
    assert!(all[0].is_deleted());
}

#[test]
fn test_resource_remove_invalid_id_fails() {
    let env = TestEnv::new();
    let result = resource::remove::execute(env.storage(), 99, true);
    assert!(result.is_err());
}

#[test]
fn test_resource_remove_does_not_affect_others() {
    let env = TestEnv::new();
    add_resource(&env, "Keep");
    add_resource(&env, "Remove");
    add_resource(&env, "Keep too");

    resource::remove::execute(env.storage(), 2, true).unwrap();

    assert_eq!(resource_count(&env), 2);
    let resources: Vec<_> = env
        .storage()
        .load_resources()
        .unwrap()
        .into_iter()
        .filter(|r| !r.is_deleted())
        .collect();
    assert!(resources.iter().any(|r| r.title == "Keep"));
    assert!(resources.iter().any(|r| r.title == "Keep too"));
}

#[test]
fn test_resource_remove_clears_note_resource_ids() {
    let env = TestEnv::new();
    add_resource(&env, "Resource");

    note::add::execute(
        env.storage(),
        NoteAddArgs {
            body: Some("Note body".to_string()),
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

    note::edit::execute(
        env.storage(),
        NoteEditArgs {
            id: 1,
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
            add_resource: vec![1],
            remove_resource: vec![],
            clear_resources: false,
        },
    )
    .unwrap();

    let notes = env.storage().load_notes().unwrap();
    assert_eq!(notes[0].resource_ids.len(), 1);

    resource::remove::execute(env.storage(), 1, true).unwrap();

    let notes = env.storage().load_notes().unwrap();
    assert!(notes[0].resource_ids.is_empty());
}

// ─── clear ────────────────────────────────────────────────────────────────────

#[test]
fn test_resource_clear_removes_all() {
    let env = TestEnv::new();
    add_resource(&env, "A");
    add_resource(&env, "B");
    add_resource(&env, "C");

    resource::clear::execute(env.storage(), true).unwrap();
    assert_eq!(resource_count(&env), 0);
}

#[test]
fn test_resource_clear_empty_is_ok() {
    let env = TestEnv::new();
    let result = resource::clear::execute(env.storage(), true);
    assert!(result.is_ok());
}

#[test]
fn test_resource_clear_cleans_note_links() {
    let env = TestEnv::new();
    add_resource(&env, "R1");
    add_resource(&env, "R2");

    note::add::execute(
        env.storage(),
        NoteAddArgs {
            body: Some("Note".to_string()),
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

    note::edit::execute(
        env.storage(),
        NoteEditArgs {
            id: 1,
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
            add_resource: vec![1, 2],
            remove_resource: vec![],
            clear_resources: false,
        },
    )
    .unwrap();

    resource::clear::execute(env.storage(), true).unwrap();

    let notes = env.storage().load_notes().unwrap();
    assert!(notes[0].resource_ids.is_empty());
}
