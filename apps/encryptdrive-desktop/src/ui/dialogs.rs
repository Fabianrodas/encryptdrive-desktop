use eframe::egui::{self, CornerRadius, Frame, Margin, RichText, Stroke};

use encryptdrive_desktop::preview::{EntryId, NameValidationError, OperationState};

use super::app::{DesktopApp, DialogKind};
use crate::design::{
    components::{
        ButtonKind, button_activated_when_allowed, dialog_body, dialog_button, dialog_footer,
        dialog_frame, labeled_text_field,
    },
    geometry::Geometry,
    icons::Icon,
    theme::Palette,
    typography::TypeScale,
};

impl DesktopApp {
    pub(super) fn render_dialogs(&mut self, ctx: &egui::Context, palette: Palette) {
        let Some(preview) = &mut self.preview else {
            return;
        };
        let Some(dialog) = preview.dialog else {
            self.dialog_invoker = None;
            self.focused_dialog = None;
            if let Some(target) = self.focus_return_target.take() {
                ctx.memory_mut(|memory| memory.request_focus(target));
            }
            return;
        };
        let request_cancel_focus = self.focused_dialog != Some(dialog);
        if request_cancel_focus {
            self.focus_return_target = self
                .dialog_invoker
                .take()
                .or_else(|| ctx.memory(|memory| memory.focused()))
                .or(self.focused_row_widget)
                .or(self.files_navigation_widget);
        }
        self.focused_dialog = Some(dialog);
        let selected = preview.explorer.selected_ids();
        let subject = item_subject(preview, &selected);
        let title = match dialog {
            DialogKind::CreateFolder => "New folder".to_owned(),
            DialogKind::Rename => "Rename".to_owned(),
            DialogKind::MoveToTrash => "Move to Trash?".to_owned(),
            DialogKind::PermanentDelete => "Delete permanently?".to_owned(),
            DialogKind::Import => "Import files".to_owned(),
            DialogKind::Export => "Export a decrypted copy?".to_owned(),
        };
        let mut outcome = None;
        let enter_pressed = ctx.input(|input| {
            crate::design::components::key_pressed_without_repeat(input, egui::Key::Enter)
        });
        let tab_pressed = ctx.input(|input| input.key_pressed(egui::Key::Tab));
        let style = ctx.global_style();
        // Activation is ignored on the frame a dialog opens: the key or click that opened it
        // must never also confirm it.
        let allowed = !request_cancel_focus;
        egui::Modal::new(egui::Id::unique("encryptdrive-preview-dialog"))
            .frame(dialog_frame(&style, palette))
            .show(ctx, |ui| {
                dialog_body(ui, &title, palette, |ui| match dialog {
                    DialogKind::CreateFolder | DialogKind::Rename => {
                        let (label, hint) = if dialog == DialogKind::CreateFolder {
                            ("Folder name", "New folder")
                        } else {
                            ("New name", "Item name")
                        };
                        let response =
                            labeled_text_field(ui, label, &mut preview.input_name, hint, false, palette);
                        if request_cancel_focus || (response.lost_focus() && !tab_pressed) {
                            response.request_focus();
                        }
                        if let Some(error) = preview.input_error.as_deref() {
                            ui.add_space(Geometry::BASE);
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = Geometry::BASE + 2.0;
                                crate::design::components::decorative_icon(ui, Icon::Error, palette.danger, TypeScale::HELPER);
                                ui.label(
                                    RichText::new(error)
                                        .size(TypeScale::HELPER)
                                        .color(palette.danger),
                                );
                            });
                        }
                        if allowed
                            && enter_submits(
                                dialog,
                                response.has_focus() || response.lost_focus(),
                                enter_pressed,
                            )
                        {
                            outcome = Some(form_outcome(dialog, preview));
                        }
                    }
                    DialogKind::MoveToTrash => {
                        body_text(ui, &format!("{subject} will be moved to Trash."), palette);
                        caption(
                            ui,
                            "Contents of selected folders move with them. You can restore items from Trash.",
                            palette,
                        );
                    }
                    DialogKind::PermanentDelete => {
                        body_text(
                            ui,
                            &format!("{subject} will be permanently deleted. This can't be undone."),
                            palette,
                        );
                        caption(ui, "Sample data — no real file or vault is affected.", palette);
                    }
                    DialogKind::Import => {
                        body_text(ui, "Files you import are encrypted into this vault.", palette);
                        caption(
                            ui,
                            "This preview simulates progress only: no file picker opens and no files are read.",
                            palette,
                        );
                    }
                    DialogKind::Export => {
                        warning_bar(
                            ui,
                            "The exported copy is plaintext outside the encrypted vault.",
                            palette,
                        );
                        ui.add_space(Geometry::SPACE_3);
                        body_text(
                            ui,
                            "Save it only to a location you trust, and delete it when you no longer need it.",
                            palette,
                        );
                        caption(ui, "This preview does not decrypt, open or write any file.", palette);
                    }
                });
                dialog_footer(ui, palette, |ui, width| {
                    let (label, kind) = match dialog {
                        DialogKind::CreateFolder => ("Create", ButtonKind::Primary),
                        DialogKind::Rename => ("Rename", ButtonKind::Primary),
                        DialogKind::MoveToTrash => ("Move to Trash", ButtonKind::Destructive),
                        DialogKind::PermanentDelete => ("Delete permanently", ButtonKind::Destructive),
                        DialogKind::Import => ("Start import", ButtonKind::Primary),
                        DialogKind::Export => ("Export copy", ButtonKind::Primary),
                    };
                    // Fluent order: primary action first (left), Cancel last (right).
                    let confirm = dialog_button(ui, label, kind, width, palette);
                    let cancel = dialog_button(ui, "Cancel", ButtonKind::Standard, width, palette);
                    let confirmation_first = matches!(
                        dialog,
                        DialogKind::MoveToTrash
                            | DialogKind::PermanentDelete
                            | DialogKind::Import
                            | DialogKind::Export
                    );
                    if request_cancel_focus && confirmation_first {
                        cancel.request_focus();
                    }
                    if button_activated_when_allowed(ui, &cancel, allowed) {
                        outcome = Some(DialogOutcome::Cancel);
                    }
                    if button_activated_when_allowed(ui, &confirm, allowed) {
                        outcome = Some(match dialog {
                            DialogKind::CreateFolder | DialogKind::Rename => {
                                form_outcome(dialog, preview)
                            }
                            DialogKind::MoveToTrash => DialogOutcome::MoveToTrash(selected.clone()),
                            DialogKind::PermanentDelete => {
                                DialogOutcome::PermanentlyDelete(selected.clone())
                            }
                            DialogKind::Import => DialogOutcome::StartOperation("Importing files"),
                            DialogKind::Export => {
                                DialogOutcome::StartOperation("Exporting a decrypted copy")
                            }
                        });
                    }
                });
            });

        if let Some(outcome) = outcome {
            match outcome {
                DialogOutcome::Cancel => {
                    if let Some(preview) = &mut self.preview {
                        preview.dialog = None;
                        preview.input_error = None;
                    }
                }
                DialogOutcome::CreateFolder(parent, name) => {
                    if let Some(preview) = &mut self.preview {
                        match preview
                            .model
                            .create_folder(parent, &name, "2026-10-19 10:00")
                        {
                            Ok(_) => {
                                preview.dialog = None;
                                preview.input_error = None;
                                preview.notice = Some(format!("Created folder “{}”.", name.trim()));
                            }
                            Err(error) => {
                                preview.input_error = Some(validation_message(error).to_owned())
                            }
                        }
                    }
                }
                DialogOutcome::Rename(id, name) => {
                    if let Some(preview) = &mut self.preview {
                        match preview.model.rename(id, &name) {
                            Ok(()) => {
                                preview.dialog = None;
                                preview.input_error = None;
                                preview.notice = Some(format!("Renamed to “{}”.", name.trim()));
                            }
                            Err(error) => {
                                preview.input_error = Some(validation_message(error).to_owned())
                            }
                        }
                    }
                }
                DialogOutcome::MoveToTrash(ids) => {
                    if let Some(preview) = &mut self.preview {
                        preview.model.move_to_trash(&ids, "2026-10-19 10:00");
                        preview.explorer.clear_selection();
                        preview.dialog = None;
                        self.focus_return_target =
                            self.focused_row_widget.or(self.files_navigation_widget);
                        preview.notice = Some(count_message("Moved", ids.len(), "to Trash"));
                    }
                }
                DialogOutcome::PermanentlyDelete(ids) => {
                    if let Some(preview) = &mut self.preview {
                        preview.model.permanently_delete(&ids);
                        preview.explorer.clear_selection();
                        preview.dialog = None;
                        self.focus_return_target =
                            self.focused_row_widget.or(self.trash_navigation_widget);
                        preview.notice = Some(count_message("Permanently deleted", ids.len(), ""));
                    }
                }
                DialogOutcome::StartOperation(message) => {
                    if let Some(preview) = &mut self.preview {
                        preview.dialog = None;
                        preview.operation = OperationState::running(12);
                        preview.operation_ticks = 0;
                        preview.operation_label = message.to_owned();
                    }
                }
            }
        }
    }
}

