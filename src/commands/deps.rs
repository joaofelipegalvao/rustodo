use anyhow::Result;
use colored::Colorize;

use crate::storage::Storage;
use crate::validation::validate_task_id;

pub fn execute(storage: &impl Storage, id: usize) -> Result<()> {
    let tasks = storage.load()?;
    validate_task_id(id, tasks.len())?;

    let task = &tasks[id - 1];

    println!(
        "\n{} #{}: {}\n",
        "Task".dimmed(),
        id,
        task.text.bright_white()
    );

    // === This task depends on ===
    if task.depends_on.is_empty() {
        println!("{}", "  No dependencies.".dimmed());
    } else {
        println!("{}:", "  Depends on".dimmed());
        for &dep_id in &task.depends_on {
            let idx = dep_id.saturating_sub(1);
            if let Some(dep) = tasks.get(idx) {
                let status = if dep.completed {
                    "✓".green()
                } else {
                    "◦".red()
                };
                let label = if dep.completed {
                    dep.text.dimmed()
                } else {
                    dep.text.bright_white()
                };
                println!("    {} #{} — {}", status, dep_id, label);
            } else {
                println!(
                    "    {} #{} — {}",
                    "?".yellow(),
                    dep_id,
                    "(task not found)".dimmed()
                );
            }
        }
    }

    // === Tasks that depend on this one ===
    let dependents: Vec<(usize, &_)> = tasks
        .iter()
        .enumerate()
        .filter(|(i, t)| *i != id - 1 && t.depends_on.contains(&id))
        .map(|(i, t)| (i + 1, t))
        .collect();

    println!();
    if dependents.is_empty() {
        println!("{}", "  No tasks depend on this one.".dimmed());
    } else {
        println!("{}:", "  Required by".dimmed());
        for (dep_id, dep_task) in &dependents {
            let status = if dep_task.completed {
                "✓".green()
            } else {
                "◦".yellow()
            };
            println!(
                "    {} #{} — {}",
                status,
                dep_id,
                dep_task.text.bright_white()
            );
        }
    }

    // === Blocked status ===
    println!();
    if task.is_blocked(&tasks) {
        let blocking = task.blocking_deps(&tasks);
        let ids = blocking
            .iter()
            .map(|id| format!("#{}", id))
            .collect::<Vec<_>>()
            .join(", ");
        println!("  {} Blocked by: {}", "[~]".red(), ids.red());
    } else if !task.depends_on.is_empty() {
        println!("  {} All dependencies satisfied", "✓".green());
    }

    println!();
    Ok(())
}
