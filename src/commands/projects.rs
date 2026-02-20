use anyhow::Result;
use colored::Colorize;

use crate::error::TodoError;
use crate::storage::Storage;

pub fn execute(storage: &impl Storage) -> Result<()> {
    let tasks = storage.load()?;

    let mut projects: Vec<String> = tasks
        .iter()
        .filter_map(|t| t.project.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    if projects.is_empty() {
        return Err(TodoError::NoProjectsFound.into());
    }

    projects.sort();

    println!("\nProjects:\n");
    for project in &projects {
        let total = tasks
            .iter()
            .filter(|t| t.project.as_deref() == Some(project))
            .count();
        let pending = tasks
            .iter()
            .filter(|t| t.project.as_deref() == Some(project) && !t.completed)
            .count();
        let done = total - pending;

        println!("  {} ({} pending, {} done)", project.cyan(), pending, done,);
    }

    println!();
    Ok(())
}
