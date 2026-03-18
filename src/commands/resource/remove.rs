//! Handler for `todo resource remove <ID>`.

use anyhow::Result;
use colored::Colorize;

use crate::storage::{EntityType, EventType, Storage};
use crate::utils::validation::resolve_visible_index;

pub fn execute(storage: &impl Storage, id: usize, yes: bool) -> Result<()> {
    let mut notes = storage.load_notes()?;
    let mut resources = storage.load_resources()?;

    let real_index = resolve_visible_index(&resources, id, |r| r.is_deleted())
        .map_err(|_| anyhow::anyhow!("Resource #{} not found", id))?;

    let resource_uuid = resources[real_index].uuid;
    let title = resources[real_index].title.clone();

    if !yes {
        println!(
            "{} Remove resource #{}: {}? [y/N] ",
            "!".yellow(),
            id,
            title.bold()
        );
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{} Cancelled.", "".dimmed());
            return Ok(());
        }
    }

    resources[real_index].soft_delete();

    for note in notes.iter_mut().filter(|n| !n.is_deleted()) {
        let before = note.resource_ids.len();
        note.resource_ids.retain(|id| *id != resource_uuid);
        if note.resource_ids.len() != before {
            note.touch();
        }
    }

    storage.save_notes(&notes)?;
    storage.save_resources(&resources)?;
    storage.record_event(EntityType::Resource, resource_uuid, EventType::Deleted)?;

    println!("{} Resource #{} removed.", "✓".green(), id);
    Ok(())
}
