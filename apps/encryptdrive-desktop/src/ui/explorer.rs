use eframe::egui::{
    self, Align, Color32, CornerRadius, Layout, RichText, Sense, Stroke, WidgetInfo,
};
use egui_extras::{Column, TableBuilder};

use encryptdrive_desktop::{
    Destination,
    preview::{EntryId, EntryKind, Modifier, OperationState, SortDirection, SortKey},
};

use super::app::{DesktopApp, DialogKind, fmt_size};
use crate::design::{
    components::{
        ButtonKind, breadcrumb_button, button_activated, command_button, menu_item,
        paint_focus_ring_rect, paint_selection_pill, search_field, secondary_button,
        subtle_icon_button, table_sort_header_button,
    },
    geometry::Geometry,
    icons::Icon,
    theme::Palette,
    typography::TypeScale,
};

impl DesktopApp {
    pub(super) fn render_explorer(&mut self, ui: &mut egui::Ui, palette: Palette) {
        let busy = self
            .preview
            .as_ref()
            .is_some_and(|preview| matches!(preview.operation, OperationState::Running { .. }));
        let actions_enabled = !busy
            && self.preview.as_ref().is_some_and(|preview| {
                preview.scenario != encryptdrive_desktop::preview::PreviewScenario::Disabled
            });
        let (destination, folder, selected_count) =
            self.preview
                .as_ref()
                .map_or((self.shell.destination, None, 0), |preview| {
                    (
                        preview.explorer.destination(),
                        preview.explorer.current_folder(),
                        preview.explorer.selected_ids().len(),
                    )
                });
        self.render_page_heading(ui, palette, destination, folder);
        ui.add_space(Geometry::SPACE_2);
        let density = command_density(ui.available_width());
        let search_has_focus = self.render_command_bar(
            ui,
            palette,
            destination,
            selected_count,
            actions_enabled,
            busy,
            density,
        );
        ui.add_space(Geometry::SPACE_3);

        if self.preview.is_some() {
            let ids = {
                let preview = self.preview.as_mut().expect("preview is present");
                std::sync::Arc::clone(&preview.explorer.visible_ids(&preview.model))
            };
            self.render_file_table(ui, palette, &ids);
        } else {
            self.render_default_trash(ui, palette);
        }
        self.search_has_focus = search_has_focus;
    }

    /// Back / Up buttons followed by an Explorer-style breadcrumb (Subtitle size).
    fn render_page_heading(
        &mut self,
        ui: &mut egui::Ui,
        palette: Palette,
        destination: Destination,
        folder: Option<EntryId>,
    ) {
        let mut breadcrumbs = Vec::new();
        if destination == Destination::Files {
            let mut current = folder;
            while let Some(id) = current {
                if let Some(entry) = self
                    .preview
                    .as_ref()
                    .and_then(|state| state.model.entry(id))
                {
                    breadcrumbs.push((id, entry.name.clone()));
                    current = entry.parent_id;
                } else {
                    break;
                }
            }
            breadcrumbs.reverse();
        }
        let width = ui.available_width();
        ui.allocate_ui_with_layout(
            egui::vec2(width, Geometry::CONTROL_HEIGHT + Geometry::BASE),
            Layout::left_to_right(Align::Center),
            |ui| {
                ui.set_min_size(egui::vec2(width, Geometry::CONTROL_HEIGHT + Geometry::BASE));
                ui.spacing_mut().item_spacing.x = Geometry::BASE;
                let Some(preview) = &mut self.preview else {
                    return;
                };
                let in_folder = folder.is_some();
                let back = ui
                    .add_enabled_ui(in_folder, |ui| {
                        subtle_icon_button(ui, Icon::Back, "Back", palette)
                    })
                    .inner;
                if button_activated(ui, &back) {
                    preview.explorer.navigate_back();
                    preview.search_input.clear();
                }
                let up = ui
                    .add_enabled_ui(in_folder, |ui| {
                        subtle_icon_button(ui, Icon::Up, "Up one level", palette)
                    })
                    .inner;
                if button_activated(ui, &up) {
                    let parent = folder
                        .and_then(|id| preview.model.entry(id))
                        .and_then(|entry| entry.parent_id);
                    preview.explorer.navigate_to(parent);
                    preview.search_input.clear();
                }
                ui.add_space(Geometry::SPACE_2);
                if destination == Destination::Trash {
                    breadcrumb_button(ui, "Trash", true, palette);
                    return;
                }
                let root = breadcrumb_button(ui, "Files", breadcrumbs.is_empty(), palette);
                if button_activated(ui, &root) && !breadcrumbs.is_empty() {
                    preview.explorer.navigate_to(None);
                    preview.search_input.clear();
                }
                let last = breadcrumbs.len().saturating_sub(1);
                for (index, (id, name)) in breadcrumbs.iter().enumerate() {
                    ui.label(Icon::ChevronRight.rich_text(palette.muted, TypeScale::HELPER));
                    let crumb = breadcrumb_button(ui, name, index == last, palette);
                    if button_activated(ui, &crumb) && index != last {
                        preview.explorer.navigate_to(Some(*id));
                        preview.search_input.clear();
                    }
                }
            },
        );
    }

