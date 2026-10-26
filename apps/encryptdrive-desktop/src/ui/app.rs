use eframe::egui;

use encryptdrive_desktop::{Appearance, AppearancePreference, Destination, ShellState};

use crate::design::{components, theme, theme::Palette};
use encryptdrive_desktop::preview::{
    DatasetSize, EntryId, ExplorerState, Modifier, OperationState, PreviewModel, PreviewScenario,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DialogKind {
    CreateFolder,
    Rename,
    MoveToTrash,
    PermanentDelete,
    Import,
    Export,
}

pub(super) struct PreviewState {
    pub model: PreviewModel,
    pub explorer: ExplorerState,
    pub scenario: PreviewScenario,
    pub dataset_size: DatasetSize,
    pub operation: OperationState,
    pub operation_ticks: u8,
    pub operation_label: String,
    pub notice: Option<String>,
    pub error: Option<String>,
    pub dialog: Option<DialogKind>,
    /// Scope of the open Trash/Delete dialog, computed once when it opens.
    pub pending_plan: Option<encryptdrive_desktop::preview::OperationPlan>,
    pub input_name: String,
    pub input_error: Option<String>,
    pub search_input: String,
    pub form_identity: String,
    pub form_secret: String,
    pub form_feedback: Option<String>,
    pub sample_vault: String,
    pub sample_account: String,
    pub lock_sample_on_exit: bool,
    pub warn_before_trash: bool,
}

impl PreviewState {
    fn new() -> Self {
        Self {
            model: PreviewModel::fixture(DatasetSize::OneHundred),
            explorer: ExplorerState::new(),
            scenario: PreviewScenario::Explorer,
            dataset_size: DatasetSize::OneHundred,
            operation: OperationState::Idle,
            operation_ticks: 0,
            operation_label: "Sample operation".to_owned(),
            notice: None,
            error: None,
            dialog: None,
            pending_plan: None,
            input_name: String::new(),
            input_error: None,
            search_input: String::new(),
            form_identity: String::new(),
            form_secret: String::new(),
            form_feedback: None,
            sample_vault: "Personal sample vault".to_owned(),
            sample_account: "Personal sample account".to_owned(),
            lock_sample_on_exit: false,
            warn_before_trash: false,
        }
    }
}

pub struct DesktopApp {
    pub(super) shell: ShellState,
    pub(super) preview: Option<PreviewState>,
    pub(super) settings_open: bool,
    pub(super) table_has_focus: bool,
    pub(super) table_page_rows: usize,
    pub(super) search_has_focus: bool,
    pub(super) focused_dialog: Option<DialogKind>,
    pub(super) focused_row_widget: Option<egui::Id>,
    pub(super) files_navigation_widget: Option<egui::Id>,
    pub(super) trash_navigation_widget: Option<egui::Id>,
    pub(super) dialog_invoker: Option<egui::Id>,
    pub(super) focus_return_target: Option<egui::Id>,
    pub(super) search_focus_requested: bool,
    pub(super) requested_row_focus: Option<EntryId>,
    pub(super) applied_appearance: Option<AppearancePreference>,
    pub(super) resolved_appearance: Option<Appearance>,
    pub(super) logo_texture: Option<egui::TextureHandle>,
    pub(super) window_fitted: bool,
}

impl DesktopApp {
    pub fn new(design_preview: bool) -> Self {
        Self {
            shell: ShellState::new(),
            preview: design_preview.then(PreviewState::new),
            settings_open: false,
            table_has_focus: false,
            table_page_rows: 10,
            search_has_focus: false,
            focused_dialog: None,
            focused_row_widget: None,
            files_navigation_widget: None,
            trash_navigation_widget: None,
            dialog_invoker: None,
            focus_return_target: None,
            search_focus_requested: false,
            requested_row_focus: None,
            applied_appearance: None,
            resolved_appearance: None,
            logo_texture: None,
            window_fitted: false,
        }
    }

    /// Starts with an explicit appearance instead of following the system (review builds).
    pub fn with_appearance(mut self, appearance: AppearancePreference) -> Self {
        self.shell.appearance = appearance;
        self
    }

    /// Starts with the Settings dialog open (review builds).
    pub fn with_settings_open(mut self, open: bool) -> Self {
        self.settings_open = open;
        self
    }

    /// Opens a specific preview scenario (review builds); ignored outside the preview.
    pub fn with_scenario_index(mut self, index: usize) -> Self {
        if let Some(scenario) = PreviewScenario::ALL.get(index).copied() {
            self.set_scenario(scenario);
        }
        self
    }

    pub(super) fn palette(&self) -> Palette {
        let appearance = self.resolved_appearance.unwrap_or(Appearance::Dark);
        Palette::from_tokens(self.shell.tokens(appearance))
    }

    pub(super) fn is_preview(&self) -> bool {
        self.preview.is_some()
    }

    pub(super) fn set_scenario(&mut self, scenario: PreviewScenario) {
        let Some(preview) = &mut self.preview else {
            return;
        };
        preview.scenario = scenario;
        preview.notice = None;
        preview.error = None;
        preview.dialog = None;
        preview.operation = OperationState::Idle;
        preview.operation_label = "Sample operation".to_owned();
        preview.explorer = ExplorerState::new();
        self.table_has_focus = false;
        self.search_has_focus = false;
        self.requested_row_focus = None;
        self.focused_row_widget = None;
        self.files_navigation_widget = None;
        self.trash_navigation_widget = None;
        self.dialog_invoker = None;
        self.focus_return_target = None;
        preview.search_input.clear();
        preview.form_identity.clear();
        preview.form_secret.clear();
        preview.form_feedback = None;
        match scenario {
            PreviewScenario::VaultSelection
            | PreviewScenario::CreateVault
            | PreviewScenario::UnlockVault
            | PreviewScenario::AccountSelection
            | PreviewScenario::AccountLogin
            | PreviewScenario::RegisterAccount
            | PreviewScenario::AccountSettings
            | PreviewScenario::VaultSettings => {}
            PreviewScenario::Trash => preview.explorer.set_destination(Destination::Trash),
            PreviewScenario::EmptyExplorer => preview.explorer.navigate_to(Some(EntryId(3))),
            PreviewScenario::FolderNavigation => {
                preview.explorer.navigate_to(Some(EntryId(1)));
                preview.explorer.navigate_to(Some(EntryId(101)));
            }
            PreviewScenario::SearchResults => {
                preview.search_input = "project".to_owned();
                preview.explorer.set_query("project");
            }
            PreviewScenario::Loading => {
                preview.operation = OperationState::running(8);
                preview.operation_ticks = 0;
                preview.operation_label = "Loading sample items".to_owned();
            }
            PreviewScenario::MultiSelection => {
                let ids = preview.explorer.visible_ids(&preview.model);
                preview
                    .explorer
                    .select_all_visible(&ids[..4.min(ids.len())]);
            }
            PreviewScenario::ImportProgress | PreviewScenario::Busy => {
                preview.operation = OperationState::running(12);
                preview.operation_ticks = 0;
                preview.operation_label = if scenario == PreviewScenario::ImportProgress {
                    "Importing sample files"
                } else {
                    "Sample operation in progress"
                }
                .to_owned();
            }
            PreviewScenario::ExportWarning => preview.dialog = Some(DialogKind::Export),
            PreviewScenario::PermanentDelete => {
                preview.explorer.set_destination(Destination::Trash);
                let ids = preview.explorer.visible_ids(&preview.model);
                if let Some(id) = ids.first() {
                    preview.explorer.select(
                        *id,
                        &ids,
                        encryptdrive_desktop::preview::Modifier::Plain,
                    );
                }
                preview.dialog = Some(DialogKind::PermanentDelete);
            }
            PreviewScenario::RecoveryNotice => {
                preview.notice = Some(
                    "Synthetic recovery example: some optional metadata could not be read. Review the report before continuing.".to_owned(),
                );
            }
            PreviewScenario::Error => {
                preview.operation = OperationState::Failed;
                preview.error = Some(
                    "The sample operation could not be completed. No data was changed.".to_owned(),
                );
            }
            PreviewScenario::Disabled => {
                preview.notice = Some("Vault creation, unlock, and file operations are represented only in this synthetic design preview.".to_owned());
            }
            PreviewScenario::Explorer => {}
        }
    }

    pub(super) fn apply_theme(&mut self, ctx: &egui::Context) {
        if self.applied_appearance != Some(self.shell.appearance) {
            theme::apply(ctx, self.shell.appearance);
            self.applied_appearance = Some(self.shell.appearance);
        }
        self.resolved_appearance = Some(match ctx.theme() {
            egui::Theme::Light => Appearance::Light,
            egui::Theme::Dark => Appearance::Dark,
        });
    }

    pub(super) fn handle_global_keys(&mut self, ctx: &egui::Context) {
        let input_is_editing = input_is_editing(ctx);
        let busy = self
            .preview
            .as_ref()
            .is_some_and(|preview| matches!(preview.operation, OperationState::Running { .. }));
        let modal_open = self
            .preview
            .as_ref()
            .is_some_and(|preview| preview.dialog.is_some());
        let popup_open = egui::Popup::is_any_open(ctx);
        let focused_widget = ctx.memory(|memory| memory.focused());
        let commands_enabled = !input_is_editing && !busy && !modal_open && !popup_open;
        let table_commands_enabled = commands_enabled && self.table_has_focus;
        let events = ctx.input_mut(|input| {
            let command = input.modifiers.command;
            let control = input.modifiers.ctrl || command;
            let alt = input.modifiers.alt;
            let shift = input.modifiers.shift;
            (
                table_commands_enabled
                    && input.consume_shortcut(&egui::KeyboardShortcut::new(
                        egui::Modifiers::COMMAND,
                        egui::Key::A,
                    )),
                commands_enabled
                    && input.consume_shortcut(&egui::KeyboardShortcut::new(
                        egui::Modifiers::COMMAND,
                        egui::Key::F,
                    )),
                table_commands_enabled
                    && components::key_pressed_without_repeat(input, egui::Key::F2),
                table_commands_enabled
                    && !shift
                    && components::key_pressed_without_repeat(input, egui::Key::Delete),
                table_commands_enabled
                    && shift
                    && components::key_pressed_without_repeat(input, egui::Key::Delete),
                !popup_open && components::key_pressed_without_repeat(input, egui::Key::Escape),
                table_commands_enabled
                    .then(|| row_navigation(input))
                    .flatten(),
                commands_enabled
                    && alt
                    && components::key_pressed_without_repeat(input, egui::Key::ArrowLeft),
                commands_enabled
                    && alt
                    && components::key_pressed_without_repeat(input, egui::Key::ArrowUp),
                table_commands_enabled
                    && components::key_pressed_without_repeat(input, egui::Key::Enter),
                control,
                shift,
            )
        });
        let Some(preview) = &mut self.preview else {
            if events.5 {
                self.settings_open = false;
            }
            return;
        };
        let visible = preview.explorer.visible_ids(&preview.model);
        let mut requested_row_focus = None;
        if events.0 {
            preview.explorer.select_all_visible(&visible);
        }
        if events.1 {
            self.search_focus_requested = true;
        }
        if events.2 && preview.explorer.destination() == Destination::Files {
            let selected = preview.explorer.selected_ids();
            if let Some(target) = super::explorer::rename_target(&selected) {
                self.dialog_invoker = focused_widget;
                preview.input_name = preview
                    .model
                    .entry(target)
                    .map_or_else(String::new, |entry| entry.name.clone());
                preview.input_error = None;
                preview.dialog = Some(DialogKind::Rename);
            }
        }
        if !preview.explorer.selected_ids().is_empty() {
            if events.3 {
                self.dialog_invoker = focused_widget;
                preview.dialog = delete_dialog_for(preview.explorer.destination(), false);
            } else if events.4 {
                self.dialog_invoker = focused_widget;
                preview.dialog = delete_dialog_for(preview.explorer.destination(), true);
            }
        }
        if events.5 {
            if preview.dialog.is_some() {
                preview.dialog = None;
            } else if self.search_has_focus {
                preview.search_input.clear();
                preview.explorer.set_query("");
            } else if !input_is_editing {
                let focused_index = self
                    .table_has_focus
                    .then(|| {
                        preview
                            .explorer
                            .focused_id()
                            .and_then(|focused| visible.iter().position(|id| *id == focused))
                    })
                    .flatten();
                preview.explorer.clear_selection();
                if let Some(index) = focused_index {
                    preview.explorer.move_focus_to_index(index, &visible, None);
                }
            }
        }
        if let Some(navigation) = events.6 {
            if navigation == RowNavigation::ToggleSelection {
                preview.explorer.toggle_focused_selection(&visible);
            } else if let Some(index) = navigation_target_index(
                navigation,
                preview
                    .explorer
                    .focused_id()
                    .and_then(|focused| visible.iter().position(|id| *id == focused)),
                visible.len(),
                self.table_page_rows,
            ) {
                preview.explorer.move_focus_to_index(
                    index,
                    &visible,
                    arrow_move_modifier(events.10, events.11),
                );
            }
            requested_row_focus = preview.explorer.focused_id();
        }
        if events.7 {
            preview.explorer.navigate_back();
            preview.search_input.clear();
        }
        if events.8
            && let Some(folder) = preview.explorer.current_folder()
        {
            let parent = preview.model.entry(folder).and_then(|item| item.parent_id);
            preview.explorer.navigate_to(parent);
            preview.search_input.clear();
        }
        if events.9 {
            if preview.explorer.destination() == Destination::Files {
                match preview.explorer.focused_id() {
                    Some(focused)
                        if preview.model.entry(focused).is_some_and(|item| {
                            item.kind == encryptdrive_desktop::preview::EntryKind::Folder
                        }) =>
                    {
                        preview.explorer.navigate_to(Some(focused));
                        preview.search_input.clear();
                    }
                    Some(focused) => {
                        preview.explorer.select(focused, &visible, Modifier::Plain);
                        self.dialog_invoker = focused_widget;
                        preview.dialog = Some(DialogKind::Export);
                    }
                    None => {}
                }
            } else {
                preview.notice = Some(
                    "Restore this sample item to Files before opening or exporting it.".to_owned(),
                );
            }
        }
        if requested_row_focus.is_some() {
            self.requested_row_focus = requested_row_focus;
        }
    }

    fn render_header(&mut self, ui: &mut egui::Ui, palette: Palette) {
        if self.logo_texture.is_none() {
            self.logo_texture = Some(components::load_brand_texture(ui.ctx()));
        }
        let vault_status = if self.preview.is_some() {
            "Sample vault"
        } else {
            "No vault open"
        };
        if components::app_header(
            ui,
            self.logo_texture
                .as_ref()
                .expect("loaded immediately above"),
            palette,
            vault_status,
        ) {
            self.settings_open = true;
        }
    }

    fn render_settings(&mut self, ctx: &egui::Context, palette: Palette) {
        if !self.settings_open {
            return;
        }
        let note = if self.is_preview() {
            "This preference lasts for this preview session."
        } else {
            "This preference lasts for the current session."
        };
        self.settings_open =
            components::settings_dialog(ctx, &mut self.shell.appearance, note, palette);
    }
}

impl eframe::App for DesktopApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        crate::design::work_area::fit_window_to_monitor(&ctx, &mut self.window_fitted);
        self.apply_theme(&ctx);
        self.handle_global_keys(&ctx);
        let palette = self.palette();
        self.render_header(ui, palette);
        components::status_bar_panel(palette).show(ui, |ui| self.render_status_bar(ui, palette));
        let window_width = ui.max_rect().width();
        components::sidebar_panel(palette, window_width)
            .show(ui, |ui| self.render_navigation(ui, palette));

        egui::CentralPanel::default()
            .frame(components::central_canvas_frame(palette))
            .show(ui, |ui| {
                components::content_frame(palette).show(ui, |ui| {
                    ui.set_min_size(ui.available_size());
                    if let Some(preview) = self.preview.as_ref() {
                        match preview.scenario {
                            PreviewScenario::VaultSelection
                            | PreviewScenario::CreateVault
                            | PreviewScenario::UnlockVault
                            | PreviewScenario::AccountSelection
                            | PreviewScenario::AccountLogin
                            | PreviewScenario::RegisterAccount
                            | PreviewScenario::AccountSettings
                            | PreviewScenario::VaultSettings => {
                                self.render_auth_screen(ui, palette)
                            }
                            _ => self.render_explorer(ui, palette),
                        }
                    } else {
                        self.render_default_empty(ui, palette);
                    }
                });
            });

        self.render_settings(&ctx, palette);
        self.render_dialogs(&ctx, palette);
    }
}

