use eframe::egui::{self, Align, CornerRadius, Frame, Layout, Margin, RichText, Stroke};

use encryptdrive_desktop::preview::PreviewScenario;

use super::app::DesktopApp;
use crate::design::{
    components::{
        ButtonKind, button_activated, dialog_button, labeled_text_field, paint_focus_ring,
        paint_selection_pill,
    },
    geometry::Geometry,
    icons::Icon,
    theme::Palette,
    typography::{TypeScale, semibold_font},
};

const CARD_WIDTH: f32 = 460.0;

impl DesktopApp {
    pub(super) fn render_auth_screen(&mut self, ui: &mut egui::Ui, palette: Palette) {
        let Some(preview) = &mut self.preview else {
            return;
        };
        let scenario = preview.scenario;
        let mut route = None;
        let available = ui.available_size();
        ui.allocate_ui_with_layout(available, Layout::top_down(Align::Center), |ui| {
            ui.set_min_size(available);
            ui.add_space((available.y * 0.08).clamp(Geometry::SPACE_4, Geometry::SPACE_8 * 2.0));
            Frame::new()
                .fill(palette.surface)
                .stroke(Stroke::new(1.0, palette.separator))
                .corner_radius(CornerRadius::same(Geometry::SURFACE_RADIUS))
                .inner_margin(Margin::same(Geometry::SPACE_8 as i8))
                .show(ui, |ui| {
                    ui.set_width(CARD_WIDTH - Geometry::SPACE_8 * 2.0);
                    ui.with_layout(Layout::top_down(Align::Min), |ui| {
                        let content_width = ui.available_width();
                        let half = (content_width - Geometry::SPACE_2) / 2.0;
                        match scenario {
                            PreviewScenario::VaultSelection => {
                                header(ui, Icon::Lock, "Open a vault", "Create a new encrypted vault or unlock one you already have.", palette);
                                ui.horizontal(|ui| {
                                    let button = dialog_button(ui, "Create a vault", ButtonKind::Primary, half, palette);
                                    if button_activated(ui, &button) {
                                        route = Some(PreviewScenario::CreateVault);
                                    }
                                    let button = dialog_button(ui, "Open existing", ButtonKind::Standard, half, palette);
                                    if button_activated(ui, &button) {
                                        route = Some(PreviewScenario::UnlockVault);
                                    }
                                });
                            }
                            PreviewScenario::CreateVault => {
                                header(ui, Icon::Lock, "Create a vault", "Choose a name and a strong password. You'll need the password to unlock the vault.", palette);
                                labeled_text_field(ui, "Vault name", &mut preview.form_identity, "For example, Personal files", false, palette);
                                ui.add_space(Geometry::SPACE_3);
                                labeled_text_field(ui, "Password", &mut preview.form_secret, "At least 12 characters", true, palette);
                                ui.add_space(Geometry::SPACE_6);
                                ui.horizontal(|ui| {
                                    let button = dialog_button(ui, "Create vault", ButtonKind::Primary, half, palette);
                                    if button_activated(ui, &button) {
                                        preview.form_feedback = Some("Preview only: no vault was created.".to_owned());
                                    }
                                    let button = dialog_button(ui, "Back", ButtonKind::Standard, half, palette);
                                    if button_activated(ui, &button) {
                                        route = Some(PreviewScenario::VaultSelection);
                                    }
                                });
                            }
                            PreviewScenario::UnlockVault => {
                                header(ui, Icon::Unlock, "Unlock vault", "Enter the vault password to access its files.", palette);
                                let vault_label = ui.label(RichText::new("Vault").size(TypeScale::BODY).color(palette.text));
                                egui::ComboBox::from_id_salt("synthetic-vault-choice")
                                    .selected_text(preview.sample_vault.as_str())
                                    .width(content_width)
                                    .show_ui(ui, |ui| {
                                        for vault in ["Personal sample vault", "Archive sample vault"] {
                                            ui.selectable_value(&mut preview.sample_vault, vault.to_owned(), vault);
                                        }
                                    })
                                    .response
                                    .labelled_by(vault_label.id);
                                ui.add_space(Geometry::SPACE_3);
                                labeled_text_field(ui, "Password", &mut preview.form_secret, "Vault password", true, palette);
                                ui.add_space(Geometry::SPACE_6);
                                ui.horizontal(|ui| {
                                    let button = dialog_button(ui, "Unlock", ButtonKind::Primary, half, palette);
                                    if button_activated(ui, &button) {
                                        preview.form_feedback = Some("Preview only: unlocking is not connected.".to_owned());
                                    }
                                    let button = dialog_button(ui, "Back", ButtonKind::Standard, half, palette);
                                    if button_activated(ui, &button) {
                                        route = Some(PreviewScenario::VaultSelection);
                                    }
                                });
                            }
                            PreviewScenario::AccountSelection => {
                                header(ui, Icon::Account, "Choose an account", "Accounts belong to the selected vault.", palette);
                                for name in ["Personal sample account", "Work sample account"] {
                                    if account_row(ui, name, preview.sample_account == name, palette) {
                                        preview.sample_account = name.to_owned();
                                        route = Some(PreviewScenario::AccountLogin);
                                    }
                                }
                                ui.add_space(Geometry::SPACE_4);
                                let button = dialog_button(ui, "Add an account", ButtonKind::Standard, content_width, palette);
                                    if button_activated(ui, &button) {
                                    route = Some(PreviewScenario::RegisterAccount);
                                }
                            }
                            PreviewScenario::AccountLogin => {
                                let title = format!("Sign in to {}", preview.sample_account);
                                header(ui, Icon::Account, &title, "Use the email address and password for this account.", palette);
                                labeled_text_field(ui, "Email address", &mut preview.form_identity, "name@example.com", false, palette);
                                ui.add_space(Geometry::SPACE_3);
                                labeled_text_field(ui, "Password", &mut preview.form_secret, "Account password", true, palette);
                                ui.add_space(Geometry::SPACE_6);
                                ui.horizontal(|ui| {
                                    let button = dialog_button(ui, "Sign in", ButtonKind::Primary, half, palette);
                                    if button_activated(ui, &button) {
                                        preview.form_feedback = Some("Preview only: sign-in is not implemented.".to_owned());
                                    }
                                    let button = dialog_button(ui, "Back", ButtonKind::Standard, half, palette);
                                    if button_activated(ui, &button) {
                                        route = Some(PreviewScenario::AccountSelection);
                                    }
                                });
                            }
                            PreviewScenario::RegisterAccount => {
                                header(ui, Icon::Account, "Create an account", "Registration and recovery are not available in this build.", palette);
                                labeled_text_field(ui, "Email address", &mut preview.form_identity, "name@example.com", false, palette);
                                ui.add_space(Geometry::SPACE_3);
                                labeled_text_field(ui, "Password", &mut preview.form_secret, "At least 12 characters", true, palette);
                                ui.add_space(Geometry::SPACE_6);
                                ui.horizontal(|ui| {
                                    let button = dialog_button(ui, "Create account", ButtonKind::Primary, half, palette);
                                    if button_activated(ui, &button) {
                                        preview.form_feedback = Some("Preview only: no account was created and no email was sent.".to_owned());
                                    }
                                    let button = dialog_button(ui, "Back", ButtonKind::Standard, half, palette);
                                    if button_activated(ui, &button) {
                                        route = Some(PreviewScenario::AccountSelection);
                                    }
                                });
                            }
                            PreviewScenario::AccountSettings => {
                                header(ui, Icon::Settings, "Account settings", "Sample account — changes are not saved.", palette);
                                setting_value(ui, "Account", "sample-user@example.invalid", palette);
                                ui.add_space(Geometry::SPACE_3);
                                ui.checkbox(&mut preview.lock_sample_on_exit, "Lock the session when the app closes");
                            }
                            PreviewScenario::VaultSettings => {
                                header(ui, Icon::Settings, "Vault settings", "Personal sample vault — no vault data is read or written.", palette);
                                setting_value(ui, "Location", "Not shown in this preview", palette);
                                ui.add_space(Geometry::SPACE_3);
                                ui.checkbox(&mut preview.warn_before_trash, "Ask before moving items to Trash");
                            }
                            _ => {}
                        }
                        if let Some(message) = preview.notice.as_ref().or(preview.form_feedback.as_ref()) {
                            ui.add_space(Geometry::SPACE_4);
                            Frame::new()
                                .fill(crate::design::theme::mix(palette.surface, palette.accent, 0.08))
                                .stroke(Stroke::new(1.0, crate::design::theme::mix(palette.separator, palette.accent, 0.3)))
                                .corner_radius(CornerRadius::same(Geometry::CONTROL_RADIUS))
                                .inner_margin(Margin::same(Geometry::SPACE_3 as i8))
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = Geometry::SPACE_2;
                                        ui.label(Icon::Info.rich_text(palette.accent, Geometry::ICON_SIZE));
                                        ui.label(RichText::new(message).size(TypeScale::HELPER).color(palette.text));
                                    });
                                });
                        }
                    });
                });
        });
        if let Some(scenario) = route {
            self.set_scenario(scenario);
        }
    }
}