    /// Stable command bar: every command is always present and only enables/disables, so
    /// selecting rows never moves the table. Search sits on the right.
    #[allow(clippy::too_many_arguments)]
    fn render_command_bar(
        &mut self,
        ui: &mut egui::Ui,
        palette: Palette,
        destination: Destination,
        selected_count: usize,
        actions_enabled: bool,
        busy: bool,
        density: CommandDensity,
    ) -> bool {
        let mut search_has_focus = false;
        let width = ui.available_width();
        // Secondary commands drop their labels first; the primary keeps its label longest.
        let compact = density != CommandDensity::Labels;
        let primary_compact = density == CommandDensity::IconsOnly;
        ui.allocate_ui_with_layout(
            egui::vec2(width, Geometry::CONTROL_HEIGHT),
            Layout::left_to_right(Align::Center),
            |ui| {
                ui.set_min_size(egui::vec2(width, Geometry::CONTROL_HEIGHT));
                ui.spacing_mut().item_spacing.x = Geometry::BASE;
                let Some(preview) = &mut self.preview else {
                    return;
                };
                let any_selected = selected_count > 0 && actions_enabled;
                if destination == Destination::Files {
                    let new_folder = ui
                        .add_enabled_ui(actions_enabled, |ui| {
                            command_button(
                                ui,
                                Icon::NewFolder,
                                "New folder",
                                ButtonKind::Primary,
                                primary_compact,
                                palette,
                            )
                        })
                        .inner;
                    if button_activated(ui, &new_folder) {
                        self.dialog_invoker = Some(new_folder.id);
                        preview.input_name.clear();
                        preview.input_error = None;
                        preview.dialog = Some(DialogKind::CreateFolder);
                    }
                    ui.add_space(Geometry::BASE);
                    let import = ui
                        .add_enabled_ui(actions_enabled, |ui| {
                            command_button(
                                ui,
                                Icon::Import,
                                "Import",
                                ButtonKind::Subtle,
                                compact,
                                palette,
                            )
                        })
                        .inner;
                    if button_activated(ui, &import) {
                        self.dialog_invoker = Some(import.id);
                        preview.dialog = Some(DialogKind::Import);
                    }
                    let export = ui
                        .add_enabled_ui(any_selected, |ui| {
                            command_button(
                                ui,
                                Icon::Export,
                                "Export",
                                ButtonKind::Subtle,
                                compact,
                                palette,
                            )
                        })
                        .inner;
                    if button_activated(ui, &export) {
                        self.dialog_invoker = Some(export.id);
                        preview.dialog = Some(DialogKind::Export);
                    }
                    command_separator(ui, palette);
                    let rename = ui
                        .add_enabled_ui(selected_count == 1 && actions_enabled, |ui| {
                            command_button(
                                ui,
                                Icon::Rename,
                                "Rename",
                                ButtonKind::Subtle,
                                compact,
                                palette,
                            )
                        })
                        .inner;
                    if button_activated(ui, &rename)
                        && let Some(id) = rename_target(&preview.explorer.selected_ids())
                    {
                        self.dialog_invoker = Some(rename.id);
                        preview.input_name = preview
                            .model
                            .entry(id)
                            .map_or_else(String::new, |entry| entry.name.clone());
                        preview.input_error = None;
                        preview.dialog = Some(DialogKind::Rename);
                    }
                    let move_to_trash = ui
                        .add_enabled_ui(any_selected, |ui| {
                            command_button(
                                ui,
                                Icon::Trash,
                                "Move to Trash",
                                ButtonKind::Subtle,
                                compact,
                                palette,
                            )
                        })
                        .inner;
                    if button_activated(ui, &move_to_trash) {
                        self.dialog_invoker = Some(move_to_trash.id);
                        preview.dialog = Some(DialogKind::MoveToTrash);
                    }
                } else {
                    let restore = ui
                        .add_enabled_ui(any_selected, |ui| {
                            command_button(
                                ui,
                                Icon::Restore,
                                "Restore",
                                ButtonKind::Subtle,
                                // Trash has only two commands, so they keep labels longer.
                                primary_compact,
                                palette,
                            )
                        })
                        .inner;
                    if button_activated(ui, &restore) {
                        let ids = preview.explorer.selected_ids();
                        preview.model.restore(&ids);
                        preview.explorer.clear_selection();
                        preview.notice = Some(restored_message(ids.len()));
                    }
                    let delete = ui
                        .add_enabled_ui(any_selected, |ui| {
                            command_button(
                                ui,
                                Icon::Trash,
                                "Delete permanently",
                                ButtonKind::SubtleDanger,
                                // Trash has only two commands, so they keep labels longer.
                                primary_compact,
                                palette,
                            )
                        })
                        .inner;
                    if button_activated(ui, &delete) {
                        self.dialog_invoker = Some(delete.id);
                        preview.dialog = Some(DialogKind::PermanentDelete);
                    }
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let search_width = search_width(width, density);
                    let hint = if destination == Destination::Trash {
                        "Search Trash"
                    } else {
                        "Search Files"
                    };
                    let response = ui
                        .add_enabled_ui(!busy, |ui| {
                            search_field(ui, &mut preview.search_input, hint, search_width, palette)
                        })
                        .inner;
                    if self.search_focus_requested {
                        response.request_focus();
                        self.search_focus_requested = false;
                    }
                    search_has_focus = response.has_focus();
                    preview.explorer.set_query(preview.search_input.clone());
                });
            },
        );
        search_has_focus
    }

    fn render_file_table(&mut self, ui: &mut egui::Ui, palette: Palette, ids: &[EntryId]) {
        let requested_row_focus = consume_row_focus_request(&mut self.requested_row_focus);
        let Some(preview) = &mut self.preview else {
            return;
        };
        let is_trash = preview.explorer.destination() == Destination::Trash;
        if preview.scenario == encryptdrive_desktop::preview::PreviewScenario::Loading {
            ui.allocate_ui_with_layout(
                ui.available_size(),
                Layout::top_down(Align::Center),
                |ui| {
                    ui.set_min_size(ui.available_size());
                    ui.add_space((ui.available_height() / 2.0 - Geometry::SPACE_8).max(0.0));
                    ui.spinner();
                    ui.add_space(Geometry::SPACE_2);
                    ui.label(
                        RichText::new("Loading items…")
                            .size(TypeScale::BODY)
                            .color(palette.muted),
                    );
                },
            );
            return;
        }
        if ids.is_empty() {
            let searching = !preview.search_input.trim().is_empty();
            let (title, description) = empty_list_copy(is_trash, searching);
            let icon = if searching {
                Icon::Search
            } else if is_trash {
                Icon::Trash
            } else {
                Icon::Folder
            };
            crate::design::components::empty_state(ui, icon, title, description, palette);
            return;
        }
        let current_sort = preview.explorer.sort_key();
        let current_direction = preview.explorer.sort_direction();
        let mut sort_to = None;
        let mut row_action: Option<(EntryId, bool, bool, bool, egui::Id)> = None;
        let mut context_action: Option<(EntryId, ContextAction)> = None;
        let selected_ids = preview
            .explorer
            .selected_ids()
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>();
        let modifiers = ui.input(|input| {
            (
                input.modifiers.ctrl || input.modifiers.command,
                input.modifiers.shift,
            )
        });
        let shift_f10 =
            ui.input(|input| input.modifiers.shift && input.key_pressed(egui::Key::F10));
        let context = ui.ctx().clone();
        let table_enabled = !matches!(preview.operation, OperationState::Running { .. })
            && preview.scenario != encryptdrive_desktop::preview::PreviewScenario::Disabled;
        let table_sense = if table_enabled {
            Sense::CLICK
        } else {
            Sense::hover()
        };
        let table_viewport = ui.available_rect_before_wrap();
        // Row decorations share the table's layer but are clipped to the body area so they
        // never paint over the sticky header.
        let mut body_clip = table_viewport;
        body_clip.min.y += Geometry::TABLE_HEADER_HEIGHT;
        let row_painter = ui
            .painter()
            .with_clip_rect(body_clip.intersect(ui.clip_rect()));
        let rows_per_page = ((table_viewport.height() - Geometry::TABLE_HEADER_HEIGHT)
            / Geometry::TABLE_ROW_HEIGHT)
            .floor()
            .max(1.0) as usize;
        self.table_page_rows = rows_per_page;
        let table_focus_response = ui.interact(
            table_viewport,
            ui.make_persistent_id("encryptdrive-explorer-list-focus"),
            if table_enabled {
                Sense::focusable_noninteractive()
            } else {
                Sense::hover()
            },
        );
        table_focus_response.widget_info(|| {
            WidgetInfo::labeled(
                egui::Role::List,
                table_enabled,
                if is_trash { "Trash items" } else { "Files" },
            )
        });
        if table_focus_response.has_focus() {
            context.memory_mut(|memory| {
                memory.set_focus_lock_filter(table_focus_response.id, table_focus_event_filter());
            });
        }
        let table_has_focus = table_focus_response.has_focus();
        let pointer = ui.input(|input| input.pointer.hover_pos());
        let header_line_y = table_viewport.top() + Geometry::TABLE_HEADER_HEIGHT - 0.5;
        ui.painter().hline(
            table_viewport.x_range(),
            header_line_y,
            Stroke::new(1.0, palette.separator),
        );

        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
            let mut table = TableBuilder::new(ui)
                .id_salt("encryptdrive-synthetic-explorer-table")
                .striped(false)
                .resizable(false)
                .sense(Sense::hover())
                .column(Column::remainder().at_least(200.0).clip(true))
                .column(Column::exact(150.0).clip(true))
                .column(Column::exact(96.0).clip(true))
                .column(Column::exact(148.0).clip(true));
            if let Some(focused) = requested_row_focus
                && let Some(index) = ids.iter().position(|id| *id == focused)
            {
                table = table.scroll_to_row(index, None);
            }
            table
                .header(Geometry::TABLE_HEADER_HEIGHT, |mut header| {
                    for (label, key, align_right) in [
                        ("Name", SortKey::Name, false),
                        ("Type", SortKey::Type, false),
                        ("Size", SortKey::Size, true),
                        ("Modified", SortKey::ModifiedSample, false),
                    ] {
                        let indicator = sort_indicator(current_sort, current_direction, key);
                        header.col(|ui| {
                            let inset = if key == SortKey::Name {
                                Geometry::SPACE_3
                            } else {
                                Geometry::SPACE_2
                            };
                            let response = table_sort_header_button(
                                ui,
                                label,
                                indicator,
                                align_right,
                                inset,
                                palette,
                            );
                            if button_activated(ui, &response) {
                                sort_to = Some(key);
                            }
                        });
                    }
                })
                .body(|body| {
                    body.rows(Geometry::TABLE_ROW_HEIGHT, ids.len(), |mut row| {
                        let id = ids[row.index()];
                        let Some(entry) = preview.model.entry(id) else {
                            return;
                        };
                        // Reserve the backplate slot before the cells so text paints on top.
                        let backplate = row_painter.add(egui::Shape::Noop);
                        let entry_name = entry.name.clone();
                        let kind = entry.kind;
                        let type_text = type_label(&entry.file_type, kind);
                        let size_text = if kind == EntryKind::Folder {
                            String::new()
                        } else {
                            fmt_size(entry.size_bytes)
                        };
                        let modified = entry.modified_at.clone();
                        let selected = selected_ids.contains(&id);
                        let mut name_response = None;
                        let mut name_cell = None;
                        row.col(|ui| {
                            let cell = ui.max_rect();
                            name_cell = Some(cell);
                            let response = ui.interact(cell, row_widget_id(id), table_sense);
                            name_response = Some(response);
                            ui.horizontal_centered(|ui| {
                                // Leaves room for the selection pill; the "Name" header uses the
                                // same 12 px inset so labels and icons line up.
                                ui.add_space(Geometry::SPACE_3);
                                ui.spacing_mut().item_spacing.x = Geometry::SPACE_3;
                                let icon = if kind == EntryKind::Folder {
                                    Icon::Folder
                                } else {
                                    Icon::File
                                };
                                ui.add(
                                    egui::Label::new(icon.rich_text(
                                        if kind == EntryKind::Folder {
                                            palette.accent
                                        } else {
                                            palette.icon
                                        },
                                        Geometry::ICON_SIZE,
                                    ))
                                    .selectable(false),
                                );
                                table_text_label(
                                    ui,
                                    RichText::new(entry_name.clone())
                                        .size(TypeScale::BODY)
                                        .color(palette.text),
                                );
                            });
                        });
                        row.col(|ui| {
                            metadata_cell(ui, &type_text, false, palette);
                        });
                        row.col(|ui| {
                            metadata_cell(ui, &size_text, true, palette);
                        });
                        row.col(|ui| {
                            metadata_cell(ui, &modified, false, palette);
                        });
                        let row_response = row.response();
                        // The name cell holds the row's true vertical extent; span it across
                        // the full table width for the backplate.
                        let cell = name_cell.unwrap_or(row_response.rect);
                        let plate =
                            egui::Rect::from_x_y_ranges(table_viewport.x_range(), cell.y_range())
                                .shrink2(egui::vec2(Geometry::BASE / 2.0, 1.0));
                        let hovered = pointer.is_some_and(|position| {
                            plate.contains(position) && body_clip.contains(position)
                        });
                        let fill = match (selected, hovered) {
                            (true, true) => {
                                crate::design::theme::mix(palette.selected, palette.hover, 0.4)
                            }
                            (true, false) => palette.selected,
                            (false, true) => palette.hover,
                            (false, false) => Color32::TRANSPARENT,
                        };
                        if fill != Color32::TRANSPARENT {
                            row_painter.set(
                                backplate,
                                egui::Shape::rect_filled(
                                    plate,
                                    CornerRadius::same(Geometry::CONTROL_RADIUS),
                                    fill,
                                ),
                            );
                        }
                        if selected {
                            paint_selection_pill(&row_painter, plate, palette);
                        }
                        let name_response = name_response.expect("the name cell is present");
                        let combined = name_response.union(row_response);
                        combined.widget_info(|| {
                            WidgetInfo::selected(
                                egui::Role::ListItem,
                                true,
                                selected,
                                format!(
                                    "{entry_name}, {}",
                                    if kind == EntryKind::Folder {
                                        "folder"
                                    } else {
                                        "file"
                                    }
                                ),
                            )
                        });
                        if preview.explorer.focused_id() == Some(id) && table_has_focus {
                            row_painter.rect_stroke(
                                plate,
                                CornerRadius::same(Geometry::CONTROL_RADIUS),
                                Stroke::new(2.0, palette.focus),
                                egui::StrokeKind::Inside,
                            );
                        }
                        if !matches!(preview.operation, OperationState::Running { .. })
                            && preview.scenario
                                != encryptdrive_desktop::preview::PreviewScenario::Disabled
                        {
                            combined.context_menu(|ui| {
                                ui.set_min_width(220.0);
                                if is_trash {
                                    if menu_item(ui, "Restore", None).clicked() {
                                        context_action = Some((id, ContextAction::Restore));
                                        ui.close();
                                    }
                                    ui.separator();
                                    if menu_item(ui, "Delete permanently…", Some("Shift+Del"))
                                        .clicked()
                                    {
                                        self.dialog_invoker = Some(combined.id);
                                        context_action = Some((id, ContextAction::PermanentDelete));
                                        ui.close();
                                    }
                                } else {
                                    if kind == EntryKind::Folder
                                        && menu_item(ui, "Open", Some("Enter")).clicked()
                                    {
                                        context_action = Some((id, ContextAction::Open));
                                        ui.close();
                                    }
                                    if kind == EntryKind::File
                                        && menu_item(ui, "Export…", Some("Enter")).clicked()
                                    {
                                        self.dialog_invoker = Some(combined.id);
                                        context_action = Some((id, ContextAction::Export));
                                        ui.close();
                                    }
                                    ui.separator();
                                    if menu_item(ui, "Rename…", Some("F2")).clicked() {
                                        self.dialog_invoker = Some(combined.id);
                                        context_action = Some((id, ContextAction::Rename));
                                        ui.close();
                                    }
                                    if menu_item(ui, "Move to Trash…", Some("Del")).clicked() {
                                        self.dialog_invoker = Some(combined.id);
                                        context_action = Some((id, ContextAction::MoveToTrash));
                                        ui.close();
                                    }
                                }
                            });
                        }
                        if shift_f10 && preview.explorer.focused_id() == Some(id) {
                            egui::Popup::open_id(
                                &context,
                                egui::Popup::default_response_id(&combined),
                            );
                        }
                        if combined.clicked() {
                            row_action = Some((
                                id,
                                modifiers.0,
                                modifiers.1,
                                combined.double_clicked(),
                                combined.id,
                            ));
                        }
                    });
                });
        });
        if table_has_focus && preview.explorer.focused_id().is_none() {
            paint_focus_ring_rect(ui, table_viewport.shrink(2.0), palette);
        }
        self.table_has_focus = table_has_focus;
        if self.table_has_focus {
            self.focused_row_widget = Some(table_focus_response.id);
        }
        if let Some(sort) = sort_to
            && let Some(preview) = &mut self.preview
        {
            self.requested_row_focus = preview.explorer.focused_id();
            preview.explorer.set_sort(sort);
        }
        if let Some((id, control, shift, double_click, row_widget)) = row_action
            && let Some(preview) = &mut self.preview
        {
            table_focus_response.request_focus();
            self.table_has_focus = true;
            self.focused_row_widget = Some(table_focus_response.id);
            let visible = std::sync::Arc::clone(&preview.explorer.visible_ids(&preview.model));
            preview.explorer.select(
                id,
                &visible,
                if shift {
                    Modifier::Shift
                } else if control {
                    Modifier::Control
                } else {
                    Modifier::Plain
                },
            );
            if double_click && !control && !shift {
                if is_trash {
                    preview.notice = Some(
                        "Restore this item to Files before opening or exporting it.".to_owned(),
                    );
                } else {
                    match preview.model.entry(id).map(|item| item.kind) {
                        Some(EntryKind::Folder) => {
                            preview.explorer.navigate_to(Some(id));
                            preview.search_input.clear();
                        }
                        Some(EntryKind::File) => {
                            self.dialog_invoker = Some(row_widget);
                            preview.dialog = Some(DialogKind::Export);
                        }
                        None => {}
                    }
                }
            }
        }
        if let Some((id, action)) = context_action
            && let Some(preview) = &mut self.preview
        {
            let current_selection = preview.explorer.selected_ids();
            let replace_selection =
                context_action_replaces_selection(action, &current_selection, id);
            if replace_selection {
                preview.explorer.select(id, ids, Modifier::Plain);
            }
            let selected_ids = if replace_selection {
                vec![id]
            } else {
                current_selection
            };
            match action {
                ContextAction::Open => {
                    preview.explorer.navigate_to(Some(id));
                    preview.search_input.clear();
                }
                ContextAction::Export => preview.dialog = Some(DialogKind::Export),
                ContextAction::Rename => {
                    preview.input_name = preview
                        .model
                        .entry(id)
                        .map_or_else(String::new, |entry| entry.name.clone());
                    preview.input_error = None;
                    preview.dialog = Some(DialogKind::Rename);
                }
                ContextAction::MoveToTrash => preview.dialog = Some(DialogKind::MoveToTrash),
                ContextAction::Restore => {
                    preview.model.restore(&selected_ids);
                    preview.explorer.clear_selection();
                    preview.notice = Some(restored_message(selected_ids.len()));
                }
                ContextAction::PermanentDelete => {
                    preview.dialog = Some(DialogKind::PermanentDelete)
                }
            }
        }
    }

    /// Explorer-style status bar: item summary, then the current message or operation,
    /// with the preview controls on the right.
    pub(super) fn render_status_bar(&mut self, ui: &mut egui::Ui, palette: Palette) {
        crate::design::components::use_compact_density(ui);
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = Geometry::SPACE_2;
            if let Some(summary) = self.status_summary() {
                ui.label(
                    RichText::new(summary)
                        .size(TypeScale::HELPER)
                        .color(palette.muted),
                );
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                self.render_preview_controls(ui, palette);
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    let busy = self.preview.as_ref().is_some_and(|preview| {
                        matches!(preview.operation, OperationState::Running { .. })
                    });
                    if busy {
                        status_divider(ui, palette);
                        self.render_operation_status(ui, palette);
                    } else if self.current_feedback().is_some() {
                        status_divider(ui, palette);
                        self.render_feedback(ui, palette);
                    }
                });
            });
        });
    }

    fn status_summary(&mut self) -> Option<String> {
        let preview = self.preview.as_mut()?;
        if matches!(
            preview.scenario,
            encryptdrive_desktop::preview::PreviewScenario::VaultSelection
                | encryptdrive_desktop::preview::PreviewScenario::CreateVault
                | encryptdrive_desktop::preview::PreviewScenario::UnlockVault
                | encryptdrive_desktop::preview::PreviewScenario::AccountSelection
                | encryptdrive_desktop::preview::PreviewScenario::AccountLogin
                | encryptdrive_desktop::preview::PreviewScenario::RegisterAccount
                | encryptdrive_desktop::preview::PreviewScenario::AccountSettings
                | encryptdrive_desktop::preview::PreviewScenario::VaultSettings
        ) {
            return None;
        }
        let total = preview.explorer.visible_ids(&preview.model).len();
        let selected = preview.explorer.selected_ids().len();
        let items = if total == 1 {
            "1 item".to_owned()
        } else {
            format!("{} items", group_thousands(total))
        };
        Some(if selected > 0 {
            format!("{items}  ·  {} selected", group_thousands(selected))
        } else {
            items
        })
    }

    fn current_feedback(&self) -> Option<(&str, FeedbackKind)> {
        let preview = self.preview.as_ref()?;
        feedback_for(
            preview.error.as_deref(),
            preview.notice.as_deref(),
            preview.operation,
        )
    }

    fn render_feedback(&mut self, ui: &mut egui::Ui, palette: Palette) {
        let Some((message, kind)) = self
            .current_feedback()
            .map(|(message, kind)| (message.to_owned(), kind))
        else {
            return;
        };
        let accent = feedback_accent(kind, palette);
        let icon = match kind {
            FeedbackKind::Error => Icon::Error,
            FeedbackKind::Warning => Icon::Warning,
            FeedbackKind::Success => Icon::Success,
            FeedbackKind::Info => Icon::Info,
        };
        let icon_color = if kind == FeedbackKind::Info {
            palette.accent
        } else {
            accent
        };
        ui.label(icon.rich_text(icon_color, Geometry::ICON_SIZE));
        let max_width =
            (ui.available_width() - Geometry::COMPACT_CONTROL_HEIGHT - Geometry::SPACE_3).max(80.0);
        let label = left_aligned_label(
            ui,
            max_width.min(560.0),
            RichText::new(message.as_str())
                .size(TypeScale::HELPER)
                .color(palette.text),
        );
        label.widget_info(|| WidgetInfo::labeled(egui::Role::Label, true, message.as_str()));
        let dismiss = subtle_icon_button(ui, Icon::Close, "Dismiss message", palette);
        if button_activated(ui, &dismiss)
            && let Some(preview) = &mut self.preview
        {
            preview.notice = None;
            preview.error = None;
            if matches!(
                preview.operation,
                OperationState::Succeeded | OperationState::Failed | OperationState::Canceled
            ) {
                preview.operation = OperationState::Idle;
            }
        }
    }

    fn render_operation_status(&mut self, ui: &mut egui::Ui, palette: Palette) {
        let Some(preview) = &mut self.preview else {
            return;
        };
        let OperationState::Running { completed, total } = preview.operation else {
            return;
        };
        let operation_label = preview.operation_label.clone();
        let label = left_aligned_label(
            ui,
            200.0,
            RichText::new(operation_label.as_str())
                .size(TypeScale::HELPER)
                .color(palette.text),
        );
        label
            .widget_info(|| WidgetInfo::labeled(egui::Role::Label, true, operation_label.as_str()));
        let percent = completed.saturating_mul(100) / total.max(1);
        let fraction = preview.operation.progress().unwrap_or(0.0);
        let (bar, _) = ui.allocate_exact_size(egui::vec2(160.0, 4.0), Sense::hover());
        ui.painter()
            .rect_filled(bar, CornerRadius::same(2), palette.separator);
        let mut filled = bar;
        filled.set_width(bar.width() * fraction);
        ui.painter()
            .rect_filled(filled, CornerRadius::same(2), palette.accent);
        let progress_response = ui.interact(
            bar,
            egui::Id::unique("encryptdrive-operation-progress"),
            Sense::hover(),
        );
        progress_response.widget_info(|| {
            WidgetInfo::labeled(
                egui::Role::ProgressIndicator,
                true,
                format!("{operation_label}: {percent}%"),
            )
        });
        ui.label(
            RichText::new(format!("{percent}%"))
                .size(TypeScale::HELPER)
                .color(palette.muted),
        );
        let cancel = secondary_button(ui, "Cancel", palette);
        if button_activated(ui, &cancel) {
            preview.operation = preview.operation.cancel();
            preview.notice = Some("Operation canceled. No files were read or changed.".to_owned());
        }
        if completed < total {
            preview.operation_ticks = preview.operation_ticks.saturating_add(1);
            if preview.operation_ticks >= 2 {
                preview.operation_ticks = 0;
                preview.operation = preview.operation.advance();
                if preview.operation == OperationState::Succeeded {
                    if preview.scenario == encryptdrive_desktop::preview::PreviewScenario::Loading {
                        preview.scenario = encryptdrive_desktop::preview::PreviewScenario::Explorer;
                        preview.operation = OperationState::Idle;
                    } else {
                        preview.notice = Some(
                            "Operation completed. No files were read or written (sample data)."
                                .to_owned(),
                        );
                    }
                }
            }
        }
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(110));
    }

    fn render_default_trash(&mut self, ui: &mut egui::Ui, palette: Palette) {
        let (title, text) = if self.shell.destination == Destination::Trash {
            (
                "Trash is unavailable",
                "Open a vault to see its deleted items.",
            )
        } else {
            ("No vault open", "Open a vault to browse its files.")
        };
        crate::design::components::empty_state(ui, Icon::Lock, title, text, palette);
    }
}