fn form_outcome(dialog: DialogKind, preview: &super::app::PreviewState) -> DialogOutcome {
    match dialog {
        DialogKind::Rename => match preview.explorer.selected_ids().first().copied() {
            Some(id) => DialogOutcome::Rename(id, preview.input_name.clone()),
            None => DialogOutcome::Cancel,
        },
        _ => DialogOutcome::CreateFolder(
            preview.explorer.current_folder(),
            preview.input_name.clone(),
        ),
    }
}

/// “Name.ext” for one item, otherwise “3 items”.
fn item_subject(preview: &super::app::PreviewState, selected: &[EntryId]) -> String {
    match selected {
        [id] => preview
            .model
            .entry(*id)
            .map_or_else(|| "1 item".to_owned(), |entry| format!("“{}”", entry.name)),
        ids => super::explorer::item_count(ids.len()),
    }
}

fn count_message(verb: &str, count: usize, suffix: &str) -> String {
    let items = super::explorer::item_count(count);
    if suffix.is_empty() {
        format!("{verb} {items}.")
    } else {
        format!("{verb} {items} {suffix}.")
    }
}

fn body_text(ui: &mut egui::Ui, text: &str, palette: Palette) {
    ui.label(
        RichText::new(text)
            .size(TypeScale::BODY)
            .color(palette.text),
    );
}

