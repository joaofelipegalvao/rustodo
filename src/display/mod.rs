//! Terminal rendering for task lists.
//!
//! The public interface is a single function, [`display_lists`], which prints
//! a formatted, colorized table of tasks to stdout.
//!
//! Internally the module is split into two layers:
//!
//! - `formatting` — pure functions that convert a [`Task`] into colored
//!   strings (due-date text, checkbox, priority letter).
//! - **`table`** — computes column widths, renders header/rows/summary
//!   renders the header/separator/rows, and prints the completion summary.
//!
//! [`Task`]: crate::models::Task

mod formatting;
mod table;

pub use table::display_lists;
