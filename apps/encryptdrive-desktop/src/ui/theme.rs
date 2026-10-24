use eframe::egui::{self, Color32, Stroke, Style};

use super::geometry::Geometry;
use super::typography::{TypeScale, semibold_font};
use encryptdrive_desktop::{Appearance, AppearancePreference, SemanticTokens, ThemeMode};

#[derive(Clone, Copy)]
pub(crate) struct Palette {
    pub canvas: Color32,
    pub surface: Color32,
    pub surface_alt: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub icon: Color32,
    pub border: Color32,
    pub separator: Color32,
    pub hover: Color32,
    pub selected: Color32,
    pub selected_text: Color32,
    pub selection_indicator: Color32,
    pub accent: Color32,
    #[cfg_attr(not(feature = "design-preview"), allow(dead_code))]
    pub on_action: Color32,
    pub focus: Color32,
    #[cfg_attr(not(feature = "design-preview"), allow(dead_code))]
    pub danger: Color32,
    #[cfg_attr(not(feature = "design-preview"), allow(dead_code))]
    pub warning: Color32,
    pub success: Color32,
}

impl Palette {
    pub fn for_theme(theme: egui::Theme) -> Self {
        let appearance = match theme {
            egui::Theme::Light => Appearance::Light,
            egui::Theme::Dark => Appearance::Dark,
        };
        let mode = match appearance {
            Appearance::Light => ThemeMode::Light,
            Appearance::Dark => ThemeMode::Dark,
        };
        Self::from_tokens(SemanticTokens::for_mode(mode))
    }

    pub(crate) fn from_tokens(tokens: SemanticTokens) -> Self {
        Self {
            canvas: rgb(tokens.canvas),
            surface: rgb(tokens.surface),
            surface_alt: rgb(tokens.surface_secondary),
            text: rgb(tokens.primary_text),
            muted: rgb(tokens.secondary_text),
            icon: rgb(tokens.icon),
            border: rgb(tokens.outline),
            separator: rgb(tokens.separator),
            hover: rgb(tokens.hover),
            selected: rgb(tokens.selection),
            selected_text: rgb(tokens.selection_text),
            selection_indicator: rgb(tokens.selection_indicator),
            accent: rgb(tokens.primary_action),
            on_action: rgb(tokens.on_primary_action),
            focus: rgb(tokens.focus),
            danger: rgb(tokens.danger),
            warning: rgb(tokens.warning),
            success: rgb(tokens.success),
        }
    }

    /// Subtle outline for buttons; inputs keep the stronger 3:1 `border`.
    pub(crate) fn control_stroke(self) -> Color32 {
        mix(self.separator, self.border, 0.35)
    }

    pub(crate) fn accent_hover(self) -> Color32 {
        mix(self.accent, self.surface, 0.12)
    }

    pub(crate) fn accent_pressed(self) -> Color32 {
        mix(self.accent, self.surface, 0.24)
    }

    pub(crate) fn disabled_text(self) -> Color32 {
        mix(self.muted, self.surface, 0.35)
    }

    #[cfg_attr(not(feature = "design-preview"), allow(dead_code))]
    fn is_dark(self) -> bool {
        self.surface.r() < 128
    }

    /// Fluent caution InfoBar background (`#FFF4CE` light / `#433519` dark). Mixing the
    /// warning foreground into the surface produced a muddy olive in Dark.
    #[cfg_attr(not(feature = "design-preview"), allow(dead_code))]
    pub(crate) fn caution_surface(self) -> Color32 {
        if self.is_dark() {
            rgb(0x433519)
        } else {
            rgb(0xFFF4CE)
        }
    }

    #[cfg_attr(not(feature = "design-preview"), allow(dead_code))]
    pub(crate) fn caution_stroke(self) -> Color32 {
        if self.is_dark() {
            rgb(0x6B5420)
        } else {
            rgb(0xE8D59A)
        }
    }