/// Truncating label in a fixed-width, left-aligned slot (`add_sized` would centre it).
fn left_aligned_label(ui: &mut egui::Ui, width: f32, text: RichText) -> egui::Response {
    ui.allocate_ui_with_layout(
        egui::vec2(width, Geometry::CONTROL_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| ui.add(egui::Label::new(text).truncate().selectable(false)),
    )
    .inner
}

/// How much text the command bar can afford at a given content width.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CommandDensity {
    /// Every command shows icon and label.
    Labels,
    /// Only the primary command keeps its label; the rest are icon buttons with tooltips.
    PrimaryLabel,
    /// Every command is an icon button with a tooltip.
    IconsOnly,
}

pub(super) fn command_density(content_width: f32) -> CommandDensity {
    if content_width >= Geometry::COMMAND_LABELS_MIN_WIDTH {
        CommandDensity::Labels
    } else if content_width >= Geometry::COMMAND_PRIMARY_LABEL_MIN_WIDTH {
        CommandDensity::PrimaryLabel
    } else {
        CommandDensity::IconsOnly
    }
}

/// Search grows with spare width up to 320 px and never squeezes the commands.
pub(super) fn search_width(content_width: f32, density: CommandDensity) -> f32 {
    let reserved_for_commands = match density {
        CommandDensity::Labels => 640.0,
        CommandDensity::PrimaryLabel => 400.0,
        CommandDensity::IconsOnly => 260.0,
    };
    (content_width - reserved_for_commands)
        .clamp(Geometry::SEARCH_MIN_WIDTH, Geometry::SEARCH_MAX_WIDTH)
}

