# UX3 Component Standardization

**Implementation:** `apps/encryptdrive-desktop/src/ui/components.rs`, `geometry.rs`, `typography.rs`, and `theme.rs`.  
**Evidence:** Rust unit tests and source review PASS; live appearance and accessibility semantics are NOT RUN.

| Component | Standardized behavior |
|---|---|
| Icon+label action | 36-point target, 16-point Phosphor icon, 14-point control label, consistent spacing, primary/secondary treatment, hover/focus/disabled feedback, human-readable Button metadata |
| Navigation row | 40-point row, 20-point icon, 14-point label, distinct selected fill/leading focus-colored marker, selected Button metadata |
| Text field | 36-point height, visible 12-point label associated by widget ID, hint text, optional password masking in preview only |
| File row | Stable EntryId widget ID, full-row hit area, selected ListItem metadata with human-readable name/kind, clipped filename cell, selected-aware icon/text, Name-cell focus indicator |
| Sort header | Plain human-readable Button name; active sort arrow is visual-only metadata; test/manual verification must confirm sort state is announced accurately |
| Surface | Shared semantic fill, outline/separator, 4-point control radius, 6-point surface radius |

`Geometry` centralizes the 4-point spacing scale, 36-point controls, 40-point navigation/table rows, icon sizes, selection marker, brand mark, and radii. Primary action color is reserved for action fills; ordinary dark-theme icon/text foregrounds use readable semantic tokens.

## Remaining component questions

- Custom-painted icon+label buttons use explicit `WidgetInfo`; verify keyboard activation, role/name, and focus state in UIA and Narrator.
- Confirm search, row, sort-state, dialog, progress, disabled, and error accessibility properties in the running application.
- Validate clipping, tooltip behavior, columns, action wrapping, and all button states at the minimum window size and higher Windows scaling.
