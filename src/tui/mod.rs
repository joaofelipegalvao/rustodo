//! Terminal User Interface for rustodo.

pub mod app;
pub mod events;
pub mod style;
pub mod ui;

use anyhow::Result;
use ratatui::crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

use crate::config::Config;
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
    let cfg = Config::load().unwrap_or_default();
    let theme = cfg.theme.resolve();

    let mut app = app::App::new(storage)?;

    loop {
        terminal.draw(|f| ui::draw(f, &mut app, &theme))?;

        if events::handle(&mut app, storage)? {
            break;
        }
    }

    Ok(())
}
