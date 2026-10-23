use std::{cell::RefCell, rc::Rc, time::Duration};

use encryptdrive_spike_common::{
    Appearance, DialogKind, EntryId, EntryKind, ExplorerModel, OperationStatus, Page,
    SemanticTokens, SortColumn, SyntheticAction, ThemeMode, make_fixture,
};
use slint::{ModelRc, Timer, TimerMode, VecModel};

slint::include_modules!();

#[derive(Clone, Copy)]
struct KeyModifiers {
    control: bool,
    shift: bool,
    alt: bool,
}

fn main() -> Result<(), slint::PlatformError> {
    let size = std::env::args()
        .collect::<Vec<_>>()
        .windows(2)
        .find(|pair| pair[0] == "--entries")
        .and_then(|pair| pair[1].parse::<usize>().ok())
        .filter(|value| matches!(value, 100 | 1_000 | 10_000))
        .unwrap_or(100);

    let model = Rc::new(RefCell::new(ExplorerModel::new(make_fixture(size))));
    let window = MainWindow::new()?;
    let operation_timer = Rc::new(RefCell::new(Timer::default()));
    let context_target = Rc::new(RefCell::new(None::<EntryId>));
    sync_window(&window, &model.borrow());

    {
        let weak = window.as_weak();
        let model = model.clone();
        window.on_navigate_page(move |page| {
            let mut state = model.borrow_mut();
            state.set_page(if page == "Trash" {
                Page::Trash
            } else {
                Page::Files
            });
            if let Some(window) = weak.upgrade() {
                sync_window(&window, &state);
            }
        });
    }
    {
        let weak = window.as_weak();
        let state = model.clone();
        window.on_navigate_back(move || {
            let mut model = state.borrow_mut();
            model.navigate_back();
            if let Some(window) = weak.upgrade() {
                sync_window(&window, &model);
            }
        });
    }
    {
        let weak = window.as_weak();
        let state = model.clone();
        window.on_navigate_up(move || {
            let mut model = state.borrow_mut();
            model.navigate_parent();
            if let Some(window) = weak.upgrade() {
                sync_window(&window, &model);
            }
        });
    }
    {
        let weak = window.as_weak();
        let state = model.clone();
        window.on_navigate_crumb(move |index| {
            let mut model = state.borrow_mut();
            if let Some((id, _)) = model.breadcrumb().get(index as usize).cloned() {
                model.navigate_to(id);
            }
            if let Some(window) = weak.upgrade() {
                sync_window(&window, &model);
            }
        });
    }
    {
        let weak = window.as_weak();
        let state = model.clone();
        window.on_search_changed(move |query| {
            let mut model = state.borrow_mut();
            model.set_search(query.to_string());
            if let Some(window) = weak.upgrade() {
                sync_window(&window, &model);
            }
        });
    }
    {
        let state = model.clone();
        let timer = operation_timer.clone();
        let context = context_target.clone();
        let window_for_action = window.as_weak();
        window.on_action(move |action| {
            let Some(window) = window_for_action.upgrade() else {
                return;
            };
            handle_action(action.as_str(), &window, &state, &timer, &context);
        });
    }
    {
        let weak = window.as_weak();
        let state = model.clone();
        window.on_row_clicked(move |id, control, shift| {
            let mut model = state.borrow_mut();
            let id = EntryId(id as u64);
            if shift {
                model.select_range(id);
            } else if control {
                model.toggle_selection(id);
            } else {
                model.select_only(id);
            }
            if let Some(window) = weak.upgrade() {
                sync_window(&window, &model);
            }
        });
    }
    {
        let weak = window.as_weak();
        let state = model.clone();
        let action_weak = window.as_weak();
        window.on_row_activated(move |id| {
            let mut model = state.borrow_mut();
            model.activate(EntryId(id as u64));
            if let Some(window) = weak.upgrade() {
                sync_window(&window, &model);
            }
            if let Some(window) = action_weak.upgrade() {
                refresh_dialog(&window, &model);
            }
        });
    }
    {
        let weak = window.as_weak();
        let state = model.clone();
        let target = context_target.clone();
        window.on_row_context(move |id| {
            let mut model = state.borrow_mut();
            let id = EntryId(id as u64);
            if !model.selected_ids().contains(&id) {
                model.select_only(id);
            }
            *target.borrow_mut() = Some(id);
            if let Some(window) = weak.upgrade() {
                window.set_context_id(id.0 as i32);
                sync_window(&window, &model);
            }
        });
    }
    {
        let weak = window.as_weak();
        let state = model.clone();
        window.on_sort(move |column| {
            let mut model = state.borrow_mut();
            let column = match column.as_str() {
                "Size" => SortColumn::Size,
                "Modified" => SortColumn::Modified,
                _ => SortColumn::Name,
            };
            model.toggle_sort(column);
            if let Some(window) = weak.upgrade() {
                sync_window(&window, &model);
            }
        });
    }
    {
        let weak = window.as_weak();
        let state = model.clone();
        let timer = operation_timer.clone();
        let context = context_target.clone();
        let action_weak = window.as_weak();
        window.on_key_input(move |key, control, shift, alt| {
            handle_key(
                key.as_str(),
                KeyModifiers {
                    control,
                    shift,
                    alt,
                },
                &action_weak,
                &state,
                &timer,
                &context,
            );
            if let Some(window) = weak.upgrade() {
                sync_window(&window, &state.borrow());
            }
        });
    }
    {
        let weak = window.as_weak();
        let state = model.clone();
        window.on_close_dialog(move || {
            let mut model = state.borrow_mut();
            model.close_dialog();
            if let Some(window) = weak.upgrade() {
                sync_window(&window, &model);
            }
        });
    }
    {
        let weak = window.as_weak();
        let state = model.clone();
        let timer = operation_timer.clone();
        let target = context_target.clone();
        window.on_confirm_dialog(move || {
            if let Some(window) = weak.upgrade() {
                confirm_dialog(&window, &state, &timer, &target);
            }
        });
    }
    {
        let weak = window.as_weak();
        let state = model.clone();
        window.on_form_changed(move |_| {
            if let Some(dialog) = state.borrow_mut().dialog_mut() {
                dialog.error = None;
            }
            if let Some(window) = weak.upgrade() {
                window.set_form_error(slint::SharedString::default());
            }
        });
    }
    {
        let weak = window.as_weak();
        let state = model.clone();
        window.on_theme_toggle(move || {
            let mut model = state.borrow_mut();
            let next_appearance = if model.appearance() == Appearance::Dark {
                Appearance::Light
            } else {
                Appearance::Dark
            };
            model.set_appearance(next_appearance);
            if let Some(window) = weak.upgrade() {
                sync_window(&window, &model);
            }
        });
    }
    {
        let weak = window.as_weak();
        let state = model.clone();
        let timer = operation_timer.clone();
        window.on_size_changed(move |size| {
            let appearance = state.borrow().appearance();
            let mut model = ExplorerModel::new(make_fixture(size.max(100) as usize));
            model.set_appearance(appearance);
            *state.borrow_mut() = model;
            timer.borrow().stop();
            if let Some(window) = weak.upgrade() {
                sync_window(&window, &state.borrow());
            }
        });
    }
    {
        let weak = window.as_weak();
        let state = model.clone();
        window.on_dismiss_notice(move || {
            let mut model = state.borrow_mut();
            model.dismiss_notice();
            if let Some(window) = weak.upgrade() {
                sync_window(&window, &model);
            }
        });
    }

    window.run()
}

