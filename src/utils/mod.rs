//! Shared terminal utilities.
//!
//! | Module | Purpose |
//! |---|---|
//! | [`confirm`] | Yes/no prompt for destructive operations |
//! | [`tag_normalizer`] | Fuzzy tag normalization with Levenshtein distance |
//! | [`date_parser`] |
//! | [`validation`] | Input validation for task fields |

pub mod confirm;
pub mod date_parser;
pub mod tag_normalizer;
pub mod validation;

pub use confirm::confirm;
