#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use encryptdrive_desktop::AppearancePreference;

mod design;
mod shell;
#[cfg(feature = "design-preview")]
mod ui;

/// `--open-settings` starts with the Settings dialog open (useful for reviews).
fn settings_argument() -> bool {
    std::env::args().any(|arg| arg == "--open-settings")
}

/// `--theme=light|dark|system` starts with an explicit appearance (useful for reviews).
fn appearance_argument() -> AppearancePreference {
    std::env::args()
        .find_map(|arg| arg.strip_prefix("--theme=").map(str::to_owned))
        .map_or(AppearancePreference::System, |value| {
            match value.to_ascii_lowercase().as_str() {
                "light" => AppearancePreference::Light,
                "dark" => AppearancePreference::Dark,
                _ => AppearancePreference::System,
            }
        })
}

#[cfg(feature = "design-preview")]
fn main() -> eframe::Result {
    let design_preview = std::env::args().any(|arg| arg == "--design-preview");
    if design_preview {
        run_preview()
    } else {
        run_shell()
    }
}

#[cfg(not(feature = "design-preview"))]
fn main() -> eframe::Result {
    run_shell()
}

#[cfg(feature = "design-preview")]
fn run_preview() -> eframe::Result {
    let mut app = ui::DesktopApp::new(true)
        .with_appearance(appearance_argument())
        .with_settings_open(settings_argument());
    // `--scenario=N` opens the N-th preview state (0-based, same order as the picker).
    if let Some(index) = std::env::args()
        .find_map(|arg| arg.strip_prefix("--scenario=").and_then(|v| v.parse().ok()))
    {
        app = app.with_scenario_index(index);
    }
    run_native(
        "EncryptDrive — Design preview",
        Box::new(move |_| Ok(Box::new(app))),
    )
}

fn run_shell() -> eframe::Result {
    let app = shell::DesktopShell::new()
        .with_appearance(appearance_argument())
        .with_settings_open(settings_argument());
    run_native("EncryptDrive", Box::new(move |_| Ok(Box::new(app))))
}

fn run_native(title: &'static str, app: eframe::AppCreator<'_>) -> eframe::Result {
    let app = Box::new(move |creation_context: &eframe::CreationContext<'_>| {
        crate::design::icons::install_font(&creation_context.egui_ctx);
        app(creation_context)
    });
    let window_icon =
        eframe::icon_data::from_png_bytes(include_bytes!("../assets/brand/encryptdrive-husky.png"))
            .expect("the approved EncryptDrive application icon is valid PNG data");
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title(title)
            .with_inner_size([1200.0, 760.0])
            .with_min_inner_size([880.0, 600.0])
            .with_icon(Arc::new(window_icon)),
        centered: true,
        ..Default::default()
    };
    eframe::run_native(title, options, app)
}