fn handle_action(
    action: &str,
    window: &MainWindow,
    model: &Rc<RefCell<ExplorerModel>>,
    timer: &Rc<RefCell<Timer>>,
    context: &Rc<RefCell<Option<EntryId>>>,
) {
    match action {
        "settings" => window.set_settings_open(!window.get_settings_open()),
        "create-folder" => open_dialog(window, model, DialogKind::CreateFolder, None),
        "import" => open_dialog(window, model, DialogKind::Import, None),
        "export" => {
            let selected = model.borrow().selected_ids();
            open_dialog(
                window,
                model,
                DialogKind::ExportWarning,
                selected.first().copied(),
            );
        }
        "rename" => {
            let selected = model.borrow().selected_ids();
            open_dialog(window, model, DialogKind::Rename, selected.first().copied());
        }
        "delete" => {
            let kind = model.borrow().delete_dialog_kind();
            open_dialog(window, model, kind, None);
        }
        "context-activate" => {
            if let Some(id) = *context.borrow() {
                model.borrow_mut().activate(id);
            }
            refresh_dialog(window, &model.borrow());
        }
        "context-rename" => {
            let id = *context.borrow();
            open_dialog(window, model, DialogKind::Rename, id);
        }
        "context-delete-restore" => {
            if let Some(id) = *context.borrow() {
                if model.borrow().page() == Page::Files {
                    model.borrow_mut().select_only(id);
                    open_dialog(window, model, DialogKind::DeleteToTrash, None);
                } else {
                    let mut state = model.borrow_mut();
                    state.restore(&[id]);
                    sync_window(window, &state);
                }
            }
        }
        "cancel-operation" => {
            timer.borrow().stop();
            let mut state = model.borrow_mut();
            state.cancel_operation();
            state.set_notice("Synthetic operation cancelled; no files were changed.");
            sync_window(window, &state);
        }
        "simulate-loading" => {
            let mut state = model.borrow_mut();
            state.simulate_loading();
            sync_window(window, &state);
        }
        "simulate-error" => {
            let mut state = model.borrow_mut();
            state.simulate_error();
            sync_window(window, &state);
        }
        "retry" => {
            let mut state = model.borrow_mut();
            state.finish_loading();
            sync_window(window, &state);
        }
        "recovery" => open_dialog(window, model, DialogKind::RecoveryNotice, None),
        "operation-success" => {
            start_operation(window, model, timer, "Synthetic operation", 36, None, None)
        }
        "operation-failure" => start_operation(
            window,
            model,
            timer,
            "Controlled failure simulation",
            36,
            Some(12),
            None,
        ),
        _ => {}
    }
}

