//! Shared, Fluent-aligned controls. Every interactive surface in the shell and the
//! design preview is painted here so states (rest, hover, pressed, focus, disabled,
//! selected) stay identical across screens.
//!
//! Rule for this module: never call `Response` accessors or `Context` methods inside a
//! `Context`/`Ui` closure (`input`, `memory`, `options` ...). egui's context lock is not
//! reentrant, so nested access deadlocks the UI thread.

use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, FontFamily, FontId, Frame, Layout, Margin, Rect,
    RichText, Sense, Stroke, StrokeKind, TextureHandle, WidgetInfo,
};

use super::typography::semibold_font;
use super::{geometry::Geometry, icons::Icon, theme::Palette, typography::TypeScale};
use crate::design::copy::OFFLINE_STATUS;
use encryptdrive_desktop::{AppearancePreference, Destination};

fn icon_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("encryptdrive-icons".into()))
}

fn control_radius() -> CornerRadius {
    CornerRadius::same(Geometry::CONTROL_RADIUS)
}

/// Visual role of a button.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(feature = "design-preview"), allow(dead_code))]
pub(crate) enum ButtonKind {
    /// Accent fill; one per surface at most.
    Primary,
    /// Neutral filled button with a subtle outline.
    Standard,
    /// Transparent until hovered (command bar, title bar, icon buttons).
    Subtle,
    /// Neutral button whose label uses the danger colour.
    Destructive,
}

/// Paints and senses a button. `label` is always the accessible name; with
/// `icon_only` it is shown as a tooltip instead of being drawn.
fn paint_button(
    ui: &mut egui::Ui,
    icon: Option<Icon>,
    label: &str,
    kind: ButtonKind,
    icon_only: bool,
    min_width: f32,
    palette: Palette,
) -> egui::Response {
    let enabled = ui.is_enabled();
    let (foreground, icon_color) = match (enabled, kind) {
        (false, _) => (palette.disabled_text(), palette.disabled_text()),
        (true, ButtonKind::Primary) => (palette.on_action, palette.on_action),
        (true, ButtonKind::Destructive) => (palette.danger, palette.danger),
        (true, _) => (palette.text, palette.icon),
    };
    let icon_galley = icon.map(|icon| {
        ui.painter().layout_no_wrap(
            icon.glyph().to_string(),
            icon_font(Geometry::ICON_SIZE),
            icon_color,
        )
    });
    let label_galley = (!icon_only).then(|| {
        let font = if kind == ButtonKind::Primary {
            semibold_font(TypeScale::CONTROL)
        } else {
            FontId::proportional(TypeScale::CONTROL)
        };
        ui.painter()
            .layout_no_wrap(label.to_owned(), font, foreground)
    });
    let gap = if icon_galley.is_some() && label_galley.is_some() {
        Geometry::SPACE_2
    } else {
        0.0
    };
    let content_width = icon_galley.as_ref().map_or(0.0, |g| g.size().x)
        + gap
        + label_galley.as_ref().map_or(0.0, |g| g.size().x);
    // Height follows the surrounding density (standard 32 px, compact 28 px in the status
    // bar) via `interact_size`, so one painter serves every context.
    let height = ui
        .spacing()
        .interact_size
        .y
        .max(Geometry::COMPACT_CONTROL_HEIGHT);
    let padding = if height < Geometry::CONTROL_HEIGHT {
        Geometry::SPACE_2
    } else {
        Geometry::SPACE_3
    };
    let width = if icon_only {
        height + Geometry::BASE
    } else {
        (content_width + padding * 2.0).max(min_width)
    };
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(width, height),
        Sense::click().union(Sense::focusable_noninteractive()),
    );

    if ui.is_rect_visible(rect) {
        let hovered = enabled && response.hovered();
        let pressed = enabled && response.is_pointer_button_down_on();
        let (fill, stroke) = match kind {
            ButtonKind::Primary if enabled => (
                if pressed {
                    palette.accent_pressed()
                } else if hovered {
                    palette.accent_hover()
                } else {
                    palette.accent
                },
                Color32::TRANSPARENT,
            ),
            ButtonKind::Subtle => (
                if pressed {
                    palette.surface_alt
                } else if hovered {
                    palette.hover
                } else {
                    Color32::TRANSPARENT
                },
                Color32::TRANSPARENT,
            ),
            _ => (
                if !enabled || pressed {
                    palette.surface_alt
                } else if hovered {
                    palette.hover
                } else {
                    palette.surface
                },
                palette.control_stroke(),
            ),
        };
        ui.painter().rect_filled(rect, control_radius(), fill);
        if stroke != Color32::TRANSPARENT {
            ui.painter().rect_stroke(
                rect,
                control_radius(),
                Stroke::new(1.0, stroke),
                StrokeKind::Inside,
            );
        }
        let mut x = rect.center().x - content_width / 2.0;
        let content_color = if pressed && kind != ButtonKind::Primary {
            palette.muted
        } else {
            foreground
        };
        if let Some(galley) = icon_galley {
            let size = galley.size();
            ui.painter().galley(
                egui::pos2(x, rect.center().y - size.y / 2.0),
                galley,
                icon_color,
            );
            x += size.x + gap;
        }
        if let Some(galley) = label_galley {
            let size = galley.size();
            ui.painter().galley(
                egui::pos2(x, rect.center().y - size.y / 2.0),
                galley,
                content_color,
            );
        }
    }

    paint_focus_ring(ui, &response, palette);
    response.widget_info(|| WidgetInfo::labeled(egui::Role::Button, enabled, label));
    if icon_only {
        response.on_hover_text(label)
    } else {
        response
    }
}

