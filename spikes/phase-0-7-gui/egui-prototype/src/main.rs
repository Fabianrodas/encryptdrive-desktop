use std::{collections::BTreeSet, time::Duration};

use eframe::egui::{self, Align, Align2, Color32, Key, Layout, Modifiers, RichText, Sense, Vec2};
use egui_extras::{Column, TableBuilder};
use encryptdrive_spike_common::{
    Appearance, DialogKind, Entry, EntryId, EntryKind, ExplorerModel, OperationStatus, Page,
    SemanticTokens, SortColumn, SyntheticAction, ThemeMode, make_fixture,
};

const ROW_HEIGHT: f32 = 36.0;

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();
    let entries = args
        .windows(2)
        .find(|pair| pair[0] == "--entries")
        .and_then(|pair| pair[1].parse::<usize>().ok())
        .filter(|value| matches!(value, 100 | 1_000 | 10_000))
        .unwrap_or(100);
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_title("EncryptDrive — Synthetic files")
            .with_inner_size([1180.0, 760.0])
            .with_min_inner_size([780.0, 520.0]),
        ..Default::default()
    };
    eframe::run_native(
        "EncryptDrive — Synthetic files",
        options,
        Box::new(move |_creation| Ok(Box::new(ExplorerApp::new(entries)))),
    )
}

struct ExplorerApp {
    model: ExplorerModel,
    entries_count: usize,
    search: String,
    form_name: String,
    dialog_focus_pending: bool,
    search_focus_pending: bool,
    table_focused: bool,
    context_target: Option<EntryId>,
    context_menu_open: bool,
    settings_open: bool,
    loading_frames: u8,
}

impl ExplorerApp {
    fn new(entries_count: usize) -> Self {
        Self {
            model: ExplorerModel::new(make_fixture(entries_count)),
            entries_count,
            search: String::new(),
            form_name: String::new(),
            dialog_focus_pending: true,
            search_focus_pending: false,
            table_focused: false,
            context_target: None,
            context_menu_open: false,
            settings_open: false,
            loading_frames: 0,
        }
    }

    fn set_size(&mut self, count: usize) {
        self.entries_count = count;
        let appearance = self.model.appearance();
        self.model = ExplorerModel::new(make_fixture(count));
        self.model.set_appearance(appearance);
        self.search.clear();
        self.table_focused = false;
    }

