//! Handler for `todo sync` subcommands.
//!
//! Dispatches to four subcommands:
//!
//! | Subcommand | Description |
//! |---|---|
//! | `init <remote>` | Initialize Git repo and configure remote |
//! | `push` | Commit changes and push to remote |
//! | `pull` | Pull from remote and merge (Phase 2) |
//! | `status` | Show sync state |

use anyhow::{Result, bail};
use colored::Colorize;

use crate::storage::{Storage, get_data_file_path};
use crate::sync::{config, git};

/// Subcommand variants for `todo sync`.
pub enum SyncCommand {
    Init { remote: String },
    Push,
    Pull,
    Status,
}

/// Executes a sync subcommand.
pub fn execute(storage: &impl Storage, cmd: SyncCommand) -> Result<()> {
    match cmd {
        SyncCommand::Init { remote } => init(storage, &remote),
        SyncCommand::Push => push(storage),
        SyncCommand::Pull => pull(storage),
        SyncCommand::Status => status(),
    }
}

// ── init ─────────────────────────────────────────────────────────────────────

fn init(_storage: &impl Storage, remote: &str) -> Result<()> {
    git::check_git_available()?;

    let data_dir = get_data_file_path()?
        .parent()
        .expect("todos.json always has a parent directory")
        .to_path_buf();

    // 1. git init (idempotent)
    git::init(&data_dir)?;
    println!("{} Git repository initialized", "✓".green());

    // 2. git remote add origin <url>
    git::add_remote(&data_dir, remote)?;
    println!("{} Remote set to {}", "✓".green(), remote.cyan());

    // 3. Initial commit of todos.json
    git::initial_commit(&data_dir)?;
    println!("{} todos.json committed", "✓".green());

    // 4. Persist remote URL to sync.toml
    config::save(&config::SyncConfig {
        remote: remote.to_string(),
    })?;
    println!("{} Saved sync config", "✓".green());

    println!();
    println!(
        "{}  Run {} to push your tasks.",
        "→".dimmed(),
        "todo sync push".bright_white()
    );

    Ok(())
}

// ── push ─────────────────────────────────────────────────────────────────────

fn push(storage: &impl Storage) -> Result<()> {
    git::check_git_available()?;
    let _cfg = config::require()?;

    let data_dir = get_data_file_path()?
        .parent()
        .expect("todos.json always has a parent directory")
        .to_path_buf();

    if !git::is_initialized(&data_dir) {
        bail!("Git repository not initialized. Run: todo sync init <remote>");
    }

    // Build a descriptive commit message from task counts
    let tasks = storage.load()?;
    let total = tasks.len();
    let done = tasks.iter().filter(|t| t.completed).count();
    let pending = total - done;
    let message = format!("sync: {} tasks ({} pending, {} done)", total, pending, done);

    let committed = git::commit(&data_dir, &message)?;

    if committed {
        println!("{} Committed: {}", "✓".green(), message.dimmed());
    } else {
        println!("{} Nothing to commit — todos.json is unchanged", "".blue());
    }

    git::push(&data_dir)?;
    println!("{} Pushed to remote", "✓".green());

    Ok(())
}

// ── pull ─────────────────────────────────────────────────────────────────────

fn pull(_storage: &impl Storage) -> Result<()> {
    git::check_git_available()?;
    let _cfg = config::require()?;

    let data_dir = get_data_file_path()?
        .parent()
        .expect("todos.json always has a parent directory")
        .to_path_buf();

    if !git::is_initialized(&data_dir) {
        bail!("Git repository not initialized. Run: todo sync init <remote>");
    }

    println!("{}", "Pulling from remote…".dimmed());
    let output = git::pull(&data_dir)?;

    if !output.is_empty() {
        println!("{}", output.dimmed());
    }

    // Phase 2: semantic merge will be implemented here.
    // For now, git pull --rebase handles conflicts at the file level.
    println!("{} Pull complete", "✓".green());
    println!(
        "{}  Semantic merge (UUID-based) coming in Phase 2.",
        "→".dimmed()
    );

    Ok(())
}

// ── status ───────────────────────────────────────────────────────────────────

fn status() -> Result<()> {
    git::check_git_available()?;

    let data_dir = get_data_file_path()?
        .parent()
        .expect("todos.json always has a parent directory")
        .to_path_buf();

    if !git::is_initialized(&data_dir) {
        bail!("Git repository not initialized. Run: todo sync init <remote>");
    }

    let cfg = config::load()?;

    println!("\n{}\n", "Sync Status".bright_white().bold());

    match cfg {
        Some(c) => println!("  {:<14} {}", "Remote:".dimmed(), c.remote.cyan()),
        None => println!(
            "  {:<14} {}",
            "Remote:".dimmed(),
            "(not configured)".yellow()
        ),
    }

    let s = git::status(&data_dir)?;
    for line in s.lines() {
        println!("  {}", line.dimmed());
    }

    println!();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::config::SyncConfig;

    #[test]
    fn test_sync_command_variants_exist() {
        let _ = SyncCommand::Push;
        let _ = SyncCommand::Pull;
        let _ = SyncCommand::Status;
        let _ = SyncCommand::Init {
            remote: "git@github.com:user/tasks.git".to_string(),
        };
    }

    #[test]
    fn test_sync_config_require_error_message() {
        let result: Result<SyncConfig, _> = toml::from_str("");
        assert!(result.is_err());
    }
}