#[cfg(feature = "design-preview")]
pub(crate) fn secondary_button(ui: &mut egui::Ui, label: &str, palette: Palette) -> egui::Response {
    paint_button(ui, None, label, ButtonKind::Standard, false, 0.0, palette)
}

/// Command-bar button: transparent at rest, icon plus label (or icon only when compact).
#[cfg(feature = "design-preview")]
pub(crate) fn command_button(
    ui: &mut egui::Ui,
    icon: Icon,
    label: &str,
    kind: ButtonKind,
    compact: bool,
    palette: Palette,
) -> egui::Response {
    paint_button(ui, Some(icon), label, kind, compact, 0.0, palette)
}

/// Square, transparent icon button with a tooltip; `label` is the accessible name.
pub(crate) fn subtle_icon_button(
    ui: &mut egui::Ui,
    icon: Icon,
    label: &str,
    palette: Palette,
) -> egui::Response {
    paint_button(
        ui,
        Some(icon),
        label,
        ButtonKind::Subtle,
        true,
        0.0,
        palette,
    )
}

/// Equal-width button used in dialog footers.
pub(crate) fn dialog_button(
    ui: &mut egui::Ui,
    label: &str,
    kind: ButtonKind,
    width: f32,
    palette: Palette,
) -> egui::Response {
    paint_button(ui, None, label, kind, false, width, palette)
}

/// Text-only breadcrumb segment: transparent, hover fill, keyboard focusable.
#[cfg(feature = "design-preview")]
pub(crate) fn breadcrumb_button(
    ui: &mut egui::Ui,
    label: &str,
    current: bool,
    palette: Palette,
) -> egui::Response {
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        semibold_font(TypeScale::PAGE_TITLE),
        if current { palette.text } else { palette.muted },
    );
    let size = egui::vec2(
        galley.size().x + Geometry::SPACE_2 * 2.0,
        galley.size().y.max(Geometry::CONTROL_HEIGHT),
    );
    let (rect, response) = ui.allocate_exact_size(
        size,
        Sense::click().union(Sense::focusable_noninteractive()),
    );
    if ui.is_rect_visible(rect) {
        if response.hovered() && !current {
            ui.painter()
                .rect_filled(rect, control_radius(), palette.hover);
        }
        let color = if current || response.hovered() {
            palette.text
        } else {
            palette.muted
        };
        ui.painter().galley(
            egui::pos2(
                rect.left() + Geometry::SPACE_2,
                rect.center().y - galley.size().y / 2.0,
            ),
            galley,
            color,
        );
    }
    paint_focus_ring(ui, &response, palette);
    response.widget_info(|| WidgetInfo::labeled(egui::Role::Link, true, label));
    response
}

#[cfg(feature = "design-preview")]
pub(crate) fn table_sort_header_button(
    ui: &mut egui::Ui,
    label: &str,
    sort_indicator: Option<Icon>,
    align_right: bool,
    inset: f32,
    palette: Palette,
) -> egui::Response {
    let enabled = ui.is_enabled();
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), Geometry::TABLE_HEADER_HEIGHT),
        Sense::click().union(Sense::focusable_noninteractive()),
    );
    if ui.is_rect_visible(rect) {
        if enabled && (response.hovered() || response.is_pointer_button_down_on()) {
            ui.painter().rect_filled(
                rect.shrink2(egui::vec2(0.0, Geometry::BASE)),
                control_radius(),
                if response.is_pointer_button_down_on() {
                    palette.surface_alt
                } else {
                    palette.hover
                },
            );
        }
        let color = if sort_indicator.is_some() {
            palette.text
        } else {
            palette.muted
        };
        let galley = ui.painter().layout_no_wrap(
            label.to_owned(),
            FontId::proportional(TypeScale::METADATA),
            color,
        );
        let chevron_space = if sort_indicator.is_some() {
            Geometry::SPACE_3
        } else {
            0.0
        };
        let text_width = galley.size().x;
        let text_left = if align_right {
            rect.right() - inset - chevron_space - text_width
        } else {
            rect.left() + inset
        };
        ui.painter().galley(
            egui::pos2(text_left, rect.center().y - galley.size().y / 2.0),
            galley,
            color,
        );
        if let Some(indicator) = sort_indicator {
            // Drawn as geometry: the icon font maps a-z to empty glyphs, so it must never
            // be used for words.
            let center = egui::pos2(text_left + text_width + Geometry::SPACE_2, rect.center().y);
            let stroke = Stroke::new(1.25, palette.muted);
            let (outer_y, inner_y) = if indicator == Icon::SortAscending {
                (center.y + 2.0, center.y - 2.0)
            } else {
                (center.y - 2.0, center.y + 2.0)
            };
            ui.painter().line_segment(
                [
                    egui::pos2(center.x - 3.5, outer_y),
                    egui::pos2(center.x, inner_y),
                ],
                stroke,
            );
            ui.painter().line_segment(
                [
                    egui::pos2(center.x, inner_y),
                    egui::pos2(center.x + 3.5, outer_y),
                ],
                stroke,
            );
        }
    }
    response.widget_info(|| WidgetInfo::labeled(egui::Role::Button, enabled, label));
    paint_focus_ring(ui, &response, palette);
    response
}

