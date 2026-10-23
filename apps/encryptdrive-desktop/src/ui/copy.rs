//! Shared product and preview copy to keep repeated labels consistent.

#[cfg(feature = "design-preview")]
pub(crate) const PREVIEW_MARKER: &str = "Design preview · Sample data";
pub(crate) const OFFLINE_STATUS: &str = "Local only";

#[cfg(test)]
mod tests {
    use super::OFFLINE_STATUS;
    #[cfg(feature = "design-preview")]
    use super::PREVIEW_MARKER;

    #[test]
    fn preview_and_offline_status_are_explicit_about_their_context() {
        #[cfg(feature = "design-preview")]
        assert!(PREVIEW_MARKER.contains("Sample data"));
        assert!(OFFLINE_STATUS.contains("Local"));
    }
}
