# UX3 Typography Specification

**Source:** `apps/encryptdrive-desktop/src/ui/typography.rs`; default eframe text roles are synchronized in `ui/theme.rs`.  
**Status:** Token and source tests pass; rendered specimens and Windows text scaling are NOT RUN.

| Semantic role | Size (logical points) | Use |
|---|---:|---|
| App identity | 18 | EncryptDrive wordmark text |
| Page title | 20 | Files, Trash, and representative screen titles |
| Section heading | 16 | Dialog and empty-state headings |
| Body | 14 | Explanatory copy |
| Control | 14 | Buttons and navigation labels |
| Metadata | 13 | Table headers and compact status |
| Helper | 12 | Supporting labels and validation hints |

The family is eframe's bundled/default proportional family with host system fonts available as configured in the shell. No remote font is loaded. Phosphor is an icon font and must not be used for words. Text is not reduced based on filename character count; the filename label is clipped to its cell and carries the full name as a tooltip. Whether Windows UI Automation exposes the full label independently of the tooltip is NOT RUN.

**Accessibility limits:** 12 logical points is the smallest current role. Scaling at Windows 100%, 150%, and 200%, font substitution for accented/CJK/emoji names, and low-vision readability need human review.
