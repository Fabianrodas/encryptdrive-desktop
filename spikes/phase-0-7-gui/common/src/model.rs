use std::collections::{BTreeMap, BTreeSet};

use crate::{fixture::Fixture, theme::Appearance};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EntryId(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntryKind {
    Folder,
    File,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entry {
    pub id: EntryId,
    pub name: String,
    pub kind: EntryKind,
    pub parent_id: Option<EntryId>,
    pub size_bytes: u64,
    pub created_at: String,
    /// Synthetic only in this experiment; Java's value is not a reliable modified time.
    pub modified_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Page {
    Files,
    Trash,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SortColumn {
    Name,
    Size,
    Modified,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DialogKind {
    CreateFolder,
    Rename,
    DeleteToTrash,
    PermanentDelete,
    Import,
    ExportWarning,
    RecoveryNotice,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dialog {
    pub kind: DialogKind,
    pub name: String,
    pub target: Option<EntryId>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperationStatus {
    Idle,
    Running {
        label: String,
        completed: u32,
        total: u32,
        fail_at: Option<u32>,
    },
    Succeeded {
        label: String,
    },
    Failed {
        label: String,
        message: String,
    },
    Cancelled {
        label: String,
    },
}

impl OperationStatus {
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running { .. })
    }

    pub fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled { .. })
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }

    pub fn progress(&self) -> Option<(u32, u32)> {
        match self {
            Self::Running {
                completed, total, ..
            } => Some((*completed, *total)),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SyntheticAction {
    Import { parent: EntryId, name: String },
    MoveToTrash(Vec<EntryId>),
    Restore(Vec<EntryId>),
    PermanentlyDelete(Vec<EntryId>),
}

#[derive(Clone, Debug)]
pub struct ExplorerModel {
    entries: BTreeMap<EntryId, Entry>,
    next_id: u64,
    page: Page,
    current_folder: EntryId,
    files_folder: EntryId,
    search_query: String,
    sort_column: SortColumn,
    sort_ascending: bool,
    selected: BTreeSet<EntryId>,
    focused: Option<EntryId>,
    anchor: Option<EntryId>,
    dialog: Option<Dialog>,
    operation: OperationStatus,
    pending_action: Option<SyntheticAction>,
    appearance: Appearance,
    notice: Option<String>,
    error_state: bool,
    loading: bool,
    navigation_history: Vec<EntryId>,
    history_index: usize,
}

impl ExplorerModel {
    pub fn new(fixture: Fixture) -> Self {
        let entries: BTreeMap<_, _> = fixture
            .into_entries()
            .into_iter()
            .map(|entry| (entry.id, entry))
            .collect();
        let next_id = entries.keys().map(|id| id.0).max().unwrap_or(0) + 1;
        Self {
            entries,
            next_id,
            page: Page::Files,
            current_folder: EntryId(0),
            files_folder: EntryId(0),
            search_query: String::new(),
            sort_column: SortColumn::Name,
            sort_ascending: true,
            selected: BTreeSet::new(),
            focused: None,
            anchor: None,
            dialog: None,
            operation: OperationStatus::Idle,
            pending_action: None,
            appearance: Appearance::Light,
            notice: Some("Synthetic vault · offline demo".to_owned()),
            error_state: false,
            loading: false,
            navigation_history: vec![EntryId(0)],
            history_index: 0,
        }
    }

    pub fn entries(&self) -> Vec<&Entry> {
        self.entries.values().collect()
    }

    pub fn entry(&self, id: EntryId) -> Option<&Entry> {
        self.entries.get(&id)
    }

    pub fn page(&self) -> Page {
        self.page
    }

    pub fn delete_dialog_kind(&self) -> DialogKind {
        match self.page {
            Page::Files => DialogKind::DeleteToTrash,
            Page::Trash => DialogKind::PermanentDelete,
        }
    }

    pub fn current_folder(&self) -> EntryId {
        self.current_folder
    }

    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    pub fn sort_column(&self) -> SortColumn {
        self.sort_column
    }

    pub fn sort_ascending(&self) -> bool {
        self.sort_ascending
    }

    pub fn selected_ids(&self) -> Vec<EntryId> {
        self.selected.iter().copied().collect()
    }

    pub fn focused_id(&self) -> Option<EntryId> {
        self.focused
    }

    pub fn dialog(&self) -> Option<&Dialog> {
        self.dialog.as_ref()
    }

    pub fn operation(&self) -> &OperationStatus {
        &self.operation
    }

    pub fn appearance(&self) -> Appearance {
        self.appearance
    }

    pub fn set_appearance(&mut self, appearance: Appearance) {
        self.appearance = appearance;
    }

    pub fn notice(&self) -> Option<&str> {
        self.notice.as_deref()
    }

    pub fn set_notice(&mut self, message: impl Into<String>) {
        self.notice = Some(message.into());
        self.error_state = false;
    }

    pub fn dismiss_notice(&mut self) {
        self.notice = None;
        self.error_state = false;
    }

    pub fn set_error(&mut self, message: impl Into<String>) {
        self.notice = Some(message.into());
        self.error_state = true;
    }

    pub fn is_error(&self) -> bool {
        self.error_state
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }

    pub fn simulate_loading(&mut self) {
        self.loading = true;
        self.notice = Some("Loading synthetic entries…".to_owned());
    }

    pub fn finish_loading(&mut self) {
        self.loading = false;
        self.notice = Some("Synthetic entries are ready.".to_owned());
    }

    pub fn simulate_error(&mut self) {
        self.loading = false;
        self.set_error("The synthetic list could not be loaded. Retry the demo view.");
    }

    pub fn visible_ids(&self) -> Vec<EntryId> {
        let query = self.search_query.trim().to_lowercase();
        let mut visible: Vec<&Entry> = self
            .entries
            .values()
            .filter(|entry| match self.page {
                Page::Files if !query.is_empty() => {
                    entry.deleted_at.is_none() && entry.name.to_lowercase().contains(&query)
                }
                Page::Files => {
                    entry.deleted_at.is_none() && entry.parent_id == Some(self.current_folder)
                }
                Page::Trash => {
                    entry.deleted_at.is_some()
                        && entry
                            .parent_id
                            .and_then(|parent| self.entries.get(&parent))
                            .is_none_or(|parent| parent.deleted_at.is_none())
                }
            })
            .collect();

        visible.sort_by(|left, right| match self.sort_column {
            SortColumn::Name => {
                let kind_order = match (left.kind, right.kind) {
                    (EntryKind::Folder, EntryKind::File) => std::cmp::Ordering::Less,
                    (EntryKind::File, EntryKind::Folder) => std::cmp::Ordering::Greater,
                    _ => std::cmp::Ordering::Equal,
                };
                let name_order = left.name.to_lowercase().cmp(&right.name.to_lowercase());
                let directed_name_order = if self.sort_ascending {
                    name_order
                } else {
                    name_order.reverse()
                };
                kind_order
                    .then(directed_name_order)
                    .then_with(|| left.id.cmp(&right.id))
            }
            SortColumn::Size => {
                let order = left.size_bytes.cmp(&right.size_bytes);
                let directed = if self.sort_ascending {
                    order
                } else {
                    order.reverse()
                };
                directed.then_with(|| left.id.cmp(&right.id))
            }
            SortColumn::Modified => {
                let order = left.modified_at.cmp(&right.modified_at);
                let directed = if self.sort_ascending {
                    order
                } else {
                    order.reverse()
                };
                directed.then_with(|| left.id.cmp(&right.id))
            }
        });
        visible.into_iter().map(|entry| entry.id).collect()
    }

    pub fn visible_entries(&self) -> Vec<&Entry> {
        self.visible_ids()
            .into_iter()
            .filter_map(|id| self.entries.get(&id))
            .collect()
    }

    pub fn set_page(&mut self, page: Page) {
        if self.page == page {
            return;
        }
        self.page = page;
        if page == Page::Trash {
            self.files_folder = self.current_folder;
            self.selected.clear();
            self.focused = None;
            self.anchor = None;
        } else {
            if !self
                .entries
                .get(&self.current_folder)
                .is_some_and(|entry| entry.kind == EntryKind::Folder && entry.deleted_at.is_none())
            {
                self.current_folder = EntryId(0);
                self.files_folder = EntryId(0);
            }
            self.current_folder = self.files_folder;
            self.prune_selection_to_visible();
        }
    }

    pub fn set_search(&mut self, query: impl Into<String>) {
        self.search_query = query.into();
        self.prune_selection_to_visible();
    }

    pub fn set_sort(&mut self, column: SortColumn, ascending: bool) {
        self.sort_column = column;
        self.sort_ascending = ascending;
    }

    pub fn toggle_sort(&mut self, column: SortColumn) {
        if self.sort_column == column {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_column = column;
            self.sort_ascending = true;
        }
    }

    pub fn breadcrumb(&self) -> Vec<(EntryId, String)> {
        let mut path = vec![(EntryId(0), "My files".to_owned())];
        if self.page != Page::Files || !self.search_query.is_empty() {
            return path;
        }
        let mut current = self.current_folder;
        let mut folders = Vec::new();
        while current != EntryId(0) {
            let Some(entry) = self.entries.get(&current) else {
                break;
            };
            folders.push((entry.id, entry.name.clone()));
            let Some(parent) = entry.parent_id else {
                break;
            };
            current = parent;
        }
        folders.reverse();
        path.extend(folders);
        path
    }

    pub fn navigate_into(&mut self, id: EntryId) {
        let Some(entry) = self.entries.get(&id) else {
            return;
        };
        if entry.kind != EntryKind::Folder || entry.deleted_at.is_some() {
            return;
        }
        if self.page == Page::Trash {
            self.page = Page::Files;
        }
        self.search_query.clear();
        self.current_folder = id;
        self.files_folder = id;
        self.push_history(id);
        self.clear_selection();
    }

    pub fn navigate_to(&mut self, id: EntryId) {
        let Some(entry) = self.entries.get(&id) else {
            return;
        };
        if entry.kind == EntryKind::Folder && entry.deleted_at.is_none() {
            self.page = Page::Files;
            self.search_query.clear();
            self.current_folder = id;
            self.files_folder = id;
            self.push_history(id);
            self.clear_selection();
        }
    }

    pub fn navigate_parent(&mut self) {
        if self.page != Page::Files {
            return;
        }
        if !self.search_query.is_empty() {
            self.search_query.clear();
        }
        let parent = self
            .entries
            .get(&self.current_folder)
            .and_then(|entry| entry.parent_id)
            .unwrap_or(EntryId(0));
        if parent != self.current_folder {
            self.current_folder = parent;
            self.files_folder = parent;
            self.push_history(parent);
            self.clear_selection();
        }
    }

    pub fn navigate_back(&mut self) {
        if self.page == Page::Trash || self.history_index == 0 {
            return;
        }
        self.history_index -= 1;
        self.current_folder = self.navigation_history[self.history_index];
        self.files_folder = self.current_folder;
        self.search_query.clear();
        self.clear_selection();
    }

    pub fn select_only(&mut self, id: EntryId) {
        if self.visible_ids().contains(&id) {
            self.selected.clear();
            self.selected.insert(id);
            self.focused = Some(id);
            self.anchor = Some(id);
        }
    }

    pub fn toggle_selection(&mut self, id: EntryId) {
        if !self.visible_ids().contains(&id) {
            return;
        }
        if !self.selected.remove(&id) {
            self.selected.insert(id);
        }
        self.focused = Some(id);
        self.anchor = Some(id);
    }

    pub fn select_range(&mut self, id: EntryId) {
        let visible = self.visible_ids();
        let Some(anchor) = self.anchor else {
            self.select_only(id);
            return;
        };
        let (Some(start), Some(end)) = (
            visible.iter().position(|candidate| *candidate == anchor),
            visible.iter().position(|candidate| *candidate == id),
        ) else {
            self.select_only(id);
            return;
        };
        self.selected.clear();
        for candidate in &visible[start.min(end)..=start.max(end)] {
            self.selected.insert(*candidate);
        }
        self.focused = Some(id);
    }

    pub fn select_all_visible(&mut self) {
        self.selected = self.visible_ids().into_iter().collect();
        if self.focused.is_none() {
            self.focused = self.selected.first().copied();
        }
        self.anchor = self.focused;
    }

    pub fn move_focus(&mut self, delta: isize, extend_selection: bool) {
        let visible = self.visible_ids();
        if visible.is_empty() {
            self.focused = None;
            return;
        }
        let current = self
            .focused
            .and_then(|id| visible.iter().position(|candidate| *candidate == id))
            .unwrap_or(0);
        let next = (current as isize + delta).clamp(0, visible.len() as isize - 1) as usize;
        let id = visible[next];
        if extend_selection {
            self.select_range(id);
        } else {
            self.select_only(id);
        }
    }

    pub fn activate(&mut self, id: EntryId) {
        let Some(entry) = self.entries.get(&id).cloned() else {
            return;
        };
        self.select_only(id);
        if entry.kind == EntryKind::Folder && entry.deleted_at.is_none() {
            self.navigate_into(id);
        } else if entry.kind == EntryKind::File {
            self.open_dialog(DialogKind::ExportWarning, Some(id));
        }
    }

    pub fn open_dialog(&mut self, kind: DialogKind, target: Option<EntryId>) {
        let name = target
            .and_then(|id| self.entries.get(&id))
            .map(|entry| entry.name.clone())
            .unwrap_or_default();
        self.dialog = Some(Dialog {
            kind,
            name,
            target,
            error: None,
        });
    }

    pub fn dialog_mut(&mut self) -> Option<&mut Dialog> {
        self.dialog.as_mut()
    }

    pub fn close_dialog(&mut self) {
        self.dialog = None;
    }

    pub fn create_folder(&mut self, name: &str) -> Result<EntryId, String> {
        self.ensure_files_destination()?;
        self.insert_name(name, EntryKind::Folder, self.current_folder, 0)
    }

    pub fn import_synthetic(&mut self, name: &str) -> Result<EntryId, String> {
        self.ensure_files_destination()?;
        self.insert_name(name, EntryKind::File, self.current_folder, 24_576)
    }

    pub fn rename(&mut self, id: EntryId, name: &str) -> Result<(), String> {
        let (parent, old_name) = self
            .entries
            .get(&id)
            .map(|entry| (entry.parent_id.unwrap_or(EntryId(0)), entry.name.clone()))
            .ok_or_else(|| "That synthetic item no longer exists.".to_owned())?;
        let cleaned = validate_name(name)?;
        if cleaned.eq_ignore_ascii_case(&old_name) {
            return Ok(());
        }
        self.ensure_name_available(parent, &cleaned, Some(id))?;
        self.entries.get_mut(&id).expect("entry checked above").name = cleaned;
        self.set_notice("Renamed synthetic item.");
        Ok(())
    }

    pub fn move_selected_to_trash(&mut self) -> Vec<EntryId> {
        let selected = self.selected_ids();
        self.move_to_trash(&selected)
    }

    pub fn move_to_trash(&mut self, ids: &[EntryId]) -> Vec<EntryId> {
        let roots = self.selected_roots(ids);
        let timestamp = "2026-10-18 12:00".to_owned();
        for id in &roots {
            for descendant in self.subtree(*id) {
                if let Some(entry) = self.entries.get_mut(&descendant) {
                    entry.deleted_at = Some(timestamp.clone());
                }
            }
        }
        self.clear_selection();
        roots
    }

    pub fn restore_selected(&mut self) -> Vec<EntryId> {
        let selected = self.selected_ids();
        self.restore(&selected)
    }

    pub fn restore(&mut self, ids: &[EntryId]) -> Vec<EntryId> {
        let roots = self.selected_roots(ids);
        let mut restored = Vec::new();
        for id in roots {
            let Some(entry) = self.entries.get(&id).cloned() else {
                continue;
            };
            let parent = entry
                .parent_id
                .filter(|parent| {
                    self.entries
                        .get(parent)
                        .is_some_and(|item| item.deleted_at.is_none())
                })
                .unwrap_or(EntryId(0));
            let name = self.available_restore_name(parent, &entry.name, id);
            if let Some(root) = self.entries.get_mut(&id) {
                root.parent_id = Some(parent);
                root.name = name;
            }
            for descendant in self.subtree(id) {
                if let Some(item) = self.entries.get_mut(&descendant) {
                    item.deleted_at = None;
                }
            }
            restored.push(id);
        }
        self.set_page(Page::Files);
        self.set_notice(format!("Restored {} synthetic item(s).", restored.len()));
        restored
    }

    pub fn permanently_delete_selected(&mut self) -> Vec<EntryId> {
        let selected = self.selected_ids();
        self.permanently_delete(&selected)
    }

    pub fn permanently_delete(&mut self, ids: &[EntryId]) -> Vec<EntryId> {
        let roots = self.selected_roots(ids);
        for id in &roots {
            for descendant in self.subtree(*id) {
                self.entries.remove(&descendant);
            }
        }
        self.clear_selection();
        self.set_notice(format!(
            "Permanently removed {} synthetic item(s).",
            roots.len()
        ));
        roots
    }

    pub fn start_operation(&mut self, label: &str, total: u32, fail_at: Option<u32>) {
        let total = total.max(1);
        self.operation = OperationStatus::Running {
            label: label.to_owned(),
            completed: 0,
            total,
            fail_at,
        };
        self.pending_action = None;
    }

    pub fn queue_after_success(&mut self, action: SyntheticAction) {
        self.pending_action = Some(action);
    }

    /// Advances the deterministic progress simulation. Returns true on any terminal state.
    pub fn advance_operation(&mut self) -> bool {
        let OperationStatus::Running {
            label,
            completed,
            total,
            fail_at,
        } = self.operation.clone()
        else {
            return false;
        };
        let completed = completed + 1;
        if fail_at == Some(completed) {
            self.operation = OperationStatus::Failed {
                label,
                message: "The synthetic operation failed at a controlled progress step.".to_owned(),
            };
            self.pending_action = None;
            return true;
        }
        if completed >= total {
            self.operation = OperationStatus::Succeeded { label };
            return true;
        }
        self.operation = OperationStatus::Running {
            label,
            completed,
            total,
            fail_at,
        };
        false
    }

    pub fn cancel_operation(&mut self) {
        if let OperationStatus::Running { label, .. } = &self.operation {
            self.operation = OperationStatus::Cancelled {
                label: label.clone(),
            };
        }
        self.pending_action = None;
    }

    pub fn finish_operation(&mut self) {
        if matches!(&self.operation, OperationStatus::Succeeded { .. }) {
            match self.pending_action.take() {
                Some(SyntheticAction::Import { parent, name }) => {
                    self.current_folder = parent;
                    if let Err(error) = self.insert_name(&name, EntryKind::File, parent, 24_576) {
                        self.set_error(error);
                    }
                }
                Some(SyntheticAction::MoveToTrash(ids)) => {
                    self.move_to_trash(&ids);
                    self.set_notice("Moved selected synthetic items to Trash.");
                }
                Some(SyntheticAction::Restore(ids)) => {
                    self.restore(&ids);
                }
                Some(SyntheticAction::PermanentlyDelete(ids)) => {
                    self.permanently_delete(&ids);
                }
                None => {}
            }
        }
    }

    pub fn fail_operation_now(&mut self) {
        if let OperationStatus::Running { label, .. } = &self.operation {
            self.operation = OperationStatus::Failed {
                label: label.clone(),
                message: "The synthetic operation failed on request.".to_owned(),
            };
        }
        self.pending_action = None;
    }

    pub fn set_loading_state(&mut self, loading: bool) {
        self.loading = loading;
    }

    fn insert_name(
        &mut self,
        name: &str,
        kind: EntryKind,
        parent: EntryId,
        size_bytes: u64,
    ) -> Result<EntryId, String> {
        let cleaned = validate_name(name)?;
        if !self
            .entries
            .get(&parent)
            .is_some_and(|entry| entry.kind == EntryKind::Folder && entry.deleted_at.is_none())
            && parent != EntryId(0)
        {
            return Err("Choose an active folder.".to_owned());
        }
        self.ensure_name_available(parent, &cleaned, None)?;
        let id = EntryId(self.next_id);
        self.next_id += 1;
        let entry = Entry {
            id,
            name: cleaned,
            kind,
            parent_id: Some(parent),
            size_bytes,
            created_at: "2026-10-18 12:00".to_owned(),
            modified_at: "2026-10-18 12:00".to_owned(),
            deleted_at: None,
        };
        self.entries.insert(id, entry);
        self.set_notice(match kind {
            EntryKind::Folder => "Created synthetic folder.",
            EntryKind::File => "Imported synthetic file.",
        });
        Ok(id)
    }

    fn ensure_files_destination(&self) -> Result<(), String> {
        if self.page != Page::Files || !self.search_query.is_empty() {
            Err("Choose a folder in Files before changing synthetic items.".to_owned())
        } else {
            Ok(())
        }
    }

    fn ensure_name_available(
        &self,
        parent: EntryId,
        name: &str,
        except: Option<EntryId>,
    ) -> Result<(), String> {
        if self.entries.values().any(|entry| {
            Some(entry.id) != except
                && entry.parent_id == Some(parent)
                && entry.deleted_at.is_none()
                && entry.name.eq_ignore_ascii_case(name)
        }) {
            Err("An item with that name already exists in this folder.".to_owned())
        } else {
            Ok(())
        }
    }

    fn available_restore_name(&self, parent: EntryId, original: &str, id: EntryId) -> String {
        if self
            .ensure_name_available(parent, original, Some(id))
            .is_ok()
        {
            return original.to_owned();
        }
        let (stem, extension) = original
            .rsplit_once('.')
            .map(|(stem, extension)| (stem, format!(".{extension}")))
            .unwrap_or((original, String::new()));
        for number in 2..=10_000 {
            let candidate = format!("{stem} (restored {number}){extension}");
            if self
                .ensure_name_available(parent, &candidate, Some(id))
                .is_ok()
            {
                return candidate;
            }
        }
        format!("{stem} ({}){extension}", id.0)
    }

    fn selected_roots(&self, ids: &[EntryId]) -> Vec<EntryId> {
        let set: BTreeSet<EntryId> = ids.iter().copied().collect();
        set.iter()
            .copied()
            .filter(|id| {
                let mut parent = self.entries.get(id).and_then(|entry| entry.parent_id);
                while let Some(parent_id) = parent {
                    if set.contains(&parent_id) {
                        return false;
                    }
                    parent = self
                        .entries
                        .get(&parent_id)
                        .and_then(|entry| entry.parent_id);
                }
                self.entries.contains_key(id)
            })
            .collect()
    }

    fn subtree(&self, root: EntryId) -> Vec<EntryId> {
        let mut children: BTreeMap<EntryId, Vec<EntryId>> = BTreeMap::new();
        for entry in self.entries.values() {
            if let Some(parent) = entry.parent_id {
                children.entry(parent).or_default().push(entry.id);
            }
        }
        let mut result = Vec::new();
        let mut pending = vec![root];
        while let Some(id) = pending.pop() {
            result.push(id);
            pending.extend(children.get(&id).into_iter().flatten().copied());
        }
        result
    }

    fn push_history(&mut self, id: EntryId) {
        if self.navigation_history.get(self.history_index) == Some(&id) {
            return;
        }
        self.navigation_history.truncate(self.history_index + 1);
        self.navigation_history.push(id);
        self.history_index = self.navigation_history.len() - 1;
    }

    fn clear_selection(&mut self) {
        self.selected.clear();
        self.focused = None;
        self.anchor = None;
    }

    fn prune_selection_to_visible(&mut self) {
        let visible: BTreeSet<EntryId> = self.visible_ids().into_iter().collect();
        self.selected.retain(|id| visible.contains(id));
        if self.focused.is_some_and(|id| !visible.contains(&id)) {
            self.focused = None;
        }
        if self.anchor.is_some_and(|id| !visible.contains(&id)) {
            self.anchor = self.focused;
        }
    }
}

fn validate_name(name: &str) -> Result<String, String> {
    let cleaned = name.trim();
    if cleaned.is_empty() {
        return Err("Enter a name.".to_owned());
    }
    if cleaned.chars().count() > 255 {
        return Err("Names must be 255 characters or fewer.".to_owned());
    }
    if cleaned.ends_with('.')
        || cleaned.chars().any(|character| {
            character.is_control()
                || matches!(
                    character,
                    '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
                )
        })
        || is_reserved_windows_device_name(cleaned)
    {
        return Err("This name is not valid for a Windows file or folder.".to_owned());
    }
    Ok(cleaned.to_owned())
}

fn is_reserved_windows_device_name(name: &str) -> bool {
    let device = name
        .split('.')
        .next()
        .unwrap_or(name)
        .trim_end_matches([' ', '.'])
        .to_ascii_uppercase();
    if matches!(
        device.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
    ) {
        return true;
    }

    let Some(suffix) = device
        .strip_prefix("COM")
        .or_else(|| device.strip_prefix("LPT"))
    else {
        return false;
    };
    matches!(
        suffix,
        "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
    )
}
