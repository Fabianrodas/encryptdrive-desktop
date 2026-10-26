use std::collections::{BTreeMap, BTreeSet};

use crate::Destination;

/// A stable synthetic identifier. It never depends on the displayed row index.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct EntryId(pub u64);

/// Synthetic folder or file.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntryKind {
    /// Folder containing other synthetic entries.
    Folder,
    /// Synthetic file; no content is created or read.
    File,
}

/// One deterministic, in-memory directory or file row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entry {
    /// Stable ID used by navigation, selection, and operations.
    pub id: EntryId,
    /// Parent folder, retained while in Trash for simulated restore.
    pub parent_id: Option<EntryId>,
    /// Sample name, never derived from user files.
    pub name: String,
    /// Synthetic entry kind.
    pub kind: EntryKind,
    /// Display-only type label.
    pub file_type: String,
    /// Sample size for files; folders have no size.
    pub size_bytes: Option<u64>,
    /// Deterministic sample creation time.
    pub created_at: String,
    /// Deterministic sample modification time, not a VF1 promise.
    pub modified_at: String,
    /// Deterministic deletion time when the item is in synthetic Trash.
    pub deleted_at: Option<String>,
}

/// The number of active rows at the synthetic vault root.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DatasetSize {
    /// One hundred active root entries.
    OneHundred,
    /// One thousand active root entries.
    OneThousand,
    /// Ten thousand active root entries.
    TenThousand,
}

impl DatasetSize {
    /// Active root row count represented by this choice.
    pub const fn count(self) -> usize {
        match self {
            Self::OneHundred => 100,
            Self::OneThousand => 1_000,
            Self::TenThousand => 10_000,
        }
    }
}

/// Validation reason for synthetic folder and rename forms.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NameValidationError {
    /// Blank or whitespace-only name.
    Empty,
    /// A path separator or control character is not allowed in a single name.
    InvalidCharacters,
    /// Another sibling already uses this name, ignoring case.
    NameAlreadyExists,
    /// The selected parent is not an active folder.
    ParentNotFolder,
    /// The entry to rename does not exist in the active fixture.
    EntryNotFound,
}

/// A recursive synthetic operation whose scope is confirmed before it runs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecursiveOperation {
    /// Active entries and their active descendants move to Trash.
    MoveToTrash,
    /// Trashed entries and their trashed descendants are removed from the fixture.
    PermanentDelete,
}

impl RecursiveOperation {
    /// The only place this operation reads from: Files for Trash moves, Trash for deletion.
    pub const fn destination(self) -> Destination {
        match self {
            Self::MoveToTrash => Destination::Files,
            Self::PermanentDelete => Destination::Trash,
        }
    }
}

/// The exact scope of a recursive operation, computed once by [`PreviewModel::plan`].
/// The confirmation shows these counts and [`PreviewModel::execute_plan`] runs exactly this
/// set, so the two can never disagree.
///
/// Production note: if the affected set changes between confirmation and execution, the
/// operation must be refused and confirmed again, never silently widened. `execute_plan`
/// enforces that here by refusing stale plans.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationPlan {
    operation: RecursiveOperation,
    selected: BTreeSet<EntryId>,
    affected: BTreeSet<EntryId>,
    revision: u64,
}

impl OperationPlan {
    /// Which operation this plan describes.
    pub const fn operation(&self) -> RecursiveOperation {
        self.operation
    }

    /// Distinct selected entries that exist in the operation's destination.
    pub const fn selected(&self) -> &BTreeSet<EntryId> {
        &self.selected
    }

    /// Every entry the operation changes: the selection plus descendants, deduplicated.
    pub const fn affected(&self) -> &BTreeSet<EntryId> {
        &self.affected
    }

    /// Number of distinct selected entries.
    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }

    /// Number of entries the operation changes.
    pub fn affected_count(&self) -> usize {
        self.affected.len()
    }

    /// Entries included only because they are inside a selected folder.
    pub fn included_count(&self) -> usize {
        self.affected.len() - self.selected.len()
    }
}

/// Why a plan was not executed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanError {
    /// The fixture changed after the plan was made; nothing was modified.
    Stale,
}

/// Deterministic in-memory directory tree and Trash fixture.
#[derive(Clone, Debug)]
pub struct PreviewModel {
    entries: BTreeMap<EntryId, Entry>,
    next_id: u64,
    dataset_size: DatasetSize,
    revision: u64,
}