/// Explorer-style size text: bytes below 1 KB, whole KB (rounded up) below 1 MB, then MB.
pub(super) fn fmt_size(size: Option<u64>) -> String {
    let Some(size) = size else {
        return String::new();
    };
    const KIB: u64 = 1024;
    if size < KIB {
        format!("{size} bytes")
    } else if size < KIB * KIB {
        let kib = size.div_ceil(KIB) as usize;
        format!("{} KB", super::explorer::group_thousands(kib))
    } else {
        format!("{:.1} MB", size as f64 / (KIB * KIB) as f64)
    }
}
fn arrow_move_modifier(control: bool, shift: bool) -> Option<Modifier> {
    if shift {
        Some(Modifier::Shift)
    } else if control {
        None
    } else {
        Some(Modifier::Plain)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RowNavigation {
    Move(isize),
    Home,
    End,
    Page(isize),
    ToggleSelection,
}

fn row_navigation(input: &egui::InputState) -> Option<RowNavigation> {
    row_navigation_by_key(|key| input.key_pressed(key))
}

fn row_navigation_by_key(mut is_pressed: impl FnMut(egui::Key) -> bool) -> Option<RowNavigation> {
    [
        (egui::Key::ArrowUp, RowNavigation::Move(-1)),
        (egui::Key::ArrowDown, RowNavigation::Move(1)),
        (egui::Key::Home, RowNavigation::Home),
        (egui::Key::End, RowNavigation::End),
        (egui::Key::PageUp, RowNavigation::Page(-1)),
        (egui::Key::PageDown, RowNavigation::Page(1)),
        (egui::Key::Space, RowNavigation::ToggleSelection),
    ]
    .into_iter()
    .find_map(|(key, command)| is_pressed(key).then_some(command))
}

fn navigation_target_index(
    navigation: RowNavigation,
    current: Option<usize>,
    row_count: usize,
    page_rows: usize,
) -> Option<usize> {
    if row_count == 0 || navigation == RowNavigation::ToggleSelection {
        return None;
    }
    let last = row_count - 1;
    Some(match navigation {
        RowNavigation::Move(delta) => current.map_or(0, |index| {
            (index as isize + delta).clamp(0, last as isize) as usize
        }),
        RowNavigation::Home => 0,
        RowNavigation::End => last,
        RowNavigation::Page(delta) => {
            let index = current.unwrap_or(0);
            let distance = page_rows.max(1) as isize * delta;
            (index as isize + distance).clamp(0, last as isize) as usize
        }
        RowNavigation::ToggleSelection => unreachable!("toggle has no row target"),
    })
}

fn delete_dialog_for(destination: Destination, shift_pressed: bool) -> Option<DialogKind> {
    match (destination, shift_pressed) {
        (Destination::Files, false) => Some(DialogKind::MoveToTrash),
        (Destination::Trash, true) => Some(DialogKind::PermanentDelete),
        (Destination::Files, true) | (Destination::Trash, false) => None,
    }
}

fn input_is_editing(ctx: &egui::Context) -> bool {
    ctx.text_edit_focused()
}

#[cfg(test)]
mod tests {
    use super::{
        DialogKind, RowNavigation, arrow_move_modifier, delete_dialog_for, input_is_editing,
        navigation_target_index, row_navigation_by_key,
    };
    use eframe::egui;
    use encryptdrive_desktop::{
        Appearance, AppearancePreference, Destination,
        preview::{DatasetSize, EntryId, Modifier, PreviewModel, PreviewScenario},
    };
    use std::time::{Duration, Instant};

    fn render_complete_app_frame(
        app: &mut super::DesktopApp,
        context: &egui::Context,
        events: Vec<egui::Event>,
        time: f64,
    ) -> (Duration, egui::FullOutput) {
        let mut frame = eframe::Frame::_new_kittest();
        let started = Instant::now();
        let output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1180.0, 792.0),
                )),
                time: Some(time),
                events,
                ..Default::default()
            },
            |ui| <super::DesktopApp as eframe::App>::ui(app, ui, &mut frame),
        );
        let mut output = output;
        output.textures_delta.clear();
        (started.elapsed(), output)
    }

    fn accessible_center(output: &egui::FullOutput, label: &str) -> Option<egui::Pos2> {
        output
            .platform_output
            .accesskit_update
            .as_ref()?
            .nodes
            .iter()
            .find_map(|(_, node)| {
                (node.label() == Some(label))
                    .then(|| node.bounds())
                    .flatten()
            })
            .map(|bounds| {
                egui::pos2(
                    (bounds.x0 + bounds.x1) as f32 / 2.0,
                    (bounds.y0 + bounds.y1) as f32 / 2.0,
                )
            })
    }

    fn accessible_center_containing(output: &egui::FullOutput, text: &str) -> Option<egui::Pos2> {
        output
            .platform_output
            .accesskit_update
            .as_ref()?
            .nodes
            .iter()
            .find_map(|(_, node)| {
                node.label()
                    .is_some_and(|label| label.contains(text))
                    .then(|| node.bounds())
                    .flatten()
            })
            .map(|bounds| {
                egui::pos2(
                    (bounds.x0 + bounds.x1) as f32 / 2.0,
                    (bounds.y0 + bounds.y1) as f32 / 2.0,
                )
            })
    }

    fn accessible_disabled(output: &egui::FullOutput, label: &str) -> Option<bool> {
        output
            .platform_output
            .accesskit_update
            .as_ref()?
            .nodes
            .iter()
            .find_map(|(_, node)| (node.label() == Some(label)).then(|| node.is_disabled()))
    }

    fn accessible_labels(output: &egui::FullOutput) -> Vec<String> {
        output
            .platform_output
            .accesskit_update
            .as_ref()
            .map(|update| {
                update
                    .nodes
                    .iter()
                    .filter_map(|(_, node)| node.label().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn send_app_key(
        app: &mut super::DesktopApp,
        context: &egui::Context,
        key: egui::Key,
        modifiers: egui::Modifiers,
        time: f64,
    ) -> Duration {
        let (duration, _) = render_complete_app_frame(
            app,
            context,
            vec![
                egui::Event::ModifiersChanged(modifiers),
                egui::Event::Key {
                    key,
                    physical_key: Some(key),
                    pressed: true,
                    repeat: false,
                    modifiers,
                },
            ],
            time,
        );
        duration
    }

    fn send_repeated_app_key(
        app: &mut super::DesktopApp,
        context: &egui::Context,
        key: egui::Key,
        time: f64,
    ) -> Duration {
        let modifiers = egui::Modifiers::default();
        let (duration, _) = render_complete_app_frame(
            app,
            context,
            vec![egui::Event::Key {
                key,
                physical_key: Some(key),
                pressed: true,
                repeat: true,
                modifiers,
            }],
            time,
        );
        duration
    }

    fn click_app_point(
        app: &mut super::DesktopApp,
        context: &egui::Context,
        position: egui::Pos2,
        time: f64,
    ) {
        for (pressed, offset) in [(true, 0.0), (false, 0.01)] {
            let (_, mut output) = render_complete_app_frame(
                app,
                context,
                vec![
                    egui::Event::PointerMoved(position),
                    egui::Event::PointerButton {
                        pos: position,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::default(),
                    },
                ],
                time + offset,
            );
            output.textures_delta.clear();
        }
    }

    fn run_key_event(
        app: &mut super::DesktopApp,
        context: &egui::Context,
        key: egui::Key,
        modifiers: egui::Modifiers,
    ) {
        let mut events = vec![egui::Event::ModifiersChanged(modifiers)];
        events.push(egui::Event::Key {
            key,
            physical_key: Some(key),
            pressed: true,
            repeat: false,
            modifiers,
        });
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800.0, 600.0),
                )),
                events,
                ..Default::default()
            },
            |ui| app.handle_global_keys(ui.ctx()),
        );
        output.textures_delta.clear();
    }

    #[test]
    fn keyboard_arrows_select_by_default_extend_with_shift_and_preserve_with_control() {
        assert_eq!(arrow_move_modifier(false, false), Some(Modifier::Plain));
        assert_eq!(arrow_move_modifier(false, true), Some(Modifier::Shift));
        assert_eq!(arrow_move_modifier(true, false), None);
    }

    #[test]
    fn keyboard_home_end_and_page_navigation_stays_inside_the_visible_results() {
        assert_eq!(
            navigation_target_index(RowNavigation::Home, Some(19), 20, 8),
            Some(0)
        );
        assert_eq!(
            navigation_target_index(RowNavigation::End, Some(1), 20, 8),
            Some(19)
        );
        assert_eq!(
            navigation_target_index(RowNavigation::Page(1), Some(15), 20, 8),
            Some(19)
        );
        assert_eq!(
            navigation_target_index(RowNavigation::Page(-1), Some(3), 20, 8),
            Some(0)
        );
        assert_eq!(
            navigation_target_index(RowNavigation::Move(1), None, 0, 8),
            None
        );
    }

    #[test]
    fn explorer_maps_home_end_page_and_space_to_navigation_commands() {
        for (key, expected) in [
            (egui::Key::Home, RowNavigation::Home),
            (egui::Key::End, RowNavigation::End),
            (egui::Key::PageUp, RowNavigation::Page(-1)),
            (egui::Key::PageDown, RowNavigation::Page(1)),
            (egui::Key::Space, RowNavigation::ToggleSelection),
        ] {
            assert_eq!(
                row_navigation_by_key(|pressed| pressed == key),
                Some(expected)
            );
        }
        assert_eq!(
            row_navigation_by_key(|pressed| pressed == egui::Key::ArrowLeft),
            None,
            "Left has no list command and must not transfer egui focus"
        );
        assert_eq!(
            row_navigation_by_key(|pressed| pressed == egui::Key::ArrowRight),
            None,
            "Right has no list command and must not transfer egui focus"
        );
    }

    #[test]
    fn focused_non_text_widget_does_not_block_explorer_shortcuts() {
        let context = egui::Context::default();
        let mut app = super::DesktopApp::new(true);
        app.table_has_focus = true;
        let preview = app.preview.as_mut().expect("preview enabled");
        preview.explorer.set_query("project item 00003");
        let visible = preview.explorer.visible_ids(&preview.model).to_vec();
        assert_eq!(visible.len(), 1);
        context.memory_mut(|memory| {
            memory.request_focus(egui::Id::unique("focused-table"));
        });

        assert!(context.egui_wants_keyboard_input());
        assert!(!input_is_editing(&context));
        run_key_event(
            &mut app,
            &context,
            egui::Key::A,
            egui::Modifiers {
                ctrl: true,
                command: true,
                ..Default::default()
            },
        );

        assert_eq!(
            app.preview.as_ref().unwrap().explorer.selected_ids(),
            visible
        );
    }

    #[test]
    fn focused_text_edit_keeps_global_shortcuts_out_of_its_input() {
        let mut value = String::new();
        let mut editing = false;
        egui::__run_test_ui(|ui| {
            let response = ui.text_edit_singleline(&mut value);
            response.request_focus();
            editing = input_is_editing(ui.ctx());
        });

        assert!(editing);
    }

    #[test]
    fn delete_key_routes_to_trash_or_permanent_confirmation_by_destination() {
        assert_eq!(
            delete_dialog_for(Destination::Files, false),
            Some(DialogKind::MoveToTrash)
        );
        assert_eq!(
            delete_dialog_for(Destination::Trash, true),
            Some(DialogKind::PermanentDelete)
        );
        assert_eq!(delete_dialog_for(Destination::Trash, false), None);
        assert_eq!(delete_dialog_for(Destination::Files, true), None);
    }

    #[test]
    fn delete_does_not_target_an_item_hidden_by_a_changed_query() {
        let mut app = super::DesktopApp::new(true);
        let preview = app.preview.as_mut().expect("preview enabled");
        let all_visible = preview.explorer.visible_ids(&preview.model);
        preview
            .explorer
            .select(EntryId(4), &all_visible, Modifier::Plain);
        preview.explorer.set_query("日本語");
        assert!(preview.explorer.selected_ids().is_empty());
        app.table_has_focus = true;
        let context = egui::Context::default();

        run_key_event(
            &mut app,
            &context,
            egui::Key::Delete,
            egui::Modifiers::default(),
        );

        assert_eq!(app.preview.as_ref().unwrap().dialog, None);
        assert!(
            app.preview
                .as_ref()
                .unwrap()
                .model
                .entry(EntryId(4))
                .is_some()
        );
    }

    #[test]
    fn ctrl_f_is_routed_to_search_when_the_table_has_focus() {
        let mut app = super::DesktopApp::new(true);
        app.table_has_focus = true;
        let context = egui::Context::default();

        run_key_event(
            &mut app,
            &context,
            egui::Key::F,
            egui::Modifiers {
                ctrl: true,
                command: true,
                ..Default::default()
            },
        );

        assert!(app.search_focus_requested);
    }

    #[test]
    fn table_arrow_home_end_page_and_space_events_update_stable_selection() {
        let context = egui::Context::default();
        let mut app = super::DesktopApp::new(true);
        app.table_has_focus = true;
        let visible = {
            let preview = app.preview.as_mut().unwrap();
            preview.explorer.visible_ids(&preview.model).to_vec()
        };

        run_key_event(
            &mut app,
            &context,
            egui::Key::ArrowDown,
            egui::Modifiers::default(),
        );
        assert_eq!(
            app.preview.as_ref().unwrap().explorer.focused_id(),
            visible.first().copied()
        );
        run_key_event(
            &mut app,
            &context,
            egui::Key::ArrowDown,
            egui::Modifiers::default(),
        );
        assert_eq!(
            app.preview.as_ref().unwrap().explorer.focused_id(),
            visible.get(1).copied()
        );
        run_key_event(
            &mut app,
            &context,
            egui::Key::ArrowUp,
            egui::Modifiers::default(),
        );
        assert_eq!(
            app.preview.as_ref().unwrap().explorer.focused_id(),
            visible.first().copied()
        );
        run_key_event(
            &mut app,
            &context,
            egui::Key::End,
            egui::Modifiers::default(),
        );
        assert_eq!(
            app.preview.as_ref().unwrap().explorer.focused_id(),
            visible.last().copied()
        );
        run_key_event(
            &mut app,
            &context,
            egui::Key::Home,
            egui::Modifiers::default(),
        );
        assert_eq!(
            app.preview.as_ref().unwrap().explorer.focused_id(),
            visible.first().copied()
        );
        run_key_event(
            &mut app,
            &context,
            egui::Key::PageDown,
            egui::Modifiers::default(),
        );
        assert!(
            app.preview
                .as_ref()
                .unwrap()
                .explorer
                .focused_id()
                .is_some()
        );
        run_key_event(
            &mut app,
            &context,
            egui::Key::Space,
            egui::Modifiers::default(),
        );
        assert!(
            app.preview
                .as_ref()
                .unwrap()
                .explorer
                .selected_ids()
                .is_empty()
        );
        assert!(app.table_has_focus);
    }

    #[test]
    fn left_right_and_escape_keep_keyboard_focus_in_the_table() {
        let context = egui::Context::default();
        let mut app = super::DesktopApp::new(true);
        app.table_has_focus = true;
        let visible = {
            let preview = app.preview.as_mut().unwrap();
            preview.explorer.visible_ids(&preview.model).to_vec()
        };
        app.preview
            .as_mut()
            .unwrap()
            .explorer
            .select(EntryId(4), &visible, Modifier::Plain);
        let focused = app.preview.as_ref().unwrap().explorer.focused_id();

        for key in [egui::Key::ArrowLeft, egui::Key::ArrowRight] {
            run_key_event(&mut app, &context, key, egui::Modifiers::default());
            assert_eq!(
                app.preview.as_ref().unwrap().explorer.focused_id(),
                focused,
                "{key:?} must not move row focus"
            );
            assert!(app.table_has_focus, "{key:?} must not leave the table");
        }

        run_key_event(
            &mut app,
            &context,
            egui::Key::Escape,
            egui::Modifiers::default(),
        );
        let explorer = &app.preview.as_ref().unwrap().explorer;
        assert!(explorer.selected_ids().is_empty());
        assert_eq!(explorer.focused_id(), focused);
        assert!(
            app.table_has_focus,
            "Escape clears selection and retains table focus"
        );
    }

    #[test]
    fn f2_and_delete_route_to_the_expected_sensitive_dialogs() {
        let context = egui::Context::default();
        let mut app = super::DesktopApp::new(true);
        app.table_has_focus = true;
        let visible = {
            let preview = app.preview.as_mut().unwrap();
            preview.explorer.visible_ids(&preview.model).to_vec()
        };
        app.preview
            .as_mut()
            .unwrap()
            .explorer
            .select(EntryId(4), &visible, Modifier::Plain);

        run_key_event(
            &mut app,
            &context,
            egui::Key::F2,
            egui::Modifiers::default(),
        );
        assert_eq!(
            app.preview.as_ref().unwrap().dialog,
            Some(DialogKind::Rename)
        );
        app.preview.as_mut().unwrap().dialog = None;
        run_key_event(
            &mut app,
            &context,
            egui::Key::Delete,
            egui::Modifiers::default(),
        );
        assert_eq!(
            app.preview.as_ref().unwrap().dialog,
            Some(DialogKind::MoveToTrash)
        );
        assert!(
            app.preview
                .as_ref()
                .unwrap()
                .model
                .entry(EntryId(4))
                .is_some()
        );
    }

    #[test]
    fn sample_account_vault_and_settings_choices_survive_screen_changes() {
        let mut app = super::DesktopApp::new(true);
        let preview = app.preview.as_mut().expect("preview is enabled");
        preview.sample_vault = "Archive sample vault".to_owned();
        preview.sample_account = "Work sample account".to_owned();
        preview.lock_sample_on_exit = true;
        preview.warn_before_trash = true;

        app.set_scenario(PreviewScenario::UnlockVault);
        app.set_scenario(PreviewScenario::AccountLogin);
        app.set_scenario(PreviewScenario::AccountSettings);
        app.set_scenario(PreviewScenario::VaultSettings);

        let preview = app.preview.as_ref().expect("preview remains enabled");
        assert_eq!(preview.sample_vault, "Archive sample vault");
        assert_eq!(preview.sample_account, "Work sample account");
        assert!(preview.lock_sample_on_exit);
        assert!(preview.warn_before_trash);
    }

    #[test]
    fn select_all_on_ten_thousand_entries_confirms_and_runs_exactly_10_005_entries() {
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut app = super::DesktopApp::new(true);
        let visible = {
            let preview = app.preview.as_mut().expect("preview is enabled");
            preview.dataset_size = DatasetSize::TenThousand;
            preview.model = PreviewModel::fixture(DatasetSize::TenThousand);
            preview.explorer.visible_ids(&preview.model).to_vec()
        };
        assert_eq!(
            visible.len(),
            10_000,
            "every root row, most of them off screen"
        );
        app.table_has_focus = true;
        context.memory_mut(|memory| memory.request_focus(egui::Id::unique("focused-table")));
        run_key_event(
            &mut app,
            &context,
            egui::Key::A,
            egui::Modifiers {
                ctrl: true,
                command: true,
                ..Default::default()
            },
        );
        let selected: std::collections::BTreeSet<EntryId> = app
            .preview
            .as_ref()
            .unwrap()
            .explorer
            .selected_ids()
            .into_iter()
            .collect();
        assert_eq!(selected, visible.iter().copied().collect());

        app.preview.as_mut().unwrap().dialog = Some(DialogKind::MoveToTrash);
        // The first frame sizes the modal invisibly; the second shows it.
        render_complete_app_frame(&mut app, &context, Vec::new(), 0.1);
        let (_, output) = render_complete_app_frame(&mut app, &context, Vec::new(), 0.12);

        // The exact affected set: 10,000 roots plus the five entries inside Documents
        // (Projects, Shared notes, and the note inside Projects) and Photos (two files).
        let mut expected: std::collections::BTreeSet<EntryId> = visible.iter().copied().collect();
        expected.extend([101, 102, 103, 201, 202].map(EntryId));
        let plan = app
            .preview
            .as_ref()
            .unwrap()
            .pending_plan
            .clone()
            .expect("plan made");
        assert_eq!(plan.selected_count(), 10_000);
        assert_eq!(plan.affected_count(), 10_005);
        assert_eq!(plan.affected(), &expected);

        let texts = accessible_labels_and_values(&output);
        let shown = "10,005 items will be moved to Trash: the 10,000 items you selected and 5 items inside them.";
        assert!(texts.iter().any(|text| text == shown), "{shown:?} missing");
        assert!(
            !texts
                .iter()
                .any(|text| text.starts_with("10,000 items will be moved")),
            "the confirmation must not understate the scope"
        );

        // Confirm with a real click on the dialog's confirm button (the bottom-most
        // "Move to Trash" button; the command bar has one too).
        let confirm = accessible_buttons_named(&output, "Move to Trash")
            .into_iter()
            .max_by(|a, b| a.y.total_cmp(&b.y))
            .expect("dialog confirm button");
        click_app_point(&mut app, &context, confirm, 0.2);

        let preview = app.preview.as_ref().unwrap();
        assert_eq!(preview.dialog, None);
        assert_eq!(
            preview.notice.as_deref(),
            Some("Moved 10,005 items to Trash.")
        );
        let trashed: std::collections::BTreeSet<EntryId> = preview
            .model
            .entries()
            .filter(|entry| entry.deleted_at.as_deref() == Some("2026-10-19 10:00"))
            .map(|entry| entry.id)
            .collect();
        assert_eq!(
            trashed, expected,
            "the operation ran exactly the confirmed set"
        );
        assert!(preview.model.list_ids(Destination::Files, None).is_empty());
    }

    fn accessible_labels_and_values(output: &egui::FullOutput) -> Vec<String> {
        output
            .platform_output
            .accesskit_update
            .as_ref()
            .map(|update| {
                update
                    .nodes
                    .iter()
                    .flat_map(|(_, node)| [node.label(), node.value()])
                    .flatten()
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default()
    }

    fn accessible_buttons_named(output: &egui::FullOutput, name: &str) -> Vec<egui::Pos2> {
        output
            .platform_output
            .accesskit_update
            .as_ref()
            .map(|update| {
                update
                    .nodes
                    .iter()
                    .filter(|(_, node)| {
                        node.role() == egui::accesskit::Role::Button && node.label() == Some(name)
                    })
                    .filter_map(|(_, node)| node.bounds())
                    .map(|bounds| {
                        egui::pos2(
                            ((bounds.x0 + bounds.x1) / 2.0) as f32,
                            ((bounds.y0 + bounds.y1) / 2.0) as f32,
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    #[test]
    fn complete_preview_app_frames_remain_bounded_with_ten_thousand_synthetic_entries() {
        let context = egui::Context::default();
        let mut app = super::DesktopApp::new(true);
        {
            let preview = app.preview.as_mut().expect("preview is enabled");
            preview.dataset_size = DatasetSize::TenThousand;
            preview.model = PreviewModel::fixture(DatasetSize::TenThousand);
        }

        let mut durations = Vec::with_capacity(24);
        for index in 0..24 {
            let (duration, _) =
                render_complete_app_frame(&mut app, &context, Vec::new(), index as f64 / 60.0);
            durations.push(duration);
        }
        durations.sort_unstable();
        let median = durations[durations.len() / 2];
        let maximum = *durations.last().expect("at least one frame was measured");
        eprintln!(
            "headless full-app UI frame timing with 10,000 synthetic entries: median={:.2}ms max={:.2}ms",
            median.as_secs_f64() * 1_000.0,
            maximum.as_secs_f64() * 1_000.0,
        );
        assert!(
            maximum < Duration::from_secs(2),
            "an ordinary full-app egui frame exceeded the 2s responsiveness bound: {maximum:?}"
        );
    }

    #[test]
    fn synthetic_full_app_click_opens_settings_and_switches_theme_without_blocking() {
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut app = super::DesktopApp::new(true);
        let (_, output) = render_complete_app_frame(&mut app, &context, Vec::new(), 0.0);
        let settings = accessible_center(&output, "Settings");
        let settings = settings
            .expect("the full app frame exposes the Settings control in its AccessKit tree");

        click_app_point(&mut app, &context, settings, 0.02);
        assert!(app.settings_open, "clicking Settings must open its window");

        let (_, output) = render_complete_app_frame(&mut app, &context, Vec::new(), 0.04);
        let light = accessible_center(&output, "Light");
        let light = light.expect("the open Settings window exposes the Light choice");
        click_app_point(&mut app, &context, light, 0.06);
        assert_eq!(app.shell.appearance, AppearancePreference::Light);

        let (duration, _) = render_complete_app_frame(&mut app, &context, Vec::new(), 0.08);
        assert_eq!(app.resolved_appearance, Some(Appearance::Light));
        assert!(duration < Duration::from_secs(2));
    }

    #[test]
    fn synthetic_full_app_click_opens_and_validates_folder_dialog() {
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut app = super::DesktopApp::new(true);
        let (_, output) = render_complete_app_frame(&mut app, &context, Vec::new(), 0.0);
        let create_folder =
            accessible_center(&output, "New folder").expect("the file toolbar exposes New folder");

        click_app_point(&mut app, &context, create_folder, 0.02);
        assert_eq!(
            app.preview.as_ref().unwrap().dialog,
            Some(DialogKind::CreateFolder)
        );

        let enter_duration = send_app_key(
            &mut app,
            &context,
            egui::Key::Enter,
            egui::Modifiers::default(),
            0.04,
        );
        let preview = app.preview.as_ref().unwrap();
        assert_eq!(preview.dialog, Some(DialogKind::CreateFolder));
        assert_eq!(preview.input_error.as_deref(), Some("Enter a name."));
        assert!(enter_duration < Duration::from_secs(2));

        send_app_key(
            &mut app,
            &context,
            egui::Key::Escape,
            egui::Modifiers::default(),
            0.06,
        );
        assert_eq!(app.preview.as_ref().unwrap().dialog, None);
    }

    /// Focusable or clickable controls that are not fully inside the window, keyed by role
    /// and name. Table rows are skipped: they live in the table's own scroll area.
    fn offscreen_controls(output: &egui::FullOutput, screen: egui::Rect) -> Vec<String> {
        use egui::accesskit::{Action, Role};
        output
            .platform_output
            .accesskit_update
            .as_ref()
            .map(|update| {
                update
                    .nodes
                    .iter()
                    .filter(|(_, node)| {
                        node.role() != Role::ListItem
                            && (node.supports_action(Action::Focus)
                                || node.supports_action(Action::Click))
                    })
                    .filter_map(|(_, node)| {
                        let bounds = node.bounds()?;
                        let rect = egui::Rect::from_min_max(
                            egui::pos2(bounds.x0 as f32, bounds.y0 as f32),
                            egui::pos2(bounds.x1 as f32, bounds.y1 as f32),
                        );
                        (!screen.expand(0.5).contains_rect(rect))
                            .then(|| format!("{:?} {:?}", node.role(), node.label()))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Controls that stay off-screen both at the top of the page and after scrolling to its
    /// end, i.e. controls the user cannot reach.
    fn unreachable_controls(app: &mut super::DesktopApp, size: egui::Vec2) -> Vec<String> {
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut frame = eframe::Frame::_new_kittest();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, size);
        let centre = screen.center();
        let mut run = |time: f64, events: Vec<egui::Event>| {
            let mut output = context.run_ui(
                egui::RawInput {
                    screen_rect: Some(screen),
                    time: Some(time),
                    events,
                    ..Default::default()
                },
                |ui| <super::DesktopApp as eframe::App>::ui(app, ui, &mut frame),
            );
            output.textures_delta.clear();
            output
        };
        let mut output = run(0.0, Vec::new());
        for step in 1..3 {
            output = run(step as f64 * 0.02, Vec::new());
        }
        let at_top = offscreen_controls(&output, screen);
        for step in 3..8 {
            output = run(
                step as f64 * 0.02,
                vec![
                    egui::Event::PointerMoved(centre),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0.0, -2000.0),
                        phase: egui::TouchPhase::Move,
                        source: egui::MouseWheelSource::Unknown,
                        modifiers: egui::Modifiers::default(),
                    },
                ],
            );
        }
        let at_bottom = offscreen_controls(&output, screen);
        at_top
            .into_iter()
            .filter(|control| at_bottom.contains(control))
            .collect()
    }

    #[test]
    fn every_control_is_reachable_at_the_smallest_supported_work_areas() {
        // Inner sizes the startup policy produces for 1920×1080 at 200 % (960×474) and
        // 1280×720 at 150 % (853×410), both with a bottom taskbar.
        let mut failures = Vec::new();
        for size in [egui::vec2(960.0, 474.0), egui::vec2(853.0, 410.0)] {
            for (index, scenario) in PreviewScenario::ALL.iter().enumerate() {
                let mut app = super::DesktopApp::new(true).with_scenario_index(index);
                let unreachable = unreachable_controls(&mut app, size);
                if !unreachable.is_empty() {
                    failures.push(format!("{size:?} {scenario:?}: {unreachable:?}"));
                }
            }
            for dialog in [
                DialogKind::CreateFolder,
                DialogKind::Rename,
                DialogKind::MoveToTrash,
                DialogKind::PermanentDelete,
                DialogKind::Import,
                DialogKind::Export,
            ] {
                let mut app = super::DesktopApp::new(true);
                {
                    let preview = app.preview.as_mut().unwrap();
                    // Documents: a folder with descendants, so delete dialogs show the
                    // longest scope wording.
                    preview.explorer.select_all_visible(&[EntryId(1)]);
                    preview.dialog = Some(dialog);
                }
                let unreachable = unreachable_controls(&mut app, size);
                if !unreachable.is_empty() {
                    failures.push(format!("{size:?} {dialog:?}: {unreachable:?}"));
                }
            }
            let mut app = super::DesktopApp::new(true).with_settings_open(true);
            let unreachable = unreachable_controls(&mut app, size);
            if !unreachable.is_empty() {
                failures.push(format!("{size:?} Settings: {unreachable:?}"));
            }
        }
        assert!(failures.is_empty(), "{failures:#?}");
    }

    #[test]
    fn accessibility_tree_has_sort_state_counts_overflow_and_no_icon_glyphs() {
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut app = super::DesktopApp::new(true);
        let mut frame = eframe::Frame::_new_kittest();
        let mut texts = Vec::new();
        for time in [0.0, 0.02] {
            let mut output = context.run_ui(
                egui::RawInput {
                    // The minimum window size, where the command bar overflows.
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(880.0, 600.0),
                    )),
                    time: Some(time),
                    ..Default::default()
                },
                |ui| <super::DesktopApp as eframe::App>::ui(&mut app, ui, &mut frame),
            );
            output.textures_delta.clear();
            texts = output
                .platform_output
                .accesskit_update
                .as_ref()
                .map(|update| {
                    update
                        .nodes
                        .iter()
                        .flat_map(|(_, node)| [node.label(), node.value()])
                        .flatten()
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default();
        }
        let private_use =
            |text: &String| text.chars().any(|c| ('\u{E000}'..='\u{F8FF}').contains(&c));
        assert!(
            !texts.iter().any(private_use),
            "icon glyphs leaked: {texts:?}"
        );
        assert!(
            texts.iter().any(|text| text == "Name, sorted ascending"),
            "{texts:?}"
        );
        assert!(texts.iter().any(|text| text == "Type"), "{texts:?}");
        assert!(
            texts
                .iter()
                .any(|text| text.starts_with("Files, ") && text.ends_with(" items")),
            "{texts:?}"
        );
        assert!(
            texts.iter().any(|text| text == "More commands"),
            "{texts:?}"
        );
        assert!(texts.iter().any(|text| text == "New folder"), "{texts:?}");
    }

    #[test]
    fn synthetic_full_app_row_click_updates_selection_toolbar_and_keyboard_focus() {
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut app = super::DesktopApp::new(true);
        let (_, output) = render_complete_app_frame(&mut app, &context, Vec::new(), 0.0);
        let row = accessible_center_containing(
            &output,
            "Annual report.txt, Text document, 73 KB, modified",
        )
        .expect("the accessible file list exposes the synthetic first file row");
        // The command bar is stable: Rename is always present, but disabled until exactly
        // one item is selected (so selecting never shifts the table).
        assert_eq!(accessible_disabled(&output, "Rename"), Some(true));

        click_app_point(&mut app, &context, row, 0.02);
        let preview = app.preview.as_ref().unwrap();
        assert_eq!(preview.explorer.selected_ids(), [EntryId(4)]);
        assert_eq!(preview.explorer.focused_id(), Some(EntryId(4)));
        assert!(app.table_has_focus);

        let (_, output) = render_complete_app_frame(&mut app, &context, Vec::new(), 0.04);
        let labels = accessible_labels(&output);
        assert_eq!(
            accessible_disabled(&output, "Rename"),
            Some(false),
            "selecting one row must enable Rename: {labels:?}"
        );
        assert!(
            accessible_center(&output, "Move to Trash").is_some(),
            "{labels:?}"
        );

        let visible = {
            let preview = app.preview.as_mut().unwrap();
            preview.explorer.visible_ids(&preview.model).to_vec()
        };
        let old_index = visible
            .iter()
            .position(|id| *id == EntryId(4))
            .expect("selected row is visible");
        send_app_key(
            &mut app,
            &context,
            egui::Key::ArrowDown,
            egui::Modifiers::default(),
            0.06,
        );
        let next_id = visible.get((old_index + 1).min(visible.len() - 1)).copied();
        assert_eq!(app.preview.as_ref().unwrap().explorer.focused_id(), next_id);
        assert!(app.table_has_focus);
    }

    #[test]
    fn synthetic_full_app_preview_state_selector_opens_with_accessible_options() {
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut app = super::DesktopApp::new(true);
        let (_, output) = render_complete_app_frame(&mut app, &context, Vec::new(), 0.0);
        let state_picker = accessible_center(&output, "Preview state: Populated Files")
            .unwrap_or_else(|| {
                panic!("preview selector missing: {:?}", accessible_labels(&output))
            });

        click_app_point(&mut app, &context, state_picker, 0.02);
        let (_, output) = render_complete_app_frame(&mut app, &context, Vec::new(), 0.04);
        assert!(
            accessible_center(&output, "Error state").is_some(),
            "opening preview selector must expose its scenario options"
        );
        assert_eq!(
            app.preview.as_ref().unwrap().scenario,
            PreviewScenario::Explorer
        );
    }

    #[test]
    fn synthetic_full_app_list_size_selector_changes_fixture_size() {
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut app = super::DesktopApp::new(true);
        let (_, output) = render_complete_app_frame(&mut app, &context, Vec::new(), 0.0);
        let list_size = accessible_center(&output, "List size: 100 items").unwrap();

        click_app_point(&mut app, &context, list_size, 0.02);
        let (_, output) = render_complete_app_frame(&mut app, &context, Vec::new(), 0.04);
        let thousand = accessible_center(&output, "1,000 items").unwrap();
        click_app_point(&mut app, &context, thousand, 0.06);

        let preview = app.preview.as_ref().unwrap();
        assert_eq!(preview.dataset_size, DatasetSize::OneThousand);
        assert_eq!(preview.model.dataset_size(), DatasetSize::OneThousand);
    }

    #[test]
    fn synthetic_full_app_scenario_transition_and_search_update_visible_results() {
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut app = super::DesktopApp::new(true);
        app.set_scenario(PreviewScenario::Error);
        let (_, output) = render_complete_app_frame(&mut app, &context, Vec::new(), 0.0);
        assert_eq!(
            app.preview.as_ref().unwrap().scenario,
            PreviewScenario::Error
        );
        assert!(app.preview.as_ref().unwrap().error.is_some());
        let search = accessible_center(&output, "Search files")
            .expect("search is exposed as a named text input in the complete app frame");

        click_app_point(&mut app, &context, search, 0.02);
        let (duration, _) = render_complete_app_frame(
            &mut app,
            &context,
            vec![egui::Event::Text("project".to_owned())],
            0.04,
        );
        let preview = app.preview.as_mut().unwrap();
        assert_eq!(preview.search_input, "project");
        let visible = preview.explorer.visible_ids(&preview.model);
        assert!(!visible.is_empty());
        assert!(visible.iter().all(|id| {
            preview
                .model
                .entry(*id)
                .is_some_and(|entry| entry.name.to_lowercase().contains("project"))
        }));
        assert!(duration < Duration::from_secs(2));
    }

    #[test]
    fn repeated_enter_on_open_trash_dialog_never_moves_the_selected_sample_item() {
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut app = super::DesktopApp::new(true);
        {
            let preview = app.preview.as_mut().unwrap();
            let visible = preview.explorer.visible_ids(&preview.model);
            preview
                .explorer
                .select(EntryId(4), &visible, Modifier::Plain);
            preview.dialog = Some(DialogKind::MoveToTrash);
        }
        let (open_duration, _) = render_complete_app_frame(&mut app, &context, Vec::new(), 0.0);
        assert_eq!(
            app.preview.as_ref().unwrap().dialog,
            Some(DialogKind::MoveToTrash)
        );

        let duration = send_repeated_app_key(&mut app, &context, egui::Key::Enter, 0.08);
        let preview = app.preview.as_ref().unwrap();
        assert!(
            matches!(preview.dialog, None | Some(DialogKind::MoveToTrash)),
            "repeat must not select a different operation"
        );
        assert!(
            preview
                .model
                .entry(EntryId(4))
                .unwrap()
                .deleted_at
                .is_none()
        );
        assert!(duration < Duration::from_secs(2));

        let cancel_duration = if app.preview.as_ref().unwrap().dialog.is_some() {
            send_app_key(
                &mut app,
                &context,
                egui::Key::Enter,
                egui::Modifiers::default(),
                0.10,
            )
        } else {
            Duration::ZERO
        };
        assert!(
            app.preview
                .as_ref()
                .unwrap()
                .model
                .entry(EntryId(4))
                .unwrap()
                .deleted_at
                .is_none()
        );
        assert!(open_duration < Duration::from_secs(2));
        assert!(cancel_duration < Duration::from_secs(2));
    }
}
