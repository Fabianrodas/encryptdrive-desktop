use crate::model::{Entry, EntryId, EntryKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fixture {
    entries: Vec<Entry>,
}

impl Fixture {
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn into_entries(self) -> Vec<Entry> {
        self.entries
    }
}

/// Build an exactly sized, repeatable fixture. IDs are generated from a fixed sequence,
/// never from the current sort or filter position.
pub fn make_fixture(size: usize) -> Fixture {
    let size = size.max(6);
    let mut entries = Vec::with_capacity(size + 4);

    let fixed = [
        ("Projects", EntryKind::Folder, Some(0), 0),
        ("Empty folder", EntryKind::Folder, Some(0), 0),
        ("résumé — 東京.txt", EntryKind::File, Some(0), 8_614),
        (
            "A deliberately long synthetic document name for truncation and tooltip checks 2026.txt",
            EntryKind::File,
            Some(0),
            82_410,
        ),
        ("Collision Lab", EntryKind::Folder, Some(0), 0),
        ("image sample.png", EntryKind::File, Some(0), 152_300),
    ];

    for (index, (name, kind, parent, size_bytes)) in fixed.into_iter().enumerate() {
        let id = EntryId(index as u64 + 1);
        entries.push(Entry {
            id,
            name: name.to_owned(),
            kind,
            parent_id: parent.map(EntryId),
            size_bytes,
            created_at: fixture_time(index as u64),
            modified_at: fixture_time(index as u64 + 2 + (index % 6) as u64),
            deleted_at: None,
        });
    }

    let mut next_id = entries.len() as u64 + 1;
    while entries.len() < size {
        let ordinal = next_id as usize;
        let make_folder = ordinal.is_multiple_of(73);
        let (kind, name, size_bytes) = if make_folder {
            (
                EntryKind::Folder,
                format!("Archive batch {:04}", ordinal / 73),
                0,
            )
        } else {
            let suffix = match ordinal % 5 {
                0 => "pdf",
                1 => "txt",
                2 => "png",
                3 => "csv",
                _ => "zip",
            };
            let name = if ordinal.is_multiple_of(101) {
                format!("Quarterly résumé 東京 {ordinal:05}.{suffix}")
            } else {
                format!("Synthetic document {ordinal:05}.{suffix}")
            };
            let bytes = ((ordinal as u64 * 7_919) % 8_000_000) + 512;
            (EntryKind::File, name, bytes)
        };

        let parent_id = Some(EntryId(0));
        let timestamp = fixture_time(ordinal as u64);
        entries.push(Entry {
            id: EntryId(next_id),
            name,
            kind,
            parent_id,
            size_bytes,
            created_at: timestamp.clone(),
            modified_at: fixture_time(ordinal as u64 + (ordinal % 31) as u64 + 1),
            deleted_at: None,
        });
        next_id += 1;
    }

    let hidden = [
        ("Research", EntryKind::Folder, EntryId(1), 0, None),
        (
            "field-notes.md",
            EntryKind::File,
            EntryId(size as u64 + 1),
            14_122,
            None,
        ),
        ("Budget.xlsx", EntryKind::File, EntryId(5), 56_001, None),
        (
            "Budget.xlsx",
            EntryKind::File,
            EntryId(5),
            45_200,
            Some(fixture_time(size as u64 + 42)),
        ),
    ];
    for (offset, (name, kind, parent, size_bytes, deleted_at)) in hidden.into_iter().enumerate() {
        let ordinal = size as u64 + offset as u64 + 1;
        entries.push(Entry {
            id: EntryId(ordinal),
            name: name.to_owned(),
            kind,
            parent_id: Some(parent),
            size_bytes,
            created_at: fixture_time(ordinal),
            modified_at: fixture_time(ordinal + 2),
            deleted_at,
        });
    }

    Fixture { entries }
}

fn fixture_time(offset: u64) -> String {
    let day = offset % 28 + 1;
    let month = (offset / 28) % 12 + 1;
    let hour = (offset / 13) % 24;
    let minute = (offset * 7) % 60;
    format!("2025-{month:02}-{day:02} {hour:02}:{minute:02}")
}