impl PreviewModel {
    /// Creates an identical fixture for a given active root size.
    pub fn fixture(dataset_size: DatasetSize) -> Self {
        let mut model = Self {
            entries: BTreeMap::new(),
            next_id: 100_000,
            dataset_size,
            revision: 0,
        };

        model.insert(entry(1, None, "Documents", EntryKind::Folder, None, 0));
        model.insert(entry(2, None, "Photos", EntryKind::Folder, None, 1));
        model.insert(entry(3, None, "Empty folder", EntryKind::Folder, None, 2));
        model.insert(entry(
            4,
            None,
            "Annual report.txt",
            EntryKind::File,
            Some(74_752),
            3,
        ));
        model.insert(entry(
            5,
            None,
            "Résumé · 日本語の資料.md",
            EntryKind::File,
            Some(12_288),
            4,
        ));
        model.insert(entry(
            101,
            Some(EntryId(1)),
            "Projects",
            EntryKind::Folder,
            None,
            5,
        ));
        model.insert(entry(
            102,
            Some(EntryId(1)),
            "Shared notes.txt",
            EntryKind::File,
            Some(4_096),
            6,
        ));
        model.insert(entry(
            103,
            Some(EntryId(101)),
            "日本語のノート.md",
            EntryKind::File,
            Some(8_192),
            7,
        ));
        model.insert(entry(
            201,
            Some(EntryId(2)),
            "Shared notes.txt",
            EntryKind::File,
            Some(32_768),
            8,
        ));
        model.insert(entry(
            202,
            Some(EntryId(2)),
            "Screenshot 2026-10-19.png",
            EntryKind::File,
            Some(524_288),
            9,
        ));

        let generated_count = dataset_size.count().saturating_sub(5);
        for index in 0..generated_count {
            let id = 1_000 + index as u64;
            let (name, extension) = generated_name(index);
            let mut sample = entry(
                id,
                None,
                &name,
                EntryKind::File,
                Some(generated_size(index)),
                index + 10,
            );
            sample.file_type = extension.to_owned();
            model.insert(sample);
        }

        for index in 0..12 {
            let id = 90_000 + index as u64;
            let mut deleted = entry(
                id,
                None,
                &format!("Deleted sample {:02}.{}", index + 1, file_extension(index)),
                if index == 4 {
                    EntryKind::Folder
                } else {
                    EntryKind::File
                },
                (index != 4).then_some(generated_size(index + 50)),
                index + 400,
            );
            deleted.deleted_at = Some(sample_timestamp(index + 800));
            model.insert(deleted);
        }

        model
    }

    /// Returns the configured root row count.
    pub const fn dataset_size(&self) -> DatasetSize {
        self.dataset_size
    }

    /// Revision used to invalidate sorted visible-list projections after edits.
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Resolves one stable ID without changing row order.
    pub fn entry(&self, id: EntryId) -> Option<&Entry> {
        self.entries.get(&id)
    }

    /// Iterates entries in stable ID order.
    pub fn entries(&self) -> impl Iterator<Item = &Entry> {
        self.entries.values()
    }

    /// Returns direct active children or the flattened Trash listing.
    pub fn list_ids(&self, destination: Destination, parent: Option<EntryId>) -> Vec<EntryId> {
        self.entries
            .values()
            .filter(|item| match destination {
                Destination::Files => item.deleted_at.is_none() && item.parent_id == parent,
                Destination::Trash => item.deleted_at.is_some(),
            })
            .map(|item| item.id)
            .collect()
    }

    /// Creates a folder in the synthetic tree after name validation.
    pub fn create_folder(
        &mut self,
        parent: Option<EntryId>,
        name: &str,
        created_at: &str,
    ) -> Result<EntryId, NameValidationError> {
        self.validate_name(parent, name, None)?;
        if let Some(parent_id) = parent {
            let is_active_folder = self
                .entries
                .get(&parent_id)
                .is_some_and(|item| item.kind == EntryKind::Folder && item.deleted_at.is_none());
            if !is_active_folder {
                return Err(NameValidationError::ParentNotFolder);
            }
        }

        let normalized = name.trim();
        let id = EntryId(self.next_id);
        self.next_id += 1;
        self.insert(Entry {
            id,
            parent_id: parent,
            name: normalized.to_owned(),
            kind: EntryKind::Folder,
            file_type: "Folder".to_owned(),
            size_bytes: None,
            created_at: created_at.to_owned(),
            modified_at: created_at.to_owned(),
            deleted_at: None,
        });
        self.revision = self.revision.wrapping_add(1);
        Ok(id)
    }