/// Context-menu row with an optional keyboard shortcut hint.
#[cfg(feature = "design-preview")]
pub(crate) fn menu_item(ui: &mut egui::Ui, label: &str, shortcut: Option<&str>) -> egui::Response {
    let mut button = egui::Button::new(label)
        .frame_when_inactive(false)
        .min_size(egui::vec2(200.0, Geometry::CONTROL_HEIGHT));
    if let Some(shortcut) = shortcut {
        button = button.shortcut_text(shortcut);
    }
    ui.add(button)
}

pub(crate) fn button_activated(ui: &egui::Ui, response: &egui::Response) -> bool {
    button_activated_when_allowed(ui, response, true)
}

pub(crate) fn button_activated_when_allowed(
    ui: &egui::Ui,
    response: &egui::Response,
    activation_allowed: bool,
) -> bool {
    if !activation_allowed {
        return false;
    }
    // Each accessor below takes egui's (non-reentrant) context write lock, so all of
    // them must be evaluated before entering `ui.input`.
    let has_focus = response.has_focus();
    let pointer_clicked = response.clicked_by(egui::PointerButton::Primary);
    let widget_clicked = response.clicked();
    ui.input(|input| {
        let enter_pressed = input.key_pressed(egui::Key::Enter);
        let space_pressed = input.key_pressed(egui::Key::Space);
        let accessibility_clicked =
            widget_clicked && !pointer_clicked && !enter_pressed && !space_pressed;
        pointer_clicked
            || accessibility_clicked
            || keyboard_activation_requested(
                true,
                has_focus,
                key_pressed_without_repeat(input, egui::Key::Enter),
                key_pressed_without_repeat(input, egui::Key::Space),
            )
    })
}

pub(crate) fn key_pressed_without_repeat(input: &egui::InputState, key: egui::Key) -> bool {
    input.events.iter().any(|event| {
        matches!(
            event,
            egui::Event::Key {
                key: pressed_key,
                pressed: true,
                repeat: false,
                ..
            } if *pressed_key == key
        )
    })
}

fn keyboard_activation_requested(
    keyboard_activation_allowed: bool,
    has_focus: bool,
    enter: bool,
    space: bool,
) -> bool {
    keyboard_activation_allowed && has_focus && (enter || space)
}

/// Fluent double focus ring: a 2 px outer ring in the focus colour drawn *outside* the
/// control, so it is visible on accent fills and selected rows alike.
pub(crate) fn paint_focus_ring(ui: &egui::Ui, response: &egui::Response, palette: Palette) {
    if response.has_focus() && ui.is_rect_visible(response.rect) {
        paint_focus_ring_rect(ui, response.rect, palette);
    }
}

pub(crate) fn paint_focus_ring_rect(ui: &egui::Ui, rect: Rect, palette: Palette) {
    let radius = CornerRadius::same(Geometry::CONTROL_RADIUS + 2);
    ui.painter().rect_stroke(
        rect.expand(2.0),
        radius,
        Stroke::new(2.0, palette.focus),
        StrokeKind::Inside,
    );
}

/// Fluent text-box focus: an accent underline in addition to the frame.
#[cfg(feature = "design-preview")]
fn paint_input_focus(ui: &egui::Ui, response: &egui::Response, palette: Palette) {
    if response.has_focus() && ui.is_rect_visible(response.rect) {
        let rect = response.rect;
        ui.painter().rect_filled(
            Rect::from_min_max(
                egui::pos2(rect.left() + 1.0, rect.bottom() - 2.0),
                egui::pos2(rect.right() - 1.0, rect.bottom()),
            ),
            CornerRadius {
                nw: 0,
                ne: 0,
                sw: Geometry::CONTROL_RADIUS,
                se: Geometry::CONTROL_RADIUS,
            },
            palette.accent,
        );
        paint_focus_ring(ui, response, palette);
    }
}

#[cfg(feature = "design-preview")]
pub(crate) fn labeled_text_field(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut String,
    hint: &str,
    password: bool,
    palette: Palette,
) -> egui::Response {
    let label_response = ui.label(
        RichText::new(label)
            .size(TypeScale::BODY)
            .color(palette.text),
    );
    ui.add_space(-Geometry::BASE);
    let width = ui.available_width();
    let mut text_edit = egui::TextEdit::singleline(value)
        .hint_text(hint)
        .desired_width(width)
        .align(Align2::LEFT_CENTER)
        .margin(Margin::symmetric(
            Geometry::SPACE_3 as i8,
            Geometry::BASE as i8,
        ));
    if password {
        text_edit = text_edit.password(true);
    }
    let response = ui
        .add_sized(egui::vec2(width, Geometry::CONTROL_HEIGHT), text_edit)
        .labelled_by(label_response.id);
    paint_input_focus(ui, &response, palette);
    response
}