fn handle_key(
    key: &str,
    modifiers: KeyModifiers,
    weak: &slint::Weak<MainWindow>,
    model: &Rc<RefCell<ExplorerModel>>,
    timer: &Rc<RefCell<Timer>>,
    context: &Rc<RefCell<Option<EntryId>>>,
) {
    if model.borrow().dialog().is_some() {
        if key == "Escape" {
            model.borrow_mut().close_dialog();
        }
        return;
    }
    let mut state = model.borrow_mut();
    if modifiers.control && key.eq_ignore_ascii_case("a") {
        state.select_all_visible();
    } else if modifiers.control && key.eq_ignore_ascii_case("f") {
        // The declarative FocusScope moves focus to the search field.
    } else if modifiers.shift && key == "F10" {
        if let Some(id) = state
            .focused_id()
            .or_else(|| state.selected_ids().first().copied())
        {
            if !state.selected_ids().contains(&id) {
                state.select_only(id);
            }
            *context.borrow_mut() = Some(id);
            if let Some(window) = weak.upgrade() {
                window.set_context_id(id.0 as i32);
            }
        }
    } else if modifiers.alt && key == "Left" {
        state.navigate_back();
    } else if modifiers.alt && key == "Up" {
        state.navigate_parent();
    } else if key == "Escape" {
        if let Some(window) = weak.upgrade() {
            window.set_context_id(-1);
        }
        if state.operation().is_running() {
            state.cancel_operation();
            state.set_notice("Synthetic operation cancelled; no files were changed.");
            timer.borrow().stop();
        } else {
            state.set_search("");
        }
    } else if key == "Down" {
        state.move_focus(1, modifiers.shift);
    } else if key == "Up" {
        state.move_focus(-1, modifiers.shift);
    } else if key == "Enter" {
        if let Some(id) = state.focused_id() {
            state.activate(id);
        }
    } else if key == "F2" {
        let selected = state.selected_ids();
        if selected.len() == 1 {
            state.open_dialog(DialogKind::Rename, Some(selected[0]));
        }
    } else if key == "Delete" && !state.selected_ids().is_empty() {
        let kind = state.delete_dialog_kind();
        state.open_dialog(kind, None);
    }
    drop(state);
    if let Some(window) = weak.upgrade() {
        if let Some(dialog) = model.borrow().dialog() {
            window.set_form_name(dialog.name.clone().into());
            refresh_dialog(&window, &model.borrow());
        }
        if key == "Escape" {
            window.set_search_text(slint::SharedString::default());
        }
        sync_window(&window, &model.borrow());
    }
}

