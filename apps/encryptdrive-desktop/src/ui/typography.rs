//! Named, shared text sizes following the Windows 11 type ramp
//! (Caption 12, Body 14, Body Strong 14, Body Large 18, Subtitle 20, Title 28).
//! Semibold is reserved for titles and emphasis; nothing smaller than 12 px is used.

use eframe::egui::{FontFamily, FontId};

pub(crate) struct TypeScale;

#[cfg_attr(not(feature = "design-preview"), allow(dead_code))]
impl TypeScale {
    /// Product name in the header (Body Strong).
    pub(crate) const APP_IDENTITY: f32 = 14.0;
    /// Page header / breadcrumb and dialog titles (Subtitle).
    pub(crate) const PAGE_TITLE: f32 = 20.0;
    /// Empty-state headings (Body Large Strong).
    pub(crate) const SECTION_HEADING: f32 = 18.0;
    pub(crate) const BODY: f32 = 14.0;
    pub(crate) const CONTROL: f32 = 14.0;
    /// Table metadata cells and column headers (Caption).
    pub(crate) const METADATA: f32 = 12.0;
    pub(crate) const HELPER: f32 = 12.0;
}

pub(crate) fn semibold_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("encryptdrive-ui-semibold".into()))
}

#[cfg(test)]
mod tests {
    use super::TypeScale;

    #[test]
    fn semantic_text_roles_follow_the_windows_type_ramp() {
        assert_eq!(TypeScale::APP_IDENTITY, 14.0);
        assert_eq!(TypeScale::PAGE_TITLE, 20.0);
        assert_eq!(TypeScale::SECTION_HEADING, 18.0);
        assert_eq!(TypeScale::BODY, 14.0);
        assert_eq!(TypeScale::CONTROL, 14.0);
        assert_eq!(TypeScale::METADATA, 12.0);
        assert_eq!(TypeScale::HELPER, 12.0);
    }

    #[test]
    fn semibold_uses_a_distinct_system_font_family_and_explicit_size() {
        let font = super::semibold_font(TypeScale::PAGE_TITLE);
        assert_eq!(font.size, TypeScale::PAGE_TITLE);
        assert_eq!(
            font.family,
            eframe::egui::FontFamily::Name("encryptdrive-ui-semibold".into())
        );
    }
}
