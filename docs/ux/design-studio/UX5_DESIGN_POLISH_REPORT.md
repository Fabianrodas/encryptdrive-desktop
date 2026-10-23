# UX5 — Design Polish and Proportions Report

**Date:** 2026-10-23
**Scope:** EncryptDrive Desktop (Rust / eframe 0.37): the production shell and the synthetic design preview.
**Status:** Implemented and verified with automated tests and genuine Release captures in Light and Dark.
- Narrator, UI Automation, High Contrast and 150/200% DPI were **not** run (see section 8).
- This report does not grant final owner approval.

## 1. Goal

Fix every design defect found in the R1–R4 reviews, make proportions consistent everywhere, and reach a polished Windows 11 look. That meant redesigning where needed, based on how Windows 11 Explorer and Fluent apps are built.

## 2. Research basis

| Source | What was adopted |
|---|---|
| Microsoft Learn, [Geometry in Windows 11](https://learn.microsoft.com/en-us/windows/apps/design/signature-experiences/geometry) | **4 px** radius for in-page controls; **8 px** for dialogs, flyouts, menus and content layers. |
| Microsoft Learn, [Content layout and spacing](https://learn.microsoft.com/en-us/windows/apps/design/basics/content-basics) | **8 px** between buttons and between a control and its header; **12 px** between content areas; **16 px** from a surface's edge to its text. |
| Microsoft Learn, [Typography in Windows](https://learn.microsoft.com/en-us/windows/apps/design/signature-experiences/typography) | Type ramp: Caption 12/16, Body 14/20, Body Strong 14 semibold, Body Large 18, Subtitle 20 semibold, Title 28. Semibold for titles only, sentence case everywhere, no uppercase labels, 12 px minimum. |
| Windows 11 Explorer (reference product) | Breadcrumb with Back/Up on the left; stable command bar; search box on the right; folders listed before files; descriptive types ("Text document"); KB rounded up; status bar with item count. |
| Cryptomator desktop docs ([accessing vaults](https://docs.cryptomator.org/desktop/accessing-vaults/)) | A centred "unlock vault" card as a separate, focused step. |
| WinUI defaults (Fluent) | Accent `#005FB8` (Light) / `#60CDFF` with black text (Dark); a rounded "pill" selection indicator; subtle (transparent) command buttons; ContentDialog with a tinted footer and two equal-width buttons. |

## 3. New design system

### 3.1 Colours (`src/lib.rs`, `SemanticTokens`)

| Role | Light | Dark |
|---|---|---|
| Canvas (window, header, sidebar, status bar) | `#F3F3F3` | `#202020` |
| Surface (content layer, dialogs, inputs) | `#FFFFFF` | `#2B2B2B` |
| Secondary surface / pressed | `#F9F9F9` | `#323232` |
| Hover | `#F0F0F0` | `#383838` |
| Selection fill | `#E6EEF8` | `#2E3A47` |
| Selection pill = Accent | `#005FB8` | `#60CDFF` |
| Text on accent | `#FFFFFF` | `#000000` |
| Primary / secondary text | `#1B1B1B` / `#5C5C5C` | `#FFFFFF` / `#C8C8C8` |
| Icon | `#424242` | `#D6D6D6` |
| Input outline (3:1) / separator | `#868686` / `#E3E3E3` | `#8E8E8E` / `#3B3B3B` |
| Focus ring | `#1B1B1B` | `#FFFFFF` |
| Danger / warning / success | `#C42B1C` / `#8A5300` / `#0F7B0F` | `#FF99A4` / `#FCE100` / `#6CCB5F` |

Every pair meets WCAG: text ≥ 4.5:1, icons/outlines/indicators/focus ≥ 3:1. The test `light_and_dark_semantic_pairs_meet_text_and_component_contrast_targets` now also covers the pill against the selection fill and the focus ring against canvas, surface and selection. The previous Dark accent (`#155EEF`, 2.70:1 against the surface) is gone.

### 3.2 Geometry (`ui/geometry.rs`)

4 px grid.
- **Controls and rows:** standard control 32 px; icon button 36×32; navigation row 36 px; table row 36 px with no gaps; table header 32 px.
- **Chrome:** header 48 px; status bar 40 px; sidebar 220 px; dialog 440 px wide.
- **Radius:** 4 px for controls, 8 px for layers and dialogs.

### 3.3 Typography (`ui/typography.rs`)

Segoe UI / Segoe UI Semibold, loaded from `%WINDIR%\Fonts`.
- **Product name:** 14 semibold.
- **Breadcrumb / dialog titles:** 20 semibold.
- **Empty-state titles:** 18 semibold.
- **Body text and controls:** 14.
- **Table metadata, headers and captions:** 12.

## 4. Changes by area

| Area | Before | Now |
|---|---|---|
| **Window** | 1180×792; at 125% on 1080p the window extended under the taskbar (R4 P2-05). | 1200×760, minimum 880×600, centred; `fit_window_to_monitor` shrinks and centres it once if it doesn't fit the work area. Verified at 125%: no clipping. |
| **Header** | Heavy border, green "Local · Offline", outlined Settings button. | On the canvas with no border: logo trimmed (padding removed) and mipmapped, "EncryptDrive" 14 semibold, a "Sample vault" chip, a green shield with "Local only" (tooltip explains it), and a subtle gear icon. |
| **Sidebar** | Uppercase "WORKSPACE", 4 px strip, borders. | Fluent NavigationView: no title, 36 px rows, subtle selection fill with a 3×16 px accent pill, semibold selected label. Disabled during vault and account steps, with a hint. |
| **Content** | Panel with a heavy border. | A rounded layer (8 px) on the canvas, as in Explorer. |
| **Page header** | "Files" plus the redundant subtitle "Files in this workspace"; Back on the right. | Back and Up (subtle icons) on the **left**, then a 20 px breadcrumb with chevrons; the current segment is emphasised. Subtitle removed. |
| **Command bar** | Commands appeared and disappeared, shifting the table (R1/R2 P1-08). Reserved empty band. Search outside its box. | **Stable** command bar: New folder (accent), Import, Export \| Rename, Move to Trash. Commands only enable or disable. Below 820 px they collapse to icon-only with tooltips. Search on the right with the icon inside the field; hint reads "Search Files" or "Search Trash". |
| **Table** | Sorted header read "^ N"; cells misaligned by 11 px; 48 px pitch with gaps; full-row saturated selection; selection bar spilling into the next row. | Header: 12 px label, chevron **after** the label, hover fill, Size right-aligned, header and icons aligned. Rows: 36 px with no gaps; rounded hover/selection backplate (4 px inset 2 px); accent pill; 2 px focus ring around the whole row, only when the table has focus; decorations clipped to the body. Folders first; blue folder icon; types "File folder", "Text document"…; sizes "73 KB" / "256 bytes" / "1.2 MB"; empty Size cell for folders. |
| **Empty / loading** | A text row inside the table. | Centred empty state (40 px icon, 18 semibold title, 14 description), specific to empty folder, empty Trash or no results. Loading shows a centred spinner. |
| **Messages and progress** | Banner above the table that pushed rows down; text centred with a large gap. | Moved to the **status bar**: item count ("100 items · 4 selected"), then the message (severity icon, left-aligned text, Dismiss button) or the progress (label, 4 px accent bar, %, Cancel). The table never moves. |
| **Context menu** | Header with "More actions" and the item name. | No header. Items with shortcuts (Enter, F2, Del, Shift+Del) and separators. |
| **Dialogs** | 12 px text, buttons of uneven width, copy with "item(s)" and "in-memory fixture". | Fluent ContentDialog: 8 px corners, 24 px padding, 20 semibold title, 14 body, tinted footer with two equal buttons (**primary left, Cancel right**). Copy names the item ("“Deleted sample 05.zip” will be permanently deleted. This can't be undone."), uses correct plurals, and keeps one honest preview line. Export shows a warning InfoBar ("The exported copy is plaintext outside the encrypted vault."). Validation errors show an icon and text in the danger colour. |
| **Settings** | Floating `egui::Window` at the top left with an oversized title; Esc didn't close it. | Modal dialog shared by the shell and the preview: 52 px Fluent radio rows with a description, session note, Close button. Esc and clicking outside close it. |
| **Vault / account screens** | Left-aligned card, "Password preview" labels, narrow ComboBox. | Centred 460 px card: tinted icon tile, 20 px title, description, full-width fields and ComboBox, equal-width buttons, account rows with initials ("PS"), pill and chevron. |
| **Production shell** | "No vault open" appeared three times (chip, empty state, status). | One centred empty state ("No vault open" / "Vault support isn't available in this engineering build.") and "Engineering build" in the status bar. |
| **Global styles** | Hover drew a coloured border (it looked like focus); `disabled_alpha` 0.82; 2 px radius on egui widgets. | Hover is a fill only; disabled controls use a dedicated colour (and alpha 0.5 on egui widgets); 4 px radius on every widget; 8 px on menus and windows; soft layered shadow; focused text fields get an accent underline. |
| **Focus** | Inner ring, invisible on accent fills. | Fluent outer 2 px ring drawn **outside** the control, visible on the accent, on selection and on surfaces. |
| **Icons** | — | Added `Close`, `ChevronRight`, `ShieldCheck` (Phosphor, mapping checked by rendering the glyphs); removed the unused `More`. |

## 5. Behaviour and safety preserved

No interaction logic was weakened:
- **Click and dialog safety:** `button_activated_when_allowed`, which evaluates every `Response` accessor before `ui.input` (the deadlock fix), is unchanged. So are the first-frame activation gate, initial focus on Cancel in confirmations, `enter_submits` (Enter never confirms Trash/Delete/Import/Export), and Escape closing dialogs.
- **Keyboard and selection:** the table's single tab stop with its focus-lock filter (arrows and Escape), Ctrl+A/Ctrl+F, F2/Delete, Home/End/PageUp/PageDown/Space, and selection by stable ID are unchanged.
- **Click test strengthened:** the reentrancy regression test now performs a real click (hover, press, release). Before, it passed even with the buggy code (R4 P2-01). It was not mutation-tested against the old helper in this round; it uses the same mechanism as the full-app tests that did deadlock in R4.

## 6. Verification

| Check | Result |
|---|---|
| `cargo +stable fmt --all -- --check` | PASS |
| `cargo +stable clippy --locked --workspace --all-targets -- -D warnings` | PASS (0 warnings) |
| Same with `--features encryptdrive-desktop/design-preview` | PASS (0 warnings) |
| `cargo +stable test --locked --workspace` | PASS: 27 tests |
| `cargo +stable test --locked --workspace --all-targets --features encryptdrive-desktop/design-preview` | PASS: 92 tests (3 core, 10 library, 61 UI, 18 model) |
| `scripts/verify-architecture.ps1` | PASS |
| Release preview (final candidate) | `target/release/encryptdrive-desktop.exe`, 15,459,328 bytes, SHA-256 **`715D59C0E3A35CC562524C56C95A7A236CAFCAE86921887B89617B696E5C18EF`** |
| Native window | Responding=True on every capture launch (27 launches), Windows 11, 125% scaling, 1080p. |

Three tests were updated to match the new design, not to hide errors:
- **Tokens and geometry:** they assert the new values.
- **Selection test:** "Rename" now exists but is *disabled* until something is selected, because the command bar is stable.
- **Table position threshold:** the absolute threshold now reflects the new page header.

Their invariants (stable geometry, Rename only with exactly one item selected) still hold.

## 7. Evidence (genuine Release captures, synthetic data)

Folder `captures/ux5/`, captured with `PrintWindow` on the app's own window only, with no mouse or keyboard input:

- `files_light.png`, `files_dark.png`: main Files view.
- `multi_light.png`: 4 rows selected with pills.
- `folder_dark.png`: breadcrumb Files › Documents › Projects (Unicode filename).
- `search_dark.png`, `empty_light.png`.
- `trash_light.png`, `delete_dark.png`: permanent-delete dialog.
- `export_dark.png`: warning dialog.
- `import_dark.png`: progress in the status bar.
- `error_light.png`: error in the status bar.
- `vaultsel_light.png`, `unlock_dark.png`, `accounts_dark.png`.
- `settings_light.png`, `settings_dark.png`.
- `shell_light.png`: production shell.

**Review flags** (they make it possible to reproduce every capture without touching the mouse):

```powershell
.\target\release\encryptdrive-desktop.exe --design-preview --theme=light --scenario=11
.\target\release\encryptdrive-desktop.exe --theme=dark --open-settings   # production shell
```

`--scenario=N` follows the order of the scenario picker (0 Vault selection … 6 Populated Files, 11 Multi-selection, 13 Export warning, 15 Permanent deletion, 20 Error state).

## 8. Not done / pending

- **Not run:** Narrator, UI Automation tree, High Contrast, 150/200% scaling. Nor real keyboard and mouse interaction on **this** SHA: the captures were passive. The behaviour checked live in R2–R4 has unit and full-app test coverage.
- **Executable icon (`.ico`):** not embedded. It would need `build.rs` plus a resource crate (a new dependency); left for the owner to decide.
- **Theme persistence:** still per session (unchanged; it needs a settings store).
- **Design decisions for the owner:** blue folder icon (Explorer uses yellow); the destructive outlined button in Trash; status-bar density.
- `verify-architecture.ps1` triggered rustup's auto-install of the pinned toolchain `1.99.1` (same version as stable); no other system changes.

## 9. Files changed

- **`apps/encryptdrive-desktop/src/`:**
  - `lib.rs` (tokens)
  - `main.rs` (window, review flags)
  - `shell.rs`
  - `preview/state.rs` (folders first)
  - `ui/`: `app.rs`, `components.rs` (new component layer), `dialogs.rs`, `explorer.rs`, `geometry.rs`, `icons.rs`, `navigation.rs`, `screens.rs`, `theme.rs`, `typography.rs`, `copy.rs`
- **Docs:** this report, `captures/ux5/`, and a note in `DESIGN_SYSTEM_V3.md` marking it superseded.
- A full pre-change copy of the source was kept outside the repository (the reviewer's scratchpad), because the repository has no commits.