fn open_dialog(
    window: &MainWindow,
    model: &Rc<RefCell<ExplorerModel>>,
    kind: DialogKind,
    target: Option<EntryId>,
) {
    model.borrow_mut().open_dialog(kind, target);
    if let Some(dialog) = model.borrow().dialog() {
        window.set_form_name(dialog.name.clone().into());
    }
    refresh_dialog(window, &model.borrow());
}

fn refresh_dialog(window: &MainWindow, model: &ExplorerModel) {
    let Some(dialog) = model.dialog() else {
        window.set_dialog_kind(slint::SharedString::default());
        window.set_dialog_title(slint::SharedString::default());
        window.set_dialog_copy(slint::SharedString::default());
        window.set_form_error(slint::SharedString::default());
        return;
    };
    let (kind, title, copy, action) = match dialog.kind {
        DialogKind::CreateFolder => (
            "create-folder",
            "Create folder",
            "Add a folder to this synthetic location.",
            "Create",
        ),
        DialogKind::Rename => (
            "rename",
            "Rename item",
            "Choose a new name for this synthetic entry.",
            "Rename",
        ),
        DialogKind::DeleteToTrash => (
            "delete",
            "Move to Trash?",
            "Only selected synthetic items will move to Trash.",
            "Move to Trash",
        ),
        DialogKind::PermanentDelete => (
            "permanent-delete",
            "Delete permanently?",
            "This permanently removes synthetic entries from memory. No secure-erasure claim applies.",
            "Delete permanently",
        ),
        DialogKind::Import => (
            "import",
            "Simulated Import",
            "No picker is opened and no local file is read.",
            "Start simulated import",
        ),
        DialogKind::ExportWarning => (
            "export",
            "Export plaintext copy?",
            "The exported copy would be plaintext outside EncryptDrive. This prototype does not decrypt, write, or launch files.",
            "Continue to simulated Export",
        ),
        DialogKind::RecoveryNotice => (
            "recovery",
            "Recovery notice",
            "Synthetic recovery notice only. No vault or metadata recovery ran.",
            "Acknowledge",
        ),
    };
    window.set_dialog_kind(kind.into());
    window.set_dialog_title(title.into());
    window.set_dialog_copy(copy.into());
    window.set_dialog_action(action.into());
    window.set_form_error(dialog.error.clone().unwrap_or_default().into());
}

