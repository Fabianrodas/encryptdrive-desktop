use eframe::egui::{self, RichText};

use encryptdrive_desktop::{Appearance, AppearancePreference, Destination, ShellState};

use crate::design::{
    components::{self, workspace_navigation},
    icons::Icon,
    theme::{self, Palette},
    typography::TypeScale,
};

pub struct DesktopShell {
    state: ShellState,
    applied_appearance: Option<AppearancePreference>,
    resolved_appearance: Option<Appearance>,
    logo_texture: Option<egui::TextureHandle>,
    window_fitted: bool,
}

impl DesktopShell {
    pub const fn new() -> Self {
        Self {
            state: ShellState::new(),
            applied_appearance: None,
            resolved_appearance: None,
            logo_texture: None,
            window_fitted: false,
        }
    }

    /// Starts with an explicit appearance instead of following the system (review builds).
    pub const fn with_appearance(mut self, appearance: AppearancePreference) -> Self {
        self.state.appearance = appearance;
        self
    }

    /// Starts with the Settings dialog open (review builds).
    pub const fn with_settings_open(mut self, open: bool) -> Self {
        self.state.settings_open = open;
        self
    }

    fn palette(&self) -> Palette {
        let appearance = self.resolved_appearance.unwrap_or(Appearance::Dark);
        Palette::from_tokens(self.state.tokens(appearance))
    }

    fn apply_theme(&mut self, ctx: &egui::Context) {
        if self.applied_appearance != Some(self.state.appearance) {
            theme::apply(ctx, self.state.appearance);
            self.applied_appearance = Some(self.state.appearance);
        }
        self.resolved_appearance = Some(match ctx.theme() {
            egui::Theme::Light => Appearance::Light,
            egui::Theme::Dark => Appearance::Dark,
        });
    }
}

impl Default for DesktopShell {
    fn default() -> Self {
        Self::new()
    }
}

impl eframe::App for DesktopShell {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        crate::design::work_area::fit_window_to_monitor(&ctx, &mut self.window_fitted);
        self.apply_theme(&ctx);
        let palette = self.palette();

        if self.logo_texture.is_none() {
            self.logo_texture = Some(components::load_brand_texture(ui.ctx()));
        }
        if components::app_header(
            ui,
            self.logo_texture
                .as_ref()
                .expect("loaded immediately above"),
            palette,
            "",
        ) {
            self.state.settings_open = true;
        }

        components::status_bar_panel(palette).show(ui, |ui| {
            components::use_compact_density(ui);
            ui.horizontal_centered(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new("Engineering build")
                            .size(TypeScale::HELPER)
                            .color(palette.muted),
                    );
                });
            });
        });

        components::sidebar_panel(palette, ui.max_rect().width()).show(ui, |ui| {
            if let Some(destination) =
                workspace_navigation(ui, self.state.destination, true, palette).chosen
            {
                self.state.destination = destination;
            }
        });

        egui::CentralPanel::default()
            .frame(components::central_canvas_frame(palette))
            .show(ui, |ui| {
                components::content_frame(palette).show(ui, |ui| {
                    ui.set_min_size(ui.available_size());
                    let is_trash = self.state.destination == Destination::Trash;
                    let (icon, title, description) = if is_trash {
                        (
                            Icon::Trash,
                            "Trash is unavailable",
                            "Deleted items will appear here once vault support is implemented.",
                        )
                    } else {
                        (
                            Icon::Lock,
                            "No vault open",
                            "Vault support isn't available in this engineering build.",
                        )
                    };
                    components::empty_state(ui, icon, title, description, palette);
                });
            });

        if self.state.settings_open {
            self.state.settings_open = components::settings_dialog(
                &ctx,
                &mut self.state.appearance,
                "This preference lasts for the current session.",
                palette,
            );
        }
    }
}