/// Card header: accent icon tile, Subtitle title, Body description.
fn header(ui: &mut egui::Ui, icon: Icon, title: &str, description: &str, palette: Palette) {
    Frame::new()
        .fill(crate::design::theme::mix(
            palette.surface,
            palette.accent,
            0.12,
        ))
        .corner_radius(CornerRadius::same(Geometry::SURFACE_RADIUS))
        .inner_margin(Margin::same(Geometry::SPACE_2 as i8 + 2))
        .show(ui, |ui| {
            ui.label(icon.rich_text(palette.accent, Geometry::MAJOR_ICON_SIZE + 4.0));
        });
    ui.add_space(Geometry::SPACE_4);
    ui.label(
        RichText::new(title)
            .font(semibold_font(TypeScale::PAGE_TITLE))
            .color(palette.text),
    );
    ui.add_space(Geometry::BASE);
    ui.label(
        RichText::new(description)
            .size(TypeScale::BODY)
            .color(palette.muted),
    );
    ui.add_space(Geometry::SPACE_6);
}

fn setting_value(ui: &mut egui::Ui, label: &str, value: &str, palette: Palette) {
    ui.label(
        RichText::new(label)
            .size(TypeScale::HELPER)
            .color(palette.muted),
    );
    ui.label(
        RichText::new(value)
            .size(TypeScale::BODY)
            .color(palette.text),
    );
}