    /// Renames one entry in the synthetic tree.
    pub fn rename(&mut self, id: EntryId, name: &str) -> Result<(), NameValidationError> {
        let Some(existing) = self.entries.get(&id) else {
            return Err(NameValidationError::EntryNotFound);
        };
        let parent = existing.parent_id;
        self.validate_name(parent, name, Some(id))?;
        let item = self
            .entries
            .get_mut(&id)
            .ok_or(NameValidationError::EntryNotFound)?;
        item.name = name.trim().to_owned();
        item.modified_at = "2026-10-19 12:00".to_owned();
        self.revision = self.revision.wrapping_add(1);
        Ok(())
    }

    /// Computes the exact scope of `operation` for a selection:
    /// - selected IDs are deduplicated and kept only if they are in the operation's
    ///   destination (stale or foreign IDs are ignored);
    /// - descendants in the same destination are added through an iterative walk over a
    ///   parent-to-children index, so deep or wide trees cost O(n log n) without recursion,
    ///   and an entry already reached (a selected descendant of a selected folder) is
    ///   counted once.
    pub fn plan(&self, operation: RecursiveOperation, selection: &[EntryId]) -> OperationPlan {
        let in_scope = |item: &Entry| match operation.destination() {
            Destination::Files => item.deleted_at.is_none(),
            Destination::Trash => item.deleted_at.is_some(),
        };
        let selected: BTreeSet<EntryId> = selection
            .iter()
            .copied()
            .filter(|id| self.entries.get(id).is_some_and(in_scope))
            .collect();
        let affected = self.expand(&selected, in_scope);
        OperationPlan {
            operation,
            selected,
            affected,
            revision: self.revision,
        }
    }

    /// Runs a plan exactly as confirmed and returns how many entries changed. A plan made
    /// before any later change to the fixture is refused without modifying anything.
    pub fn execute_plan(
        &mut self,
        plan: &OperationPlan,
        deleted_at: &str,
    ) -> Result<usize, PlanError> {
        let selection: Vec<EntryId> = plan.selected.iter().copied().collect();
        if plan.revision != self.revision || self.plan(plan.operation, &selection) != *plan {
            return Err(PlanError::Stale);
        }
        for id in &plan.affected {
            match plan.operation {
                RecursiveOperation::MoveToTrash => {
                    if let Some(item) = self.entries.get_mut(id) {
                        item.deleted_at = Some(deleted_at.to_owned());
                    }
                }
                RecursiveOperation::PermanentDelete => {
                    self.entries.remove(id);
                }
            }
        }
        self.revision = self.revision.wrapping_add(1);
        Ok(plan.affected.len())
    }

    /// Moves synthetic entries and their descendants to Trash, preserving their IDs.
    /// Same scope as the confirmation: see [`Self::plan`].
    pub fn move_to_trash(&mut self, ids: &[EntryId], deleted_at: &str) -> usize {
        let plan = self.plan(RecursiveOperation::MoveToTrash, ids);
        self.execute_plan(&plan, deleted_at)
            .expect("a plan made just now is current")
    }

    /// Restores synthetic entries and descendants to their original parent IDs.
    pub fn restore(&mut self, ids: &[EntryId]) {
        let mut roots = ids.to_vec();
        for id in ids {
            let mut parent = self.entries.get(id).and_then(|item| item.parent_id);
            while let Some(parent_id) = parent {
                let Some(item) = self.entries.get(&parent_id) else {
                    break;
                };
                if item.deleted_at.is_some() {
                    roots.push(parent_id);
                }
                parent = item.parent_id;
            }
        }
        let affected = self.expand(&roots.into_iter().collect(), |_| true);
        for id in affected {
            if let Some(item) = self.entries.get_mut(&id) {
                item.deleted_at = None;
            }
        }
        self.revision = self.revision.wrapping_add(1);
    }

