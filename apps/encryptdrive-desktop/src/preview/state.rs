use std::{collections::BTreeSet, sync::Arc};

use super::{EntryId, EntryKind, PreviewModel};
use crate::Destination;

/// How a visible table row was selected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Modifier {
    /// Replace selection with a single row.
    Plain,
    /// Toggle the clicked row without changing the others.
    Control,
    /// Replace selection with the inclusive anchor-to-row range.
    Shift,
}

/// Sort column used by the design preview.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SortKey {
    /// Sort by the sample name.
    Name,
    /// Sort by the sample type label.
    Type,
    /// Sort by synthetic byte size.
    Size,
    /// Sort by a clearly labeled sample-only timestamp.
    ModifiedSample,
}

/// Direction of the current sort.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SortDirection {
    /// Ascending order.
    Ascending,
    /// Descending order.
    Descending,
}

/// Navigation, search, sort, and stable-ID selection state for the file table.
#[derive(Clone, Debug)]
pub struct ExplorerState {
    destination: Destination,
    current_folder: Option<EntryId>,
    folder_history: Vec<Option<EntryId>>,
    history_index: usize,
    query: String,
    sort_key: SortKey,
    sort_direction: SortDirection,
    selected: BTreeSet<EntryId>,
    anchor: Option<EntryId>,
    focused: Option<EntryId>,
    cached_projection: Option<ProjectionCache>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProjectionKey {
    destination: Destination,
    folder: Option<EntryId>,
    query: String,
    sort_key: SortKey,
    sort_direction: SortDirection,
    model_revision: u64,
}

#[derive(Clone, Debug)]
struct ProjectionCache {
    key: ProjectionKey,
    ids: Arc<[EntryId]>,
}

impl ExplorerState {
    /// Starts at the Files root with an ascending Name sort and no selection.
    pub fn new() -> Self {
        Self {
            destination: Destination::Files,
            current_folder: None,
            folder_history: vec![None],
            history_index: 0,
            query: String::new(),
            sort_key: SortKey::Name,
            sort_direction: SortDirection::Ascending,
            selected: BTreeSet::new(),
            anchor: None,
            focused: None,
            cached_projection: None,
        }
    }

    /// Current destination.
    pub const fn destination(&self) -> Destination {
        self.destination
    }

    /// Current folder; `None` denotes the root.
    pub const fn current_folder(&self) -> Option<EntryId> {
        self.current_folder
    }

    /// Current local name query.
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Current sort key.
    pub const fn sort_key(&self) -> SortKey {
        self.sort_key
    }

    /// Current sort direction.
    pub const fn sort_direction(&self) -> SortDirection {
        self.sort_direction
    }

    /// Selected stable IDs, in ID order.
    pub fn selected_ids(&self) -> Vec<EntryId> {
        self.selected.iter().copied().collect()
    }

    /// Current focused row ID.
    pub const fn focused_id(&self) -> Option<EntryId> {
        self.focused
    }

    /// Changes destination and clears row selection.
    pub fn set_destination(&mut self, destination: Destination) {
        if self.destination != destination {
            self.destination = destination;
            self.current_folder = None;
            self.query.clear();
            self.clear_selection();
        }
    }

    /// Sets a transient query and drops selection if the effective result set changes.
    pub fn set_query(&mut self, query: impl Into<String>) {
        let query = query.into();
        if normalized_query(&self.query) != normalized_query(&query) {
            self.clear_selection();
        }
        self.query = query;
    }

    /// Changes sort; selecting the current key toggles its direction.
    pub fn set_sort(&mut self, sort_key: SortKey) {
        if self.sort_key == sort_key {
            self.sort_direction = match self.sort_direction {
                SortDirection::Ascending => SortDirection::Descending,
                SortDirection::Descending => SortDirection::Ascending,
            };
        } else {
            self.sort_key = sort_key;
            self.sort_direction = SortDirection::Ascending;
        }
    }

    /// Navigates into a folder and appends a back-history entry.
    pub fn navigate_to(&mut self, folder: Option<EntryId>) {
        if self.current_folder == folder {
            return;
        }
        self.folder_history.truncate(self.history_index + 1);
        self.folder_history.push(folder);
        self.history_index = self.folder_history.len() - 1;
        self.current_folder = folder;
        self.query.clear();
        self.clear_selection();
    }

    /// Returns to the previous folder when available.
    pub fn navigate_back(&mut self) -> bool {
        if self.history_index == 0 {
            return false;
        }
        self.history_index -= 1;
        self.current_folder = self.folder_history[self.history_index];
        self.query.clear();
        self.clear_selection();
        true
    }