/// Search box with the magnifier drawn inside the field (Explorer / Fluent AutoSuggestBox).
#[cfg(feature = "design-preview")]
pub(crate) fn search_field(
    ui: &mut egui::Ui,
    value: &mut String,
    hint: &str,
    width: f32,
    palette: Palette,
) -> egui::Response {
    let response = ui.add_sized(
        egui::vec2(width, Geometry::CONTROL_HEIGHT),
        egui::TextEdit::singleline(value)
            .hint_text(hint)
            .align(Align2::LEFT_CENTER)
            .margin(Margin {
                left: (Geometry::SPACE_8) as i8,
                right: Geometry::SPACE_2 as i8,
                top: Geometry::BASE as i8,
                bottom: Geometry::BASE as i8,
            }),
    );
    if ui.is_rect_visible(response.rect) {
        ui.painter().text(
            egui::pos2(
                response.rect.left() + Geometry::SPACE_4,
                response.rect.center().y,
            ),
            Align2::CENTER_CENTER,
            Icon::Search.glyph(),
            icon_font(Geometry::ICON_SIZE),
            palette.muted,
        );
    }
    paint_input_focus(ui, &response, palette);
    response.widget_info(|| {
        WidgetInfo::labeled(egui::Role::TextInput, ui.is_enabled(), "Search files")
    });
    response
}

/// Paints the Fluent selection pill on the leading edge of `rect`.
pub(crate) fn paint_selection_pill(painter: &egui::Painter, rect: Rect, palette: Palette) {
    let height = Geometry::SELECTION_PILL_HEIGHT.min(rect.height() - Geometry::BASE * 2.0);
    painter.rect_filled(
        Rect::from_center_size(
            egui::pos2(
                rect.left() + Geometry::SELECTION_PILL_WIDTH / 2.0 + 1.0,
                rect.center().y,
            ),
            egui::vec2(Geometry::SELECTION_PILL_WIDTH, height),
        ),
        CornerRadius::same(2),
        palette.selection_indicator,
    );
}

pub(crate) fn nav_item(
    ui: &mut egui::Ui,
    label: &str,
    icon: Icon,
    selected: bool,
    enabled: bool,
    palette: Palette,
) -> egui::Response {
    let sense = if enabled {
        Sense::click().union(Sense::focusable_noninteractive())
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), Geometry::NAVIGATION_ROW_HEIGHT),
        sense,
    );
    if ui.is_rect_visible(rect) {
        let hovered = enabled && response.hovered();
        let pressed = enabled && response.is_pointer_button_down_on();
        let fill = if selected {
            if hovered {
                super::theme::mix(palette.selected, palette.hover, 0.5)
            } else {
                palette.selected
            }
        } else if pressed {
            palette.surface_alt
        } else if hovered {
            palette.hover
        } else {
            Color32::TRANSPARENT
        };
        ui.painter().rect_filled(rect, control_radius(), fill);
        if selected {
            paint_selection_pill(ui.painter(), rect, palette);
        }
        let foreground = if !enabled {
            palette.disabled_text()
        } else if selected {
            palette.selected_text
        } else {
            palette.text
        };
        ui.painter().text(
            egui::pos2(rect.left() + Geometry::NAV_ICON_CENTER, rect.center().y),
            Align2::CENTER_CENTER,
            icon.glyph(),
            icon_font(Geometry::ICON_SIZE),
            if enabled {
                palette.icon_foreground(selected)
            } else {
                foreground
            },
        );
        ui.painter().text(
            egui::pos2(rect.left() + Geometry::NAV_LABEL_INSET, rect.center().y),
            Align2::LEFT_CENTER,
            label,
            if selected {
                semibold_font(TypeScale::BODY)
            } else {
                FontId::proportional(TypeScale::BODY)
            },
            foreground,
        );
    }
    paint_focus_ring(ui, &response, palette);
    response.widget_info(|| WidgetInfo::selected(egui::Role::Tab, enabled, selected, label));
    response
}