fn caption(ui: &mut egui::Ui, text: &str, palette: Palette) {
    ui.add_space(Geometry::SPACE_2);
    ui.label(
        RichText::new(text)
            .size(TypeScale::HELPER)
            .color(palette.muted),
    );
}

/// Fluent InfoBar (warning severity): tinted background, coloured icon, body text.
fn warning_bar(ui: &mut egui::Ui, text: &str, palette: Palette) {
    Frame::new()
        .fill(palette.caution_surface())
        .stroke(Stroke::new(1.0, palette.caution_stroke()))
        .corner_radius(CornerRadius::same(Geometry::CONTROL_RADIUS))
        .inner_margin(Margin::symmetric(
            Geometry::SPACE_3 as i8,
            Geometry::SPACE_3 as i8,
        ))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = Geometry::SPACE_2;
                crate::design::components::decorative_icon(
                    ui,
                    Icon::Warning,
                    palette.warning,
                    Geometry::ICON_SIZE,
                );
                ui.add(
                    egui::Label::new(
                        RichText::new(text)
                            .font(crate::design::typography::semibold_font(TypeScale::BODY))
                            .color(palette.text),
                    )
                    .wrap(),
                );
            });
        });
}

enum DialogOutcome {
    Cancel,
    CreateFolder(Option<EntryId>, String),
    Rename(EntryId, String),
    MoveToTrash(Vec<EntryId>),
    PermanentlyDelete(Vec<EntryId>),
    StartOperation(&'static str),
}

fn validation_message(error: NameValidationError) -> &'static str {
    match error {
        NameValidationError::Empty => "Enter a name.",
        NameValidationError::InvalidCharacters => {
            "A name cannot contain a slash or control character."
        }
        NameValidationError::NameAlreadyExists => "An item with this name already exists here.",
        NameValidationError::ParentNotFolder => "The selected location is no longer available.",
        NameValidationError::EntryNotFound => "The selected item is no longer available.",
    }
}

fn enter_submits(dialog: DialogKind, text_field_focused: bool, enter_pressed: bool) -> bool {
    enter_pressed
        && match dialog {
            DialogKind::CreateFolder | DialogKind::Rename => text_field_focused,
            DialogKind::MoveToTrash
            | DialogKind::PermanentDelete
            | DialogKind::Import
            | DialogKind::Export => false,
        }
}