/// Selectable account row (Fluent list item with avatar initials).
fn account_row(ui: &mut egui::Ui, name: &str, selected: bool, palette: Palette) -> bool {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 48.0),
        egui::Sense::click().union(egui::Sense::focusable_noninteractive()),
    );
    if ui.is_rect_visible(rect) {
        let fill = if selected {
            palette.selected
        } else if response.hovered() {
            palette.hover
        } else {
            egui::Color32::TRANSPARENT
        };
        ui.painter()
            .rect_filled(rect, CornerRadius::same(Geometry::CONTROL_RADIUS), fill);
        if selected {
            paint_selection_pill(ui.painter(), rect, palette);
        }
        let avatar = egui::pos2(rect.left() + Geometry::SPACE_6, rect.center().y);
        ui.painter().circle_filled(
            avatar,
            14.0,
            crate::design::theme::mix(palette.surface, palette.accent, 0.2),
        );
        let initials: String = name
            .split_whitespace()
            .take(2)
            .filter_map(|w| w.chars().next())
            .flat_map(char::to_uppercase)
            .collect();
        ui.painter().text(
            avatar,
            egui::Align2::CENTER_CENTER,
            initials,
            semibold_font(TypeScale::HELPER),
            palette.accent,
        );
        ui.painter().text(
            egui::pos2(rect.left() + Geometry::SPACE_6 + 24.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            name,
            egui::FontId::proportional(TypeScale::BODY),
            palette.text,
        );
        ui.painter().text(
            egui::pos2(rect.right() - Geometry::SPACE_4, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            Icon::ChevronRight.glyph(),
            egui::FontId::new(
                TypeScale::HELPER,
                egui::FontFamily::Name("encryptdrive-icons".into()),
            ),
            palette.muted,
        );
    }
    paint_focus_ring(ui, &response, palette);
    response.widget_info(|| egui::WidgetInfo::selected(egui::Role::ListItem, true, selected, name));
    button_activated(ui, &response)
}
