//! Semantic colors shared by both prototypes. Values follow the Phase 0.5 tokens.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThemeMode {
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Appearance {
    Light,
    Dark,
    System,
}

impl Appearance {
    pub fn resolved(self, system_dark: bool) -> ThemeMode {
        match self {
            Self::Light => ThemeMode::Light,
            Self::Dark => ThemeMode::Dark,
            Self::System if system_dark => ThemeMode::Dark,
            Self::System => ThemeMode::Light,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SemanticTokens {
    pub canvas: u32,
    pub surface: u32,
    pub surface_secondary: u32,
    pub hover: u32,
    pub selection: u32,
    pub selection_text: u32,
    pub primary_text: u32,
    pub secondary_text: u32,
    pub primary_action: u32,
    pub on_primary_action: u32,
    pub focus: u32,
    pub outline: u32,
    pub separator: u32,
    pub error: u32,
    pub error_surface: u32,
    pub success: u32,
    pub success_surface: u32,
    pub warning: u32,
    pub warning_surface: u32,
}

impl SemanticTokens {
    pub const fn for_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Light => Self {
                canvas: 0xF7F8FA,
                surface: 0xFFFFFF,
                surface_secondary: 0xEEF1F4,
                hover: 0xF1F4F7,
                selection: 0xE6F0FC,
                selection_text: 0x183C66,
                primary_text: 0x18212B,
                secondary_text: 0x596573,
                primary_action: 0x155EEF,
                on_primary_action: 0xFFFFFF,
                focus: 0x155EEF,
                outline: 0x7D8995,
                separator: 0xE0E4E8,
                error: 0xB42318,
                error_surface: 0xFCE8E7,
                success: 0x166534,
                success_surface: 0xE8F5EC,
                warning: 0x714000,
                warning_surface: 0xFFF2D1,
            },
            ThemeMode::Dark => Self {
                canvas: 0x1A1D21,
                surface: 0x242930,
                surface_secondary: 0x2C333B,
                hover: 0x2E353E,
                selection: 0x254A73,
                selection_text: 0xD6E8FF,
                primary_text: 0xF2F4F7,
                secondary_text: 0xB5C0CB,
                primary_action: 0x155EEF,
                on_primary_action: 0xFFFFFF,
                focus: 0x91BEFF,
                outline: 0x707D89,
                separator: 0x3B444E,
                error: 0xB42318,
                error_surface: 0x3A1F20,
                success: 0x91D7A7,
                success_surface: 0x1D3427,
                warning: 0xFFD28A,
                warning_surface: 0x3B2E17,
            },
        }
    }
}
