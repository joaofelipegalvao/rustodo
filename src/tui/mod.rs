//! Terminal User Interface for rustodo.
//!
//! Launched when `todo` is invoked with no subcommand.
//! Built with [Ratatui](https://ratatui.rs) + Crossterm.
//!
//! # Layout
//!
//! ```text
//! ┌─ rustodo ──────────────────────────────────────────────────────┐
//! │ # │ P │  S  │ Task                  │ Project  │ Tags  │ Due   │
//! │───┼───┼─────┼───────────────────────┼──────────┼───────┼───────│
//! │ 1 │ H │ [ ] │ Fix login bug         │ Backend  │ work  │ 2d    │
//! │ 2 │ M │ [x] │ Write tests           │          │       │       │
//! └────────────────────────────────────────────────────────────────┘
//! ┌─ Details ──────────────────────────────────────────────────────┐
//! │ Fix login bug                                                   │
//! │ Priority : High                                                 │
//! │ Project  : Backend                                              │
//! │ Tags     : work                                                 │
//! │ Due      : 2026-03-05                                           │
//! │ UUID     : ef257b05-...                                         │
//! └────────────────────────────────────────────────────────────────┘
//! [j/k] navigate  [d] done/undone  [x] remove  [q] quit
//! ```

pub mod app;
pub mod events;
pub mod ui;

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

use crate::storage::Storage;

/// Entry point for the TUI. Sets up the terminal, runs the event loop,
/// and restores the terminal on exit (even on panic).
pub fn run(storage: &impl Storage) -> Result<()> {
    // ── setup ────────────────────────────────────────────────────────────────
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Ensure terminal is always restored, even on panic
    let result = run_app(&mut terminal, storage);

    // ── teardown ─────────────────────────────────────────────────────────────
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    storage: &impl Storage,
) -> Result<()> {
    let mut app = app::App::new(storage)?;

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        if events::handle(&mut app, storage)? {
            break;
        }
    }

    Ok(())
}