fn confirm_dialog(
    window: &MainWindow,
    model: &Rc<RefCell<ExplorerModel>>,
    timer: &Rc<RefCell<Timer>>,
    context: &Rc<RefCell<Option<EntryId>>>,
) {
    let Some(dialog) = model.borrow().dialog().cloned() else {
        return;
    };
    let name = window.get_form_name().to_string();
    let mut operation: Option<(&str, Option<u32>, Option<SyntheticAction>)> = None;
    {
        let mut state = model.borrow_mut();
        match dialog.kind {
            DialogKind::CreateFolder => match state.create_folder(&name) {
                Ok(_) => state.close_dialog(),
                Err(error) => {
                    state.dialog_mut().unwrap().error = Some(error);
                }
            },
            DialogKind::Rename => match dialog.target.map(|id| state.rename(id, &name)) {
                Some(Ok(())) => state.close_dialog(),
                Some(Err(error)) => {
                    state.dialog_mut().unwrap().error = Some(error);
                }
                None => {
                    state.dialog_mut().unwrap().error =
                        Some("Select one item to rename.".to_owned());
                }
            },
            DialogKind::DeleteToTrash => {
                operation = Some((
                    "Moving items to Trash",
                    None,
                    Some(SyntheticAction::MoveToTrash(state.selected_ids())),
                ));
                state.close_dialog();
            }
            DialogKind::PermanentDelete => {
                operation = Some((
                    "Deleting synthetic items",
                    None,
                    Some(SyntheticAction::PermanentlyDelete(state.selected_ids())),
                ));
                state.close_dialog();
            }
            DialogKind::Import => {
                let import = SyntheticAction::Import {
                    parent: state.current_folder(),
                    name,
                };
                state.close_dialog();
                operation = Some(("Importing synthetic item", None, Some(import)));
            }
            DialogKind::ExportWarning => {
                state.close_dialog();
                operation = Some(("Preparing simulated Export", None, None));
            }
            DialogKind::RecoveryNotice => {
                state.close_dialog();
                state.set_notice("Synthetic recovery notification acknowledged. No recovery ran.");
            }
        }
    }
    if let Some((label, fail_at, queued)) = operation {
        start_operation(window, model, timer, label, 36, fail_at, queued);
    }
    *context.borrow_mut() = None;
    sync_window(window, &model.borrow());
    refresh_dialog(window, &model.borrow());
}

fn start_operation(
    window: &MainWindow,
    model: &Rc<RefCell<ExplorerModel>>,
    timer: &Rc<RefCell<Timer>>,
    label: &str,
    total: u32,
    fail_at: Option<u32>,
    action: Option<SyntheticAction>,
) {
    {
        let mut state = model.borrow_mut();
        state.start_operation(label, total, fail_at);
        if let Some(action) = action {
            state.queue_after_success(action);
        }
    }
    start_timer(window, model, timer, label);
}

fn start_timer(
    window: &MainWindow,
    model: &Rc<RefCell<ExplorerModel>>,
    timer: &Rc<RefCell<Timer>>,
    label: &str,
) {
    let weak = window.as_weak();
    let state = model.clone();
    let timer_for_callback = timer.clone();
    let label = label.to_owned();
    timer
        .borrow()
        .start(TimerMode::Repeated, Duration::from_millis(45), move || {
            let mut model = state.borrow_mut();
            let ended = model.advance_operation();
            if ended {
                timer_for_callback.borrow().stop();
                match model.operation().clone() {
                    OperationStatus::Succeeded { .. } => {
                        model.finish_operation();
                        model.set_notice(format!("{label} completed.").to_owned());
                    }
                    OperationStatus::Failed { message, .. } => model.set_error(message),
                    _ => {}
                }
            }
            if let Some(window) = weak.upgrade() {
                sync_window(&window, &model);
            }
        });
    sync_window(window, &model.borrow());
}

