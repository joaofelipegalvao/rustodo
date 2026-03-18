//! Handler for `todo resource add`.

use anyhow::Result;
use colored::Colorize;

use crate::cli::ResourceAddArgs;
use crate::models::Resource;
use crate::services::tag_service::collect_all_tag_names;
use crate::storage::{EntityType, EventType, Storage};
use crate::utils::tag_normalizer::normalize_tags;

pub fn execute(storage: &impl Storage, args: ResourceAddArgs) -> Result<()> {
    let resources = storage.load_resources()?;
    let tasks = storage.load()?;
    let notes = storage.load_notes()?;

    let duplicate = if let Some(ref url) = args.url {
        resources
            .iter()
            .filter(|r| !r.is_deleted())
            .find(|r| r.url.as_deref() == Some(url.as_str()))
    } else {
        resources
            .iter()
            .filter(|r| !r.is_deleted())
            .find(|r| r.title.to_lowercase() == args.title.to_lowercase())
    };

    if let Some(existing) = duplicate {
        let visible_id = resources
            .iter()
            .filter(|r| !r.is_deleted())
            .position(|r| r.uuid == existing.uuid)
            .map(|i| i + 1)
            .unwrap_or(0);
        let reason = if args.url.is_some() { "URL" } else { "title" };
        eprintln!(
            "{} Resource with same {} \"{}\" already exists (#{}). Add anyway? [y/N] ",
            "".yellow(),
            reason,
            existing.title,
            visible_id
        );
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            println!("{}", "Cancelled.".dimmed());
            return Ok(());
        }
    }

    let existing_tags = collect_all_tag_names(&tasks, &notes, &resources);
    let (normalized_tags, normalization_messages) = normalize_tags(args.tag, &existing_tags);

    let mut resource = Resource::new(args.title);
    resource.resource_type = args.r#type;
    resource.url = args.url;
    resource.description = args.description;
    resource.tags = normalized_tags;

    let resource_uuid = resource.uuid;
    let visible_id = resources.iter().filter(|r| !r.is_deleted()).count() + 1;
    storage.upsert_resource(&resource)?;
    storage.record_event(EntityType::Resource, resource_uuid, EventType::Created)?;

    for msg in &normalization_messages {
        println!("  {} Tag normalized: {}", "~".yellow(), msg.yellow());
    }
    println!("{} Added resource #{}", "✓".green(), visible_id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ResourceType;
    use crate::storage::InMemoryStorage;

    fn args(title: &str) -> ResourceAddArgs {
        ResourceAddArgs {
            title: title.into(),
            r#type: None,
            url: None,
            description: None,
            tag: vec![],
        }
    }

    // ── basic add ─────────────────────────────────────────────────────────────

    #[test]
    fn test_resource_add_creates_resource() {
        let storage = InMemoryStorage::default();
        execute(&storage, args("SQLx docs")).unwrap();

        let resources = storage.load_resources().unwrap();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].title, "SQLx docs");
    }

    #[test]
    fn test_resource_add_with_url() {
        let storage = InMemoryStorage::default();
        execute(
            &storage,
            ResourceAddArgs {
                url: Some("https://docs.rs/sqlx".into()),
                ..args("SQLx docs")
            },
        )
        .unwrap();

        assert_eq!(
            storage.load_resources().unwrap()[0].url.as_deref(),
            Some("https://docs.rs/sqlx")
        );
    }

    #[test]
    fn test_resource_add_with_type() {
        let storage = InMemoryStorage::default();
        execute(
            &storage,
            ResourceAddArgs {
                r#type: Some(ResourceType::Docs),
                ..args("SQLx docs")
            },
        )
        .unwrap();

        assert_eq!(
            storage.load_resources().unwrap()[0].resource_type,
            Some(ResourceType::Docs)
        );
    }

    #[test]
    fn test_resource_add_with_description() {
        let storage = InMemoryStorage::default();
        execute(
            &storage,
            ResourceAddArgs {
                description: Some("Async SQL for Rust".into()),
                ..args("SQLx docs")
            },
        )
        .unwrap();

        assert_eq!(
            storage.load_resources().unwrap()[0].description.as_deref(),
            Some("Async SQL for Rust")
        );
    }

    #[test]
    fn test_resource_add_with_tags() {
        let storage = InMemoryStorage::default();
        execute(
            &storage,
            ResourceAddArgs {
                tag: vec!["rust".into(), "backend".into()],
                ..args("SQLx docs")
            },
        )
        .unwrap();

        let tags = &storage.load_resources().unwrap()[0].tags;
        assert!(tags.contains(&"rust".to_string()));
        assert!(tags.contains(&"backend".to_string()));
    }

    #[test]
    fn test_resource_add_multiple_increments_count() {
        let storage = InMemoryStorage::default();
        execute(&storage, args("Resource A")).unwrap();
        execute(&storage, args("Resource B")).unwrap();
        execute(&storage, args("Resource C")).unwrap();

        assert_eq!(storage.load_resources().unwrap().len(), 3);
    }

    // ── deleted resource allows reuse of title ────────────────────────────────

    #[test]
    fn test_resource_add_deleted_title_allows_new() {
        let storage = InMemoryStorage::default();
        let mut r = Resource::new("SQLx docs".into());
        r.soft_delete();
        storage.save_resources(&[r]).unwrap();

        // Should succeed — existing one is deleted
        execute(&storage, args("SQLx docs")).unwrap();

        let active: Vec<_> = storage
            .load_resources()
            .unwrap()
            .into_iter()
            .filter(|r| !r.is_deleted())
            .collect();
        assert_eq!(active.len(), 1);
    }

    // ── tag normalization ─────────────────────────────────────────────────────

    #[test]
    fn test_resource_add_normalizes_tags_to_existing() {
        let storage = InMemoryStorage::default();

        // Add first resource with tag "rust"
        execute(
            &storage,
            ResourceAddArgs {
                tag: vec!["rust".into()],
                ..args("Resource A")
            },
        )
        .unwrap();

        // Add second with slightly different casing — should normalize to "rust"
        execute(
            &storage,
            ResourceAddArgs {
                tag: vec!["Rust".into()],
                ..args("Resource B")
            },
        )
        .unwrap();

        let resources = storage.load_resources().unwrap();
        let active: Vec<_> = resources.iter().filter(|r| !r.is_deleted()).collect();
        assert_eq!(active[1].tags[0], "rust");
    }
}