/// Loads the husky mark with its transparent padding trimmed and mipmaps enabled, so the
/// 500 px source downsamples cleanly to header size at every display scale.
pub(crate) fn load_brand_texture(ctx: &egui::Context) -> TextureHandle {
    let logo = eframe::icon_data::from_png_bytes(include_bytes!(
        "../../assets/brand/encryptdrive-husky.png"
    ))
    .expect("the approved EncryptDrive logo asset is valid PNG data");
    let (width, height) = (logo.width as usize, logo.height as usize);
    let alpha = |x: usize, y: usize| logo.rgba[(y * width + x) * 4 + 3];
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (width, height, 0, 0);
    for y in 0..height {
        for x in 0..width {
            if alpha(x, y) > 8 {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }
    let image = if min_x <= max_x && min_y <= max_y {
        // Square crop centred on the opaque bounds keeps the mark's aspect ratio.
        let side = (max_x - min_x).max(max_y - min_y) + 1;
        let center_x = (min_x + max_x) / 2;
        let center_y = (min_y + max_y) / 2;
        let left = center_x
            .saturating_sub(side / 2)
            .min(width - side.min(width));
        let top = center_y
            .saturating_sub(side / 2)
            .min(height - side.min(height));
        let side = side.min(width).min(height);
        let mut pixels = Vec::with_capacity(side * side * 4);
        for y in top..top + side {
            let start = (y * width + left) * 4;
            pixels.extend_from_slice(&logo.rgba[start..start + side * 4]);
        }
        egui::ColorImage::from_rgba_unmultiplied([side, side], &pixels)
    } else {
        egui::ColorImage::from_rgba_unmultiplied([width, height], &logo.rgba)
    };
    ctx.load_texture(
        "encryptdrive-brand-mark",
        image,
        egui::TextureOptions {
            mipmap_mode: Some(egui::TextureFilter::Linear),
            ..egui::TextureOptions::LINEAR
        },
    )
}

pub(crate) fn brand_mark(ui: &mut egui::Ui, texture: &TextureHandle) {
    let size = egui::vec2(Geometry::BRAND_MARK_SIZE, Geometry::BRAND_MARK_SIZE);
    let response = ui.add(
        egui::Image::new((texture.id(), size))
            .fit_to_exact_size(size)
            .sense(Sense::hover()),
    );
    response
        .widget_info(|| WidgetInfo::labeled(egui::Role::Image, true, "EncryptDrive husky logo"));
}

pub(crate) fn workspace_navigation(
    ui: &mut egui::Ui,
    current: Destination,
    enabled: bool,
    palette: Palette,
) -> WorkspaceNavigationResult {
    let mut result = WorkspaceNavigationResult::default();
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = Geometry::BASE;
        for (destination, icon, label) in [
            (Destination::Files, Icon::Files, "Files"),
            (Destination::Trash, Icon::Trash, "Trash"),
        ] {
            let response = nav_item(
                ui,
                label,
                icon,
                enabled && current == destination,
                enabled,
                palette,
            );
            match destination {
                Destination::Files => result.files_id = Some(response.id),
                Destination::Trash => result.trash_id = Some(response.id),
            }
            if button_activated(ui, &response) {
                result.chosen = Some(destination);
            }
        }
    });
    result
}

#[derive(Default)]
pub(crate) struct WorkspaceNavigationResult {
    pub(crate) chosen: Option<Destination>,
    pub(crate) files_id: Option<egui::Id>,
    pub(crate) trash_id: Option<egui::Id>,
}

/// Fixed-width navigation pane shared by the shell and the preview.
pub(crate) fn sidebar_panel(palette: Palette) -> egui::Panel {
    egui::Panel::left("encryptdrive-shared-sidebar")
        .resizable(false)
        .exact_size(Geometry::SIDEBAR_WIDTH)
        .show_separator_line(false)
        .frame(Frame::new().fill(palette.canvas).inner_margin(Margin {
            left: Geometry::SPACE_2 as i8,
            right: Geometry::SPACE_2 as i8,
            top: Geometry::SPACE_2 as i8,
            bottom: Geometry::SPACE_2 as i8,
        }))
}

/// Rounded content "layer" that sits on the canvas, as in Windows 11 Explorer.
pub(crate) fn content_frame(palette: Palette) -> Frame {
    Frame::new()
        .fill(palette.surface)
        .stroke(Stroke::new(1.0, palette.separator))
        .corner_radius(CornerRadius::same(Geometry::SURFACE_RADIUS))
        .inner_margin(Margin {
            left: Geometry::SPACE_6 as i8,
            right: Geometry::SPACE_6 as i8,
            top: Geometry::SPACE_4 as i8,
            bottom: Geometry::SPACE_2 as i8,
        })
}

/// Canvas around the content layer: flush with the sidebar on the left.
pub(crate) fn central_canvas_frame(palette: Palette) -> Frame {
    Frame::new().fill(palette.canvas).inner_margin(Margin {
        left: 0,
        right: Geometry::SPACE_2 as i8,
        top: 0,
        bottom: Geometry::SPACE_2 as i8,
    })
}

pub(crate) fn app_header(
    ui: &mut egui::Ui,
    logo: &TextureHandle,
    palette: Palette,
    vault_status: &str,
) -> bool {
    let mut settings_clicked = false;
    egui::Panel::top("encryptdrive-shared-app-header")
        .exact_size(Geometry::HEADER_HEIGHT)
        .show_separator_line(false)
        .frame(Frame::new().fill(palette.canvas).inner_margin(Margin {
            left: Geometry::SPACE_4 as i8,
            right: Geometry::SPACE_3 as i8,
            top: 0,
            bottom: 0,
        }))
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                brand_mark(ui, logo);
                ui.add_space(Geometry::BASE);
                ui.label(
                    RichText::new("EncryptDrive")
                        .font(semibold_font(TypeScale::APP_IDENTITY))
                        .color(palette.text),
                );
                if !vault_status.is_empty() {
                    ui.add_space(Geometry::SPACE_2);
                    status_chip(ui, Icon::Lock, vault_status, palette);
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let settings = subtle_icon_button(ui, Icon::Settings, "Settings", palette);
                    settings_clicked = button_activated(ui, &settings);
                    ui.add_space(Geometry::BASE);
                    ui.label(
                        RichText::new(OFFLINE_STATUS)
                            .size(TypeScale::HELPER)
                            .color(palette.muted),
                    )
                    .on_hover_text("EncryptDrive works locally; nothing is sent over the network.");
                    ui.label(Icon::ShieldCheck.rich_text(palette.success, Geometry::ICON_SIZE));
                });
            });
        });
    settings_clicked
}