fn sync_window(window: &MainWindow, model: &ExplorerModel) {
    let selected = model.selected_ids();
    let focused = model.focused_id();
    let rows = model
        .visible_entries()
        .into_iter()
        .map(|entry| {
            let kind = if entry.kind == EntryKind::Folder {
                "folder"
            } else {
                "file"
            };
            FileRow {
                id: entry.id.0 as i32,
                name: entry.name.clone().into(),
                size: if entry.kind == EntryKind::Folder {
                    "—".into()
                } else {
                    format_size(entry.size_bytes).into()
                },
                modified: entry.modified_at.clone().into(),
                kind: kind.into(),
                selected: selected.contains(&entry.id),
                focused: focused == Some(entry.id),
                accessible_name: format!(
                    "{} {}, {}, modified {}{}",
                    if kind == "folder" { "Folder" } else { "File" },
                    entry.name,
                    if kind == "folder" {
                        "".to_owned()
                    } else {
                        format_size(entry.size_bytes)
                    },
                    entry.modified_at,
                    if selected.contains(&entry.id) {
                        ", selected"
                    } else {
                        ""
                    }
                )
                .into(),
            }
        })
        .collect::<Vec<_>>();
    window.set_rows(ModelRc::new(VecModel::from(rows)));
    window.set_current_page(if model.page() == Page::Files {
        "Files".into()
    } else {
        "Trash".into()
    });
    let crumbs = model
        .breadcrumb()
        .into_iter()
        .map(|(_, name)| name)
        .collect::<Vec<_>>();
    window.set_breadcrumb(crumbs.join("  ›  ").into());
    window.set_breadcrumb_parts(ModelRc::new(VecModel::from(
        crumbs
            .into_iter()
            .map(slint::SharedString::from)
            .collect::<Vec<_>>(),
    )));
    window.set_search_text(model.search_query().into());
    window.set_selected_count(format!("{} selected", selected.len()).into());
    window.set_notice(model.notice().unwrap_or_default().into());
    window.set_notice_is_error(model.is_error());
    window.set_loading(model.is_loading());
    window.set_theme_name(if model.appearance() == Appearance::Dark {
        "Dark".into()
    } else {
        "Light".into()
    });
    apply_tokens(window, model.appearance().resolved(false));
    match model.operation() {
        OperationStatus::Running {
            label,
            completed,
            total,
            ..
        } => {
            window.set_operation_running(true);
            window.set_operation_label(label.clone().into());
            window.set_operation_progress((completed.saturating_mul(100) / (*total).max(1)) as i32);
        }
        OperationStatus::Succeeded { label } => {
            window.set_operation_running(false);
            window.set_operation_label(format!("{label} complete").into());
            window.set_operation_progress(100);
        }
        OperationStatus::Failed { label, .. } => {
            window.set_operation_running(false);
            window.set_operation_label(format!("{label} failed").into());
        }
        OperationStatus::Cancelled { label } => {
            window.set_operation_running(false);
            window.set_operation_label(format!("{label} cancelled").into());
        }
        OperationStatus::Idle => {
            window.set_operation_running(false);
            window.set_operation_label(slint::SharedString::default());
            window.set_operation_progress(0);
        }
    }
    refresh_dialog(window, model);
}

fn apply_tokens(window: &MainWindow, mode: ThemeMode) {
    let t = SemanticTokens::for_mode(mode);
    window.set_canvas(color(t.canvas));
    window.set_surface(color(t.surface));
    window.set_surface_secondary(color(t.surface_secondary));
    window.set_primary_text(color(t.primary_text));
    window.set_secondary_text(color(t.secondary_text));
    window.set_border(color(t.separator));
    window.set_accent(color(t.primary_action));
    window.set_selection(color(t.selection));
    window.set_selection_text(color(t.selection_text));
    window.set_hover(color(t.hover));
    window.set_focus_ring(color(t.focus));
    window.set_error(color(t.error));
    window.set_error_surface(color(t.error_surface));
    window.set_success(color(t.success));
    window.set_success_surface(color(t.success_surface));
}

fn color(rgb: u32) -> slint::Color {
    slint::Color::from_rgb_u8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::format_size;

    #[test]
    fn format_size_matches_eframe_binary_units() {
        assert_eq!(format_size(82_410), "80.5 KB");
        assert_eq!(format_size(152_300), "148.7 KB");
        assert_eq!(format_size(1_048_576), "1.0 MB");
    }
}
