# Interaction and accessibility review

> **Pre-UX3 source review; not a current acceptance result.** Keyboard and selection fixes are summarized in [UX3 iteration log](UX3_ITERATION_LOG.md). Actual UI Automation and Narrator results remain NOT RUN; see [UX3 owner review guide](UX3_OWNER_REVIEW_GUIDE.md).

## Implemented in source

- Keyboard handlers include Enter, Escape, Ctrl+A, Ctrl+F, F2, Delete, Shift+Delete, Arrow Up/Down, Shift+F10, Alt+Left, and Alt+Up. Row commands require table focus; Tab and Shift+Tab are left to standard egui traversal.
- Plain Delete moves selected Files items to Trash; Shift+Delete opens permanent-delete confirmation only for selected Trash items. Plain Delete in Trash and Shift+Delete in Files do nothing. F2 is limited to Files.
- Escape clears a focused search query, closes a dialog, or clears the table selection. An open popup receives Escape first.
- Row selection is modeled by stable `EntryId`; single, Ctrl toggle, Shift visible-range, and Ctrl+A visible-result selection are covered by model tests.
- Arrow focus begins at the first displayed row and does not silently select it. The focused row receives a distinct outline; selected state uses both fill and accessible selected state.
- Custom Files/Trash navigation exposes a selected `Role::Tab`; custom row semantics expose `Role::ListItem`, a descriptive name, and selected state. Ordinary widgets rely on egui semantics.
- Both the production shell and the preview request focus on Files once on their first rendered frame. The visible focus and actual screen-reader announcement are still unverified.
- Create Folder and Rename request initial focus on their name field; destructive confirmations request initial focus on Cancel.
- Dialogs save the invoking widget ID when opened by a toolbar action, row action, or keyboard command. Cancellation requests focus back to that control, with a last-focused-row/Files-navigation fallback.
- After confirmed Move to Trash or Permanent Delete, focus returns to the stable destination tab (Files or Trash), since the invoking row or selection toolbar may have disappeared.
- Context actions invoked on a selected row preserve that multi-selection for group actions; invoking on an unselected row targets that row. Rename and Open remain single-item actions.
- Disabled-action explanations are visible text in the default shell. Dialogs provide titles, field labels, inline name errors, and a visible Cancel action.
- Destructive confirmations request focus on Cancel when first opened. Confirmed row removal returns focus to the stable destination tab. Focus trapping, announcements, and all return behavior remain unverified live.
- Files/Trash navigation is disabled on representative vault/account selection, create, unlock, login, and registration screens. A note directs reviewers to the Preview state picker.
- The synthetic operation reports progress, disables conflicting actions, repaints while running, and can be canceled without external work.
- Light/Dark semantic palettes and a visible focus color are defined. No motion is required for core interactions.

## Automated evidence

**PASS — model tests:** 15 preview-model tests cover fixture counts/stability, nested and empty directories, Unicode/long names, query/sort stability, Ctrl/Shift selection rules, Ctrl+A scope, name collisions, create/rename, navigation, Trash/restore/permanent delete, cached projection invalidation, arrow-focus start, and operation progress/cancel/failure. One additional feature-gated binary unit test covers destination/modifier routing for Delete and Shift+Delete.

**PASS — source/build checks:** `cargo fmt --all -- --check`; workspace check and tests; feature-enabled desktop tests (25 tests total); strict Clippy for workspace and feature-enabled targets; Release builds; and the architecture boundary script. The standard workspace suite contains 10 tests (3 core + 7 desktop library); preview mode adds 3 UI unit tests and 15 model integration tests. This does not verify a running UI or accessible output.

## Not run / blocked

| Check | Status | Why / owner-assisted action |
|---|---|---|
| Physical pointer, double click, row context menu | NOT RUN | GUI input is stopped by the prior Escape instruction. Owner can use manual checklist. |
| Keyboard-only end-to-end workflow | NOT RUN | Source and model exist, but no keyboard events were injected. |
| Shift+F10 context menu | NOT RUN | Popup routing is implemented but not exercised live. |
| Windows UI Automation roles/names/states | NOT RUN | No live accessibility inspector was run. Inspect row, tab, button, dialog, and progress nodes. |
| Windows Narrator | NOT RUN | Manual speech and focus announcements remain an owner validation gate. |
| High Contrast | NOT RUN | Palette provides Light/Dark only; system High Contrast adaptation is unresolved. |
| Text scaling | NOT RUN | Needs 100%, 150%, and 200% display/text scaling review. |
| Contrast measurements on rendered states | NOT RUN | Token hex values alone do not certify the rendered contrast. |
| Dialog focus entry/return and focus trap | NOT RUN | Requires running GUI and accessibility inspection. |

## Precise manual acceptance sequence

1. Build and run the preview command in the [Owner review guide](OWNER_REVIEW_GUIDE.md); keep the sample-data label visible.
2. With keyboard only, Tab through navigation, search, toolbar, list, and Settings. Confirm a visible focus indicator at each stop and a logical return after closing Settings/dialogs.
3. In Explorer, click/keyboard-focus a row; verify focused and selected states are distinct. Exercise Ctrl-click, Shift-click, Ctrl+A, search, and sort; confirm IDs remain stable and hidden selected items are disclosed.
4. Press Ctrl+F, Enter on folder and file, F2, Delete, Escape, arrows, Shift+F10, Alt+Left, and Alt+Up. Confirm file Enter/double-click only shows the explicit export warning.
5. Inspect UIA names, roles, and selected/focused states for Files/Trash, table headers/rows, buttons, input fields, dialogs, and progress. Run Narrator through the same path.
6. Repeat Light and Dark at 960×640, 1180×792, 1600×900, and 1920×1080; check 100%, 150%, 200%, and Windows High Contrast. Confirm no clipping of file names, dialog buttons, or navigation.
