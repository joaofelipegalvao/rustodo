//! Application state for the TUI.

use crate::models::{Priority, Recurrence, StatusFilter, Task};
use crate::storage::Storage;
use anyhow::Result;

// ── Mode ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    ConfirmDelete,
    ConfirmClearAll,
    Search,
    /// Multi-field edit form shown in the right panel.
    EditForm,
    /// Blank form for creating a new task.
    AddForm,
    /// Keybindings help popup.
    Help,
}

// ── FocusedPanel ──────────────────────────────────────────────────────────────

/// Which panel currently has keyboard focus.
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum FocusedPanel {
    Left,
    Right,
}

impl FocusedPanel {
    pub fn toggle(self) -> Self {
        match self {
            FocusedPanel::Left => FocusedPanel::Right,
            FocusedPanel::Right => FocusedPanel::Left,
        }
    }
}

// ── RightPanel ────────────────────────────────────────────────────────────────

/// Which tab is shown in the right panel.
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum RightPanel {
    Details,
    Stats,
    Deps,
}

impl RightPanel {
    pub fn next(self) -> Self {
        match self {
            RightPanel::Details => RightPanel::Stats,
            RightPanel::Stats => RightPanel::Deps,
            RightPanel::Deps => RightPanel::Details,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            RightPanel::Details => RightPanel::Deps,
            RightPanel::Stats => RightPanel::Details,
            RightPanel::Deps => RightPanel::Stats,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            RightPanel::Details => "Details",
            RightPanel::Stats => "Stats",
            RightPanel::Deps => "Deps",
        }
    }
}

// ── EditField ─────────────────────────────────────────────────────────────────

/// Which field is focused in the edit form.
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum EditField {
    Text,
    Priority,
    Due,
    Recurrence,
    Project,
    Tags,
    Deps,
}

impl EditField {
    pub fn next(self) -> Self {
        match self {
            EditField::Text => EditField::Priority,
            EditField::Priority => EditField::Due,
            EditField::Due => EditField::Recurrence,
            EditField::Recurrence => EditField::Project,
            EditField::Project => EditField::Tags,
            EditField::Tags => EditField::Deps,
            EditField::Deps => EditField::Text,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            EditField::Text => EditField::Deps,
            EditField::Priority => EditField::Text,
            EditField::Due => EditField::Priority,
            EditField::Recurrence => EditField::Due,
            EditField::Project => EditField::Recurrence,
            EditField::Tags => EditField::Project,
            EditField::Deps => EditField::Tags,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            EditField::Text => "Text",
            EditField::Priority => "Priority",
            EditField::Due => "Due",
            EditField::Recurrence => "Recurrence",
            EditField::Project => "Project",
            EditField::Tags => "Tags",
            EditField::Deps => "Deps (IDs)",
        }
    }
}

/// State for the edit form.
#[derive(Debug, Clone)]
pub struct EditFormState {
    pub focused: EditField,
    pub text: String,
    pub priority: Priority,
    pub due: String,
    pub recurrence: Option<Recurrence>,
    pub project: String,
    pub tags: String,
    /// Comma-separated list of dep IDs (e.g. "1, 3")
    pub deps: String,
}

impl EditFormState {
    /// Create a blank form for new task creation.
    pub fn blank() -> Self {
        Self {
            focused: EditField::Text,
            text: String::new(),
            priority: Priority::Medium,
            due: String::new(),
            recurrence: None,
            project: String::new(),
            tags: String::new(),
            deps: String::new(),
        }
    }

    pub fn from_task(task: &Task, all_tasks: &[Task]) -> Self {
        // Resolve dep UUIDs → visible 1-based IDs
        let visible: Vec<&Task> = all_tasks.iter().filter(|t| !t.is_deleted()).collect();
        let deps = task
            .depends_on
            .iter()
            .filter_map(|uuid| {
                let pos = visible.iter().position(|t| t.uuid == *uuid)?;
                Some((pos + 1).to_string())
            })
            .collect::<Vec<_>>()
            .join(", ");

        Self {
            focused: EditField::Text,
            text: task.text.clone(),
            priority: task.priority,
            due: task
                .due_date
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default(),
            recurrence: task.recurrence,
            project: task.project.clone().unwrap_or_default(),
            tags: task.tags.join(", "),
            deps,
        }
    }

