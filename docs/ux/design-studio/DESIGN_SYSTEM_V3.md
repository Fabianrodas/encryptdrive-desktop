# Design System V3 — UX3 implementation candidate

> **Superseded (2026-10-23):** colours, geometry and typography were replaced by the Windows 11 Fluent-aligned system documented in [UX5 design polish report](UX5_DESIGN_POLISH_REPORT.md). The values below are historical.


**Status:** Implemented in source; visual, Windows accessibility, DPI, and owner acceptance are pending.  
**Direction:** Option C — Native Explorer × Premium Desktop.  
**Implementation:** `apps/encryptdrive-desktop/src/design.rs` and `apps/encryptdrive-desktop/src/ui/`.

## Semantic colors

Colors are sourced from `SemanticTokens` in `apps/encryptdrive-desktop/src/lib.rs` and converted to an eframe palette by `ui/theme.rs`.

| Role | Light | Dark | Intended use |
|---|---:|---:|---|
| Canvas | `#F7F8FA` | `#1A1D21` | Main work area |
| Surface | `#FFFFFF` | `#242930` | Header, sidebar, dialogs, rows |
| Secondary surface | `#EEF1F4` | `#2C333B` | Quiet grouped controls |
| Hover | `#EEF3F8` | `#323B46` | Pointer hover |
| Selection | `#DCEBFB` | `#244668` | Selected row/destination |
| Selection text | `#13355B` | `#E2F0FF` | Selected labels |
| Primary text | `#18212B` | `#F2F4F7` | Main content |
| Secondary text | `#56616E` | `#C3CCD6` | Supporting content |
| Primary action | `#155EEF` | `#155EEF` | Primary action fill only |
| On primary action | `#FFFFFF` | `#FFFFFF` | Primary action label/icon |
| Focus | `#075FCC` | `#98C5FF` | Focus outline and indicator |
| Icon | `#3E4D5D` | `#B4C3D3` | Ordinary semantic icon |
| Outline | `#697887` | `#8491A0` | Control boundary |
| Separator | `#D6DCE3` | `#414B56` | Noninteractive divider |
| Danger | `#B42318` | `#FF8A80` | Destructive/status foreground |
| Warning | `#7A3E00` | `#F2C078` | Caution/status foreground |
| Success | `#126B45` | `#75D6A6` | Success/status foreground |

Do not use the Dark action blue for small foreground icons or text. Shared custom controls use the disabled surface, muted foreground, and outline tokens; standard widgets use egui's disabled styling. Rendered disabled contrast remains unverified. Windows High Contrast is not implemented or verified.

## Type and geometry

| Token | Logical points |
|---|---:|
| App identity | 18 |
| Page title | 20 |
| Section heading | 16 |
| Body / control | 14 |
| Metadata | 13 |
| Helper | 12 |
| Small icon / major icon | 16 / 20 |
| Standard control height | 36 |
| Navigation row / table row / table header | 40 / 40 / 40 |
| Spacing grid | 4-point base; common 8, 12, 16, 20, 24 |
| Control / surface radius | 4 / 6 |

The named roles live in `ui/typography.rs` and `ui/geometry.rs`. eframe's default proportional font family remains in use. Windows text scaling and fallback for every Unicode name have not been visually inspected.

## Interaction state rules

- System is the default appearance preference. Light and Dark are explicit overrides; unknown system appearance falls back to Dark.
- Selection is keyed by stable entry ID; a changed effective search clears selection; sorting and repeating the same query preserve it.
- Hover, selection, and keyboard focus use separate state. The row focus indicator is painted within the Name cell.
- Shared controls use semantic state treatments: hover uses the Hover token and separator outline; keyboard focus uses the Focus outline; selection uses the Selection fill and indicator; pressed controls use the secondary surface and outline; disabled controls use the surface, muted foreground, and outline token. No state adds an ad hoc color.
- The explorer reserves a 36-point feedback region before the table. Selection actions stay in the command area when space permits and reserve a stable row in compact layouts; selection and feedback changes must not alter the table’s row origin.
- Rename requires exactly one selected item. Group context commands preserve the current selected group when the target belongs to it.
- Unimplemented vault actions are described in empty-state copy; the shell no longer presents permanently disabled Create/Open buttons.

These are implementation contracts, not rendered acceptance findings. See [component standardization](COMPONENT_STANDARDIZATION.md), [theme audit](THEME_AND_CONTRAST_AUDIT.md), [UX3 final visual report](UX3_FINAL_VISUAL_REPORT.md), and the [R3 remediation matrix](UX3_R3_REMEDIATION_MATRIX.md). Release screenshots, native input, UIA/Narrator, High Contrast, and DPI behavior remain pending owner validation.
