use crate::design::{geometry::Geometry, icons::Icon, typography::TypeScale};
use eframe::egui::{self, RichText, WidgetInfo};

use encryptdrive_desktop::{
    Destination,
    preview::{DatasetSize, PreviewModel, PreviewScenario},
};

use super::app::DesktopApp;
use crate::design::{
    components::{empty_state, workspace_navigation},
    theme::Palette,
};

impl DesktopApp {
    pub(super) fn render_navigation(&mut self, ui: &mut egui::Ui, palette: Palette) {
        let enabled = self.preview.as_ref().is_none_or(|preview| {
            !matches!(
                preview.scenario,
                PreviewScenario::VaultSelection
                    | PreviewScenario::CreateVault
                    | PreviewScenario::UnlockVault
                    | PreviewScenario::AccountSelection
                    | PreviewScenario::AccountLogin
                    | PreviewScenario::RegisterAccount
            )
        });
        let current = self
            .preview
            .as_ref()
            .map_or(self.shell.destination, |state| state.explorer.destination());
        let navigation = workspace_navigation(ui, current, enabled, palette);
        self.files_navigation_widget = navigation.files_id;
        self.trash_navigation_widget = navigation.trash_id;

        if let Some(destination) = navigation.chosen {
            self.shell.destination = destination;
            if let Some(state) = &mut self.preview {
                state.explorer.set_destination(destination);
                state.scenario = match destination {
                    Destination::Files => PreviewScenario::Explorer,
                    Destination::Trash => PreviewScenario::Trash,
                };
                state.search_input.clear();
            }
        }
        if !enabled {
            ui.add_space(Geometry::SPACE_3);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = Geometry::BASE;
                ui.label(Icon::Lock.rich_text(palette.muted, TypeScale::HELPER));
                ui.label(
                    RichText::new("Unlock a vault to browse its files.")
                        .size(TypeScale::HELPER)
                        .color(palette.muted),
                );
            });
        }
    }

    /// Design-preview scenario and list-size selectors, placed in the status bar so they
    /// stay out of the product surface and out of review screenshots' content area.
    pub(super) fn render_preview_controls(&mut self, ui: &mut egui::Ui, palette: Palette) {
        let mut chosen_scenario = None;
        let mut dataset_changed = None;
        let Some(preview) = &mut self.preview else {
            return;
        };
        let selected_dataset = match preview.dataset_size {
            DatasetSize::OneHundred => "100 items",
            DatasetSize::OneThousand => "1,000 items",
            DatasetSize::TenThousand => "10,000 items",
        };
        // Laid out right-to-left: list size, scenario, then the "Preview" caption.
        let dataset_picker = egui::ComboBox::from_id_salt("preview-dataset-size")
            .selected_text(RichText::new(selected_dataset).size(TypeScale::HELPER))
            .width(104.0)
            .show_ui(ui, |ui| {
                for (size, label) in [
                    (DatasetSize::OneHundred, "100 items"),
                    (DatasetSize::OneThousand, "1,000 items"),
                    (DatasetSize::TenThousand, "10,000 items"),
                ] {
                    if ui
                        .selectable_label(preview.dataset_size == size, label)
                        .clicked()
                    {
                        dataset_changed = Some(size);
                    }
                }
            });
        dataset_picker.response.widget_info(|| {
            WidgetInfo::labeled(
                egui::Role::ComboBox,
                true,
                format!("List size: {selected_dataset}"),
            )
        });
        let selected_scenario = preview.scenario.label();
        let scenario_picker = egui::ComboBox::from_id_salt("preview-scenario-picker")
            .selected_text(RichText::new(selected_scenario).size(TypeScale::HELPER))
            .width(176.0)
            .show_ui(ui, |ui| {
                for scenario in PreviewScenario::ALL {
                    if ui
                        .selectable_label(scenario == preview.scenario, scenario.label())
                        .clicked()
                    {
                        chosen_scenario = Some(scenario);
                    }
                }
            });
        scenario_picker.response.widget_info(|| {
            WidgetInfo::labeled(
                egui::Role::ComboBox,
                true,
                format!("Preview state: {selected_scenario}"),
            )
        });
        ui.label(
            RichText::new(crate::design::copy::PREVIEW_MARKER)
                .size(TypeScale::HELPER)
                .color(palette.muted),
        );
        if let Some(scenario) = chosen_scenario {
            self.set_scenario(scenario);
        }
        if let Some(size) = dataset_changed {
            let scenario = self
                .preview
                .as_ref()
                .map(|state| state.scenario)
                .unwrap_or(PreviewScenario::Explorer);
            if let Some(preview) = &mut self.preview {
                preview.dataset_size = size;
                preview.model = PreviewModel::fixture(size);
                preview.explorer = encryptdrive_desktop::preview::ExplorerState::new();
            }
            self.set_scenario(scenario);
        }
    }

    pub(super) fn render_default_empty(&mut self, ui: &mut egui::Ui, palette: Palette) {
        empty_state(
            ui,
            Icon::Lock,
            "No vault open",
            "Vault support isn't available in this engineering build.",
            palette,
        );
    }
}