    /// Return a mutable reference to the string buffer of the focused field.
    pub fn focused_buf_mut(&mut self) -> Option<&mut String> {
        match self.focused {
            EditField::Text => Some(&mut self.text),
            EditField::Due => Some(&mut self.due),
            EditField::Project => Some(&mut self.project),
            EditField::Tags => Some(&mut self.tags),
            EditField::Deps => Some(&mut self.deps),
            // Selector fields — no text buffer
            EditField::Priority => None,
            EditField::Recurrence => None,
        }
    }

    pub fn priority_prev(&mut self) {
        self.priority = match self.priority {
            Priority::High => Priority::Low,
            Priority::Medium => Priority::High,
            Priority::Low => Priority::Medium,
        };
    }

    pub fn priority_next(&mut self) {
        self.priority = match self.priority {
            Priority::High => Priority::Medium,
            Priority::Medium => Priority::Low,
            Priority::Low => Priority::High,
        };
    }

    /// Cycle recurrence: None → Daily → Weekly → Monthly → None.
    pub fn recurrence_next(&mut self) {
        self.recurrence = match self.recurrence {
            None => Some(Recurrence::Daily),
            Some(Recurrence::Daily) => Some(Recurrence::Weekly),
            Some(Recurrence::Weekly) => Some(Recurrence::Monthly),
            Some(Recurrence::Monthly) => None,
        };
    }

    pub fn recurrence_prev(&mut self) {
        self.recurrence = match self.recurrence {
            None => Some(Recurrence::Monthly),
            Some(Recurrence::Daily) => None,
            Some(Recurrence::Weekly) => Some(Recurrence::Daily),
            Some(Recurrence::Monthly) => Some(Recurrence::Weekly),
        };
    }

    pub fn recurrence_label(&self) -> &'static str {
        match self.recurrence {
            None => "None",
            Some(Recurrence::Daily) => "Daily",
            Some(Recurrence::Weekly) => "Weekly",
            Some(Recurrence::Monthly) => "Monthly",
        }
    }
}

// ── ListFilter ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ListFilter {
    Pending,
    Done,
    All,
}

impl ListFilter {
    pub fn next(self) -> Self {
        match self {
            ListFilter::Pending => ListFilter::Done,
            ListFilter::Done => ListFilter::All,
            ListFilter::All => ListFilter::Pending,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ListFilter::Pending => "Pending",
            ListFilter::Done => "Done",
            ListFilter::All => "All",
        }
    }

    pub fn as_status_filter(self) -> StatusFilter {
        match self {
            ListFilter::Pending => StatusFilter::Pending,
            ListFilter::Done => StatusFilter::Done,
            ListFilter::All => StatusFilter::All,
        }
    }
}

// ── PriorityFilter ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum PriorityFilter {
    All,
    High,
    Medium,
    Low,
}

impl PriorityFilter {
    pub fn next(self) -> Self {
        match self {
            PriorityFilter::All => PriorityFilter::High,
            PriorityFilter::High => PriorityFilter::Medium,
            PriorityFilter::Medium => PriorityFilter::Low,
            PriorityFilter::Low => PriorityFilter::All,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            PriorityFilter::All => "All",
            PriorityFilter::High => "High",
            PriorityFilter::Medium => "Med",
            PriorityFilter::Low => "Low",
        }
    }
}

// ── App ───────────────────────────────────────────────────────────────────────

pub struct App {
    pub tasks: Vec<Task>,
    pub filtered_indices: Vec<usize>,
    pub selected: usize,
    pub mode: Mode,
    pub status_msg: Option<String>,
    pub details_scroll: usize,
    pub list_filter: ListFilter,
    pub priority_filter: PriorityFilter,
    /// Text input buffer for Search mode.
    pub input: String,
    /// State for the multi-field edit form.
    pub edit_form: Option<EditFormState>,
    /// Selected row in the help popup.
    pub help_selected: usize,
    /// Which tab is active in the right panel.
    pub right_panel: RightPanel,
    /// Which panel has keyboard focus.
    pub focused_panel: FocusedPanel,
}

impl App {
    pub fn new(storage: &impl Storage) -> Result<Self> {
        let tasks = Self::load_visible(storage)?;
        let filtered_indices = tasks
            .iter()
            .enumerate()
            .filter(|(_, t)| !t.completed)
            .map(|(i, _)| i)
            .collect();
        Ok(Self {
            tasks,
            filtered_indices,
            selected: 0,
            mode: Mode::Normal,
            status_msg: None,
            details_scroll: 0,
            list_filter: ListFilter::Pending,
            priority_filter: PriorityFilter::All,
            input: String::new(),
            edit_form: None,
            help_selected: 0,
            right_panel: RightPanel::Details,
            focused_panel: FocusedPanel::Left,
        })
    }

