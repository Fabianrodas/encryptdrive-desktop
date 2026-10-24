//! Shared spacing and control dimensions in logical pixels.
//!
//! Values follow the Windows 11 Fluent geometry: a 4 px grid, 32 px standard controls,
//! 4 px radius for in-page controls and 8 px for cards, dialogs and flyouts.

pub(crate) struct Geometry;

#[cfg_attr(not(feature = "design-preview"), allow(dead_code))]
impl Geometry {
    pub(crate) const BASE: f32 = 4.0;
    pub(crate) const SPACE_2: f32 = 8.0;
    pub(crate) const SPACE_3: f32 = 12.0;
    pub(crate) const SPACE_4: f32 = 16.0;
    pub(crate) const SPACE_6: f32 = 24.0;
    pub(crate) const SPACE_8: f32 = 32.0;
    pub(crate) const BRAND_MARK_SIZE: f32 = 28.0;
    pub(crate) const HEADER_HEIGHT: f32 = 48.0;
    pub(crate) const STATUS_BAR_HEIGHT: f32 = 36.0;
    pub(crate) const SIDEBAR_WIDTH: f32 = 220.0;
    /// Narrow windows trade sidebar width for content (still fits "Files" / "Trash").
    pub(crate) const SIDEBAR_WIDTH_NARROW: f32 = 184.0;
    /// Window width (logical points) at which the sidebar uses its full width.
    pub(crate) const SIDEBAR_BREAKPOINT: f32 = 1100.0;
    pub(crate) const CONTROL_HEIGHT: f32 = 32.0;
    /// Secondary chrome (status bar): Fluent compact sizing.
    pub(crate) const COMPACT_CONTROL_HEIGHT: f32 = 28.0;
    pub(crate) const CHIP_HEIGHT: f32 = 24.0;
    pub(crate) const SEARCH_MIN_WIDTH: f32 = 160.0;
    pub(crate) const SEARCH_MAX_WIDTH: f32 = 320.0;
    pub(crate) const NAVIGATION_ROW_HEIGHT: f32 = 36.0;
    pub(crate) const NAV_ICON_CENTER: f32 = 24.0;
    pub(crate) const NAV_LABEL_INSET: f32 = 44.0;
    /// Fluent selection "pill": a short rounded accent bar on the leading edge.
    pub(crate) const SELECTION_PILL_WIDTH: f32 = 3.0;
    pub(crate) const SELECTION_PILL_HEIGHT: f32 = 16.0;
    pub(crate) const TABLE_ROW_HEIGHT: f32 = 36.0;
    pub(crate) const TABLE_HEADER_HEIGHT: f32 = 32.0;
    pub(crate) const DIALOG_WIDTH: f32 = 440.0;
    pub(crate) const CONTROL_RADIUS: u8 = 4;
    pub(crate) const SURFACE_RADIUS: u8 = 8;
    pub(crate) const ICON_SIZE: f32 = 16.0;
    pub(crate) const MAJOR_ICON_SIZE: f32 = 20.0;
    pub(crate) const HERO_ICON_SIZE: f32 = 40.0;
}

#[cfg(test)]
mod tests {
    use super::Geometry;

    #[test]
    fn component_dimensions_follow_the_four_point_grid_and_readable_targets() {
        for value in [
            Geometry::SPACE_2,
            Geometry::SPACE_3,
            Geometry::SPACE_4,
            Geometry::SPACE_6,
            Geometry::SPACE_8,
            Geometry::CONTROL_HEIGHT,
            Geometry::COMPACT_CONTROL_HEIGHT,
            Geometry::CHIP_HEIGHT,
            Geometry::NAVIGATION_ROW_HEIGHT,
            Geometry::TABLE_ROW_HEIGHT,
            Geometry::TABLE_HEADER_HEIGHT,
            Geometry::HEADER_HEIGHT,
            Geometry::STATUS_BAR_HEIGHT,
            Geometry::SIDEBAR_WIDTH,
            Geometry::SIDEBAR_WIDTH_NARROW,
            Geometry::SEARCH_MIN_WIDTH,
            Geometry::SEARCH_MAX_WIDTH,
            Geometry::DIALOG_WIDTH,
        ] {
            assert_eq!(value % Geometry::BASE, 0.0);
        }
        assert_eq!(Geometry::CONTROL_HEIGHT, 32.0);
        assert_eq!(Geometry::TABLE_ROW_HEIGHT, 36.0);
        assert_eq!(Geometry::CONTROL_RADIUS, 4);
        assert_eq!(Geometry::SURFACE_RADIUS, 8);
        assert_eq!(Geometry::ICON_SIZE, 16.0);
        assert_eq!(Geometry::MAJOR_ICON_SIZE, 20.0);
    }
}
