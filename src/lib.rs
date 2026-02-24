//! # rustodo
//!
//! A fast, powerful, and colorful task manager for the terminal
//!
//! ## Library structure
//!
//! | Module | Purpose |
//! |---|---|
//! | [`cli`] | Command-line argument definitions (clap) |
//! | [`commands`] | One submodule per CLI command |
//! | [`date_parser`] | Natural language and strict date parsing |
//! | [`display`] | Table rendering and formatting |
//! | [`error`] | Typed error variants via `thiserror` |
//! | [`models`] | Core domain types: `Task`, `Priority`, `Recurrence` |
//! | [`storage`] | Storage trait with JSON and in-memory implementations |
//! | [`tag_normalizer`] | Fuzzy tag normalization with Levenshtein distance |
//! | [`utils`] | Shared utilities (interactive confirmation prompt) |
//! | [`validation`] | Input validation for task fields |

pub mod cli;
pub mod commands;
pub mod date_parser;
pub mod display;
pub mod error;
pub mod models;
pub mod storage;
pub mod tag_normalizer;
pub mod utils;
pub mod validation;
