//! Ratatui rendering.
//!
//! Layout:
//! ```text
//! ┌─ rustodo (3/7) [Pending] ──────────────┬─ Details — #1 ──────┐
//! │  # │ P │  S  │ Task               │ Due │  Fix login bug       │
//! │  1 │ H │ [ ] │ Fix login          │ 2d  │  Priority : High     │
//! └────────────────────────────────────────┴─────────────────────-┘
//! [j/k] nav  [d] done  [e] edit  [/] search  [Tab] filter  [q] quit
//! ```

use chrono::Local;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Clear, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState, Wrap,
    },
};

use crate::models::Task;

use super::app::{App, EditField, FocusedPanel, Mode, PriorityFilter, RightPanel};

// ── palette ───────────────────────────────────────────────────────────────────

const COLOR_HIGH: Color = Color::Red;
const COLOR_MEDIUM: Color = Color::Yellow;
const COLOR_LOW: Color = Color::Green;
const COLOR_DONE: Color = Color::DarkGray;
const COLOR_BLOCKED: Color = Color::Rgb(150, 150, 150);
const COLOR_SELECTED_BG: Color = Color::Rgb(40, 40, 60);
const COLOR_ACCENT: Color = Color::Cyan;
const COLOR_SEARCH_BG: Color = Color::Rgb(30, 30, 50);
const COLOR_FOCUSED_BG: Color = Color::Rgb(30, 40, 60);
const COLOR_FOCUSED_BORDER: Color = Color::Cyan;

// ── column visibility ─────────────────────────────────────────────────────────

struct Cols {
    project: bool,
    tags: bool,
    due: bool,
}

impl Cols {
    fn from(tasks: &[Task]) -> Self {
        Self {
            project: tasks.iter().any(|t| t.project.is_some()),
            tags: tasks.iter().any(|t| !t.tags.is_empty()),
            due: tasks.iter().any(|t| t.due_date.is_some()),
        }
    }
}

// ── entry point ───────────────────────────────────────────────────────────────

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(area);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(rows[0]);

    draw_task_list(f, app, cols[0]);

    // Right panel: tab bar (1 line) + content
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(cols[1]);

    draw_right_tabs(f, app, right_chunks[0]);

    match app.mode {
        Mode::EditForm => draw_edit_form(f, app, right_chunks[1], false),
        Mode::AddForm => draw_edit_form(f, app, right_chunks[1], true),
        _ => match app.right_panel {
            RightPanel::Details => draw_details(f, app, right_chunks[1]),
            RightPanel::Stats => draw_stats(f, app, right_chunks[1]),
            RightPanel::Deps => draw_deps(f, app, right_chunks[1]),
        },
    }

    draw_footer(f, app, rows[1]);

    if app.mode == Mode::Search {
        draw_input_overlay(f, app, rows[1]);
    }

    if app.mode == Mode::Help {
        draw_help_popup(f, app, area);
    }
}

// ── right panel tab bar ───────────────────────────────────────────────────────

