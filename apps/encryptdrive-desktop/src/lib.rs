//! Native-shell state and semantic visual tokens, independent of eframe widgets.

#[cfg(any(feature = "design-preview", test))]
pub mod preview;

/// The top-level destination displayed by the shell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Destination {
    /// Root of the active vault's file listing.
    Files,
    /// Deleted-entry listing.
    Trash,
}

/// Resolved application appearance used to choose semantic tokens.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Appearance {
    /// Light semantic palette.
    Light,
    /// Dark semantic palette.
    Dark,
}

/// User preference, separate from the resolved Light/Dark application appearance.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AppearancePreference {
    /// Follow the current operating-system application theme.
    #[default]
    System,
    /// Use the Light appearance for this session.
    Light,
    /// Use the Dark appearance for this session.
    Dark,
}

impl AppearancePreference {
    /// Resolves the preference, using Dark when the host cannot report a system theme.
    pub const fn resolve(self, system_appearance: Option<Appearance>) -> Appearance {
        match self {
            Self::System => match system_appearance {
                Some(appearance) => appearance,
                None => Appearance::Dark,
            },
            Self::Light => Appearance::Light,
            Self::Dark => Appearance::Dark,
        }
    }
}

/// Session view state presented by the shell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionView {
    /// No vault is opened or inspected by this engineering preview.
    NoVaultOpen,
}

/// Minimal, framework-independent view state for the native shell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShellState {
    /// Current Files or Trash destination.
    pub destination: Destination,
    /// User preference; System is the initial choice.
    pub appearance: AppearancePreference,
    /// Current vault/session view.
    pub session: SessionView,
    /// Whether the in-memory Settings window is visible.
    pub settings_open: bool,
}

impl ShellState {
    /// Creates the honest, empty initial state without touching user data.
    pub const fn new() -> Self {
        Self {
            destination: Destination::Files,
            appearance: AppearancePreference::System,
            session: SessionView::NoVaultOpen,
            settings_open: false,
        }
    }

    /// Resolves the selected appearance to its semantic palette.
    pub const fn tokens(self, appearance: Appearance) -> SemanticTokens {
        let mode = match appearance {
            Appearance::Light => ThemeMode::Light,
            Appearance::Dark => ThemeMode::Dark,
        };
        SemanticTokens::for_mode(mode)
    }
}

impl Default for ShellState {
    fn default() -> Self {
        Self::new()
    }
}

/// Semantic palette mode; values are adapted from the Phase 0.5 UX token set.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThemeMode {
    /// Light appearance.
    Light,
    /// Dark appearance.
    Dark,
}

/// Colors used by shell surfaces, text, selection, focus, and separators.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SemanticTokens {
    /// Window and page background.
    pub canvas: u32,
    /// Header, sidebar, and table surface.
    pub surface: u32,
    /// Secondary surface for subtle group backgrounds.
    pub surface_secondary: u32,
    /// Hover fill for interactive rows and controls.
    pub hover: u32,
    /// Selection fill for active navigation and rows.
    pub selection: u32,
    /// Primary text on selection fill.
    pub selection_text: u32,
    /// Leading marker that exposes selection without relying on the fill alone.
    pub selection_indicator: u32,
    /// Main text color.
    pub primary_text: u32,
    /// Muted supporting text.
    pub secondary_text: u32,
    /// Primary action color reserved for future enabled actions.
    pub primary_action: u32,
    /// Text color on the primary action color.
    pub on_primary_action: u32,
    /// Keyboard focus ring color.
    pub focus: u32,
    /// Foreground color for ordinary semantic icons.
    pub icon: u32,
    /// Control outline color.
    pub outline: u32,
    /// Table and panel separator color.
    pub separator: u32,
    /// Destructive-state foreground.
    pub danger: u32,
    /// Warning-state foreground.
    pub warning: u32,
    /// Success-state foreground.
    pub success: u32,
}