    /// Returns the sorted visible stable IDs for the current folder/search/Trash view.
    pub fn visible_ids(&mut self, model: &PreviewModel) -> Arc<[EntryId]> {
        let query = self.query.trim().to_lowercase();
        let key = ProjectionKey {
            destination: self.destination,
            folder: self.current_folder,
            query: query.clone(),
            sort_key: self.sort_key,
            sort_direction: self.sort_direction,
            model_revision: model.revision(),
        };
        if self
            .cached_projection
            .as_ref()
            .is_some_and(|cache| cache.key == key)
        {
            return Arc::clone(&self.cached_projection.as_ref().expect("cache checked").ids);
        }
        let mut ids: Vec<EntryId> = model
            .entries()
            .filter(|entry| match self.destination {
                Destination::Files => {
                    entry.deleted_at.is_none()
                        && if query.is_empty() {
                            entry.parent_id == self.current_folder
                        } else {
                            entry.name.to_lowercase().contains(&query)
                        }
                }
                Destination::Trash => {
                    entry.deleted_at.is_some()
                        && (query.is_empty() || entry.name.to_lowercase().contains(&query))
                }
            })
            .map(|entry| entry.id)
            .collect();

        ids.sort_by(|left, right| {
            let left_item = model.entry(*left).expect("IDs come from the model");
            let right_item = model.entry(*right).expect("IDs come from the model");
            let by_key = match self.sort_key {
                SortKey::Name => left_item
                    .name
                    .to_lowercase()
                    .cmp(&right_item.name.to_lowercase()),
                SortKey::Type => left_item.file_type.cmp(&right_item.file_type),
                SortKey::Size => left_item.size_bytes.cmp(&right_item.size_bytes),
                SortKey::ModifiedSample => left_item.modified_at.cmp(&right_item.modified_at),
            };
            let by_key = if self.sort_direction == SortDirection::Descending {
                by_key.reverse()
            } else {
                by_key
            };
            // Like Windows Explorer, folders stay grouped before files in either direction.
            let folders_first =
                (left_item.kind != EntryKind::Folder).cmp(&(right_item.kind != EntryKind::Folder));
            folders_first.then(by_key).then_with(|| left.cmp(right))
        });
        let ids: Arc<[EntryId]> = ids.into();
        self.cached_projection = Some(ProjectionCache {
            key,
            ids: Arc::clone(&ids),
        });
        ids
    }

    /// Applies a stable-ID row selection against the displayed order.
    pub fn select(&mut self, id: EntryId, visible: &[EntryId], modifier: Modifier) {
        let Some(index) = visible.iter().position(|candidate| *candidate == id) else {
            return;
        };
        self.focused = Some(id);
        match modifier {
            Modifier::Plain => {
                self.selected.clear();
                self.selected.insert(id);
                self.anchor = Some(id);
            }
            Modifier::Control => {
                if !self.selected.remove(&id) {
                    self.selected.insert(id);
                }
                self.anchor = Some(id);
            }
            Modifier::Shift => {
                let anchor_index = self
                    .anchor
                    .and_then(|anchor| visible.iter().position(|candidate| *candidate == anchor))
                    .unwrap_or(index);
                self.selected.clear();
                for candidate in &visible[anchor_index.min(index)..=anchor_index.max(index)] {
                    self.selected.insert(*candidate);
                }
            }
        }
    }

    /// Selects exactly the currently visible result set.
    pub fn select_all_visible(&mut self, visible: &[EntryId]) {
        self.selected.clear();
        self.selected.extend(visible.iter().copied());
        self.anchor = visible.first().copied();
        self.focused = visible.last().copied();
    }

    /// Clears selected and focused table rows.
    pub fn clear_selection(&mut self) {
        self.selected.clear();
        self.anchor = None;
        self.focused = None;
    }

    /// Moves table focus and updates selection with standard keyboard modifiers.
    pub fn move_focus(&mut self, delta: isize, visible: &[EntryId], modifier: Option<Modifier>) {
        if visible.is_empty() {
            self.clear_selection();
            return;
        }
        let next = self
            .focused
            .and_then(|focused| visible.iter().position(|id| *id == focused))
            .map_or(0, |current| {
                (current as isize + delta).clamp(0, visible.len() as isize - 1) as usize
            });
        self.move_focus_to_index(next, visible, modifier);
    }

    /// Moves focus directly to a displayed row, clamping keyboard navigation to the result set.
    pub fn move_focus_to_index(
        &mut self,
        index: usize,
        visible: &[EntryId],
        modifier: Option<Modifier>,
    ) {
        if visible.is_empty() {
            self.clear_selection();
            return;
        }
        let id = visible[index.min(visible.len() - 1)];
        if let Some(modifier) = modifier {
            self.select(id, visible, modifier);
        } else {
            self.focused = Some(id);
        }
    }

    /// Toggles the focused row while preserving other selected stable IDs.
    pub fn toggle_focused_selection(&mut self, visible: &[EntryId]) {
        if let Some(id) = self.focused.filter(|id| visible.contains(id)) {
            self.select(id, visible, Modifier::Control);
        } else if let Some(id) = visible.first() {
            self.select(*id, visible, Modifier::Plain);
        }
    }
}

fn normalized_query(query: &str) -> String {
    query.trim().to_lowercase()
}

impl Default for ExplorerState {
    fn default() -> Self {
        Self::new()
    }
}
