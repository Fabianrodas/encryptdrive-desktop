# UX3 Iconography Audit

**Status:** Source mapping and glyph uniqueness tests PASS. Visual rendering, high-DPI legibility, UI Automation, and Narrator are NOT RUN.

## Source and usage

The preview embeds `apps/encryptdrive-desktop/assets/icons/Phosphor-Regular.ttf` (488,636 bytes; SHA-256 `06B91E022B7EE899A63EFCED879392A74F0BACBDA54E4467E9F663220D173A10`) and maps semantic `Icon` variants in `apps/encryptdrive-desktop/src/ui/icons.rs`. The font file was copied from the `egui-phosphor` 0.14.0 package resource; it is embedded and does not require a runtime package download. `apps/encryptdrive-desktop/assets/licenses/PHOSPHOR_LICENSE.txt` carries the MIT notice. Product redistribution still needs a complete dependency and attribution review.

The app uses named semantic icons for files, folders, destinations, commands, status, and sorting. PUA glyphs are not intended as accessible names. Custom icon+label controls assign a human-readable Button label; rows, sort buttons, navigation controls, search, and logo receive source-level widget metadata. None of those annotations proves the Windows UIA tree or Narrator speech.

See the full 23-entry [icon mapping](ICON_MAPPING.md) and [brand asset provenance](BRAND_ASSET_PROVENANCE.md). The product's husky mark is separate from this icon font.

## Open verification items

- Inspect every glyph in Light and Dark at 100%, 150%, and 200% scaling.
- Confirm symbol alignment and fallback for the complete fixture, including Unicode filenames.
- Inspect accessible Name, Role, Focused, Selected, and Enabled values using Windows Accessibility Insights or equivalent; then run Narrator manually.
- Review dependency and attribution obligations with the product owner before redistribution.