/// Small rounded chip: neutral icon and caption text.
pub(crate) fn status_chip(ui: &mut egui::Ui, icon: Icon, text: &str, palette: Palette) {
    Frame::new()
        .fill(palette.surface_alt)
        .stroke(Stroke::new(1.0, palette.separator))
        .corner_radius(CornerRadius::same((Geometry::CHIP_HEIGHT / 2.0) as u8))
        .inner_margin(Margin::symmetric(Geometry::SPACE_3 as i8 - 2, 0))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = Geometry::BASE + 2.0;
            // `horizontal` would otherwise grow to the 32 px interact height.
            ui.spacing_mut().interact_size.y = Geometry::CHIP_HEIGHT - 2.0;
            ui.horizontal(|ui| {
                ui.label(icon.rich_text(palette.icon, TypeScale::HELPER));
                ui.label(
                    RichText::new(text)
                        .size(TypeScale::HELPER)
                        .color(palette.text),
                );
            });
        });
}

/// Bottom status bar shared by the shell and the preview (Explorer-style).
pub(crate) fn status_bar_panel(palette: Palette) -> egui::Panel {
    egui::Panel::bottom("encryptdrive-status-bar")
        .exact_size(Geometry::STATUS_BAR_HEIGHT)
        .show_separator_line(false)
        .frame(Frame::new().fill(palette.canvas).inner_margin(Margin {
            left: Geometry::SPACE_4 as i8,
            right: Geometry::SPACE_3 as i8,
            top: 0,
            bottom: 0,
        }))
}

/// Fluent compact sizing for secondary chrome such as the status bar: 28 px controls with
/// tighter padding. Text keeps the semantic type roles.
pub(crate) fn use_compact_density(ui: &mut egui::Ui) {
    let spacing = ui.spacing_mut();
    spacing.interact_size.y = Geometry::COMPACT_CONTROL_HEIGHT;
    spacing.button_padding = egui::vec2(Geometry::SPACE_2, Geometry::BASE / 2.0);
}

/// Centred empty state: hero icon, Body Large Strong title, Body description.
pub(crate) fn empty_state(
    ui: &mut egui::Ui,
    icon: Icon,
    title: &str,
    description: &str,
    palette: Palette,
) {
    let available = ui.available_size();
    let block_height = Geometry::HERO_ICON_SIZE + 72.0;
    ui.allocate_ui_with_layout(available, Layout::top_down(Align::Center), |ui| {
        ui.set_min_size(available);
        ui.add_space(((available.y - block_height) / 2.0 - Geometry::SPACE_6).max(0.0));
        ui.label(icon.rich_text(palette.muted, Geometry::HERO_ICON_SIZE));
        ui.add_space(Geometry::SPACE_3);
        ui.label(
            RichText::new(title)
                .font(semibold_font(TypeScale::SECTION_HEADING))
                .color(palette.text),
        );
        ui.add_space(Geometry::BASE);
        ui.label(
            RichText::new(description)
                .size(TypeScale::BODY)
                .color(palette.muted),
        );
    });
}

/// Frame of a Fluent ContentDialog (8 px radius, layered shadow, no inner padding: the
/// body and footer bands bring their own).
pub(crate) fn dialog_frame(ui_style: &egui::Style, palette: Palette) -> Frame {
    Frame::new()
        .fill(palette.surface)
        .stroke(Stroke::new(1.0, palette.separator))
        .corner_radius(CornerRadius::same(Geometry::SURFACE_RADIUS))
        .shadow(ui_style.visuals.window_shadow)
        .inner_margin(Margin::ZERO)
}

/// Dialog body band: 24 px padding, Subtitle title.
pub(crate) fn dialog_body<R>(
    ui: &mut egui::Ui,
    title: &str,
    palette: Palette,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    Frame::new()
        .inner_margin(Margin::same(Geometry::SPACE_6 as i8))
        .show(ui, |ui| {
            ui.set_width(Geometry::DIALOG_WIDTH - Geometry::SPACE_6 * 2.0);
            ui.label(
                RichText::new(title)
                    .font(semibold_font(TypeScale::PAGE_TITLE))
                    .color(palette.text),
            );
            ui.add_space(Geometry::SPACE_3);
            add_contents(ui)
        })
        .inner
}

/// Dialog footer band (tinted, 24 px padding). `add_buttons` receives the width each of
/// two equal buttons should use; buttons are laid out left-to-right, primary first.
pub(crate) fn dialog_footer<R>(
    ui: &mut egui::Ui,
    palette: Palette,
    add_buttons: impl FnOnce(&mut egui::Ui, f32) -> R,
) -> R {
    Frame::new()
        .fill(palette.canvas)
        .corner_radius(CornerRadius {
            nw: 0,
            ne: 0,
            sw: Geometry::SURFACE_RADIUS,
            se: Geometry::SURFACE_RADIUS,
        })
        .inner_margin(Margin::same(Geometry::SPACE_6 as i8))
        .show(ui, |ui| {
            let width = Geometry::DIALOG_WIDTH - Geometry::SPACE_6 * 2.0;
            ui.set_width(width);
            let button_width = (width - Geometry::SPACE_2) / 2.0;
            ui.horizontal(|ui| add_buttons(ui, button_width)).inner
        })
        .inner
}

