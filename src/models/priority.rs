use clap::ValueEnum;
use colored::{ColoredString, Colorize};
use serde::{Deserialize, Serialize};

/// Priority levels for tasks.
///
/// Tasks can be categorized as High, Medium, or Low priority,
/// which affects their sorting order and visual presentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    /// High priority - urgent and important tasks
    High,
    /// Medium priority - default for most tasks
    Medium,
    /// Low priority - nice to have, not urgent
    Low,
}

impl Priority {
    /// Returns the sort order for this priority level.
    ///
    /// Lower numbers indicate higher priority.
    pub fn order(&self) -> u8 {
        match self {
            Priority::High => 0,
            Priority::Medium => 1,
            Priority::Low => 2,
        }
    }

    /// Returns a colored single-letter representation of this priority.
    ///
    /// - High: Red 'H'
    /// - Medium: Yellow 'M'
    /// - Low: Green 'L'
    pub fn letter(&self) -> ColoredString {
        match self {
            Priority::High => "H".red(),
            Priority::Medium => "M".yellow(),
            Priority::Low => "L".green(),
        }
    }
}
