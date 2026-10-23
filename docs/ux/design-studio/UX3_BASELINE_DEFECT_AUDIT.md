# UX3 baseline defect audit

**Audit date:** 2026-10-19  
**Baseline:** existing Phase 1A-UX source candidate before UX3 changes  
**Evidence:** UI source, existing design-studio records, owner-reported defects, and one independent model's read-only source review.  
**Visual status:** NOT RUN. No owner screenshot is present in this repository and GUI interaction is explicitly stopped. Source findings below are not rendered observations.

## Repository and implementation scope

- Rust repository: unborn `main` branch; all existing project paths are untracked. Preserve all files and history; no commit, reset, clean, or stage operation is authorized.
- Private documentation repository: pre-existing Phase 0.6A modified and untracked files; treated as read-only.
- Java repository: existing branch with untracked `target_test-classes/`; treated as read-only.
- Normal application launch goes through `apps/encryptdrive-desktop/src/shell.rs`; `ui/*` is a separate `design-preview` feature path. Fixes limited to the preview would leave the ordinary application unchanged.
- Rust `1.99.1`, Cargo `1.99.1`, target `x86_64-pc-windows-msvc`; workspace pins eframe/egui `0.37.0`.

## Confirmed source defects

| Severity | Screen/component | Reproducible source observation | Expected behavior / correction | Verification |
|---|---|---|---|---|
| P1 | System appearance (`src/lib.rs`, `src/shell.rs`, `src/ui/app.rs`) | `ShellState::new()` selects Light. The only preference values are Light/Dark, and each UI path calls `ctx.set_theme` with a fixed theme. | Store System/Light/Dark preference separately from the resolved theme; initialize and display System; use egui's system preference and synchronized native window theme; use a documented non-Light fallback if detection is unavailable. | Pure resolution tests; source/API inspection; owner verifies Windows app mode changes and title bar live. |
| P1 | Branding (`src/shell.rs`, `src/ui/components.rs`) | Both application paths draw an original geometric animal placeholder; neither uses the legacy product logo. | Use the exact legacy husky artwork, preserve transparency/aspect ratio, show it in the header and native window, and document the source and redistribution scope. | Asset hash/pixel metadata; inspect genuine header and title-bar captures at 16/32/48 px on both backgrounds. |
| P1 | Typography (all `src/ui/*`, `src/shell.rs`) | The code specifies unrelated font sizes from 9 to 23 logical points; there is no shared named typography API. | Use a small semantic scale with consistent family, weight and line height; keep ordinary interactive/metadata text legible; document genuine exceptions. | Search source for per-control size overrides; token tests; owner checks a rendered specimen and Unicode fallback. |
| P1 | Color semantics (`src/ui/theme.rs`, `src/lib.rs`) | Border uses the decorative separator token. Source-value contrast is 1.28:1 for Light separator/surface and 1.48:1 for Dark. Dark `#155EEF` action color also appears on icons/feedback at 2.70:1 on the surface and 1.68:1 on the selection fill. | Separate outlines, separators, text/icon accent and primary-action fill/foreground; test every meaningful token pair against contrast targets. | Automated contrast tests over actual token values; rendered theme review remains owner-only. |
| P1 | Icons and action hierarchy (`src/ui/components.rs`, `src/ui/navigation.rs`, `src/ui/explorer.rs`, `src/shell.rs`) | Files/Trash/folder/account state use mixed Unicode glyphs; navigation includes multiple simultaneous selection markers; buttons embed plus and arrow characters; sort headers are raised buttons with Unicode arrows. | Use a single redistributable icon family and semantic map; standardize 16/20 logical px icons and hit boxes; remove decorative symbols from button text; distinguish primary, secondary, destructive and disabled actions. | Icon-map/source audit plus genuine Light/Dark idle/hover/focus/selected/disabled screenshots. |
| P1 | Filtered selection (`src/preview/state.rs`, `src/ui/explorer.rs`) | Query updates preserve stable selected IDs while toolbar and destructive dialogs act on the entire selection; the UI displays a small warning that some selected entries can be hidden by search. | Clear selection only when the effective query changes, preserve it across sorting and repeated same-query frames, and make destructive commands operate only on displayed/explicitly selected IDs. | Model tests for query change, unchanged query, Ctrl+A, sort, filter, and destructive target set. |
| P1 | Table focus and selection (`src/ui/explorer.rs`) | Focused row is painted as hovered; the name cell paints another hover fill; an outside painter draws an outline around the union of row/cell rectangles, which can cross table columns. | Keep hover, selected and keyboard-focus states distinct; paint focus within a clipped row-level indicator that does not cross cell separators or text. | Source/model tests; owner screenshot check after pointer/keyboard changes, sorting, resize and scrolling. |
| P1 | Keyboard targeting (`src/ui/app.rs`, `src/preview/state.rs`) | Arrow keys move focus without selection; F2 uses the first selected ID while Enter acts on the focused ID; Shift+Arrow does not extend selection. | Make command target rules consistent and visible, keep focus and selection distinct, support documented Shift-range keyboard behavior, and name the item affected by dialogs. | Tests for focus/selection/action-target state and owner keyboard-only review. |
| P1 | Representative settings/forms (`src/ui/screens.rs`) | Settings use checkboxes bound to temporary `false` values, so the controls cannot retain a change. Submit feedback is a frame-local variable and disappears on the next repaint. | Persist illustrative settings/feedback in preview state for the session or disable controls with an honest explanation. Make validation and feedback stable across frames. | UI state tests; owner keyboard/form validation review. |
| P1 risk, visual/runtime NOT RUN | Responsive layout and truncation (`src/ui/explorer.rs`, `src/ui/dialogs.rs`, `src/ui/navigation.rs`) | Several command/status rows are non-wrapping; filename truncation is character-count-based; fixed form widths and dialog widths can outgrow narrow layouts at larger scaling. | Define compact responsive overflow/wrap rules and truncate to actual available width while retaining a full accessible name/tooltip. | Owner checks 960×640, 1180×792, 1600×900, 1920×1080 and Windows 100/150/200% scaling. |
| P1 risk, accessibility NOT RUN | Custom navigation, rows, dialogs and forms | Some custom widgets provide AccessKit metadata, but source alone does not establish correct UIA roles, labels, focus containment/return, error announcements, or Narrator output. | Prefer standard egui widgets, add explicit truthful semantics for custom rows/navigation, and verify UIA/Narrator. | Windows Accessibility Insights/UIA and manual Narrator transcript. |
| P2 | Redundant preview copy (`src/ui/app.rs`, `src/ui/navigation.rs`, `src/ui/explorer.rs`, `src/ui/screens.rs`) | Header, sidebar, picker, table and footer independently repeat synthetic/offline/preview statements. | Keep one discreet persistent `Design preview · Sample data` indicator near preview controls; preserve only contextual safety disclosures. | Central copy catalog and source search; owner checks density. |
| P2 | Sort/table hierarchy (`src/ui/explorer.rs`) | Column labels are styled as buttons; status/metadata vary in size; numeric values use ordinary left-aligned labels. | Use consistent plain sortable headings, one secondary-text style and right-aligned numeric values. | Source review plus populated/empty/selected visual evidence. |