    pub(crate) fn icon_foreground(self, selected: bool) -> Color32 {
        if selected {
            self.selected_text
        } else {
            self.icon
        }
    }
}

pub(crate) fn apply(ctx: &egui::Context, preference: AppearancePreference) {
    for (theme, palette) in [
        (egui::Theme::Light, Palette::for_theme(egui::Theme::Light)),
        (egui::Theme::Dark, Palette::for_theme(egui::Theme::Dark)),
    ] {
        ctx.style_mut_of(theme, |style| configure_style(style, palette, theme));
    }

    ctx.options_mut(|options| {
        options.theme_preference = match preference {
            AppearancePreference::System => egui::ThemePreference::System,
            AppearancePreference::Light => egui::ThemePreference::Light,
            AppearancePreference::Dark => egui::ThemePreference::Dark,
        };
        options.fallback_theme = egui::Theme::Dark;
        options.sync_window_theme = true;
    });
}

pub(super) fn configure_style(style: &mut Style, palette: Palette, theme: egui::Theme) {
    style.visuals = theme.default_visuals();
    for (text_style, size) in [
        (egui::TextStyle::Heading, TypeScale::PAGE_TITLE),
        (egui::TextStyle::Body, TypeScale::BODY),
        (egui::TextStyle::Button, TypeScale::CONTROL),
        (egui::TextStyle::Small, TypeScale::HELPER),
    ] {
        let font_id = if text_style == egui::TextStyle::Heading {
            semibold_font(size)
        } else {
            egui::FontId::proportional(size)
        };
        style.text_styles.insert(text_style, font_id);
    }
    style.visuals.panel_fill = palette.canvas;
    style.visuals.window_fill = palette.surface;
    style.visuals.extreme_bg_color = palette.surface;
    style.visuals.faint_bg_color = palette.surface_alt;
    style.visuals.override_text_color = Some(palette.text);
    style.visuals.selection.bg_fill = palette.selected;
    // egui uses `selection.stroke` both for the focused TextEdit frame and for selected text.
    style.visuals.selection.stroke = Stroke::new(1.0, palette.accent);
    style.visuals.hyperlink_color = palette.accent;
    style.visuals.widgets.noninteractive.bg_fill = palette.surface;
    style.visuals.widgets.noninteractive.weak_bg_fill = palette.surface;
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, palette.text);
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, palette.separator);
    style.visuals.widgets.inactive.bg_fill = palette.surface;
    style.visuals.widgets.inactive.weak_bg_fill = palette.surface;
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, palette.text);
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, palette.border);
    // Hover is a fill change only; a coloured outline is reserved for keyboard focus.
    style.visuals.widgets.hovered.bg_fill = palette.hover;
    style.visuals.widgets.hovered.weak_bg_fill = palette.hover;
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, palette.text);
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, palette.separator);
    style.visuals.widgets.hovered.expansion = 0.0;
    style.visuals.widgets.active.bg_fill = palette.surface_alt;
    style.visuals.widgets.active.weak_bg_fill = palette.surface_alt;
    style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, palette.text);
    style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, palette.border);
    style.visuals.widgets.active.expansion = 0.0;
    style.visuals.widgets.open.bg_fill = palette.surface_alt;
    style.visuals.widgets.open.weak_bg_fill = palette.surface_alt;
    style.visuals.widgets.open.fg_stroke = Stroke::new(1.0, palette.text);
    style.visuals.widgets.open.bg_stroke = Stroke::new(1.0, palette.border);
    let control_radius = egui::CornerRadius::same(Geometry::CONTROL_RADIUS);
    for widget in [
        &mut style.visuals.widgets.noninteractive,
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.active,
        &mut style.visuals.widgets.open,
    ] {
        widget.corner_radius = control_radius;
    }
    style.visuals.disabled_alpha = 0.5;
    style.visuals.window_fill = palette.surface;
    style.visuals.window_stroke = Stroke::new(1.0, palette.separator);
    style.visuals.window_corner_radius = egui::CornerRadius::same(Geometry::SURFACE_RADIUS);
    style.visuals.menu_corner_radius = egui::CornerRadius::same(Geometry::SURFACE_RADIUS);
    let shadow = egui::epaint::Shadow {
        offset: [0, 8],
        blur: 24,
        spread: 0,
        color: Color32::from_black_alpha(if theme == egui::Theme::Dark { 96 } else { 40 }),
    };
    style.visuals.window_shadow = shadow;
    style.visuals.popup_shadow = egui::epaint::Shadow {
        offset: [0, 4],
        blur: 12,
        ..shadow
    };
    style.spacing.item_spacing = egui::vec2(Geometry::SPACE_2, Geometry::SPACE_2);
    style.spacing.button_padding = egui::vec2(Geometry::SPACE_3, Geometry::BASE + 2.0);
    style.spacing.menu_margin = egui::Margin::same(Geometry::BASE as i8);
    style.spacing.interact_size.y = Geometry::CONTROL_HEIGHT;
    style.spacing.combo_height = 320.0;
}