fn command_separator(ui: &mut egui::Ui, palette: Palette) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(Geometry::SPACE_3, 20.0), Sense::hover());
    ui.painter().vline(
        rect.center().x,
        rect.y_range(),
        Stroke::new(1.0, palette.separator),
    );
}

fn status_divider(ui: &mut egui::Ui, palette: Palette) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(Geometry::SPACE_3, 16.0), Sense::hover());
    ui.painter().vline(
        rect.center().x,
        rect.y_range(),
        Stroke::new(1.0, palette.separator),
    );
}

fn metadata_cell(ui: &mut egui::Ui, text: &str, align_right: bool, palette: Palette) {
    let layout = if align_right {
        Layout::right_to_left(Align::Center)
    } else {
        Layout::left_to_right(Align::Center)
    };
    ui.horizontal_centered(|ui| {
        ui.with_layout(layout, |ui| {
            ui.add_space(Geometry::SPACE_2);
            table_text_label(
                ui,
                RichText::new(text)
                    .size(TypeScale::METADATA)
                    .color(palette.muted),
            );
        });
    });
}

/// Explorer-style type descriptions for the synthetic file kinds.
pub(super) fn type_label(file_type: &str, kind: EntryKind) -> String {
    if kind == EntryKind::Folder {
        return "File folder".to_owned();
    }
    match file_type {
        "TXT" => "Text document".to_owned(),
        "PDF" => "PDF document".to_owned(),
        "PNG" => "PNG image".to_owned(),
        "DOCX" => "Word document".to_owned(),
        "ZIP" => "ZIP archive".to_owned(),
        "CSV" => "CSV file".to_owned(),
        other => format!("{other} file"),
    }
}