impl SemanticTokens {
    /// Returns the semantic palette for a theme mode, aligned with Windows 11 Fluent defaults
    /// (accent `#005FB8` / `#60CDFF`, neutral Mica-like canvases).
    pub const fn for_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Light => Self {
                canvas: 0xF3F3F3,
                surface: 0xFFFFFF,
                surface_secondary: 0xF9F9F9,
                hover: 0xF0F0F0,
                selection: 0xE6EEF8,
                selection_text: 0x1B1B1B,
                selection_indicator: 0x005FB8,
                primary_text: 0x1B1B1B,
                secondary_text: 0x5C5C5C,
                primary_action: 0x005FB8,
                on_primary_action: 0xFFFFFF,
                focus: 0x1B1B1B,
                icon: 0x424242,
                outline: 0x868686,
                separator: 0xE3E3E3,
                danger: 0xC42B1C,
                warning: 0x8A5300,
                success: 0x0F7B0F,
            },
            ThemeMode::Dark => Self {
                canvas: 0x202020,
                surface: 0x2B2B2B,
                surface_secondary: 0x323232,
                hover: 0x383838,
                selection: 0x2E3A47,
                selection_text: 0xFFFFFF,
                selection_indicator: 0x60CDFF,
                primary_text: 0xFFFFFF,
                secondary_text: 0xC8C8C8,
                primary_action: 0x60CDFF,
                on_primary_action: 0x000000,
                focus: 0xFFFFFF,
                icon: 0xD6D6D6,
                outline: 0x8E8E8E,
                separator: 0x3B3B3B,
                danger: 0xFF99A4,
                warning: 0xFCE100,
                success: 0x6CCB5F,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Appearance, AppearancePreference, Destination, SemanticTokens, SessionView, ShellState,
        ThemeMode,
    };

    #[test]
    fn shell_starts_on_files_without_an_open_vault() {
        let state = ShellState::new();

        assert_eq!(state.destination, Destination::Files);
        assert_eq!(state.appearance, AppearancePreference::System);
        assert_eq!(state.session, SessionView::NoVaultOpen);
    }

    #[test]
    fn system_appearance_resolves_from_the_current_application_theme() {
        assert_eq!(
            AppearancePreference::System.resolve(Some(Appearance::Light)),
            Appearance::Light
        );
        assert_eq!(
            AppearancePreference::System.resolve(Some(Appearance::Dark)),
            Appearance::Dark
        );
    }

    #[test]
    fn explicit_appearance_overrides_ignore_the_system_theme() {
        assert_eq!(
            AppearancePreference::Light.resolve(Some(Appearance::Dark)),
            Appearance::Light
        );
        assert_eq!(
            AppearancePreference::Dark.resolve(Some(Appearance::Light)),
            Appearance::Dark
        );
    }

    #[test]
    fn unknown_system_appearance_uses_the_documented_dark_fallback() {
        assert_eq!(AppearancePreference::System.resolve(None), Appearance::Dark);
    }

    #[test]
    fn default_shell_state_matches_the_safe_initial_state() {
        assert_eq!(ShellState::default(), ShellState::new());
    }

    #[test]
    fn semantic_tokens_resolve_to_the_windows_light_palette() {
        let tokens = SemanticTokens::for_mode(ThemeMode::Light);

        assert_eq!(tokens.canvas, 0xF3F3F3);
        assert_eq!(tokens.surface, 0xFFFFFF);
        assert_eq!(tokens.primary_text, 0x1B1B1B);
        assert_eq!(tokens.primary_action, 0x005FB8);
        assert_eq!(tokens.selection_indicator, 0x005FB8);
    }

    #[test]
    fn semantic_tokens_resolve_to_the_windows_dark_palette() {
        let tokens = SemanticTokens::for_mode(ThemeMode::Dark);

        assert_eq!(tokens.canvas, 0x202020);
        assert_eq!(tokens.surface, 0x2B2B2B);
        assert_eq!(tokens.primary_text, 0xFFFFFF);
        assert_eq!(tokens.primary_action, 0x60CDFF);
        assert_eq!(tokens.on_primary_action, 0x000000);
        assert_eq!(tokens.selection_indicator, 0x60CDFF);
        assert_eq!(tokens.focus, 0xFFFFFF);
    }
}