    fn open_dialog(&mut self, kind: DialogKind, target: Option<EntryId>) {
        self.model.open_dialog(kind, target);
        if let Some(dialog) = self.model.dialog() {
            self.form_name = dialog.name.clone();
        }
        self.dialog_focus_pending = true;
        self.context_menu_open = false;
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let text_input_focused = ctx.egui_wants_keyboard_input();
        if self.model.dialog().is_some() {
            if ctx.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape)) {
                self.model.close_dialog();
            }
            return;
        }

        if ctx.input_mut(|input| input.consume_key(Modifiers::CTRL, Key::F)) {
            self.search_focus_pending = true;
            self.table_focused = false;
        }
        if !text_input_focused && ctx.input_mut(|input| input.consume_key(Modifiers::CTRL, Key::A))
        {
            self.model.select_all_visible();
        }
        if !text_input_focused
            && ctx.input_mut(|input| input.consume_key(Modifiers::SHIFT, Key::F10))
        {
            self.context_target = self
                .model
                .focused_id()
                .or_else(|| self.model.selected_ids().first().copied());
            self.context_menu_open = self.context_target.is_some();
        }
        if !text_input_focused
            && ctx.input_mut(|input| input.consume_key(Modifiers::ALT, Key::ArrowLeft))
        {
            self.model.navigate_back();
        }
        if !text_input_focused
            && ctx.input_mut(|input| input.consume_key(Modifiers::ALT, Key::ArrowUp))
        {
            self.model.navigate_parent();
        }
        if !text_input_focused && self.table_focused {
            if ctx.input_mut(|input| input.consume_key(Modifiers::NONE, Key::ArrowDown)) {
                let extend = ctx.input(|input| input.modifiers.shift);
                self.model.move_focus(1, extend);
            }
            if ctx.input_mut(|input| input.consume_key(Modifiers::NONE, Key::ArrowUp)) {
                let extend = ctx.input(|input| input.modifiers.shift);
                self.model.move_focus(-1, extend);
            }
            if ctx.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Enter))
                && let Some(id) = self.model.focused_id()
            {
                self.model.activate(id);
                self.dialog_focus_pending = true;
            }
            if ctx.input_mut(|input| input.consume_key(Modifiers::NONE, Key::F2)) {
                let selected = self.model.selected_ids();
                if selected.len() == 1 {
                    self.open_dialog(DialogKind::Rename, Some(selected[0]));
                }
            }
            if ctx.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Delete))
                && !self.model.selected_ids().is_empty()
            {
                let kind = self.model.delete_dialog_kind();
                self.open_dialog(kind, None);
            }
        }
        if ctx.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape)) {
            if self.context_menu_open {
                self.context_menu_open = false;
            } else if !self.search.is_empty() {
                self.search.clear();
                self.model.set_search("");
            } else {
                self.table_focused = false;
            }
        }
    }

    fn theme(&self) -> ThemeMode {
        self.model.appearance().resolved(false)
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        let mode = self.theme();
        let tokens = SemanticTokens::for_mode(mode);
        let mut visuals = match mode {
            ThemeMode::Light => egui::Visuals::light(),
            ThemeMode::Dark => egui::Visuals::dark(),
        };
        visuals.panel_fill = color(tokens.canvas);
        visuals.window_fill = color(tokens.surface);
        visuals.faint_bg_color = color(tokens.surface_secondary);
        visuals.extreme_bg_color = color(tokens.surface);
        visuals.selection.bg_fill = color(tokens.selection);
        visuals.selection.stroke.color = color(tokens.selection_text);
        visuals.widgets.inactive.bg_fill = color(tokens.surface_secondary);
        visuals.widgets.inactive.fg_stroke.color = color(tokens.primary_text);
        visuals.widgets.hovered.bg_fill = color(tokens.hover);
        visuals.widgets.hovered.fg_stroke.color = color(tokens.primary_text);
        visuals.widgets.active.bg_fill = color(tokens.selection);
        visuals.widgets.active.fg_stroke.color = color(tokens.selection_text);
        visuals.widgets.noninteractive.fg_stroke.color = color(tokens.primary_text);
        ctx.set_visuals(visuals);
    }

    fn render_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("EncryptDrive").size(18.0));
            ui.separator();
            ui.label(
                RichText::new("Synthetic vault open · Offline demo")
                    .color(color(SemanticTokens::for_mode(self.theme()).secondary_text)),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let theme_label = match self.model.appearance() {
                    Appearance::Light => "Light",
                    Appearance::Dark => "Dark",
                    Appearance::System => "Light (System fallback)",
                };
                if ui.button(format!("Appearance: {theme_label}")).clicked() {
                    let next = match self.model.appearance() {
                        Appearance::Light => Appearance::Dark,
                        Appearance::Dark | Appearance::System => Appearance::Light,
                    };
                    self.model.set_appearance(next);
                }
                if ui.button("Settings").clicked() {
                    self.settings_open = true;
                }
                ui.label("Demo account");
            });
        });
    }

    fn render_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(12.0);
        ui.label(RichText::new("WORKSPACE").small().strong());
        ui.add_space(8.0);
        for (page, label) in [(Page::Files, "Files"), (Page::Trash, "Trash")] {
            let selected = self.model.page() == page;
            if ui.selectable_label(selected, label).clicked() {
                self.model.set_page(page);
                self.table_focused = false;
            }
        }
        ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
            ui.separator();
            ui.label(RichText::new(format!("{} synthetic entries", self.entries_count)).small());
            ui.label(RichText::new("No files are read or written").small());
        });
    }

    fn render_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            let back = ui.add_enabled(
                self.model.page() == Page::Files,
                egui::Button::new("← Back"),
            );
            if back.clicked() {
                self.model.navigate_back();
            }
            let up = ui.add_enabled(
                self.model.page() == Page::Files && self.model.current_folder() != EntryId(0),
                egui::Button::new("↑ Up"),
            );
            if up.clicked() {
                self.model.navigate_parent();
            }
            for (id, name) in self.model.breadcrumb() {
                if ui.small_button(name).clicked() {
                    self.model.navigate_to(id);
                }
                ui.label("›");
            }
            let search = ui.add_sized(
                [190.0, 28.0],
                egui::TextEdit::singleline(&mut self.search)
                    .hint_text("Search names")
                    .id_salt("file_search"),
            );
            if self.search_focus_pending {
                search.request_focus();
                self.search_focus_pending = false;
            }
            if search.changed() {
                self.model.set_search(self.search.clone());
            }
            if search.has_focus() {
                self.table_focused = false;
            }

            let busy = self.model.operation().is_running();
            let is_files = self.model.page() == Page::Files && self.search.is_empty();
            if ui
                .add_enabled(!busy && is_files, egui::Button::new("New folder"))
                .clicked()
            {
                self.open_dialog(DialogKind::CreateFolder, None);
            }
            if ui
                .add_enabled(!busy && is_files, egui::Button::new("Import"))
                .clicked()
            {
                self.open_dialog(DialogKind::Import, None);
            }
            if ui
                .add_enabled(
                    !busy && !self.model.selected_ids().is_empty(),
                    egui::Button::new("Export"),
                )
                .clicked()
            {
                let selected = self.model.selected_ids();
                self.open_dialog(DialogKind::ExportWarning, selected.first().copied());
            }
            if ui
                .add_enabled(
                    !busy && self.model.selected_ids().len() == 1,
                    egui::Button::new("Rename"),
                )
                .clicked()
            {
                let selected = self.model.selected_ids();
                self.open_dialog(DialogKind::Rename, selected.first().copied());
            }
            if ui
                .add_enabled(
                    !busy && !self.model.selected_ids().is_empty(),
                    egui::Button::new("Delete"),
                )
                .clicked()
            {
                self.open_dialog(
                    if self.model.page() == Page::Files {
                        DialogKind::DeleteToTrash
                    } else {
                        DialogKind::PermanentDelete
                    },
                    None,
                );
            }
            if ui.button("More ▾").clicked() {
                self.context_target = self.model.selected_ids().first().copied();
                self.context_menu_open = self.context_target.is_some();
            }
        });
        ui.separator();
    }

    fn render_table(&mut self, ui: &mut egui::Ui) {
        if self.model.is_loading() {
            ui.label("Loading synthetic entries…");
            return;
        }
        if self.model.is_error() {
            ui.horizontal(|ui| {
                ui.colored_label(
                    color(SemanticTokens::for_mode(self.theme()).error),
                    self.model.notice().unwrap_or("List unavailable"),
                );
                if ui.button("Retry").clicked() {
                    self.model.finish_loading();
                    self.model.dismiss_notice();
                }
            });
        }
        let entries: Vec<Entry> = self.model.visible_entries().into_iter().cloned().collect();
        if entries.is_empty() {
            ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                ui.add_space(40.0);
                ui.heading(if self.model.page() == Page::Trash {
                    "Trash is empty"
                } else {
                    "This folder is empty"
                });
                ui.label("Synthetic demo data only. No file contents can be opened.");
            });
            return;
        }

        let selected: BTreeSet<EntryId> = self.model.selected_ids().into_iter().collect();
        let input_modifiers = ui.input(|input| input.modifiers);
        let mut table = TableBuilder::new(ui)
            .id_salt("synthetic_file_table")
            .striped(false)
            .resizable(true)
            .sense(Sense::click())
            .column(Column::remainder().at_least(260.0))
            .column(Column::initial(125.0).at_least(100.0).clip(true))
            .column(Column::initial(180.0).at_least(145.0).clip(true));
        table = table.min_scrolled_height(120.0);
        table
            .header(34.0, |mut header| {
                header.col(|ui| {
                    if ui
                        .button(sort_label(
                            "Name",
                            self.model.sort_column(),
                            self.model.sort_ascending(),
                            SortColumn::Name,
                        ))
                        .clicked()
                    {
                        self.model.toggle_sort(SortColumn::Name);
                    }
                });
                header.col(|ui| {
                    if ui
                        .button(sort_label(
                            "Size",
                            self.model.sort_column(),
                            self.model.sort_ascending(),
                            SortColumn::Size,
                        ))
                        .clicked()
                    {
                        self.model.toggle_sort(SortColumn::Size);
                    }
                });
                header.col(|ui| {
                    if ui
                        .button(sort_label(
                            "Modified",
                            self.model.sort_column(),
                            self.model.sort_ascending(),
                            SortColumn::Modified,
                        ))
                        .clicked()
                    {
                        self.model.toggle_sort(SortColumn::Modified);
                    }
                });
            })
            .body(|body| {
                body.rows(ROW_HEIGHT, entries.len(), |mut row| {
                    let entry = &entries[row.index()];
                    let is_selected = selected.contains(&entry.id);
                    row.set_selected(is_selected);
                    let mut label_response = None;
                    row.col(|ui| {
                        label_response = Some(
                            ui.push_id(entry.id.0, |ui| {
                                let kind = if entry.kind == EntryKind::Folder {
                                    "Folder"
                                } else {
                                    "File"
                                };
                                ui.selectable_label(is_selected, format!("{kind}  {}", entry.name))
                            })
                            .inner,
                        );
                    });
                    let name_response =
                        label_response.expect("name cell creates a selectable label");
                    row.col(|ui| {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(if entry.kind == EntryKind::Folder {
                                "—".to_owned()
                            } else {
                                format_size(entry.size_bytes)
                            });
                        });
                    });
                    row.col(|ui| {
                        ui.label(&entry.modified_at);
                    });
                    let whole_row = row.response();

                    if name_response.clicked() || whole_row.clicked() {
                        self.table_focused = true;
                        if input_modifiers.shift {
                            self.model.select_range(entry.id);
                        } else if input_modifiers.ctrl || input_modifiers.command {
                            self.model.toggle_selection(entry.id);
                        } else {
                            self.model.select_only(entry.id);
                        }
                    }
                    if name_response.double_clicked() {
                        self.model.activate(entry.id);
                        if self.model.dialog().is_some() {
                            self.dialog_focus_pending = true;
                        }
                    }
                    name_response.context_menu(|ui| {
                        if !self.model.selected_ids().contains(&entry.id) {
                            self.model.select_only(entry.id);
                        }
                        if ui
                            .button(if entry.kind == EntryKind::Folder {
                                "Open folder"
                            } else {
                                "Simulated Export…"
                            })
                            .clicked()
                        {
                            self.model.activate(entry.id);
                            if self.model.dialog().is_some() {
                                self.dialog_focus_pending = true;
                            }
                        }
                        if ui.button("Rename…").clicked() {
                            self.open_dialog(DialogKind::Rename, Some(entry.id));
                        }
                        if ui
                            .button(if self.model.page() == Page::Files {
                                "Move to Trash…"
                            } else {
                                "Restore"
                            })
                            .clicked()
                        {
                            if self.model.page() == Page::Files {
                                self.model.select_only(entry.id);
                                self.open_dialog(DialogKind::DeleteToTrash, None);
                            } else {
                                self.model.restore(&[entry.id]);
                            }
                        }
                    });
                });
            });

        ui.allocate_space(Vec2::ZERO);
        let selected_count = self.model.selected_ids().len();
        ui.horizontal(|ui| {
            ui.label(format!("{} items", entries.len()));
            if selected_count > 0 {
                ui.label(format!("{selected_count} selected"));
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.small_button("Simulate recovery notice").clicked() {
                    self.model.set_notice(
                        "A synthetic metadata recovery notice is being previewed. No recovery ran.",
                    );
                }
                ui.label("Double-click a file to see the explicit Export warning.");
            });
        });
    }

    fn render_operation(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        match self.model.operation().clone() {
            OperationStatus::Running {
                label,
                completed,
                total,
                ..
            } => {
                ui.horizontal(|ui| {
                    ui.label(label);
                    ui.add(
                        egui::ProgressBar::new(completed as f32 / total as f32)
                            .text(format!("{completed}/{total}")),
                    );
                    if ui.button("Cancel").clicked() {
                        self.model.cancel_operation();
                        self.model
                            .set_notice("Synthetic operation cancelled; no files were changed.");
                    }
                    if ui.button("Simulate failure").clicked() {
                        self.model.fail_operation_now();
                    }
                });
                ctx.request_repaint_after(Duration::from_millis(38));
                if self.model.advance_operation() {
                    if matches!(self.model.operation(), OperationStatus::Succeeded { .. }) {
                        self.model.finish_operation();
                        self.model.set_notice("Synthetic operation completed.");
                    } else if let OperationStatus::Failed { message, .. } =
                        self.model.operation().clone()
                    {
                        self.model.set_error(message);
                    }
                }
            }
            OperationStatus::Succeeded { ref label } => {
                ui.label(format!("{label} complete."));
            }
            OperationStatus::Failed {
                ref label,
                ref message,
            } => {
                ui.colored_label(
                    color(SemanticTokens::for_mode(self.theme()).error),
                    format!("{label}: {message}"),
                );
            }
            OperationStatus::Cancelled { ref label } => {
                ui.label(format!("{label} cancelled."));
            }
            OperationStatus::Idle => {}
        }
    }

    fn render_dialog(&mut self, ctx: &egui::Context) {
        let Some(dialog) = self.model.dialog().cloned() else {
            return;
        };
        let title = match dialog.kind {
            DialogKind::CreateFolder => "Create folder",
            DialogKind::Rename => "Rename item",
            DialogKind::DeleteToTrash => "Move to Trash?",
            DialogKind::PermanentDelete => "Delete permanently?",
            DialogKind::Import => "Simulated Import",
            DialogKind::ExportWarning => "Export plaintext copy?",
            DialogKind::RecoveryNotice => "Recovery notice",
        };
        egui::Window::new(title)
            .id(egui::Id::unique("operation_dialog"))
            .collapsible(false)
            .resizable(false)
            .fixed_size([470.0, 230.0])
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                match dialog.kind {
                    DialogKind::CreateFolder | DialogKind::Rename | DialogKind::Import => {
                        let label = match dialog.kind {
                            DialogKind::CreateFolder => "Folder name",
                            DialogKind::Rename => "New name",
                            _ => "Synthetic file name",
                        };
                        ui.label(label);
                        let response = ui.text_edit_singleline(&mut self.form_name);
                        response.widget_info(|| {
                            egui::WidgetInfo::labeled(egui::Role::TextInput, true, label)
                        });
                        if let Some(error) = dialog.error.as_deref() {
                            ui.colored_label(
                                color(SemanticTokens::for_mode(self.theme()).error),
                                error,
                            );
                        }
                        if dialog.kind == DialogKind::Import {
                            ui.label("No picker is opened and no local file is read.");
                        }
                    }
                    DialogKind::DeleteToTrash => {
                        ui.label(format!(
                            "Move {} selected synthetic item(s) to Trash?",
                            self.model.selected_ids().len()
                        ));
                        ui.label("This changes only in-memory demo state.");
                    }
                    DialogKind::PermanentDelete => {
                        ui.label(format!(
                            "Permanently remove {} synthetic item(s)?",
                            self.model.selected_ids().len()
                        ));
                        ui.label("No secure-erasure claim applies; this is a simulation.");
                    }
                    DialogKind::ExportWarning => {
                        ui.label("The exported copy would be plaintext outside EncryptDrive.");
                        ui.label("This prototype does not decrypt, write, or launch files.");
                    }
                    DialogKind::RecoveryNotice => {
                        ui.label(
                            "Synthetic recovery notice only. No vault or metadata recovery ran.",
                        );
                    }
                }
                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    let cancel = ui.button("Cancel");
                    if self.dialog_focus_pending {
                        cancel.request_focus();
                        self.dialog_focus_pending = false;
                    }
                    if cancel.clicked() {
                        self.model.close_dialog();
                    }
                    let action = match dialog.kind {
                        DialogKind::CreateFolder => Some("Create"),
                        DialogKind::Rename => Some("Rename"),
                        DialogKind::DeleteToTrash => Some("Move to Trash"),
                        DialogKind::PermanentDelete => Some("Delete permanently"),
                        DialogKind::Import => Some("Start simulated import"),
                        DialogKind::ExportWarning => Some("Continue to simulated Export"),
                        DialogKind::RecoveryNotice => Some("Acknowledge"),
                    };
                    if ui.button(action.unwrap_or("Continue")).clicked() {
                        self.confirm_dialog(dialog.clone());
                    }
                });
            });
    }

    fn confirm_dialog(&mut self, dialog: encryptdrive_spike_common::Dialog) {
        match dialog.kind {
            DialogKind::CreateFolder => match self.model.create_folder(&self.form_name) {
                Ok(_) => self.model.close_dialog(),
                Err(error) => self.set_dialog_error(error),
            },
            DialogKind::Rename => match dialog
                .target
                .map(|id| self.model.rename(id, &self.form_name))
            {
                Some(Ok(())) => self.model.close_dialog(),
                Some(Err(error)) => self.set_dialog_error(error),
                None => self.set_dialog_error("Select one item to rename.".to_owned()),
            },
            DialogKind::DeleteToTrash => {
                let ids = self.model.selected_ids();
                self.start_operation("Moving items to Trash", SyntheticAction::MoveToTrash(ids));
                self.model.close_dialog();
            }
            DialogKind::PermanentDelete => {
                let ids = self.model.selected_ids();
                self.start_operation(
                    "Deleting synthetic items",
                    SyntheticAction::PermanentlyDelete(ids),
                );
                self.model.close_dialog();
            }
            DialogKind::Import => {
                let action = SyntheticAction::Import {
                    parent: self.model.current_folder(),
                    name: self.form_name.clone(),
                };
                self.start_operation("Importing synthetic item", action);
                self.model.close_dialog();
            }
            DialogKind::ExportWarning => {
                self.model
                    .start_operation("Preparing simulated Export", 36, None);
                self.model.close_dialog();
            }
            DialogKind::RecoveryNotice => self.model.close_dialog(),
        }
    }

    fn set_dialog_error(&mut self, error: String) {
        if let Some(dialog) = self.model.dialog_mut() {
            dialog.error = Some(error);
        }
        self.dialog_focus_pending = true;
    }

    fn start_operation(&mut self, label: &str, action: SyntheticAction) {
        self.model.start_operation(label, 36, None);
        self.model.queue_after_success(action);
    }

    fn render_context_menu(&mut self, ctx: &egui::Context) {
        if !self.context_menu_open {
            return;
        }
        let Some(target) = self.context_target else {
            self.context_menu_open = false;
            return;
        };
        let name = self
            .model
            .entry(target)
            .map(|entry| entry.name.clone())
            .unwrap_or_default();
        egui::Window::new("Item actions")
            .id(egui::Id::unique("keyboard_context_menu"))
            .collapsible(false)
            .resizable(false)
            .default_pos(
                ctx.input(|input| input.pointer.latest_pos())
                    .unwrap_or(egui::pos2(300.0, 180.0)),
            )
            .show(ctx, |ui| {
                ui.label(RichText::new(name).strong());
                if ui.button("Open / simulated Export").clicked() {
                    self.model.activate(target);
                    self.dialog_focus_pending = true;
                    self.context_menu_open = false;
                }
                if ui.button("Rename…").clicked() {
                    self.open_dialog(DialogKind::Rename, Some(target));
                }
                if ui
                    .button(if self.model.page() == Page::Files {
                        "Move to Trash…"
                    } else {
                        "Restore"
                    })
                    .clicked()
                {
                    if self.model.page() == Page::Files {
                        self.model.select_only(target);
                        self.open_dialog(DialogKind::DeleteToTrash, None);
                    } else {
                        self.model.restore(&[target]);
                    }
                    self.context_menu_open = false;
                }
                if ui.button("Close menu").clicked() {
                    self.context_menu_open = false;
                }
            });
    }

    fn render_settings(&mut self, ctx: &egui::Context) {
        if !self.settings_open {
            return;
        }
        let mut open = self.settings_open;
        egui::Window::new("Settings · prototype controls")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(380.0)
            .show(ctx, |ui| {
                ui.label("These controls change in-memory synthetic data only.");
                ui.horizontal(|ui| {
                    ui.label("List size");
                    for size in [100, 1_000, 10_000] {
                        if ui
                            .selectable_label(self.entries_count == size, size.to_string())
                            .clicked()
                        {
                            self.set_size(size);
                        }
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button("Simulate loading").clicked() {
                        self.model.simulate_loading();
                        self.loading_frames = 12;
                    }
                    if ui.button("Simulate error").clicked() {
                        self.model.simulate_error();
                    }
                    if ui.button("Retry list").clicked() {
                        self.model.dismiss_notice();
                        self.model.set_loading_state(false);
                    }
                });
                ui.horizontal(|ui| {
                    for appearance in [Appearance::Light, Appearance::Dark] {
                        let label = match appearance {
                            Appearance::Light => "Light",
                            Appearance::Dark => "Dark",
                            Appearance::System => unreachable!(),
                        };
                        if ui
                            .selectable_label(self.model.appearance() == appearance, label)
                            .clicked()
                        {
                            self.model.set_appearance(appearance);
                        }
                    }
                });
                ui.label("No preference is written to disk.");
            });
        self.settings_open = open;
    }
}

