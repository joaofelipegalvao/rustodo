//! Handler for `todo note preview <ID>`.
//!
//! Opens the note body in `glow` for rich markdown rendering with pagination.
//! If `glow` is not installed, prints a clear installation message.

use anyhow::Result;
use colored::Colorize;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;

use crate::storage::Storage;

pub fn execute(storage: &impl Storage, id: usize) -> Result<()> {
    let (_, _, notes, _) = storage.load_all_with_resources()?;

    let visible_notes: Vec<_> = notes.iter().filter(|n| !n.is_deleted()).collect();

    let note = visible_notes
        .get(id.saturating_sub(1))
        .ok_or_else(|| anyhow::anyhow!("Note #{} not found", id))?;

    // ── Check note format ─────────────────────────────────────────────────────
    if !note.is_markdown() {
        eprintln!("{} Note #{} is plain text, not markdown.", "✗".red(), id);
        eprintln!("  Use {} to view it.", "todo note show".cyan());
        return Ok(());
    }

    // ── Check glow is available ───────────────────────────────────────────────
    if Command::new("glow").arg("--version").output().is_err() {
        eprintln!("{} {} is not installed.", "✗".red(), "glow".yellow());
        eprintln!("  Install it with: {}", "brew install glow".cyan());
        eprintln!(
            "  Or visit: {}",
            "https://github.com/charmbracelet/glow".dimmed()
        );
        return Ok(());
    }

    // ── Write note body to a temp file ────────────────────────────────────────
    let mut tmp = NamedTempFile::with_suffix(".md")?;
    write!(tmp, "{}", note.body)?;
    tmp.flush()?;

    // ── Open with glow ────────────────────────────────────────────────────────
    Command::new("glow")
        .arg(tmp.path())
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    Ok(())
}
