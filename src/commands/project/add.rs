use anyhow::Result;
use colored::Colorize;

use crate::cli::ProjectAddArgs;
use crate::models::Project;
use crate::storage::{EntityType, EventType, Storage};
use crate::utils::date_parser;

pub fn execute(storage: &impl Storage, args: ProjectAddArgs) -> Result<()> {
    let projects = storage.load_projects()?;

    if projects
        .iter()
        .any(|p| p.name.to_lowercase() == args.name.to_lowercase() && !p.is_deleted())
    {
        return Err(anyhow::anyhow!("Project \"{}\" already exists", args.name));
    }

    let due = if let Some(ref due_str) = args.due {
        Some(date_parser::parse_date_not_in_past(due_str)?)
    } else {
        None
    };

    let mut project = Project::new(args.name.clone());
    if let Some(difficulty) = args.difficulty {
        project.difficulty = difficulty;
    }
    if !args.tech.is_empty() {
        project.tech = args
            .tech
            .into_iter()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();
    }
    if let Some(d) = due {
        project.due_date = Some(d);
    }

    let project_uuid = project.uuid;
    let visible_id = projects.iter().filter(|p| !p.is_deleted()).count() + 1;
    storage.upsert_project(&project)?;
    storage.record_event(EntityType::Project, project_uuid, EventType::Created)?;

    println!(
        "{} Added project #{}: {}",
        "✓".green(),
        visible_id,
        args.name.cyan()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::ProjectAddArgs;
    use crate::models::Difficulty;
    use crate::storage::InMemoryStorage;

    fn args(name: &str) -> ProjectAddArgs {
        ProjectAddArgs {
            name: name.into(),
            difficulty: None,
            tech: vec![],
            due: None,
        }
    }

    #[test]
    fn test_project_add_creates_project() {
        let storage = InMemoryStorage::default();
        execute(&storage, args("Rustodo")).unwrap();

        let projects = storage.load_projects().unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Rustodo");
    }

    #[test]
    fn test_project_add_duplicate_name_returns_error() {
        let storage = InMemoryStorage::default();
        execute(&storage, args("Rustodo")).unwrap();

        let err = execute(&storage, args("Rustodo")).unwrap_err();
        assert!(err.to_string().contains("already exists"));
    }

    #[test]
    fn test_project_add_duplicate_case_insensitive() {
        let storage = InMemoryStorage::default();
        execute(&storage, args("Rustodo")).unwrap();

        let err = execute(&storage, args("rustodo")).unwrap_err();
        assert!(err.to_string().contains("already exists"));
    }

    #[test]
    fn test_project_add_with_difficulty() {
        let storage = InMemoryStorage::default();
        execute(
            &storage,
            ProjectAddArgs {
                name: "Hard project".into(),
                difficulty: Some(Difficulty::Hard),
                tech: vec![],
                due: None,
            },
        )
        .unwrap();

        let projects = storage.load_projects().unwrap();
        assert_eq!(projects[0].difficulty, Difficulty::Hard);
    }

    #[test]
    fn test_project_add_with_tech() {
        let storage = InMemoryStorage::default();
        execute(
            &storage,
            ProjectAddArgs {
                name: "Tech project".into(),
                difficulty: None,
                tech: vec!["rust".into(), "sqlite".into()],
                due: None,
            },
        )
        .unwrap();

        let projects = storage.load_projects().unwrap();
        assert_eq!(projects[0].tech, vec!["rust", "sqlite"]);
    }

    #[test]
    fn test_project_add_deleted_project_allows_reuse_of_name() {
        let storage = InMemoryStorage::default();
        let mut p = crate::models::Project::new("Rustodo".into());
        p.soft_delete();
        storage.save_projects(&[p]).unwrap();

        // Should succeed since the existing one is deleted
        execute(&storage, args("Rustodo")).unwrap();
        let active: Vec<_> = storage
            .load_projects()
            .unwrap()
            .into_iter()
            .filter(|p| !p.is_deleted())
            .collect();
        assert_eq!(active.len(), 1);
    }
}