#[cfg(test)]
mod tests {
    use super::{DesktopApp, DialogKind, enter_submits};
    use eframe::egui::{self, Pos2, Rect, Vec2};
    use encryptdrive_desktop::preview::{EntryId, Modifier, OperationState};

    fn key_event(key: egui::Key, pressed: bool, repeat: bool) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: Some(key),
            pressed,
            repeat,
            modifiers: egui::Modifiers::default(),
        }
    }

    fn run_dialog_events(app: &mut DesktopApp, context: &egui::Context, events: Vec<egui::Event>) {
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(640.0, 480.0))),
                events,
                ..Default::default()
            },
            |ui| {
                app.handle_global_keys(ui.ctx());
                let palette = app.palette();
                app.render_dialogs(ui.ctx(), palette);
            },
        );
        output.textures_delta.clear();
    }

    fn run_dialog_frame(app: &mut DesktopApp, context: &egui::Context, key: Option<egui::Key>) {
        let events = key
            .map(|key| vec![key_event(key, true, false)])
            .unwrap_or_default();
        run_dialog_events(app, context, events);
    }

    #[test]
    fn enter_only_submits_create_and_rename_forms_from_their_text_field() {
        assert!(enter_submits(DialogKind::CreateFolder, true, true));
        assert!(!enter_submits(DialogKind::CreateFolder, false, true));
        assert!(enter_submits(DialogKind::Rename, true, true));
        assert!(!enter_submits(DialogKind::Rename, false, true));
        assert!(!enter_submits(DialogKind::MoveToTrash, false, true));
        assert!(!enter_submits(DialogKind::PermanentDelete, false, true));
        assert!(!enter_submits(DialogKind::Import, false, true));
        assert!(!enter_submits(DialogKind::Export, false, true));
        assert!(!enter_submits(DialogKind::Export, false, false));
    }

    #[test]
    fn entering_a_file_and_rendering_the_same_enter_keeps_export_warning_open() {
        let mut app = DesktopApp::new(true);
        let preview = app.preview.as_mut().expect("preview enabled");
        let visible = preview.explorer.visible_ids(&preview.model);
        preview
            .explorer
            .select(EntryId(4), &visible, Modifier::Plain);
        app.table_has_focus = true;
        let context = egui::Context::default();
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(640.0, 480.0))),
                events: vec![egui::Event::Key {
                    key: egui::Key::Enter,
                    physical_key: Some(egui::Key::Enter),
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::default(),
                }],
                ..Default::default()
            },
            |ui| {
                let context = ui.ctx().clone();
                app.handle_global_keys(&context);
                let palette = app.palette();
                app.render_dialogs(&context, palette);
            },
        );
        output.textures_delta.clear();

        let preview = app.preview.as_ref().expect("preview enabled");
        assert_eq!(preview.dialog, Some(DialogKind::Export));
        assert_eq!(preview.operation, OperationState::Idle);
    }

    #[test]
    fn export_confirmation_requires_a_distinct_enter_on_the_confirm_control() {
        let mut app = DesktopApp::new(true);
        let preview = app.preview.as_mut().expect("preview enabled");
        let visible = preview.explorer.visible_ids(&preview.model);
        preview
            .explorer
            .select(EntryId(4), &visible, Modifier::Plain);
        app.table_has_focus = true;
        let context = egui::Context::default();

        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(640.0, 480.0))),
                events: vec![key_event(egui::Key::Enter, true, false)],
                ..Default::default()
            },
            |ui| {
                let context = ui.ctx().clone();
                app.handle_global_keys(&context);
                let palette = app.palette();
                app.render_dialogs(&context, palette);
            },
        );
        output.textures_delta.clear();
        assert_eq!(
            app.preview.as_ref().unwrap().dialog,
            Some(DialogKind::Export)
        );
        assert_eq!(
            app.preview.as_ref().unwrap().operation,
            OperationState::Idle
        );

        run_dialog_events(
            &mut app,
            &context,
            vec![key_event(egui::Key::Enter, false, false)],
        );
        run_dialog_events(
            &mut app,
            &context,
            vec![key_event(egui::Key::Tab, true, false)],
        );
        run_dialog_frame(&mut app, &context, Some(egui::Key::Enter));

        let preview = app.preview.as_ref().unwrap();
        assert_eq!(preview.dialog, None);
        assert!(matches!(preview.operation, OperationState::Running { .. }));
    }

    #[test]
    fn held_enter_cannot_dismiss_or_confirm_the_export_dialog_that_it_opened() {
        let mut app = DesktopApp::new(true);
        let preview = app.preview.as_mut().expect("preview enabled");
        let visible = preview.explorer.visible_ids(&preview.model);
        preview
            .explorer
            .select(EntryId(4), &visible, Modifier::Plain);
        app.table_has_focus = true;
        let context = egui::Context::default();

        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(640.0, 480.0))),
                events: vec![egui::Event::Key {
                    key: egui::Key::Enter,
                    physical_key: Some(egui::Key::Enter),
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::default(),
                }],
                ..Default::default()
            },
            |ui| {
                let context = ui.ctx().clone();
                app.handle_global_keys(&context);
                let palette = app.palette();
                app.render_dialogs(&context, palette);
            },
        );
        output.textures_delta.clear();
        assert_eq!(
            app.preview.as_ref().unwrap().dialog,
            Some(DialogKind::Export)
        );
        assert_eq!(
            app.preview.as_ref().unwrap().operation,
            OperationState::Idle
        );

        run_dialog_events(
            &mut app,
            &context,
            vec![key_event(egui::Key::Enter, true, true)],
        );

        let preview = app.preview.as_ref().unwrap();
        assert_eq!(
            preview.dialog,
            Some(DialogKind::Export),
            "key repeat must not activate the initial Cancel or export action"
        );
        assert_eq!(preview.operation, OperationState::Idle);

        run_dialog_events(
            &mut app,
            &context,
            vec![key_event(egui::Key::Enter, true, true)],
        );
        assert_eq!(
            app.preview.as_ref().unwrap().dialog,
            Some(DialogKind::Export),
            "a held Enter remains inert across subsequent repeat events"
        );

        run_dialog_events(
            &mut app,
            &context,
            vec![key_event(egui::Key::Enter, false, false)],
        );
        run_dialog_frame(&mut app, &context, Some(egui::Key::Enter));
        let preview = app.preview.as_ref().unwrap();
        assert_eq!(
            preview.dialog, None,
            "a distinct Enter on focused Cancel cancels"
        );
        assert_eq!(preview.operation, OperationState::Idle);
    }

    #[test]
    fn enter_on_the_initial_cancel_focus_cancels_export_instead_of_starting_it() {
        let mut app = DesktopApp::new(true);
        app.preview.as_mut().expect("preview enabled").dialog = Some(DialogKind::Export);
        let context = egui::Context::default();

        run_dialog_frame(&mut app, &context, None);
        run_dialog_frame(&mut app, &context, Some(egui::Key::Enter));

        let preview = app.preview.as_ref().expect("preview enabled");
        assert_eq!(preview.dialog, None);
        assert_eq!(preview.operation, OperationState::Idle);
    }

    #[test]
    fn held_enter_cannot_submit_a_form_until_a_distinct_key_press() {
        let mut app = DesktopApp::new(true);
        let preview = app.preview.as_mut().expect("preview enabled");
        preview.dialog = Some(DialogKind::CreateFolder);
        preview.input_name = "Prepared folder".to_owned();
        let context = egui::Context::default();

        run_dialog_events(
            &mut app,
            &context,
            vec![key_event(egui::Key::Enter, true, false)],
        );
        assert_eq!(
            app.preview.as_ref().unwrap().dialog,
            Some(DialogKind::CreateFolder),
            "the dialog-opening frame cannot also submit the form"
        );
        assert!(
            !app.preview
                .as_ref()
                .unwrap()
                .model
                .entries()
                .any(|entry| entry.name == "Prepared folder")
        );

        run_dialog_events(
            &mut app,
            &context,
            vec![key_event(egui::Key::Enter, true, true)],
        );
        assert_eq!(
            app.preview.as_ref().unwrap().dialog,
            Some(DialogKind::CreateFolder),
            "held-key repeat cannot submit after initial field focus"
        );

        run_dialog_events(
            &mut app,
            &context,
            vec![key_event(egui::Key::Enter, false, false)],
        );
        run_dialog_frame(&mut app, &context, Some(egui::Key::Enter));
        let preview = app.preview.as_ref().unwrap();
        assert_eq!(preview.dialog, None);
        assert!(
            preview
                .model
                .entries()
                .any(|entry| entry.name == "Prepared folder")
        );
    }

    #[test]
    fn enter_on_cancel_cannot_move_or_permanently_delete_selected_items() {
        for (dialog, destination, id) in [
            (
                DialogKind::MoveToTrash,
                encryptdrive_desktop::Destination::Files,
                EntryId(4),
            ),
            (
                DialogKind::PermanentDelete,
                encryptdrive_desktop::Destination::Trash,
                EntryId(90_000),
            ),
        ] {
            let mut app = DesktopApp::new(true);
            let preview = app.preview.as_mut().expect("preview enabled");
            preview.explorer.set_destination(destination);
            let visible = preview.explorer.visible_ids(&preview.model);
            preview.explorer.select(id, &visible, Modifier::Plain);
            preview.dialog = Some(dialog);
            let context = egui::Context::default();

            run_dialog_frame(&mut app, &context, None);
            run_dialog_frame(&mut app, &context, Some(egui::Key::Enter));

            let preview = app.preview.as_ref().expect("preview enabled");
            assert_eq!(preview.dialog, None, "{dialog:?} should cancel on Enter");
            assert_eq!(preview.operation, OperationState::Idle);
            let entry = preview.model.entry(id).expect("Cancel preserves the entry");
            assert_eq!(
                entry.deleted_at.is_some(),
                destination == encryptdrive_desktop::Destination::Trash
            );
        }
    }

    #[test]
    fn repeated_enter_cannot_confirm_after_focus_moves_to_a_destructive_action() {
        let mut app = DesktopApp::new(true);
        let context = egui::Context::default();

        // Simulate Enter being held by a different focused control before the destructive
        // dialog appears. Tab changes dialog focus while the original Enter remains down.
        run_dialog_events(
            &mut app,
            &context,
            vec![key_event(egui::Key::Enter, true, false)],
        );
        let preview = app.preview.as_mut().expect("preview enabled");
        preview
            .explorer
            .set_destination(encryptdrive_desktop::Destination::Trash);
        let visible = preview.explorer.visible_ids(&preview.model);
        let id = visible[0];
        preview.explorer.select(id, &visible, Modifier::Plain);
        preview.dialog = Some(DialogKind::PermanentDelete);

        run_dialog_frame(&mut app, &context, None);
        let cancel_focus = context.memory(|memory| memory.focused());
        run_dialog_events(
            &mut app,
            &context,
            vec![key_event(egui::Key::Tab, true, false)],
        );
        let confirm_focus = context.memory(|memory| memory.focused());
        assert_ne!(
            cancel_focus, confirm_focus,
            "Tab must move focus to another dialog control"
        );

        run_dialog_events(
            &mut app,
            &context,
            vec![key_event(egui::Key::Enter, true, true)],
        );
        assert_eq!(
            app.preview.as_ref().unwrap().dialog,
            Some(DialogKind::PermanentDelete),
            "held-key repeat after a focus change must not confirm deletion"
        );
        assert!(app.preview.as_ref().unwrap().model.entry(id).is_some());

        run_dialog_events(
            &mut app,
            &context,
            vec![key_event(egui::Key::Enter, false, false)],
        );
        run_dialog_frame(&mut app, &context, Some(egui::Key::Enter));
        assert_eq!(app.preview.as_ref().unwrap().dialog, None);
        assert!(
            app.preview.as_ref().unwrap().model.entry(id).is_none(),
            "a separate Enter on the now-focused destructive action is intentional confirmation"
        );
    }

    #[test]
    fn escape_closes_export_without_starting_an_operation() {
        let mut app = DesktopApp::new(true);
        app.preview.as_mut().expect("preview enabled").dialog = Some(DialogKind::Export);
        let context = egui::Context::default();

        run_dialog_frame(&mut app, &context, Some(egui::Key::Escape));

        let preview = app.preview.as_ref().expect("preview enabled");
        assert_eq!(preview.dialog, None);
        assert_eq!(preview.operation, OperationState::Idle);
    }
}