/// Fluent radio row (36 px) used in Settings.
fn radio_row(
    ui: &mut egui::Ui,
    selected: bool,
    label: &str,
    description: &str,
    palette: Palette,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 52.0),
        Sense::click().union(Sense::focusable_noninteractive()),
    );
    if ui.is_rect_visible(rect) {
        if response.hovered() {
            ui.painter()
                .rect_filled(rect, control_radius(), palette.hover);
        }
        let center = egui::pos2(rect.left() + Geometry::SPACE_4 + 2.0, rect.center().y);
        if selected {
            ui.painter().circle_filled(center, 10.0, palette.accent);
            ui.painter().circle_filled(center, 4.0, palette.on_action);
        } else {
            ui.painter()
                .circle_stroke(center, 9.5, Stroke::new(1.0, palette.border));
        }
        let text_left = rect.left() + Geometry::SPACE_8 + Geometry::SPACE_2;
        ui.painter().text(
            egui::pos2(text_left, rect.center().y - 9.0),
            Align2::LEFT_CENTER,
            label,
            FontId::proportional(TypeScale::BODY),
            palette.text,
        );
        ui.painter().text(
            egui::pos2(text_left, rect.center().y + 10.0),
            Align2::LEFT_CENTER,
            description,
            FontId::proportional(TypeScale::HELPER),
            palette.muted,
        );
    }
    paint_focus_ring(ui, &response, palette);
    response.widget_info(|| WidgetInfo::selected(egui::Role::RadioButton, true, selected, label));
    response
}

/// Modal Settings dialog shared by the shell and the preview. Returns `false` once the
/// user closes it (Close button, Escape or click outside).
pub(crate) fn settings_dialog(
    ctx: &egui::Context,
    appearance: &mut AppearancePreference,
    note: &str,
    palette: Palette,
) -> bool {
    let style = ctx.global_style();
    let mut keep_open = true;
    let modal = egui::Modal::new(egui::Id::unique("encryptdrive-settings-dialog"))
        .frame(dialog_frame(&style, palette))
        .show(ctx, |ui| {
            dialog_body(ui, "Settings", palette, |ui| {
                ui.label(
                    RichText::new("Appearance")
                        .font(semibold_font(TypeScale::BODY))
                        .color(palette.text),
                );
                ui.add_space(Geometry::BASE);
                ui.spacing_mut().item_spacing.y = 0.0;
                for (value, label, description) in [
                    (
                        AppearancePreference::System,
                        "Use system setting",
                        "Follow the Windows app mode",
                    ),
                    (
                        AppearancePreference::Light,
                        "Light",
                        "Always use the light theme",
                    ),
                    (
                        AppearancePreference::Dark,
                        "Dark",
                        "Always use the dark theme",
                    ),
                ] {
                    let row = radio_row(ui, *appearance == value, label, description, palette);
                    if button_activated(ui, &row) {
                        *appearance = value;
                    }
                }
                ui.add_space(Geometry::SPACE_3);
                ui.label(
                    RichText::new(note)
                        .size(TypeScale::HELPER)
                        .color(palette.muted),
                );
            });
            dialog_footer(ui, palette, |ui, width| {
                ui.add_space(width + Geometry::SPACE_2);
                let close = dialog_button(ui, "Close", ButtonKind::Standard, width, palette);
                if button_activated(ui, &close) {
                    keep_open = false;
                }
            });
        });
    keep_open && !modal.should_close()
}

/// Shrinks and centres the window once if its default size does not fit the monitor
/// work area (e.g. 1080p at 125 % or 150 % scaling). Call every frame; it acts once.
pub(crate) fn fit_window_to_monitor(ctx: &egui::Context, done: &mut bool) {
    if *done {
        return;
    }
    let (monitor, inner) =
        ctx.input(|input| (input.viewport().monitor_size, input.viewport().inner_rect));
    let (Some(monitor), Some(inner)) = (monitor, inner) else {
        return;
    };
    *done = true;
    // Leave room for the title bar and the taskbar.
    let available = egui::vec2(monitor.x - 32.0, monitor.y - 120.0);
    let size = inner.size();
    if size.x <= available.x && size.y <= available.y {
        return;
    }
    let fitted = egui::vec2(size.x.min(available.x), size.y.min(available.y));
    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(fitted));
    ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
        ((monitor.x - fitted.x) / 2.0).max(0.0),
        ((monitor.y - fitted.y - 80.0) / 2.0).max(0.0),
    )));
}

#[cfg(test)]
mod tests {
    use super::{
        button_activated, key_pressed_without_repeat, keyboard_activation_requested,
        paint_focus_ring,
    };
    use crate::design::theme::Palette;
    use eframe::egui;

