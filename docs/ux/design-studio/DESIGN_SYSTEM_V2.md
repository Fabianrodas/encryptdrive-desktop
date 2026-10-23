# Design system V2 — implementation map

> **Superseded by [Design System V3](DESIGN_SYSTEM_V3.md).** Values and brand/icon claims below describe an earlier candidate and must not be used as current implementation facts. The original husky placeholder was replaced by a provenance-recorded PNG; typography, geometry, iconography and color roles changed in UX3. See [UX3 iteration log](UX3_ITERATION_LOG.md) for the evidence boundary.

This candidate extends the Phase 0.5 semantic token set through `apps/encryptdrive-desktop/src/ui/theme.rs` and reusable controls in `src/ui/components.rs`. The design uses one semantic palette for Light and Dark.

## Color tokens

| Role | Light | Dark | Use |
|---|---:|---:|---|
| Canvas | `#F7F8FA` | `#1A1D21` | Main content surface |
| Surface | `#FFFFFF` | `#242930` | Header, dialogs, table |
| Secondary surface | `#EEF1F4` | `#2C333B` | Preview controls, selected context |
| Primary text | `#18212B` | `#F2F4F7` | Main content |
| Secondary text | `#596573` | `#B5C0CB` | Supporting labels |
| Selection | `#E6F0FC` | `#254A73` | Selected rows and destinations |
| Selection text | `#183C66` | `#D6E8FF` | Text within selected state |
| Action / focus | `#155EEF` | `#91BEFF` focus; action remains `#155EEF` | Primary action and focus ring |
| Destructive | `#B42318` | `#FF8A80` | Delete warnings/actions |
| Warning | `#8A4B08` | `#F2C078` | Caution state |
| Success | `#18794E` | `#75D6A6` | Local/offline and success state |
| Disabled controls | eframe disabled widget alpha (`0.55`) | eframe disabled widget alpha (`0.55`) | Unavailable controls; a visible separate reason is shown in the default shell |

The table records token values, not contrast certification. Rendered text, icons, selection and focus contrast have **NOT RUN** status pending live visual checks.

## Typography and content

- eframe `default_fonts` and `system_fonts` are enabled; no external font file or remote font is used.
- Controls and supporting text generally use 10.5–12 px; primary page title 23 px; app name 16 px. Actual Windows text scaling has not been checked.
- Filenames remain Unicode `String`s; names longer than the display allowance are shortened by Unicode characters and shown with an ellipsis. Full names remain in the row accessible label and context menu.
- Folder/file symbols are text glyphs; the app does not bundle an icon package. Their glyph appearance depends on host font fallback and requires visual verification.
- The temporary husky mark is an original geometric drawing created from egui shapes; it is not copied artwork.

## Geometry and responsive rules

| Element | Candidate value |
|---|---:|
| Window initial size | 1180 × 792 logical points |
| Window minimum | 960 × 640 logical points |
| Header | approximately 46 points |
| Sidebar | 214 default; 168–284 resizable |
| Main padding | 22 points |
| Table header / data row | 37 / 34 points |
| Button/control minimum height | 30 points |
| Dialog width | 440–500 points |
| Common control radius | 4–8 points |

The 960×640 minimum matches the smallest requested design target. The window remains resizable above that size; live checks at 960×640, 1180×792, 1600×900, and 1920×1080 are blocked. Table columns are resizable, but their behavior under window resizing is not visually verified.

## Components and state rules

- **Header:** temporary husky mark, EncryptDrive identity, one compact sample/no-vault status, Settings, and one preview label.
- **Navigation:** Files and Trash labels; selected marker and accessible selected state; visible unavailable-action reason in the production shell.
- **Explorer:** full breadcrumb path, search, primary New Folder/Import, selection-dependent Export/Rename/Move to Trash, and row context actions.
- **Table:** virtualized rows, semantic headers, Name/Type/Size/Modified · sample, stable-ID selection, hover/focus/selected distinction.
- **Dialogs:** named title, concise explanation, persistent field label, inline validation, safe Cancel path, and destructive wording specific to the synthetic fixture.
- **Operation:** progress bar with current/total text, cancel, success, failure, and busy-action disablement.
- **Feedback:** in-window notices only. No filenames, vault path, credentials, or operation details are sent to OS notifications.

Accessibility state follows the guidance in [Interaction and accessibility review](INTERACTION_AND_ACCESSIBILITY_REVIEW.md). High Contrast support and actual UIA/Narrator output are unresolved.