fn restored_message(count: usize) -> String {
    if count == 1 {
        "Restored 1 item to Files.".to_owned()
    } else {
        format!("Restored {count} items to Files.")
    }
}

pub(super) fn group_thousands(value: usize) -> String {
    let digits = value.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    grouped
}

#[derive(Clone, Copy)]
enum ContextAction {
    Open,
    Export,
    Rename,
    MoveToTrash,
    Restore,
    PermanentDelete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FeedbackKind {
    Info,
    Warning,
    Error,
    Success,
}

fn consume_row_focus_request(request: &mut Option<EntryId>) -> Option<EntryId> {
    request.take()
}

fn feedback_for<'a>(
    error: Option<&'a str>,
    notice: Option<&'a str>,
    operation: OperationState,
) -> Option<(&'a str, FeedbackKind)> {
    if let Some(message) = error {
        return Some((message, FeedbackKind::Error));
    }
    if let Some(message) = notice {
        return Some((message, FeedbackKind::Info));
    }
    match operation {
        OperationState::Canceled => Some(("Operation canceled.", FeedbackKind::Warning)),
        OperationState::Succeeded => {
            Some(("Operation completed successfully.", FeedbackKind::Success))
        }
        OperationState::Failed => Some((
            "Operation failed. No data was changed.",
            FeedbackKind::Error,
        )),
        OperationState::Idle | OperationState::Running { .. } => None,
    }
}