    pub fn reload(&mut self, storage: &impl Storage) -> Result<()> {
        self.tasks = Self::load_visible(storage)?;
        self.refilter();
        if self.selected >= self.filtered_indices.len() {
            self.selected = self.filtered_indices.len().saturating_sub(1);
        }
        Ok(())
    }

    pub fn refilter(&mut self) {
        let raw = self.input.to_lowercase();
        let status = self.list_filter.as_status_filter();

        // Parse search tokens: @project, #tag, and free text
        let mut project_filter: Option<String> = None;
        let mut tag_filters: Vec<String> = Vec::new();
        let mut text_tokens: Vec<String> = Vec::new();

        if self.mode == Mode::Search && !raw.is_empty() {
            for token in raw.split_whitespace() {
                if let Some(proj) = token.strip_prefix('@') {
                    project_filter = Some(proj.to_string());
                } else if let Some(tag) = token.strip_prefix('#') {
                    tag_filters.push(tag.to_string());
                } else {
                    text_tokens.push(token.to_string());
                }
            }
        }

        self.filtered_indices = self
            .tasks
            .iter()
            .enumerate()
            .filter(|(_, t)| t.matches_status(status))
            .filter(|(_, t)| match self.priority_filter {
                PriorityFilter::All => true,
                PriorityFilter::High => t.priority == Priority::High,
                PriorityFilter::Medium => t.priority == Priority::Medium,
                PriorityFilter::Low => t.priority == Priority::Low,
            })
            .filter(|(_, t)| {
                if self.mode != Mode::Search || raw.is_empty() {
                    return true;
                }
                // @project filter
                if let Some(ref pf) = project_filter {
                    match t.project.as_deref() {
                        Some(proj) if proj.to_lowercase().contains(pf.as_str()) => {}
                        _ => return false,
                    }
                }
                // #tag filters — all must match
                for tf in &tag_filters {
                    if !t
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(tf.as_str()))
                    {
                        return false;
                    }
                }
                // free text tokens — all must match task text
                for token in &text_tokens {
                    if !t.text.to_lowercase().contains(token.as_str()) {
                        return false;
                    }
                }
                true
            })
            .map(|(i, _)| i)
            .collect();
    }

    /// Open the edit form for the currently selected task.
    pub fn open_edit_form(&mut self) {
        if let Some(real) = self.selected_real_index() {
            let task = self.tasks[real].clone();
            let all_tasks = self.tasks.clone();
            self.edit_form = Some(EditFormState::from_task(&task, &all_tasks));
            self.mode = Mode::EditForm;
            self.status_msg = None;
        }
    }

    /// Open a blank form for creating a new task.
    pub fn open_add_form(&mut self) {
        self.edit_form = Some(EditFormState::blank());
        self.mode = Mode::AddForm;
        self.status_msg = None;
    }

    pub fn selected_task(&self) -> Option<&Task> {
        let real = *self.filtered_indices.get(self.selected)?;
        self.tasks.get(real)
    }

    pub fn selected_real_index(&self) -> Option<usize> {
        self.filtered_indices.get(self.selected).copied()
    }

    /// 1-based ID into the *full* visible list — used when calling commands.
    pub fn selected_visible_id(&self) -> Option<usize> {
        let real = self.selected_real_index()?;
        Some(real + 1)
    }

    pub fn move_down(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.selected = (self.selected + 1).min(self.filtered_indices.len() - 1);
            self.details_scroll = 0;
        }
    }

    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
        self.details_scroll = 0;
    }

    pub fn scroll_details_down(&mut self) {
        self.details_scroll = self.details_scroll.saturating_add(1);
    }

    pub fn scroll_details_up(&mut self) {
        self.details_scroll = self.details_scroll.saturating_sub(1);
    }

    pub fn cycle_status_filter(&mut self) {
        self.list_filter = self.list_filter.next();
        self.selected = 0;
        self.details_scroll = 0;
        self.refilter();
    }

    pub fn cycle_priority_filter(&mut self) {
        self.priority_filter = self.priority_filter.next();
        self.selected = 0;
        self.details_scroll = 0;
        self.refilter();
    }

    pub fn pending_count(&self) -> usize {
        self.tasks.iter().filter(|t| !t.completed).count()
    }

    pub fn total_count(&self) -> usize {
        self.tasks.len()
    }

    fn load_visible(storage: &impl Storage) -> Result<Vec<Task>> {
        Ok(storage
            .load()?
            .into_iter()
            .filter(|t| !t.is_deleted())
            .collect())
    }
}