/// Linear blend in sRGB space: `t = 0` returns `from`, `t = 1` returns `to`.
pub(crate) fn mix(from: Color32, to: Color32, t: f32) -> Color32 {
    let blend = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * t).round() as u8;
    Color32::from_rgb(
        blend(from.r(), to.r()),
        blend(from.g(), to.g()),
        blend(from.b(), to.b()),
    )
}

pub(crate) fn rgb(value: u32) -> Color32 {
    Color32::from_rgb((value >> 16) as u8, (value >> 8) as u8, value as u8)
}

#[cfg(test)]
fn contrast_ratio(foreground: Color32, background: Color32) -> f64 {
    let first = relative_luminance(foreground);
    let second = relative_luminance(background);
    let lighter = first.max(second);
    let darker = first.min(second);
    (lighter + 0.05) / (darker + 0.05)
}

#[cfg(test)]
fn relative_luminance(color: Color32) -> f64 {
    let linear = |channel: u8| {
        let srgb = f64::from(channel) / 255.0;
        if srgb <= 0.04045 {
            srgb / 12.92
        } else {
            ((srgb + 0.055) / 1.055).powf(2.4)
        }
    };

    0.2126 * linear(color.r()) + 0.7152 * linear(color.g()) + 0.0722 * linear(color.b())
}

#[cfg(test)]
mod tests {
    use super::{Palette, contrast_ratio};
    use crate::design::typography::TypeScale;
    use eframe::egui::{self, Style};
    use encryptdrive_desktop::{Appearance, SemanticTokens, ThemeMode};

    #[test]
    fn light_and_dark_semantic_pairs_meet_text_and_component_contrast_targets() {
        for (mode, appearance) in [
            (ThemeMode::Light, Appearance::Light),
            (ThemeMode::Dark, Appearance::Dark),
        ] {
            let palette = Palette::from_tokens(SemanticTokens::for_mode(mode));
            for background in [palette.canvas, palette.surface, palette.surface_alt] {
                assert!(
                    contrast_ratio(palette.text, background) >= 4.5,
                    "primary text contrast failed in {appearance:?} on {background:?}"
                );
                assert!(
                    contrast_ratio(palette.muted, background) >= 4.5,
                    "secondary text contrast failed in {appearance:?} on {background:?}"
                );
                assert!(
                    contrast_ratio(palette.icon, background) >= 3.0,
                    "icon contrast failed in {appearance:?} on {background:?}"
                );
                assert!(
                    contrast_ratio(palette.border, background) >= 3.0,
                    "control boundary contrast failed in {appearance:?} on {background:?}"
                );
                assert!(
                    contrast_ratio(palette.focus, background) >= 3.0,
                    "focus contrast failed in {appearance:?} on {background:?}"
                );
            }
            assert!(
                contrast_ratio(palette.selected_text, palette.selected) >= 4.5,
                "selected text contrast failed in {appearance:?}"
            );
            assert!(
                contrast_ratio(palette.icon_foreground(true), palette.selected) >= 3.0,
                "selected icon contrast failed in {appearance:?}"
            );
            assert!(
                contrast_ratio(palette.icon_foreground(false), palette.hover) >= 3.0,
                "hovered icon contrast failed in {appearance:?}"
            );
            assert!(
                contrast_ratio(palette.selection_indicator, palette.selected) >= 3.0,
                "selection pill must stand out from the selection fill in {appearance:?}"
            );
            assert!(
                contrast_ratio(palette.on_action, palette.accent) >= 4.5,
                "primary action label contrast failed in {appearance:?}"
            );
            for semantic in [palette.danger, palette.warning, palette.success] {
                assert!(
                    contrast_ratio(semantic, palette.surface) >= 4.5,
                    "status text contrast failed in {appearance:?} for {semantic:?}"
                );
            }
        }
    }

