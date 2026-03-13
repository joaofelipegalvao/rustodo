//! Handler for `todo sync` subcommands.
//!
//! Git-based sync has been removed in favour of the SQLite storage backend.
//! All subcommands currently return a not-implemented message.

use anyhow::Result;
use colored::Colorize;

use crate::storage::Storage;

/// Subcommand variants for `todo sync`.
pub enum SyncCommand {
    Init { remote: String },
    Push,
    Pull,
    Status,
}

/// Executes a sync subcommand.
pub fn execute(_storage: &impl Storage, cmd: SyncCommand) -> Result<()> {
    match cmd {
        SyncCommand::Init { .. } | SyncCommand::Push | SyncCommand::Pull | SyncCommand::Status => {
            println!(
                "\n{} Git sync is not available in this version.\n",
                "".yellow()
            );
            println!("  The sync feature is being redesigned for the SQLite backend.");
            println!(
                "  Your data is stored at: {}",
                crate::storage::get_db_path()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|_| "(unknown)".to_string())
                    .cyan()
            );
            println!();
            Ok(())
        }
    }
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