    /// Permanently removes selected synthetic entries and their descendants.
    /// Same scope as the confirmation: see [`Self::plan`].
    pub fn permanently_delete(&mut self, ids: &[EntryId]) -> usize {
        let plan = self.plan(RecursiveOperation::PermanentDelete, ids);
        self.execute_plan(&plan, "")
            .expect("a plan made just now is current")
    }

    /// Validates a proposed one-segment name against active siblings.
    pub fn validate_name(
        &self,
        parent: Option<EntryId>,
        candidate: &str,
        exclude: Option<EntryId>,
    ) -> Result<(), NameValidationError> {
        let candidate = candidate.trim();
        if candidate.is_empty() {
            return Err(NameValidationError::Empty);
        }
        if candidate
            .chars()
            .any(|ch| ch.is_control() || matches!(ch, '/' | '\\'))
        {
            return Err(NameValidationError::InvalidCharacters);
        }
        let normalized = candidate.to_lowercase();
        if self.entries.values().any(|item| {
            item.id != exclude.unwrap_or(EntryId(u64::MAX))
                && item.deleted_at.is_none()
                && item.parent_id == parent
                && item.name.to_lowercase() == normalized
        }) {
            return Err(NameValidationError::NameAlreadyExists);
        }
        Ok(())
    }

    /// `roots` plus every descendant accepted by `in_scope`, without recursion. The visited
    /// set also guards against a malformed parent cycle.
    fn expand(
        &self,
        roots: &BTreeSet<EntryId>,
        in_scope: impl Fn(&Entry) -> bool,
    ) -> BTreeSet<EntryId> {
        let mut children: BTreeMap<EntryId, Vec<EntryId>> = BTreeMap::new();
        for item in self.entries.values().filter(|item| in_scope(item)) {
            if let Some(parent) = item.parent_id {
                children.entry(parent).or_default().push(item.id);
            }
        }
        let mut affected = BTreeSet::new();
        let mut pending: Vec<EntryId> = roots.iter().copied().collect();
        while let Some(id) = pending.pop() {
            if affected.insert(id)
                && let Some(kids) = children.get(&id)
            {
                pending.extend(kids);
            }
        }
        affected
    }

    fn insert(&mut self, item: Entry) {
        self.entries.insert(item.id, item);
    }
}

fn entry(
    id: u64,
    parent_id: Option<EntryId>,
    name: &str,
    kind: EntryKind,
    size_bytes: Option<u64>,
    stamp: usize,
) -> Entry {
    let file_type = if kind == EntryKind::Folder {
        "Folder".to_owned()
    } else {
        name.rsplit_once('.').map_or_else(
            || "File".to_owned(),
            |(_, extension)| extension.to_uppercase(),
        )
    };
    Entry {
        id: EntryId(id),
        parent_id,
        name: name.to_owned(),
        kind,
        file_type,
        size_bytes,
        created_at: sample_timestamp(stamp),
        modified_at: sample_timestamp(stamp + 37),
        deleted_at: None,
    }
}

fn generated_name(index: usize) -> (String, &'static str) {
    match index {
        0 => ("Budget overview.pdf".to_owned(), "PDF"),
        1 => ("Project notes – résumé.txt".to_owned(), "TXT"),
        2 => (
            format!(
                "Quarterly archive — {}.zip",
                "very-long-納品資料-".repeat(9)
            ),
            "ZIP",
        ),
        _ => {
            let extension = file_extension(index);
            (
                format!("Project item {index:05}.{extension}"),
                extension_upper(extension),
            )
        }
    }
}

fn generated_size(index: usize) -> u64 {
    ((index as u64 * 7919) % 22_000_000) + 256
}

fn file_extension(index: usize) -> &'static str {
    ["txt", "pdf", "png", "docx", "zip", "csv"][index % 6]
}

fn extension_upper(extension: &str) -> &'static str {
    match extension {
        "txt" => "TXT",
        "pdf" => "PDF",
        "png" => "PNG",
        "docx" => "DOCX",
        "zip" => "ZIP",
        "csv" => "CSV",
        _ => "File",
    }
}

fn sample_timestamp(index: usize) -> String {
    let month = (index / 28) % 12 + 1;
    let day = index % 28 + 1;
    let year = 2024 + index / 336;
    let hour = index % 24;
    let minute = (index * 7) % 60;
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}")
}