fn draw_right_tabs(f: &mut Frame, app: &App, area: Rect) {
    if matches!(app.mode, Mode::EditForm | Mode::AddForm) {
        return;
    }

    let tabs = [RightPanel::Details, RightPanel::Stats, RightPanel::Deps];
    let mut spans: Vec<Span> = Vec::new();

    for (i, &tab) in tabs.iter().enumerate() {
        let is_active = app.right_panel == tab;

        if i > 0 {
            spans.push(Span::styled(" - ", Style::default().fg(Color::DarkGray)));
        }

        if is_active {
            spans.push(Span::styled(
                format!("[{}]", tab.label()),
                Style::default()
                    .fg(COLOR_ACCENT)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(
                tab.label(),
                Style::default().fg(Color::DarkGray),
            ));
        }
    }

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

// ── task list ─────────────────────────────────────────────────────────────────

fn draw_task_list(f: &mut Frame, app: &mut App, area: Rect) {
    let c = Cols::from(&app.tasks);

    let current = if app.filtered_indices.is_empty() {
        0
    } else {
        app.selected + 1
    };
    let total_filtered = app.filtered_indices.len();
    let filter_label = app.list_filter.label();
    let title = if app.priority_filter == PriorityFilter::All {
        format!(
            " rustodo ({}/{}) [{}] ",
            current, total_filtered, filter_label
        )
    } else {
        format!(
            " rustodo ({}/{}) [{} | P:{}] ",
            current,
            total_filtered,
            filter_label,
            app.priority_filter.label()
        )
    };

    let mut header_cells = vec![
        Cell::from("S").style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Task").style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
    ];
    if c.project {
        header_cells.push(
            Cell::from("Project").style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
        );
    }
    if c.tags {
        header_cells.push(
            Cell::from("Tags").style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
        );
    }
    if c.due {
        header_cells.push(
            Cell::from("Due").style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
        );
    }
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = app
        .filtered_indices
        .iter()
        .map(|&real_idx| task_row(&app.tasks[real_idx], &app.tasks, &c))
        .collect();

    let mut widths = vec![
        Constraint::Length(3), // status P/B/D
        Constraint::Min(10),   // task text
    ];
    if c.project {
        widths.push(Constraint::Length(12));
    }
    if c.tags {
        widths.push(Constraint::Length(12));
    }
    if c.due {
        widths.push(Constraint::Length(8));
    }

    let mut state = TableState::default();
    if !app.filtered_indices.is_empty() {
        state.select(Some(app.selected));
    }

    let border_style = if app.focused_panel == FocusedPanel::Left {
        Style::default().fg(COLOR_ACCENT)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style),
        )
        .row_highlight_style(
            Style::default()
                .bg(COLOR_SELECTED_BG)
                .add_modifier(Modifier::BOLD),
        )
        .column_spacing(1);

    f.render_stateful_widget(table, area, &mut state);
}

fn task_row<'a>(task: &'a Task, all_tasks: &'a [Task], c: &Cols) -> Row<'a> {
    let blocked = !task.completed && task.is_blocked(all_tasks);

    let row_style = if task.completed {
        Style::default().fg(COLOR_DONE)
    } else if blocked {
        Style::default().fg(COLOR_BLOCKED)
    } else {
        Style::default().fg(Color::White)
    };

    // Status: single letter colored — P/B/D like lazygit M/A/D
    let (status_text, status_color) = if task.completed {
        ("D", Color::Green)
    } else if blocked {
        ("B", Color::Red)
    } else {
        ("P", Color::Blue)
    };
    let s_cell = Cell::from(status_text).style(Style::default().fg(status_color));

    let text_cell = Cell::from(truncate(&task.text, 36)).style(row_style);
    let mut cells = vec![s_cell, text_cell];

    if c.project {
        let proj = task.project.as_deref().unwrap_or("");
        let proj_style = if task.completed {
            Style::default().fg(COLOR_DONE).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD)
        };
        cells.push(Cell::from(truncate(proj, 11)).style(proj_style));
    }
    if c.tags {
        let tags = if task.tags.is_empty() {
            String::new()
        } else {
            truncate(&task.tags.join(", "), 11)
        };
        let tags_style = if task.completed {
            Style::default().fg(COLOR_DONE).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(COLOR_ACCENT)
                .add_modifier(Modifier::BOLD)
        };
        cells.push(Cell::from(tags).style(tags_style));
    }
    if c.due {
        cells.push(
            Cell::from(due_display(task)).style(due_style(task).add_modifier(Modifier::BOLD)),
        );
    }

    Row::new(cells).style(row_style).height(1)
}

// ── details panel ─────────────────────────────────────────────────────────────

fn draw_details(f: &mut Frame, app: &App, area: Rect) {
    let content: Vec<Line> = match app.selected_task() {
        None => vec![Line::from(Span::styled(
            "No tasks",
            Style::default().fg(Color::DarkGray),
        ))],
        Some(task) => build_details(task, &app.tasks),
    };

    let inner_height = area.height.saturating_sub(2) as usize;
    let content_len = content.len();
    let max_scroll = content_len.saturating_sub(inner_height);
    let scroll = app.details_scroll.min(max_scroll);

    let border_style = if app.focused_panel == FocusedPanel::Right {
        Style::default().fg(COLOR_ACCENT)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let para = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .wrap(Wrap { trim: true })
        .scroll((scroll as u16, 0));
    f.render_widget(para, area);

    if content_len > inner_height {
        let mut ss = ScrollbarState::new(max_scroll).position(scroll);
        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            area,
            &mut ss,
        );
    }
}

