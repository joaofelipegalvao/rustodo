/*
A modern, powerful task manager built with Rust.

This CLI application provides a simple yet feature-rich interface for managing
todo tasks with support for priorities, tags, due dates, recurring patterns,
and advanced filtering.
*/

use std::process;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

mod cli;
mod commands;
mod date_parser;
mod display;
mod error;
mod models;
mod storage;
mod utils;
mod validation;

use cli::{Cli, Commands};

use crate::storage::{JsonStorage, Storage};

/// Main entry point for the todo-list application.
///
/// Parses command-line arguments and executes the requested command.
/// If an error occurs, it prints a formatted error message and exits
/// with a non-zero status code.
fn main() {
    let cli = Cli::parse();

    // Create storage (JSON in production)
    let storage = match JsonStorage::new() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{} Failed to initialize storage: {}", "✗".red(), e);
            process::exit(1);
        }
    };

    if let Err(e) = run(cli, &storage) {
        eprintln!("{} {}", "✗".red(), e);

        // Print the full error chain for better debugging
        let mut source = e.source();
        while let Some(cause) = source {
            eprintln!("  {} {}", "↳".red(), cause);
            source = cause.source();
        }

        process::exit(1);
    }
}

/// Main application logic dispatcher.
///
/// This function processes the parsed CLI arguments and executes the
/// appropriate command. It handles all the core functionality of the
/// todo application including adding, listing, completing, managing tasks,
/// and setting up recurring patterns.
///
/// # Errors
///
/// Returns an error if:
/// - File I/O operations fail
/// - Task validation fails
/// - No tasks match the specified filters
fn run(cli: Cli, storage: &impl Storage) -> Result<()> {
    match cli.command {
        Commands::Add(args) => {
            // Parse due date from string (supports natural language)
            let due_date = if let Some(due_str) = args.due {
                Some(date_parser::parse_date_not_in_past(&due_str)?)
            } else {
                None
            };

            commands::add::execute(
                storage,
                args.text,
                args.priority,
                args.tag,
                args.project,
                due_date,
                args.recurrence,
            )
        }

        Commands::List {
            status,
            priority,
            due,
            sort,
            tag,
            project,
            recurrence: recur,
        } => commands::list::execute(storage, status, priority, due, sort, tag, project, recur),

        Commands::Done { id } => commands::done::execute(storage, id),

        Commands::Undone { id } => commands::undone::execute(storage, id),

        Commands::Remove { id, yes } => commands::remove::execute(storage, id, yes),

        Commands::Edit {
            id,
            text,
            priority,
            add_tag,
            remove_tag,
            project,
            clear_project,
            due,
            clear_due,
            clear_tags,
        } => {
            // Parse due date from string (supports natural language)
            let due_date = if let Some(due_str) = due {
                Some(date_parser::parse_date(&due_str)?)
            } else {
                None
            };

            commands::edit::execute(
                storage,
                id,
                text,
                priority,
                add_tag,
                remove_tag,
                project,
                clear_project,
                due_date,
                clear_due,
                clear_tags,
            )
        }

        Commands::Clear { yes } => commands::clear::execute(storage, yes),

        Commands::Search {
            query,
            tag,
            project,
            status,
        } => commands::search::execute(storage, query, tag, project, status),

        Commands::Tags => commands::tags::execute(storage),

        Commands::Projects => commands::projects::execute(storage),

        Commands::Info => commands::info::execute(),

        Commands::Recur { id, pattern } => commands::recur::execute(storage, id, pattern),

        Commands::ClearRecur { id } => commands::clear_recur::execute(storage, id),
    }
}
