// ── models/mod.rs ─────────────────────────────────────────────────────────────

//! Core domain types for rustodo.
//!
//! | Type | Description |
//! |---|---|
//! | [`Task`] | A single todo item with all its metadata |
//! | [`Priority`] | High / Medium / Low priority levels |
//! | [`Recurrence`] | Daily / Weekly / Monthly repeat patterns |
//! | [`StatusFilter`] | Filter tasks by completion status |
//! | [`DueFilter`] | Filter tasks by due-date window |
//! | [`RecurrenceFilter`] | Filter tasks by recurrence pattern |
//! | [`SortBy`] | Sort order options for task lists |
//! | [`detect_cycle`] | Dependency cycle detection via iterative DFS |

mod filters;
mod priority;
mod recurrence;
mod task;

pub use filters::{DueFilter, RecurrenceFilter, SortBy, StatusFilter};
pub use priority::Priority;
pub use recurrence::Recurrence;
pub use task::{Task, detect_cycle};