fn feedback_accent(kind: FeedbackKind, palette: Palette) -> Color32 {
    match kind {
        FeedbackKind::Info => palette.text,
        FeedbackKind::Warning => palette.warning,
        FeedbackKind::Error => palette.danger,
        FeedbackKind::Success => palette.success,
    }
}

fn empty_list_copy(is_trash: bool, searching: bool) -> (&'static str, &'static str) {
    if searching {
        (
            "No matching items",
            "Try a different name, or clear the search.",
        )
    } else if is_trash {
        ("Trash is empty", "Deleted items will appear here.")
    } else {
        (
            "This folder is empty",
            "Create a folder or import sample files to get started.",
        )
    }
}

fn sort_indicator(
    current_sort: SortKey,
    current_direction: SortDirection,
    column: SortKey,
) -> Option<Icon> {
    (current_sort == column).then_some(match current_direction {
        SortDirection::Ascending => Icon::SortAscending,
        SortDirection::Descending => Icon::SortDescending,
    })
}

fn context_action_replaces_selection(
    action: ContextAction,
    selected_ids: &[EntryId],
    target: EntryId,
) -> bool {
    let keeps_group = matches!(
        action,
        ContextAction::Export
            | ContextAction::MoveToTrash
            | ContextAction::Restore
            | ContextAction::PermanentDelete
    );
    !keeps_group || !selected_ids.contains(&target)
}

fn row_widget_id(id: EntryId) -> egui::Id {
    egui::Id::unique(("encryptdrive-preview-row", id.0))
}

#[cfg(test)]
fn centered_table_cell(ui: &mut egui::Ui, text: impl Into<egui::WidgetText>) -> egui::Response {
    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
        table_text_label(ui, text)
    })
    .inner
}

fn table_text_label(ui: &mut egui::Ui, text: impl Into<egui::WidgetText>) -> egui::Response {
    ui.add(egui::Label::new(text).truncate().selectable(false))
}

fn table_focus_event_filter() -> egui::EventFilter {
    egui::EventFilter {
        vertical_arrows: true,
        horizontal_arrows: true,
        escape: true,
        ..Default::default()
    }
}