impl eframe::App for ExplorerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.apply_theme(&ctx);
        self.handle_shortcuts(&ctx);

        if self.loading_frames > 0 {
            self.loading_frames -= 1;
            ctx.request_repaint_after(Duration::from_millis(30));
            if self.loading_frames == 0 {
                self.model.finish_loading();
            }
        }

        egui::Panel::top("compact_header").show(ui, |ui| self.render_header(ui));
        egui::Panel::left("workspace_navigation")
            .resizable(true)
            .default_size(184.0)
            .min_size(150.0)
            .show(ui, |ui| self.render_sidebar(ui));
        egui::CentralPanel::default().show(ui, |ui| {
            ui.vertical(|ui| {
                self.render_toolbar(ui);
                self.render_table(ui);
                ui.separator();
                if let Some(notice) = self.model.notice().map(str::to_owned) {
                    let tokens = SemanticTokens::for_mode(self.theme());
                    let fill = if self.model.is_error() {
                        tokens.error_surface
                    } else {
                        tokens.success_surface
                    };
                    let foreground = if self.model.is_error() {
                        tokens.error
                    } else {
                        tokens.success
                    };
                    egui::Frame::new()
                        .fill(color(fill))
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.colored_label(color(foreground), notice);
                                if ui.small_button("Dismiss").clicked() {
                                    self.model.dismiss_notice();
                                }
                            });
                        });
                }
                self.render_operation(ui, &ctx);
            });
        });

        self.render_dialog(&ctx);
        self.render_context_menu(&ctx);
        self.render_settings(&ctx);
    }
}

