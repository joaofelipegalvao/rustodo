//! Terminal rendering for note lists.

use colored::Colorize;

use crate::models::{Note, NoteFormat, Project};
use crate::render::formatting::{project_colored, project_name, truncate};

pub struct NoteTableLayout {
    pub body_w: usize,
    pub proj_w: usize,
    pub lang_w: usize,
    pub tags_w: usize,
    pub show_project: bool,
    pub show_lang: bool,
    pub show_tags: bool,
    pub show_resources: bool,
    pub show_format: bool,
    pub total_w: usize,
}

/// Returns a single-line preview of the note body.
/// Uses the title if available, otherwise the first non-empty line of the body.
fn note_preview(note: &Note) -> String {
    if let Some(ref title) = note.title {
        return title.clone();
    }
    note.body
        .lines()
        .find(|l| !l.trim().is_empty())
        .map(|l| l.trim_start_matches('#').trim().to_string())
        .unwrap_or_default()
}

impl NoteTableLayout {
    pub fn new(
        notes: &[&Note],
        projects: &[Project],
        resources: &[crate::models::Resource],
    ) -> Self {
        let body_w = notes
            .iter()
            .map(|n| note_preview(n).len())
            .max()
            .unwrap_or(10)
            .clamp(10, 48);

        let proj_w = notes
            .iter()
            .filter_map(|n| {
                n.project_id
                    .and_then(|pid| projects.iter().find(|p| p.uuid == pid))
                    .map(|p| p.name.len())
            })
            .max()
            .unwrap_or(0)
            .clamp(7, 24);

        let lang_w = notes
            .iter()
            .filter_map(|n| n.language.as_deref().map(|l| l.len()))
            .max()
            .unwrap_or(0)
            .clamp(4, 16);

        let tags_w = notes
            .iter()
            .map(|n| {
                if n.tags.is_empty() {
                    0
                } else {
                    n.tags.join(", ").len()
                }
            })
            .max()
            .unwrap_or(0)
            .clamp(4, 24);

        let show_project = notes.iter().any(|n| n.project_id.is_some());
        let show_lang = notes.iter().any(|n| n.language.is_some());
        let show_tags = notes.iter().any(|n| !n.tags.is_empty());
        let show_resources = notes.iter().any(|n| {
            n.resource_ids
                .iter()
                .any(|rid| resources.iter().any(|r| !r.is_deleted() && r.uuid == *rid))
        });
        let show_format = notes.iter().any(|n| n.format.is_markdown());

        let mut total_w = 4 + 2 + body_w;
        if show_project {
            total_w += 2 + proj_w;
        }
        if show_lang {
            total_w += 2 + lang_w;
        }
        if show_tags {
            total_w += 2 + tags_w;
        }
        if show_resources {
            total_w += 2 + 3;
        }
        if show_format {
            total_w += 2 + 8; // "markdown" = 8 chars
        }

        Self {
            body_w,
            proj_w,
            lang_w,
            tags_w,
            show_project,
            show_lang,
            show_tags,
            show_resources,
            show_format,
            total_w,
        }
    }

    pub fn display_header(&self) {
        print!("{:>4}  ", "ID".dimmed());
        if self.show_project {
            print!("{:<proj_w$}  ", "Project".dimmed(), proj_w = self.proj_w);
        }
        if self.show_lang {
            print!("{:<lang_w$}  ", "Lang".dimmed(), lang_w = self.lang_w);
        }
        if self.show_tags {
            print!("{:<tags_w$}  ", "Tags".dimmed(), tags_w = self.tags_w);
        }
        if self.show_resources {
            print!("{:^3}  ", "Res".dimmed());
        }
        if self.show_format {
            print!("{:<8}  ", "Format".dimmed());
        }
        print!("{:<body_w$}", "Note".dimmed(), body_w = self.body_w);
        println!();
        println!("{}", "─".repeat(self.total_w).dimmed());
    }

    pub fn display_row(
        &self,
        id: usize,
        note: &Note,
        projects: &[Project],
        resources: &[crate::models::Resource],
    ) {
        let preview = truncate(&note_preview(note), self.body_w);

        let name = project_name(note.project_id, projects);
        let proj_str = truncate(name, self.proj_w);
        let proj_colored = project_colored(&proj_str);

        let lang_str = note
            .language
            .as_deref()
            .map(|l| truncate(l, self.lang_w))
            .unwrap_or_else(|| "—".to_string());

        let lang_colored = if note.language.is_some() {
            lang_str.yellow()
        } else {
            lang_str.dimmed()
        };

        let tags_str = if note.tags.is_empty() {
            String::new()
        } else {
            truncate(&note.tags.join(", "), self.tags_w)
        };

        print!("{:>4}  ", format!("#{}", id).dimmed());
        if self.show_project {
            print!("{:<proj_w$}  ", proj_colored, proj_w = self.proj_w);
        }
        if self.show_lang {
            print!("{:<lang_w$}  ", lang_colored, lang_w = self.lang_w);
        }
        if self.show_tags {
            print!("{:<tags_w$}  ", tags_str.cyan(), tags_w = self.tags_w);
        }
        if self.show_resources {
            let count = note
                .resource_ids
                .iter()
                .filter(|rid| resources.iter().any(|r| !r.is_deleted() && r.uuid == **rid))
                .count();
            let res_str = if count > 0 {
                format!("{:^3}", count).dimmed().to_string()
            } else {
                format!("{:^3}", "—").dimmed().to_string()
            };
            print!("{}  ", res_str);
        }
        if self.show_format {
            match note.format {
                NoteFormat::Markdown => print!("{:<8}  ", "markdown".cyan()),
                NoteFormat::Plain => print!("{:<8}  ", "—".dimmed()),
            }
        }
        print!("{:<body_w$}", preview, body_w = self.body_w);
        println!();
    }

    pub fn display_separator(&self) {
        println!("{}", "─".repeat(self.total_w).dimmed());
    }
}

pub fn display_notes(notes: &[&Note], projects: &[Project], resources: &[crate::models::Resource]) {
    println!("\nNotes:\n");
    let layout = NoteTableLayout::new(notes, projects, resources);
    layout.display_header();
    for (i, note) in notes.iter().enumerate() {
        layout.display_row(i + 1, note, projects, resources);
    }
    layout.display_separator();
    println!();
}
