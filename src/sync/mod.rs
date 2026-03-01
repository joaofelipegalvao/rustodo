//! Git sync module for rustodo.
//!
//! Provides sync operations over the rustodo data directory using the
//! system's Git installation via [`std::process::Command`].
//!
//! | Submodule | Purpose |
//! |---|---|
//! | [`config`] | Read/write `sync.toml` (remote URL) |
//! | [`git`] | Git operations: init, commit, push, pull, status |
//! | [`merge`] | Semantic 3-way merge via UUID + `updated_at` |
//!
//! # Typical flow
//!
//! ```text
//! todo sync init git@github.com:user/tasks.git
//!   → git init
//!   → git remote add origin <url>
//!   → git add todos.json && git commit -m "sync: initial commit"
//!   → writes sync.toml
//!
//! todo sync push
//!   → builds commit message from diff (e.g. "sync: 2 added, 1 completed")
//!   → git add todos.json && git commit -m "<message>"
//!   → git push origin HEAD
//!
//! todo sync pull
//!   → git pull --rebase origin HEAD
//!   → semantic merge (Phase 2)
//!   → storage.save(merged)
//!
//! todo sync status
//!   → git status summary
//! ``
pub mod config;
pub mod git;
pub mod merge;