## Owner-reported visual defect

The owner reports that the current table selection/focus outline crosses column boundaries. This is consistent with the row union rectangle being stroked by the outer painter in `ui/explorer.rs`, but the original screenshot is not available in this repository. The defect is therefore recorded as **owner-reported plus source-correlated**, not independently visually confirmed.

## Brand provenance baseline

The legacy source image is `EncryptDrive/src/main/resources/com/fabianrodas/images/logo.png`, shown in the Java README as the EncryptDrive logo. The Java README ends with `Copyright © 2026 Fabián Rodas. All rights reserved.` and the repository has no product-level open-source license. The user's explicit instruction for this phase authorizes reuse in this product repository; this is not evidence of a general public asset license. The destination must keep the artwork's proprietary status clear and must not imply that the Rust repository's future software license covers this image.

## Prior review and evidence correction

The prior `CRITIC_REVIEW.md` reports a final source-only `0 P0 / 0 P1 / 0 P2` review. The owner rejected the visual candidate after opening the app. That source-only result is not visual approval and is not used as the UX3 baseline acceptance evidence.

An independent separate-model critic performed an additional source-only pass before reading internal critic findings. It independently confirmed the separate default-shell path, Light-only theme, absent legacy logo, low-contrast outlines/accent, typography/icon issues, hidden-selection behavior, table focus drawing, nonpersistent sample settings/feedback, and keyboard focus/selection target ambiguity. It made no visual finding and gave no approval because live evidence was unavailable.

## Material specification inconsistencies to preserve

- The owner now requires System as the appearance default; the old design decision register records Light/Dark only and defers System. UX3 supersedes that old candidate choice.
- The prior Phase 1A source contract says selection persists through filtering; UX3 explicitly authorizes the safer simpler policy of clearing selection when the query changes. UX3 supersedes the prior behavior.
- The prior review's 0-defect statement was source-only; it cannot establish appearance, accessibility, high-contrast, DPI, or owner acceptance.
- High Contrast is separate from ordinary Light/Dark. eframe/winit can report a resolved theme but this app's custom-paint palette does not automatically map to Windows Contrast Themes; support is unverified and must be documented honestly.

## Verification boundary

No running UI, screenshot, screen recording, UI Automation tree, Narrator output, High Contrast session, system-theme transition, or live DPI/text-scaling result was available at baseline. Owner-run visual evidence remains required. No real vault, file, password, encrypted object, or private user data was used by this audit.