fn color(hex: u32) -> Color32 {
    Color32::from_rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

fn sort_label(label: &str, current: SortColumn, ascending: bool, column: SortColumn) -> String {
    if current != column {
        label.to_owned()
    } else if ascending {
        format!("{label} ↑")
    } else {
        format!("{label} ↓")
    }
}

fn format_size(size: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut value = size as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{size} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::epaint::text::{FontDefinitions, Fonts, MissingGlyphPolicy, TextOptions};
    use std::sync::Arc;

    #[test]
    fn unicode_fixture_shapes_without_missing_glyphs_and_keeps_its_name() {
        let text = make_fixture(100)
            .entries()
            .iter()
            .find(|entry| entry.name.contains("東京"))
            .expect("the deterministic fixture includes a Japanese filename")
            .name
            .clone();
        let provider: Arc<dyn egui::FontProvider> =
            Arc::new(egui_system_font_provider::SystemFontProvider::new());
        let mut fonts = Fonts::new(TextOptions::default(), FontDefinitions::default())
            .with_font_providers(vec![provider])
            .with_missing_glyph_policy(MissingGlyphPolicy::Panic);
        let galley = fonts.with_pixels_per_point(1.0).layout_no_wrap(
            text.clone(),
            egui::FontId::proportional(14.0),
            Color32::WHITE,
        );

        assert_eq!(galley.rows[0].row.text(), text);
    }
}