fn build_details(task: &Task, all_tasks: &[Task]) -> Vec<Line<'static>> {
    let label = |s: &str| Span::styled(format!("{:<10}", s), Style::default().fg(Color::DarkGray));
    let value = |s: String| Span::styled(s, Style::default().fg(Color::White));
    let accent = |s: String| Span::styled(s, Style::default().fg(COLOR_ACCENT));

    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            task.text.clone(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            label("Priority"),
            match task.priority {
                crate::models::Priority::High => Span::styled(
                    "High",
                    Style::default().fg(COLOR_HIGH).add_modifier(Modifier::BOLD),
                ),
                crate::models::Priority::Medium => {
                    Span::styled("Medium", Style::default().fg(COLOR_MEDIUM))
                }
                crate::models::Priority::Low => Span::styled("Low", Style::default().fg(COLOR_LOW)),
            },
        ]),
        Line::from(vec![
            label("Status"),
            if task.completed {
                Span::styled("Done", Style::default().fg(COLOR_DONE))
            } else {
                Span::styled("Pending", Style::default().fg(Color::Green))
            },
        ]),
    ];

    if let Some(ref p) = task.project {
        lines.push(Line::from(vec![label("Project"), accent(p.clone())]));
    }
    if !task.tags.is_empty() {
        lines.push(Line::from(vec![label("Tags"), value(task.tags.join(", "))]));
    }
    if let Some(due) = task.due_date {
        lines.push(Line::from(vec![
            label("Due"),
            value(due.format("%Y-%m-%d").to_string()),
        ]));
    }
    if let Some(rec) = task.recurrence {
        lines.push(Line::from(vec![label("Recurs"), value(format!("{}", rec))]));
    }
    if !task.depends_on.is_empty() {
        let visible: Vec<&Task> = all_tasks.iter().filter(|t| !t.is_deleted()).collect();
        let dep_labels: Vec<String> = task
            .depends_on
            .iter()
            .filter_map(|dep_uuid| {
                let pos = visible.iter().position(|t| t.uuid == *dep_uuid)?;
                Some(format!("#{}", pos + 1))
            })
            .collect();
        let blocked = task.is_blocked(all_tasks);
        lines.push(Line::from(vec![
            label("Deps"),
            Span::styled(
                dep_labels.join(", "),
                if blocked {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default().fg(Color::White)
                },
            ),
        ]));
    }
    lines.push(Line::from(vec![
        label("Created"),
        value(task.created_at.format("%Y-%m-%d").to_string()),
    ]));
    if task.completed
        && let Some(done_at) = task.completed_at
    {
        lines.push(Line::from(vec![
            label("Completed"),
            value(done_at.format("%Y-%m-%d").to_string()),
        ]));
    }

    lines
}

// ── edit form panel ───────────────────────────────────────────────────────────