pub(super) fn rename_target(selected: &[EntryId]) -> Option<EntryId> {
    if selected.len() == 1 {
        selected.first().copied()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ContextAction, FeedbackKind, centered_table_cell, consume_row_focus_request,
        context_action_replaces_selection, empty_list_copy, feedback_accent, feedback_for,
        rename_target, row_widget_id, sort_indicator, table_focus_event_filter, table_text_label,
    };
    use eframe::egui;
    use eframe::egui::{Align, Layout};
    use encryptdrive_desktop::preview::{
        EntryId, EntryKind, OperationState, SortDirection, SortKey,
    };

    fn render_table_frame(
        app: &mut super::DesktopApp,
        context: &egui::Context,
        events: Vec<egui::Event>,
        time: f64,
    ) {
        let ids = {
            let preview = app.preview.as_mut().expect("preview enabled");
            preview.explorer.visible_ids(&preview.model).to_vec()
        };
        let palette = app.palette();
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1180.0, 780.0),
                )),
                time: Some(time),
                events,
                ..Default::default()
            },
            |ui| app.render_file_table(ui, palette, &ids),
        );
        output.textures_delta.clear();
    }

    fn render_explorer_frame(app: &mut super::DesktopApp, context: &egui::Context, time: f64) {
        let palette = app.palette();
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1180.0, 780.0),
                )),
                time: Some(time),
                ..Default::default()
            },
            |ui| app.render_explorer(ui, palette),
        );
        output.textures_delta.clear();
    }

    fn render_compact_explorer_frame(
        app: &mut super::DesktopApp,
        context: &egui::Context,
        time: f64,
    ) {
        let palette = app.palette();
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(960.0, 640.0),
                )),
                time: Some(time),
                ..Default::default()
            },
            |ui| {
                ui.set_width(744.0);
                app.render_explorer(ui, palette);
            },
        );
        output.textures_delta.clear();
    }

    fn render_explorer_and_dialogs_frame(
        app: &mut super::DesktopApp,
        context: &egui::Context,
        events: Vec<egui::Event>,
        time: f64,
    ) {
        let palette = app.palette();
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1180.0, 780.0),
                )),
                time: Some(time),
                events,
                ..Default::default()
            },
            |ui| {
                app.render_explorer(ui, palette);
                app.render_dialogs(ui.ctx(), palette);
            },
        );
        output.textures_delta.clear();
    }

    fn pointer_button(position: egui::Pos2, pressed: bool) -> egui::Event {
        egui::Event::PointerButton {
            pos: position,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::default(),
        }
    }

    #[test]
    fn operation_error_and_notice_states_render_only_one_feedback_message() {
        assert_eq!(
            feedback_for(Some("The operation failed."), None, OperationState::Failed),
            Some(("The operation failed.", FeedbackKind::Error))
        );
        assert_eq!(
            feedback_for(
                None,
                Some("The sample operation was canceled."),
                OperationState::Canceled
            ),
            Some(("The sample operation was canceled.", FeedbackKind::Info))
        );
    }

    #[test]
    fn feedback_states_use_existing_semantic_status_colors() {
        for theme in [egui::Theme::Light, egui::Theme::Dark] {
            let palette = crate::design::theme::Palette::for_theme(theme);
            assert_eq!(
                feedback_accent(FeedbackKind::Error, palette),
                palette.danger
            );
            assert_eq!(
                feedback_accent(FeedbackKind::Warning, palette),
                palette.warning
            );
            assert_eq!(
                feedback_accent(FeedbackKind::Success, palette),
                palette.success
            );
            assert_ne!(palette.hover, palette.focus);
            assert_ne!(palette.selected, palette.surface_alt);
        }
    }

    #[test]
    fn pending_row_scroll_request_is_consumed_after_one_frame() {
        let mut request = Some(EntryId(42));

        assert_eq!(consume_row_focus_request(&mut request), Some(EntryId(42)));
        assert_eq!(consume_row_focus_request(&mut request), None);
    }

    #[test]
    fn table_focus_keeps_arrows_and_escape_and_allows_tab_traversal() {
        let filter = table_focus_event_filter();
        assert!(filter.vertical_arrows);
        assert!(filter.horizontal_arrows);
        assert!(filter.escape);
        assert!(
            !filter.tab,
            "Tab and Shift+Tab intentionally leave the table"
        );
        for modifiers in [
            egui::Modifiers::default(),
            egui::Modifiers {
                shift: true,
                ..Default::default()
            },
        ] {
            assert!(!filter.matches(&egui::Event::Key {
                key: egui::Key::Tab,
                physical_key: Some(egui::Key::Tab),
                pressed: true,
                repeat: false,
                modifiers,
            }));
        }
    }

    #[test]
    fn sorted_header_keeps_its_text_separate_from_the_icon_glyph() {
        assert_eq!(
            sort_indicator(SortKey::Name, SortDirection::Ascending, SortKey::Name),
            Some(super::Icon::SortAscending)
        );
        assert_eq!(
            sort_indicator(SortKey::Name, SortDirection::Descending, SortKey::Name),
            Some(super::Icon::SortDescending)
        );
        assert_eq!(
            sort_indicator(
                SortKey::Name,
                SortDirection::Ascending,
                SortKey::ModifiedSample
            ),
            None
        );
    }

    #[test]
    fn metadata_cells_center_their_text_in_the_row_height() {
        let mut vertical_offset = f32::MAX;
        egui::__run_test_ui(|ui| {
            let cell_rect = ui.max_rect();
            let label = centered_table_cell(ui, "modified");
            vertical_offset = label.rect.center().y - cell_rect.center().y;
        });

        assert!(vertical_offset.abs() <= 0.5);
    }

    #[test]
    fn clicking_file_name_text_reaches_the_parent_row() {
        let context = egui::Context::default();
        let run_frame = |events: Vec<egui::Event>| {
            let mut label_rect = egui::Rect::NOTHING;
            let mut row_clicked = false;
            let mut output = context.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(640.0, 480.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ui| {
                    let (row_id, row_rect) = ui.allocate_space(egui::vec2(400.0, 40.0));
                    let row = ui.interact(row_rect, row_id, egui::Sense::CLICK);
                    let mut row_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .id_salt("clickable-file-row")
                            .max_rect(row_rect)
                            .layout(Layout::left_to_right(Align::Center)),
                    );
                    label_rect = table_text_label(&mut row_ui, "quarterly-report.pdf").rect;
                    row_clicked = row.clicked();
                },
            );
            output.textures_delta.clear();
            (label_rect, row_clicked)
        };

        let (label_rect, _) = run_frame(Vec::new());
        let position = label_rect.center();
        let _ = run_frame(vec![
            egui::Event::PointerMoved(position),
            egui::Event::PointerButton {
                pos: position,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::default(),
            },
        ]);
        let (_, row_clicked) = run_frame(vec![
            egui::Event::PointerMoved(position),
            egui::Event::PointerButton {
                pos: position,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::default(),
            },
        ]);

        assert!(row_clicked);
    }

    #[test]
    fn clicking_the_rendered_filename_selects_its_stable_entry_id() {
        let context = egui::Context::default();
        let mut app = super::DesktopApp::new(true);
        render_table_frame(&mut app, &context, Vec::new(), 0.0);
        let row = context
            .read_response(row_widget_id(EntryId(4)))
            .expect("real file row is rendered");
        // The filename follows the 16 px file icon and 4 px spacing in the name cell.
        let position = egui::pos2(row.rect.left() + 44.0, row.rect.center().y);

        render_table_frame(
            &mut app,
            &context,
            vec![
                egui::Event::PointerMoved(position),
                pointer_button(position, true),
            ],
            0.1,
        );
        render_table_frame(
            &mut app,
            &context,
            vec![pointer_button(position, false)],
            0.15,
        );

        assert_eq!(
            app.preview.as_ref().unwrap().explorer.selected_ids(),
            vec![EntryId(4)]
        );
    }

    #[test]
    fn double_clicking_the_rendered_folder_name_navigates_into_that_folder() {
        let context = egui::Context::default();
        let mut app = super::DesktopApp::new(true);
        render_table_frame(&mut app, &context, Vec::new(), 0.0);
        let row = context
            .read_response(row_widget_id(EntryId(1)))
            .expect("real Documents row is rendered");
        let position = egui::pos2(row.rect.left() + 44.0, row.rect.center().y);

        render_table_frame(
            &mut app,
            &context,
            vec![
                egui::Event::PointerMoved(position),
                pointer_button(position, true),
            ],
            0.1,
        );
        render_table_frame(
            &mut app,
            &context,
            vec![pointer_button(position, false)],
            0.15,
        );
        render_table_frame(
            &mut app,
            &context,
            vec![
                egui::Event::PointerMoved(position),
                pointer_button(position, true),
            ],
            0.25,
        );
        render_table_frame(
            &mut app,
            &context,
            vec![pointer_button(position, false)],
            0.3,
        );

        let preview = app.preview.as_ref().unwrap();
        assert_eq!(preview.explorer.current_folder(), Some(EntryId(1)));
        assert_eq!(
            preview.model.entry(EntryId(1)).map(|entry| entry.kind),
            Some(EntryKind::Folder)
        );
    }

    #[test]
    fn double_clicking_a_file_opens_the_export_warning_without_starting_it() {
        let context = egui::Context::default();
        let mut app = super::DesktopApp::new(true);
        render_explorer_and_dialogs_frame(&mut app, &context, Vec::new(), 0.0);
        let row = context
            .read_response(row_widget_id(EntryId(4)))
            .expect("real Annual report row is rendered");
        let position = egui::pos2(row.rect.left() + 44.0, row.rect.center().y);

        render_explorer_and_dialogs_frame(
            &mut app,
            &context,
            vec![
                egui::Event::PointerMoved(position),
                pointer_button(position, true),
            ],
            0.1,
        );
        render_explorer_and_dialogs_frame(
            &mut app,
            &context,
            vec![pointer_button(position, false)],
            0.15,
        );
        render_explorer_and_dialogs_frame(
            &mut app,
            &context,
            vec![
                egui::Event::PointerMoved(position),
                pointer_button(position, true),
            ],
            0.25,
        );
        render_explorer_and_dialogs_frame(
            &mut app,
            &context,
            vec![pointer_button(position, false)],
            0.3,
        );

        let preview = app.preview.as_ref().unwrap();
        assert_eq!(preview.dialog, Some(super::DialogKind::Export));
        assert_eq!(preview.operation, OperationState::Idle);
    }

    #[test]
    fn selecting_a_row_does_not_move_the_rendered_table() {
        let context = egui::Context::default();
        let mut app = super::DesktopApp::new(true);
        render_explorer_frame(&mut app, &context, 0.0);
        let before = context
            .read_response(row_widget_id(EntryId(4)))
            .expect("selected test row is rendered")
            .rect
            .top();

        {
            let preview = app.preview.as_mut().expect("preview enabled");
            let visible = preview.explorer.visible_ids(&preview.model);
            preview
                .explorer
                .select(EntryId(4), &visible, super::Modifier::Plain);
        }
        render_explorer_frame(&mut app, &context, 0.1);
        let after = context
            .read_response(row_widget_id(EntryId(4)))
            .expect("selected test row remains rendered")
            .rect
            .top();

        assert_eq!(before, after, "selection must not change table geometry");
    }

    #[test]
    fn selection_and_feedback_transitions_preserve_the_rendered_table_geometry() {
        let context = egui::Context::default();
        let mut app = super::DesktopApp::new(true);
        render_explorer_frame(&mut app, &context, 0.0);
        let initial_top = context
            .read_response(row_widget_id(EntryId(4)))
            .expect("real file row is rendered")
            .rect
            .top();
        assert!(
            initial_top < 320.0,
            "the table should start right after the page header and the command bar"
        );

        {
            let preview = app.preview.as_mut().expect("preview enabled");
            let visible = preview.explorer.visible_ids(&preview.model);
            preview
                .explorer
                .select(EntryId(4), &visible, super::Modifier::Plain);
            preview.notice =
                Some("Synthetic recovery status: some optional metadata was skipped.".to_owned());
        }
        render_explorer_frame(&mut app, &context, 0.1);
        let selected_with_notice_top = context
            .read_response(row_widget_id(EntryId(4)))
            .expect("real file row remains rendered")
            .rect
            .top();

        {
            let preview = app.preview.as_mut().expect("preview enabled");
            preview.notice = None;
            preview.error = Some("The sample operation failed without changing data.".to_owned());
        }
        render_explorer_frame(&mut app, &context, 0.2);
        let error_top = context
            .read_response(row_widget_id(EntryId(4)))
            .expect("real file row remains rendered")
            .rect
            .top();

        assert_eq!(initial_top, selected_with_notice_top);
        assert_eq!(initial_top, error_top);

        app.preview.as_mut().unwrap().error = None;
        app.preview.as_mut().unwrap().operation = OperationState::running(12);
        render_explorer_frame(&mut app, &context, 0.3);
        let operation_top = context
            .read_response(row_widget_id(EntryId(4)))
            .expect("real file row remains rendered during operation")
            .rect
            .top();
        assert_eq!(initial_top, operation_top);
    }

    #[test]
    fn compact_commandbar_keeps_table_geometry_when_selection_actions_appear() {
        let context = egui::Context::default();
        let mut app = super::DesktopApp::new(true);
        render_compact_explorer_frame(&mut app, &context, 0.0);
        let before = context
            .read_response(row_widget_id(EntryId(4)))
            .expect("compact file row is rendered")
            .rect
            .top();

        {
            let preview = app.preview.as_mut().expect("preview enabled");
            let visible = preview.explorer.visible_ids(&preview.model);
            preview
                .explorer
                .select(EntryId(4), &visible, super::Modifier::Plain);
        }
        render_compact_explorer_frame(&mut app, &context, 0.1);
        let after = context
            .read_response(row_widget_id(EntryId(4)))
            .expect("compact file row remains rendered")
            .rect
            .top();

        assert_eq!(before, after);
    }

    #[test]
    fn command_bar_collapses_progressively_and_search_stays_within_bounds() {
        use super::{CommandDensity, command_density, search_width};
        assert_eq!(command_density(1200.0), CommandDensity::Labels);
        assert_eq!(command_density(720.0), CommandDensity::PrimaryLabel);
        assert_eq!(command_density(560.0), CommandDensity::IconsOnly);
        for width in [400.0, 560.0, 720.0, 904.0, 1300.0, 1800.0] {
            let search = search_width(width, command_density(width));
            assert!((160.0..=320.0).contains(&search), "{width} -> {search}");
        }
        assert_eq!(search_width(1800.0, CommandDensity::Labels), 320.0);
    }

    #[test]
    fn empty_folder_copy_does_not_claim_that_a_search_is_active() {
        assert_eq!(
            empty_list_copy(false, false),
            (
                "This folder is empty",
                "Create a folder or import sample files to get started."
            )
        );
        assert_eq!(
            empty_list_copy(false, true),
            (
                "No matching items",
                "Try a different name, or clear the search."
            )
        );
        assert_eq!(
            empty_list_copy(true, false),
            ("Trash is empty", "Deleted items will appear here.")
        );
    }

    #[test]
    fn group_context_actions_preserve_a_selected_multi_selection() {
        let selected = [EntryId(7), EntryId(9)];

        assert!(!context_action_replaces_selection(
            ContextAction::MoveToTrash,
            &selected,
            EntryId(9)
        ));
        assert!(!context_action_replaces_selection(
            ContextAction::Restore,
            &selected,
            EntryId(7)
        ));
    }

    #[test]
    fn context_action_selects_an_unselected_target_and_rename_is_single_item() {
        let selected = [EntryId(7), EntryId(9)];

        assert!(context_action_replaces_selection(
            ContextAction::Export,
            &selected,
            EntryId(11)
        ));
        assert!(context_action_replaces_selection(
            ContextAction::Rename,
            &selected,
            EntryId(9)
        ));
    }

    #[test]
    fn row_widget_focus_ids_are_stable_across_rebuilds_and_sorts() {
        let first = row_widget_id(EntryId(42));
        let rebuilt = row_widget_id(EntryId(42));
        let other = row_widget_id(EntryId(43));

        assert_eq!(first, rebuilt);
        assert_ne!(first, other);
    }

    #[test]
    fn rename_requires_exactly_one_selected_target() {
        assert_eq!(rename_target(&[EntryId(8)]), Some(EntryId(8)));
        assert_eq!(rename_target(&[EntryId(3), EntryId(8)]), None);
        assert_eq!(rename_target(&[]), None);
    }
}