    #[test]
    fn keyboard_activation_requires_focus_on_the_button() {
        assert!(keyboard_activation_requested(true, true, true, false));
        assert!(keyboard_activation_requested(true, true, false, true));
        assert!(!keyboard_activation_requested(true, false, true, false));
        assert!(!keyboard_activation_requested(false, true, true, false));
        assert!(!keyboard_activation_requested(false, true, false, true));
    }

    #[test]
    fn custom_button_activation_survives_a_real_pointer_click_without_reentrant_locking() {
        // A genuine press/release makes `Response::clicked_by` reach egui's context lock.
        // If the helper ever evaluates it inside `ui.input` again, this test deadlocks
        // (debug builds panic after 10 s) instead of passing.
        let context = egui::Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(200.0, 80.0));
        let target = egui::pos2(40.0, 20.0);
        let mut activated = false;
        let hover = vec![egui::Event::PointerMoved(target)];
        let press = |pressed| {
            vec![
                egui::Event::PointerMoved(target),
                egui::Event::PointerButton {
                    pos: target,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::default(),
                },
            ]
        };
        // Hover first so egui knows the button's rect, then press and release on it.
        for (time, events) in [(0.0, hover), (0.02, press(true)), (0.06, press(false))] {
            let mut output = context.run_ui(
                egui::RawInput {
                    screen_rect: Some(screen),
                    time: Some(time),
                    events,
                    ..Default::default()
                },
                |ui| {
                    let response = ui.add_sized([120.0, 32.0], egui::Button::new("Test button"));
                    activated |= button_activated(ui, &response);
                },
            );
            output.textures_delta.clear();
        }

        assert!(activated, "a real pointer click must activate the button");
    }

    #[test]
    fn key_activation_ignores_repeat_events_and_accepts_a_fresh_press() {
        let context = egui::Context::default();
        let mut is_fresh = false;
        let mut output = context.run_ui(
            egui::RawInput {
                events: vec![egui::Event::Key {
                    key: egui::Key::Enter,
                    physical_key: Some(egui::Key::Enter),
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::default(),
                }],
                ..Default::default()
            },
            |ui| ui.input(|input| is_fresh = key_pressed_without_repeat(input, egui::Key::Enter)),
        );
        output.textures_delta.clear();
        assert!(is_fresh, "the initial key press must remain actionable");

        let mut output = context.run_ui(
            egui::RawInput {
                events: vec![egui::Event::Key {
                    key: egui::Key::Enter,
                    physical_key: Some(egui::Key::Enter),
                    pressed: true,
                    repeat: true,
                    modifiers: egui::Modifiers::default(),
                }],
                ..Default::default()
            },
            |ui| ui.input(|input| is_fresh = key_pressed_without_repeat(input, egui::Key::Enter)),
        );
        output.textures_delta.clear();
        assert!(!is_fresh, "held-key repeats must not activate a control");

        let mut output = context.run_ui(
            egui::RawInput {
                events: vec![egui::Event::Key {
                    key: egui::Key::Enter,
                    physical_key: Some(egui::Key::Enter),
                    pressed: false,
                    repeat: false,
                    modifiers: egui::Modifiers::default(),
                }],
                ..Default::default()
            },
            |_| {},
        );
        output.textures_delta.clear();
        let mut output = context.run_ui(
            egui::RawInput {
                events: vec![egui::Event::Key {
                    key: egui::Key::Enter,
                    physical_key: Some(egui::Key::Enter),
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::default(),
                }],
                ..Default::default()
            },
            |ui| ui.input(|input| is_fresh = key_pressed_without_repeat(input, egui::Key::Enter)),
        );
        output.textures_delta.clear();
        assert!(is_fresh, "a new press after key-up must be actionable");
    }

    #[test]
    fn focused_light_text_field_emits_a_semantic_focus_outline() {
        let context = egui::Context::default();
        let palette = Palette::for_theme(egui::Theme::Light);
        let mut value = String::new();
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(400.0, 120.0),
                )),
                ..Default::default()
            },
            |ui| {
                let response = ui.text_edit_singleline(&mut value);
                response.request_focus();
                paint_focus_ring(ui, &response, palette);
            },
        );
        output.textures_delta.clear();

        assert!(output.shapes.iter().any(|clipped| {
            matches!(
                &clipped.shape,
                egui::Shape::Rect(rect)
                    if rect.stroke.color == palette.focus && rect.stroke.width == 2.0
            )
        }));
    }

    #[test]
    fn hover_pressed_focus_and_disabled_styles_use_distinct_semantic_treatments() {
        for theme in [egui::Theme::Light, egui::Theme::Dark] {
            let palette = Palette::for_theme(theme);
            let mut style = egui::Style::default();
            crate::design::theme::configure_style(&mut style, palette, theme);
            assert_eq!(style.visuals.widgets.hovered.bg_fill, palette.hover);
            assert_eq!(
                style.visuals.widgets.hovered.bg_stroke.color,
                palette.separator
            );
            assert_eq!(style.visuals.widgets.active.bg_fill, palette.surface_alt);
            assert_eq!(style.visuals.widgets.active.bg_stroke.color, palette.border);
            assert_eq!(style.visuals.selection.bg_fill, palette.selected);
            assert_ne!(palette.focus, palette.separator);
            assert!(style.visuals.disabled_alpha < 1.0);
        }
    }
}
