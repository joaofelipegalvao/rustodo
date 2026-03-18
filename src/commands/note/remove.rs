//! Handler for `todo note remove <ID>`.

use anyhow::Result;
use colored::Colorize;

use crate::storage::{EntityType, EventType, Storage};
use crate::utils::validation::resolve_visible_index;

pub fn execute(storage: &impl Storage, id: usize, yes: bool) -> Result<()> {
    let mut notes = storage.load_notes()?;

    let real_index = resolve_visible_index(&notes, id, |n| n.is_deleted())
        .map_err(|_| anyhow::anyhow!("Note #{} not found", id))?;

    let preview = notes[real_index].title.clone().unwrap_or_else(|| {
        let b = notes[real_index].body.as_str();
        if b.len() > 60 {
            b[..60].to_string()
        } else {
            b.to_string()
        }
    });

    if !yes {
        println!(
            "{} Remove note #{}: {}? [y/N] ",
            "!".yellow(),
            id,
            preview.bold()
        );
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{} Cancelled.", "".dimmed());
            return Ok(());
        }
    }

    let note_uuid = notes[real_index].uuid;
    notes[real_index].soft_delete();
    storage.save_notes(&notes)?;
    storage.record_event(EntityType::Note, note_uuid, EventType::Deleted)?;

    println!("{} Note #{} removed.", "✓".green(), id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Note;
    use crate::storage::InMemoryStorage;

    fn make_note(body: &str) -> Note {
        Note::new(body.into())
    }

    #[test]
    fn test_note_remove_soft_deletes() {
        let storage = InMemoryStorage::default();
        storage.save_notes(&[make_note("Note body")]).unwrap();

        execute(&storage, 1, true).unwrap();

        assert!(storage.load_notes().unwrap()[0].is_deleted());
    }

    #[test]
    fn test_note_remove_invalid_id_returns_error() {
        let storage = InMemoryStorage::default();
        storage.save_notes(&[make_note("Note")]).unwrap();

        assert!(execute(&storage, 99, true).is_err());
    }

    #[test]
    fn test_note_remove_does_not_affect_other_notes() {
        let storage = InMemoryStorage::default();
        storage
            .save_notes(&[make_note("Note A"), make_note("Note B")])
            .unwrap();

        execute(&storage, 1, true).unwrap();

        let notes = storage.load_notes().unwrap();
        assert!(notes[0].is_deleted());
        assert!(!notes[1].is_deleted());
    }

    #[test]
    fn test_note_remove_skips_deleted_in_id_resolution() {
        let storage = InMemoryStorage::default();
        let mut deleted = make_note("Deleted");
        deleted.soft_delete();
        let active = make_note("Active");
        storage.save_notes(&[deleted, active]).unwrap();

        execute(&storage, 1, true).unwrap();

        let notes = storage.load_notes().unwrap();
        assert!(notes[0].is_deleted()); // was already deleted
        assert!(notes[1].is_deleted()); // #1 resolved to active
    }
}
