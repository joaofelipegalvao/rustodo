//! Shared terminal utilities.
//!
//! Currently exposes a single helper, [`confirm`], which prints a yes/no
//! prompt and reads a single line from stdin. Used by [`commands::remove`]
//! and [`commands::clear`] before destructive operations.
//!
//! [`commands::remove`]: crate::commands::remove
//! [`commands::clear`]: crate::commands::clear

use anyhow::Result;
use colored::Colorize;
use std::io::{self, Write};

/// Prompts the user for confirmation.
///
/// # Arguments
///
/// * `message` - The confirmation message to display
///
/// # Returns
///
/// `true` if the user confirms (y/Y/yes), `false` otherwise
pub fn confirm(message: &str) -> Result<bool> {
    print!("{} ", message.yellow());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let response = input.trim().to_lowercase();
    Ok(matches!(response.as_str(), "y" | "yes"))
}
