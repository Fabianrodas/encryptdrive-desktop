use std::sync::Arc;

use eframe::egui::{self, FontData, FontFamily, FontId, RichText};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(feature = "design-preview"), allow(dead_code))]
pub(crate) enum Icon {
    Files,
    File,
    Trash,
    Folder,
    NewFolder,
    Search,
    Rename,
    Import,
    Export,
    Restore,
    Back,
    Up,
    Settings,
    Account,
    #[cfg_attr(not(feature = "design-preview"), allow(dead_code))]
    Lock,
    Unlock,
    Warning,
    Info,
    Error,
    Success,
    SortAscending,
    SortDescending,
    Close,
    ChevronRight,
    ShieldCheck,
    More,
}

impl Icon {
    #[cfg(test)]
    pub(crate) const ALL: [Self; 26] = [
        Self::Files,
        Self::File,
        Self::Trash,
        Self::Folder,
        Self::NewFolder,
        Self::Search,
        Self::Rename,
        Self::Import,
        Self::Export,
        Self::Restore,
        Self::Back,
        Self::Up,
        Self::Settings,
        Self::Account,
        Self::Lock,
        Self::Unlock,
        Self::Warning,
        Self::Info,
        Self::Error,
        Self::Success,
        Self::SortAscending,
        Self::SortDescending,
        Self::Close,
        Self::ChevronRight,
        Self::ShieldCheck,
        Self::More,
    ];

    pub(crate) fn glyph(self) -> char {
        char::from_u32(match self {
            Self::Files => 0xE710,
            Self::File => 0xE230,
            Self::Trash => 0xE4A6,
            Self::Folder => 0xE24A,
            Self::NewFolder => 0xE258,
            Self::Search => 0xE30C,
            Self::Rename => 0xE3B4,
            Self::Import => 0xE4C0,
            Self::Export => 0xE20C,
            Self::Restore => 0xE024,
            Self::Back => 0xE058,
            Self::Up => 0xE08E,
            Self::Settings => 0xE270,
            Self::Account => 0xE4C2,
            Self::Lock => 0xE2FA,
            Self::Unlock => 0xE306,
            Self::Warning => 0xE4E0,
            Self::Info => 0xE2CE,
            Self::Error => 0xE4F8,
            Self::Success => 0xE184,
            Self::SortAscending => 0xE13C,
            Self::SortDescending => 0xE136,
            Self::Close => 0xE4F6,
            Self::ChevronRight => 0xE13A,
            Self::ShieldCheck => 0xE40C,
            // Phosphor `dots-three` (horizontal ellipsis).
            Self::More => 0xE1FE,
        })
        .expect("all named Phosphor codepoints are valid Unicode scalars")
    }

    /// Only for icon atoms inside widgets that set their own accessible name.
    #[cfg_attr(not(feature = "design-preview"), allow(dead_code))]
    pub(crate) fn rich_text(self, color: egui::Color32, size: f32) -> RichText {
        RichText::new(self.glyph())
            .font(FontId::new(
                size,
                FontFamily::Name(Arc::from("encryptdrive-icons")),
            ))
            .color(color)
    }
}

pub(crate) fn install_font(ctx: &egui::Context) {
    let mut definitions = egui::FontDefinitions::default();

    // Use fonts already installed by Windows. They remain outside the repository and the
    // executable, so this does not redistribute Segoe UI. Other platforms retain egui's
    // bundled fallback fonts.
    #[cfg(target_os = "windows")]
    let (regular, semibold) = install_windows_ui_fonts(&mut definitions);
    #[cfg(not(target_os = "windows"))]
    let (regular, semibold): (Option<String>, Option<String>) = (None, None);

    let proportional = definitions
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    if let Some(regular) = regular.as_ref() {
        let mut family = vec![regular.clone()];
        family.extend(proportional.iter().filter(|name| *name != regular).cloned());
        definitions
            .families
            .insert(FontFamily::Proportional, family);
    }
    let strong_family = FontFamily::Name(Arc::from("encryptdrive-ui-semibold"));
    let strong_font = semibold
        .or(regular)
        .or_else(|| proportional.first().cloned());
    if let Some(strong_font) = strong_font {
        definitions
            .families
            .insert(strong_family, vec![strong_font]);
    }

    let font_name = "encryptdrive-phosphor-regular".to_owned();
    definitions.font_data.insert(
        font_name.clone(),
        Arc::new(FontData::from_static(include_bytes!(
            "../../assets/icons/Phosphor-Regular.ttf"
        ))),
    );
    let family = FontFamily::Name(Arc::from("encryptdrive-icons"));
    let mut fallbacks = vec![font_name];
    fallbacks.extend(
        definitions.families[&FontFamily::Proportional]
            .iter()
            .cloned(),
    );
    definitions.families.insert(family, fallbacks);
    ctx.set_fonts(definitions);
}

#[cfg(target_os = "windows")]
fn install_windows_ui_fonts(
    definitions: &mut egui::FontDefinitions,
) -> (Option<String>, Option<String>) {
    let Some(windows_directory) = std::env::var_os("WINDIR") else {
        return (None, None);
    };
    let fonts_directory = std::path::PathBuf::from(windows_directory).join("Fonts");
    let regular = load_system_font(
        definitions,
        &fonts_directory,
        "segoeui.ttf",
        "encryptdrive-segoe-ui",
    );
    let semibold = load_system_font(
        definitions,
        &fonts_directory,
        "seguisb.ttf",
        "encryptdrive-segoe-ui-semibold",
    )
    .or_else(|| {
        load_system_font(
            definitions,
            &fonts_directory,
            "segoeuib.ttf",
            "encryptdrive-segoe-ui-bold",
        )
    });
    (regular, semibold)
}

#[cfg(target_os = "windows")]
fn load_system_font(
    definitions: &mut egui::FontDefinitions,
    fonts_directory: &std::path::Path,
    filename: &str,
    font_name: &str,
) -> Option<String> {
    let bytes = std::fs::read(fonts_directory.join(filename)).ok()?;
    definitions
        .font_data
        .insert(font_name.to_owned(), Arc::new(FontData::from_owned(bytes)));
    Some(font_name.to_owned())
}

#[cfg(test)]
mod tests {
    use super::Icon;

    #[test]
    fn semantic_icons_map_to_unique_private_use_glyphs() {
        let glyphs = Icon::ALL.map(Icon::glyph);
        for (index, glyph) in glyphs.iter().enumerate() {
            assert!(*glyph as u32 >= 0xE000 && *glyph as u32 <= 0xF8FF);
            assert!(!glyphs[index + 1..].contains(glyph));
        }
    }

    #[test]
    fn unlock_uses_the_phosphor_lock_open_glyph() {
        assert_eq!(Icon::Unlock.glyph() as u32, 0xE306);
    }
}