fn draw_edit_form(f: &mut Frame, app: &App, area: Rect, is_add: bool) {
    let form = match &app.edit_form {
        Some(f) => f,
        None => return,
    };

    let title = if is_add {
        " New Task ".to_string()
    } else {
        format!(" Edit — #{} ", app.selected_visible_id().unwrap_or(0))
    };

    let all_fields = [
        EditField::Text,
        EditField::Priority,
        EditField::Due,
        EditField::Recurrence,
        EditField::Project,
        EditField::Tags,
        EditField::Deps,
    ];
    let fields: &[EditField] = &all_fields;

    let inner = Block::default().borders(Borders::ALL).title(title);
    let inner_area = inner.inner(area);
    f.render_widget(inner, area);

    let constraints: Vec<Constraint> = fields.iter().map(|_| Constraint::Length(3)).collect();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner_area);

    for (i, &field) in fields.iter().enumerate() {
        let chunk = chunks[i];
        let focused = form.focused == field;

        let label_style = if focused {
            Style::default()
                .fg(COLOR_FOCUSED_BORDER)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let input_bg = if focused {
            COLOR_FOCUSED_BG
        } else {
            Color::Reset
        };

        let label_line = Line::from(Span::styled(format!(" {} ", field.label()), label_style));

        let input_line = if field == EditField::Priority {
            let (h_s, m_s, l_s) = match form.priority {
                crate::models::Priority::High => (
                    Style::default()
                        .fg(COLOR_HIGH)
                        .add_modifier(Modifier::BOLD | Modifier::REVERSED),
                    Style::default().fg(Color::DarkGray),
                    Style::default().fg(Color::DarkGray),
                ),
                crate::models::Priority::Medium => (
                    Style::default().fg(Color::DarkGray),
                    Style::default()
                        .fg(COLOR_MEDIUM)
                        .add_modifier(Modifier::BOLD | Modifier::REVERSED),
                    Style::default().fg(Color::DarkGray),
                ),
                crate::models::Priority::Low => (
                    Style::default().fg(Color::DarkGray),
                    Style::default().fg(Color::DarkGray),
                    Style::default()
                        .fg(COLOR_LOW)
                        .add_modifier(Modifier::BOLD | Modifier::REVERSED),
                ),
            };
            let arrow_style = if focused {
                Style::default().fg(COLOR_ACCENT)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            Line::from(vec![
                Span::styled(" ◀ ", arrow_style),
                Span::styled(" High ", h_s),
                Span::raw("  "),
                Span::styled(" Medium ", m_s),
                Span::raw("  "),
                Span::styled(" Low ", l_s),
                Span::styled(" ▶ ", arrow_style),
            ])
        } else if field == EditField::Recurrence {
            let options = ["None", "Daily", "Weekly", "Monthly"];
            let current = form.recurrence_label();
            let arrow_style = if focused {
                Style::default().fg(COLOR_ACCENT)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let mut spans = vec![Span::styled(" ◀ ", arrow_style)];
            for opt in &options {
                let s = if *opt == current {
                    Style::default()
                        .fg(COLOR_ACCENT)
                        .add_modifier(Modifier::BOLD | Modifier::REVERSED)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                spans.push(Span::styled(format!(" {} ", opt), s));
                spans.push(Span::raw("  "));
            }
            spans.push(Span::styled(" ▶ ", arrow_style));
            Line::from(spans)
        } else {
            let buf = match field {
                EditField::Text => &form.text,
                EditField::Due => &form.due,
                EditField::Project => &form.project,
                EditField::Tags => &form.tags,
                EditField::Deps => &form.deps,
                EditField::Priority | EditField::Recurrence => unreachable!(),
            };
            let cursor = if focused { "█" } else { "" };
            Line::from(vec![Span::styled(
                format!(" {}{} ", buf, cursor),
                Style::default().fg(Color::White).bg(input_bg),
            )])
        };

        let block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(if focused {
                Style::default().fg(COLOR_FOCUSED_BORDER)
            } else {
                Style::default().fg(Color::Rgb(50, 50, 50))
            });

        let para = Paragraph::new(vec![label_line, input_line]).block(block);
        f.render_widget(para, chunk);
    }
}

// ── deps panel ────────────────────────────────────────────────────────────────

fn draw_deps(f: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.focused_panel == FocusedPanel::Right {
        Style::default().fg(COLOR_ACCENT)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let tasks = &app.tasks;
    let visible: Vec<(usize, &Task)> = tasks
        .iter()
        .enumerate()
        .filter(|(_, t)| !t.is_deleted())
        .collect();

    let mut lines: Vec<Line> = Vec::new();

    match app.selected_task() {
        None => {
            lines.push(Line::from(Span::styled(
                "No task selected",
                Style::default().fg(Color::DarkGray),
            )));
        }
        Some(task) => {
            let task_vis_id = app.selected_visible_id().unwrap_or(0);

            // Title
            lines.push(Line::from(vec![
                Span::styled("Task #", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{}: ", task_vis_id),
                    Style::default()
                        .fg(COLOR_ACCENT)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    truncate(&task.text, 30),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(Line::from(""));

            // ── Depends on (upstream) ─────────────────────────────────────────
            lines.push(Line::from(Span::styled(
                "Depends on:",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));

            if task.depends_on.is_empty() {
                lines.push(Line::from(Span::styled(
                    "  No dependencies.",
                    Style::default().fg(Color::DarkGray),
                )));
            } else {
                for dep_uuid in &task.depends_on {
                    if let Some(vis_id) = visible.iter().position(|(_, t)| t.uuid == *dep_uuid) {
                        let dep_task = visible[vis_id].1;
                        let num = vis_id + 1;
                        let is_done = dep_task.completed;
                        lines.push(Line::from(vec![
                            Span::styled(
                                if is_done { "  ✓ " } else { "  ◦ " },
                                Style::default().fg(if is_done {
                                    Color::Green
                                } else {
                                    Color::Red
                                }),
                            ),
                            Span::styled(
                                format!("#{} — ", num),
                                Style::default().fg(Color::DarkGray),
                            ),
                            Span::styled(
                                truncate(&dep_task.text, 28),
                                Style::default().fg(if is_done {
                                    COLOR_DONE
                                } else {
                                    Color::White
                                }),
                            ),
                        ]));
                    }
                }
            }

            // ── Required by (downstream) ──────────────────────────────────────
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Required by:",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));

            let downstream: Vec<(usize, &Task)> = visible
                .iter()
                .filter(|(_, t)| t.depends_on.contains(&task.uuid))
                .map(|(i, t)| (*i, *t))
                .collect();

            if downstream.is_empty() {
                lines.push(Line::from(Span::styled(
                    "  No tasks depend on this one.",
                    Style::default().fg(Color::DarkGray),
                )));
            } else {
                for (_, dep_task) in &downstream {
                    let vis_id = visible
                        .iter()
                        .position(|(_, t)| t.uuid == dep_task.uuid)
                        .map(|p| p + 1)
                        .unwrap_or(0);
                    let is_done = dep_task.completed;
                    lines.push(Line::from(vec![
                        Span::styled(
                            if is_done { "  ✓ " } else { "  ◦ " },
                            Style::default().fg(if is_done { Color::Green } else { Color::Yellow }),
                        ),
                        Span::styled(
                            format!("#{} — ", vis_id),
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::styled(
                            truncate(&dep_task.text, 28),
                            Style::default().fg(if is_done { COLOR_DONE } else { Color::White }),
                        ),
                    ]));
                }
            }

            // ── Status ────────────────────────────────────────────────────────
            lines.push(Line::from(""));
            let is_blocked = task.is_blocked(tasks);
            if is_blocked {
                let blockers: Vec<String> = task
                    .depends_on
                    .iter()
                    .filter_map(|uuid| {
                        let pos = visible.iter().position(|(_, t)| t.uuid == *uuid)?;
                        let t = visible[pos].1;
                        if !t.completed {
                            Some(format!("#{}", pos + 1))
                        } else {
                            None
                        }
                    })
                    .collect();
                lines.push(Line::from(vec![
                    Span::styled("[~] ", Style::default().fg(COLOR_BLOCKED)),
                    Span::styled("Blocked by: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        blockers.join(", "),
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                ]));
            } else if !task.completed {
                lines.push(Line::from(Span::styled(
                    "[ ] Not blocked",
                    Style::default().fg(Color::Green),
                )));
            } else {
                lines.push(Line::from(Span::styled(
                    "[x] Completed",
                    Style::default().fg(COLOR_DONE),
                )));
            }
        }
    }

    let inner_height = area.height.saturating_sub(2) as usize;
    let content_len = lines.len();
    let max_scroll = content_len.saturating_sub(inner_height);
    let scroll = app.details_scroll.min(max_scroll);

    f.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .wrap(Wrap { trim: true })
            .scroll((scroll as u16, 0)),
        area,
    );
}

// ── stats panel ───────────────────────────────────────────────────────────────

fn draw_stats(f: &mut Frame, app: &App, area: Rect) {
    let tasks = &app.tasks;
    let total = tasks.len();
    let completed = tasks.iter().filter(|t| t.completed).count();
    let pending = total - completed;
    let overdue = tasks.iter().filter(|t| t.is_overdue()).count();
    let blocked = tasks
        .iter()
        .filter(|t| !t.completed && t.is_blocked(tasks))
        .count();
    let pct = if total == 0 {
        0
    } else {
        (completed * 100) / total
    };

    let high = tasks
        .iter()
        .filter(|t| t.priority == crate::models::Priority::High && !t.completed)
        .count();
    let medium = tasks
        .iter()
        .filter(|t| t.priority == crate::models::Priority::Medium && !t.completed)
        .count();
    let low = tasks
        .iter()
        .filter(|t| t.priority == crate::models::Priority::Low && !t.completed)
        .count();

    // ── Projects breakdown ────────────────────────────────────────────────────
    let mut project_map: std::collections::BTreeMap<String, (usize, usize)> =
        std::collections::BTreeMap::new();
    for t in tasks.iter() {
        if let Some(ref p) = t.project {
            let e = project_map.entry(p.clone()).or_insert((0, 0));
            if t.completed {
                e.1 += 1;
            } else {
                e.0 += 1;
            }
        }
    }

    // ── Tags breakdown ────────────────────────────────────────────────────────
    let mut tag_map: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for t in tasks.iter().filter(|t| !t.completed) {
        for tag in &t.tags {
            *tag_map.entry(tag.clone()).or_insert(0) += 1;
        }
    }

    let bar_width = (area.width as usize).saturating_sub(6).min(20);
    let filled = if total == 0 {
        0
    } else {
        (completed * bar_width) / total
    };
    let bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(bar_width - filled));
    let bar_color = if pct == 100 {
        Color::Green
    } else if pct >= 50 {
        Color::Yellow
    } else {
        Color::Red
    };

    let today = Local::now().naive_local().date();
    let activity: Vec<(String, usize)> = (0..7)
        .rev()
        .map(|i| {
            let date = today - chrono::Duration::days(i);
            let count = tasks
                .iter()
                .filter(|t| t.completed_at == Some(date))
                .count();
            (date.format("%b %d").to_string(), count)
        })
        .collect();
    let max_act = activity.iter().map(|(_, c)| *c).max().unwrap_or(1).max(1);

    let lbl = |s: &str| Span::styled(format!("{:<12}", s), Style::default().fg(Color::DarkGray));
    let section = |s: &str| {
        Line::from(Span::styled(
            s.to_string(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
    };

    let mut lines: Vec<Line> = vec![
        section("Overview"),
        Line::from(""),
        Line::from(vec![
            lbl("Progress"),
            Span::styled(bar, Style::default().fg(bar_color)),
            Span::styled(
                format!(" {}%", pct),
                Style::default().fg(bar_color).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            lbl("Total"),
            Span::styled(total.to_string(), Style::default().fg(COLOR_ACCENT)),
        ]),
        Line::from(vec![
            lbl("Pending"),
            Span::styled(pending.to_string(), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            lbl("Completed"),
            Span::styled(completed.to_string(), Style::default().fg(Color::Green)),
        ]),
    ];
    if overdue > 0 {
        lines.push(Line::from(vec![
            lbl("Overdue"),
            Span::styled(
                overdue.to_string(),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        ]));
    }
    if blocked > 0 {
        lines.push(Line::from(vec![
            lbl("Blocked"),
            Span::styled(blocked.to_string(), Style::default().fg(COLOR_BLOCKED)),
        ]));
    }

    // ── By Priority ───────────────────────────────────────────────────────────
    lines.push(Line::from(""));
    lines.push(section("By Priority (pending)"));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        lbl("High"),
        Span::styled(high.to_string(), Style::default().fg(COLOR_HIGH)),
    ]));
    lines.push(Line::from(vec![
        lbl("Medium"),
        Span::styled(medium.to_string(), Style::default().fg(COLOR_MEDIUM)),
    ]));
    lines.push(Line::from(vec![
        lbl("Low"),
        Span::styled(low.to_string(), Style::default().fg(COLOR_LOW)),
    ]));

    // ── By Project ────────────────────────────────────────────────────────────
    if !project_map.is_empty() {
        lines.push(Line::from(""));
        lines.push(section("By Project"));
        lines.push(Line::from(""));
        let max_name = project_map
            .keys()
            .map(|k| k.len())
            .max()
            .unwrap_or(8)
            .min(16);
        for (name, (pend, done)) in &project_map {
            let proj_total = pend + done;
            let proj_pct = (done * 100) / proj_total;
            let truncated = truncate(name, max_name + 1);
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{:<width$}", truncated, width = max_name + 2),
                    Style::default().fg(Color::Magenta),
                ),
                Span::styled(
                    format!("{} pending", pend),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("  {} done", done),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("  ({}%)", proj_pct),
                    Style::default().fg(if proj_pct == 100 {
                        Color::Green
                    } else {
                        Color::DarkGray
                    }),
                ),
            ]));
        }
    }

    // ── By Tag ────────────────────────────────────────────────────────────────
    if !tag_map.is_empty() {
        lines.push(Line::from(""));
        lines.push(section("Tags (pending)"));
        lines.push(Line::from(""));
        let max_tag = tag_map.keys().map(|k| k.len()).max().unwrap_or(6).min(16);
        let mut tags_sorted: Vec<(&String, &usize)> = tag_map.iter().collect();
        tags_sorted.sort_by(|a, b| b.1.cmp(a.1));
        for (tag, count) in tags_sorted {
            let truncated = truncate(tag, max_tag + 1);
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{:<width$}", truncated, width = max_tag + 2),
                    Style::default().fg(COLOR_ACCENT),
                ),
                Span::styled(
                    format!("{} task{}", count, if *count == 1 { "" } else { "s" }),
                    Style::default().fg(Color::White),
                ),
            ]));
        }
    }

    // ── Last 7 days ───────────────────────────────────────────────────────────
    lines.push(Line::from(""));
    lines.push(section("Last 7 days"));
    lines.push(Line::from(""));

    let act_bar_w = (area.width as usize).saturating_sub(14).min(10);
    for (label, count) in &activity {
        let filled = (count * act_bar_w) / max_act;
        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(act_bar_w - filled));
        let bar_style = if *count == 0 {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Green)
        };
        lines.push(Line::from(vec![
            Span::styled(format!("{} ", label), Style::default().fg(Color::DarkGray)),
            Span::styled(bar, bar_style),
            Span::styled(
                format!(" {}", count),
                Style::default().fg(if *count == 0 {
                    Color::DarkGray
                } else {
                    Color::White
                }),
            ),
        ]));
    }

    let inner_height = area.height.saturating_sub(2) as usize;
    let content_len = lines.len();
    let max_scroll = content_len.saturating_sub(inner_height);
    let scroll = app.details_scroll.min(max_scroll);

    let border_style = if app.focused_panel == FocusedPanel::Right {
        Style::default().fg(COLOR_ACCENT)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    f.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .wrap(Wrap { trim: false })
            .scroll((scroll as u16, 0)),
        area,
    );
}

// ── search overlay ────────────────────────────────────────────────────────────

fn draw_input_overlay(f: &mut Frame, app: &App, area: Rect) {
    f.render_widget(Clear, area);
    let content = Line::from(vec![
        Span::styled(
            "Search: ",
            Style::default()
                .fg(COLOR_ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(app.input.clone(), Style::default().fg(Color::White)),
        Span::styled("█", Style::default().fg(COLOR_ACCENT)),
    ]);
    f.render_widget(
        Paragraph::new(content).style(Style::default().bg(COLOR_SEARCH_BG)),
        area,
    );
}

// ── footer ────────────────────────────────────────────────────────────────────

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    if app.mode == Mode::Search {
        return;
    }

    let text = if let Some(ref msg) = app.status_msg {
        let color = if app.mode == Mode::ConfirmDelete {
            Color::Yellow
        } else {
            Color::Green
        };
        Line::from(Span::styled(msg.clone(), Style::default().fg(color)))
    } else {
        match app.mode {
            Mode::ConfirmDelete => Line::from(vec![
                Span::styled(
                    "[y]",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" confirm  "),
                Span::styled("[n]", Style::default().fg(COLOR_ACCENT)),
                Span::raw(" cancel"),
            ]),
            Mode::EditForm | Mode::AddForm => Line::from(vec![
                Span::styled("[Tab]", Style::default().fg(COLOR_ACCENT)),
                Span::raw(" next  "),
                Span::styled("[S-Tab]", Style::default().fg(COLOR_ACCENT)),
                Span::raw(" prev  "),
                Span::styled("[←/→]", Style::default().fg(COLOR_ACCENT)),
                Span::raw(" priority  "),
                Span::styled("[Enter]", Style::default().fg(Color::Green)),
                Span::raw(" save  "),
                Span::styled("[Esc]", Style::default().fg(Color::Red)),
                Span::raw(" cancel"),
            ]),
            _ => Line::from(vec![
                Span::styled("Navigate: ", Style::default().fg(Color::DarkGray)),
                Span::styled("j/k", Style::default().fg(COLOR_ACCENT)),
                Span::raw("  "),
                Span::styled("Focus: ", Style::default().fg(Color::DarkGray)),
                Span::styled("Tab", Style::default().fg(COLOR_ACCENT)),
                Span::raw("  "),
                Span::styled("Switch tab: ", Style::default().fg(Color::DarkGray)),
                Span::styled("[ ]", Style::default().fg(COLOR_ACCENT)),
                Span::raw("  "),
                Span::styled("Add: ", Style::default().fg(Color::DarkGray)),
                Span::styled("a", Style::default().fg(Color::Green)),
                Span::raw("  "),
                Span::styled("Edit: ", Style::default().fg(Color::DarkGray)),
                Span::styled("e", Style::default().fg(Color::Yellow)),
                Span::raw("  "),
                Span::styled("Done: ", Style::default().fg(Color::DarkGray)),
                Span::styled("d", Style::default().fg(COLOR_ACCENT)),
                Span::raw("  "),
                Span::styled("Delete: ", Style::default().fg(Color::DarkGray)),
                Span::styled("x", Style::default().fg(Color::Red)),
                Span::raw("  "),
                Span::styled("Quit: ", Style::default().fg(Color::DarkGray)),
                Span::styled("q", Style::default().fg(COLOR_ACCENT)),
                Span::raw("  "),
                Span::styled("Keybindings: ", Style::default().fg(Color::DarkGray)),
                Span::styled("?", Style::default().fg(COLOR_ACCENT)),
            ]),
        }
    };

    f.render_widget(Paragraph::new(text), area);
}

// ── help popup ────────────────────────────────────────────────────────────────

struct HelpEntry {
    key: &'static str,
    action: &'static str,
    description: Option<&'static str>,
}

fn help_entries() -> Vec<HelpEntry> {
    vec![
        HelpEntry {
            key: "──── Navigation",
            action: "",
            description: None,
        },
        HelpEntry {
            key: "j / k",
            action: "Move cursor down / up",
            description: None,
        },
        HelpEntry {
            key: "↑ / ↓",
            action: "Scroll details panel",
            description: Some(
                "Scrolls the right-hand details or stats panel without moving the selected task.",
            ),
        },
        HelpEntry {
            key: "g / G",
            action: "Jump to first / last task",
            description: None,
        },
        HelpEntry {
            key: "Tab",
            action: "Toggle panel focus",
            description: Some(
                "Moves focus between the task list (left) and the details/stats panel (right). The focused panel has a cyan border.",
            ),
        },
        HelpEntry {
            key: "[ / ]",
            action: "Cycle right panel tabs",
            description: Some(
                "Switches the right panel between Details, Stats and Deps. [ goes back, ] goes forward. Works from either panel.",
            ),
        },
        HelpEntry {
            key: "──── Actions",
            action: "",
            description: None,
        },
        HelpEntry {
            key: "a",
            action: "Add new task",
            description: Some(
                "Opens a blank form in the right panel. Fill in fields with Tab, confirm with Enter.",
            ),
        },
        HelpEntry {
            key: "e",
            action: "Edit selected task",
            description: Some(
                "Opens the edit form pre-filled with the selected task's current values. Tab navigates fields, Enter saves.",
            ),
        },
        HelpEntry {
            key: "d",
            action: "Toggle done / undone",
            description: Some(
                "Marks a pending task as completed. If the task has recurrence, a new occurrence is created automatically.",
            ),
        },
        HelpEntry {
            key: "x",
            action: "Delete task",
            description: Some(
                "Prompts for confirmation [y/n]. Tasks are soft-deleted so sync can propagate the deletion to other devices.",
            ),
        },
        HelpEntry {
            key: "X",
            action: "Clear all tasks",
            description: Some(
                "Prompts for confirmation [y/n]. Deletes all tasks currently visible (respects active filters).",
            ),
        },
        HelpEntry {
            key: "──── Filters",
            action: "",
            description: None,
        },
        HelpEntry {
            key: "f",
            action: "Cycle status filter",
            description: Some(
                "Cycles through Pending → Done → All. The active filter is shown in the list title.",
            ),
        },
        HelpEntry {
            key: "p",
            action: "Cycle priority filter",
            description: Some(
                "Cycles through All → High → Medium → Low. Combine with f to narrow down tasks.",
            ),
        },
        HelpEntry {
            key: "/",
            action: "Search tasks",
            description: Some(
                "Opens an inline search bar. Results filter live as you type. Enter confirms, Esc cancels and clears the filter.",
            ),
        },
        HelpEntry {
            key: "──── General",
            action: "",
            description: None,
        },
        HelpEntry {
            key: "?",
            action: "Open this help",
            description: None,
        },
        HelpEntry {
            key: "Esc",
            action: "Close popup / cancel",
            description: None,
        },
        HelpEntry {
            key: "q",
            action: "Quit",
            description: None,
        },
    ]
}

fn draw_help_popup(f: &mut Frame, app: &App, area: Rect) {
    let entries = help_entries();

    let selectable: Vec<usize> = entries
        .iter()
        .enumerate()
        .filter(|(_, e)| !e.action.is_empty())
        .map(|(i, _)| i)
        .collect();
    let selectable_count = selectable.len();

    let sel_pos = app.help_selected.min(selectable_count.saturating_sub(1));
    let sel_real = selectable[sel_pos];

    let desc_text = entries[sel_real].description.unwrap_or("");
    let desc_h = 4u16;

    // Fixed dimensions — never changes size while navigating
    let popup_w = (area.width as f32 * 0.70) as u16;
    let popup_h = (area.height as f32 * 0.80) as u16;
    let popup_x = (area.width.saturating_sub(popup_w)) / 2;
    let popup_y = (area.height.saturating_sub(popup_h)) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_w, popup_h);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(desc_h)])
        .split(popup_area);

    f.render_widget(Clear, popup_area);

    let inner_list_h = chunks[0].height.saturating_sub(2) as usize;

    let scroll = if sel_real >= inner_list_h {
        sel_real - inner_list_h + 1
    } else {
        0
    };

    let rows: Vec<Row> = entries
        .iter()
        .enumerate()
        .skip(scroll)
        .take(inner_list_h)
        .map(|(i, e)| {
            let is_section = e.action.is_empty();
            let is_selected = i == sel_real;

            if is_section {
                Row::new(vec![
                    Cell::from(e.key).style(
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Cell::from(""),
                ])
            } else {
                let key_style = Style::default()
                    .fg(COLOR_ACCENT)
                    .add_modifier(if is_selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    });
                let action_style = Style::default()
                    .fg(Color::White)
                    .add_modifier(if is_selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    });
                let row_style = if is_selected {
                    Style::default().bg(COLOR_SELECTED_BG)
                } else {
                    Style::default()
                };
                Row::new(vec![
                    Cell::from(format!("  {}", e.key)).style(key_style),
                    Cell::from(e.action).style(action_style),
                ])
                .style(row_style)
            }
        })
        .collect();

    let counter = format!(" {} of {} ", sel_pos + 1, selectable_count);

    let table = Table::new(rows, [Constraint::Length(18), Constraint::Min(10)]).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Keybindings ")
            .title_bottom(
                Line::from(vec![Span::styled(
                    counter,
                    Style::default().fg(Color::DarkGray),
                )])
                .right_aligned(),
            )
            .border_style(Style::default().fg(COLOR_ACCENT)),
    );

    f.render_widget(table, chunks[0]);

    // Description box — border only when has content, space always reserved
    let desc_para = Paragraph::new(desc_text)
        .block(
            Block::default()
                .borders(if desc_text.is_empty() {
                    Borders::NONE
                } else {
                    Borders::ALL
                })
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(Color::White));
    f.render_widget(desc_para, chunks[1]);
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn truncate(s: &str, max: usize) -> String {
    let mut chars = s.chars();
    let mut result: String = chars.by_ref().take(max.saturating_sub(1)).collect();
    if chars.next().is_some() {
        result.push('…');
    }
    result
}

fn due_display(task: &Task) -> String {
    let Some(due) = task.due_date else {
        return String::new();
    };
    let today = Local::now().naive_local().date();
    let days = (due - today).num_days();
    match days {
        d if d < 0 => format!("{}d late", d.abs()),
        0 => "today".into(),
        d => format!("{}d", d),
    }
}

fn due_style(task: &Task) -> Style {
    if task.completed {
        return Style::default().fg(COLOR_DONE);
    }
    let Some(due) = task.due_date else {
        return Style::default();
    };
    let today = Local::now().naive_local().date();
    let days = (due - today).num_days();
    if days < 0 {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else if days == 0 {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else if days <= 7 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(COLOR_ACCENT)
    }
}
