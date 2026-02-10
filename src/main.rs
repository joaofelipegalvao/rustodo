/*
A modern, powerful task manager built with Rust.

This CLI application provides a simple yet feature-rich interface for managing
todo tasks with support for priorities, tags, due dates, and advanced filtering.
*/

use std::process;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

mod cli;
mod commands;
mod display;
mod error;
mod models;
mod storage;
mod utils;
mod validation;

use cli::{Cli, Commands};

/// Main entry point for the todo-list application.
///
/// Parses command-line arguments and executes the requested command.
/// If an error occurs, it prints a formatted error message and exits
/// with a non-zero status code.
fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("{} {}", "Error:".red().bold(), format!("{}", e).red());

        // Print the full error chain for better debugging
        let mut source = e.source();
        while let Some(cause) = source {
            eprintln!("{} {}", "Caused by:".red(), cause);
            source = cause.source();
        }

        process::exit(1);
    }
}

/// Main application logic dispatcher.
///
/// This function processes the parsed CLI arguments and executes the
/// appropriate command. It handles all the core functionality of the
/// todo application including adding, listing, completing, and managing tasks.
///
/// # Errors
///
/// Returns an error if:
/// - File I/O operations fail
/// - Task validation fails
/// - No tasks match the specified filters
fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Add(args) => commands::add::execute(args.text, args.priority, args.tag, args.due),

        Commands::List {
            status,
            priority,
            due,
            sort,
            tag,
        } => commands::list::execute(status, priority, due, sort, tag),

        Commands::Done { id } => commands::done::execute(id),

        Commands::Undone { id } => commands::undone::execute(id),

        Commands::Remove { id, yes } => commands::remove::execute(id, yes),

        Commands::Edit {
            id,
            text,
            priority,
            tag,
            due,
            clear_due,
            clear_tags,
        } => commands::edit::execute(id, text, priority, tag, due, clear_due, clear_tags),

        Commands::Clear { yes } => commands::clear::execute(yes),

        Commands::Search { query, tag } => commands::search::execute(query, tag),

        Commands::Tags => commands::tags::execute(),

        Commands::Info => commands::info::execute(),
    }
}