    #[test]
    fn caution_surfaces_keep_text_and_icon_readable_in_both_themes() {
        for theme in [egui::Theme::Light, egui::Theme::Dark] {
            let palette = Palette::for_theme(theme);
            let background = palette.caution_surface();
            assert!(contrast_ratio(palette.text, background) >= 4.5, "{theme:?}");
            assert!(
                contrast_ratio(palette.warning, background) >= 3.0,
                "{theme:?}"
            );
        }
    }

    #[test]
    fn egui_default_text_roles_use_the_shared_semantic_scale() {
        let mut style = Style::default();
        super::configure_style(
            &mut style,
            Palette::for_theme(egui::Theme::Light),
            egui::Theme::Light,
        );

        assert_eq!(
            style.text_styles[&egui::TextStyle::Heading].size,
            TypeScale::PAGE_TITLE
        );
        assert_eq!(
            style.text_styles[&egui::TextStyle::Body].size,
            TypeScale::BODY
        );
        assert_eq!(
            style.text_styles[&egui::TextStyle::Button].size,
            TypeScale::CONTROL
        );
        assert_eq!(
            style.text_styles[&egui::TextStyle::Small].size,
            TypeScale::HELPER
        );
    }

    #[test]
    fn external_audit_selection_and_focus_contrast_contract() {
        for appearance in [Appearance::Light, Appearance::Dark] {
            let theme = match appearance {
                Appearance::Light => egui::Theme::Light,
                Appearance::Dark => egui::Theme::Dark,
            };
            let palette = Palette::for_theme(theme);

            assert!(
                contrast_ratio(palette.selection_indicator, palette.canvas) >= 3.0,
                "selection indicator must be distinguishable from the canvas in {appearance:?}"
            );
            assert!(
                contrast_ratio(palette.selection_indicator, palette.surface) >= 3.0,
                "selection indicator must be distinguishable from the table surface in {appearance:?}"
            );
            assert!(
                contrast_ratio(palette.selected_text, palette.selected) >= 4.5,
                "selected text must remain readable on the selection fill in {appearance:?}"
            );
            // Focus rings are painted outside the control bounds (see `paint_focus_ring`),
            // so they are measured against the surfaces controls sit on, not the accent fill.
            for background in [palette.canvas, palette.surface, palette.selected] {
                assert!(
                    contrast_ratio(palette.focus, background) >= 3.0,
                    "keyboard focus must stay visible around controls in {appearance:?}"
                );
            }

            let mut style = Style::default();
            super::configure_style(&mut style, palette, theme);
            assert_eq!(
                style.text_styles[&egui::TextStyle::Heading].family,
                egui::FontFamily::Name("encryptdrive-ui-semibold".into()),
                "headings must use the explicit semibold family in {appearance:?}"
            );
            assert_eq!(style.visuals.override_text_color, Some(palette.text));
        }
    }
}
