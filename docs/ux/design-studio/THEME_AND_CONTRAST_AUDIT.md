# UX3 Theme and Contrast Audit

**Status:** Automated semantic color-pair checks PASS; rendered, OS integration, and accessibility review remain NOT RUN.

## Implementation

- `AppearancePreference::System` is the default; Light and Dark can be selected. The selected preference is distinct from the resolved appearance.
- eframe's system theme preference and native-window theme synchronization are enabled. If the host does not report a theme, the code's documented fallback is Dark.
- Both themes use `SemanticTokens` in `apps/encryptdrive-desktop/src/lib.rs`. `ui/theme.rs` applies the values to default widget roles and interaction states.
- Theme choice is session state; persistent settings are not implemented.

## Automated evidence

`cargo test -p encryptdrive-desktop --features design-preview --locked` passed on 2026-10-19, including `light_and_dark_semantic_pairs_meet_text_and_component_contrast_targets`. That test checks 46 semantic pair instances across both themes: primary and secondary text, icons, outlines and focus against canvas/surface/secondary surfaces; selected text/icon and focus against selection; icon against hover; action foreground against action fill; and danger/warning/success foregrounds against the main surface. Textual pairs use a 4.5:1 threshold; non-text component pairs use 3:1.

This is a test of source token values, not rendered pixel sampling. It does not cover every disabled state, every contextual feedback/background pairing, forced colors, or contrast in the actual title bar. The application was not launched for this audit.

## Required manual checks

1. Follow System while Windows switches Light/Dark; confirm both client area and native title bar.
2. Force Light, Dark, then System and review all states: idle, hover, focus, selected, disabled, validation error, progress, success, warning, and destructive confirmation.
3. Repeat at 100%, 150%, and 200% scaling, with Windows High Contrast/Contrast Themes enabled.
4. Inspect using Windows Accessibility Insights and Narrator. Record the actual UIA tree, speech, focus order, and any missed state.
