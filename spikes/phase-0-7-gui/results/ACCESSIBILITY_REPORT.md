# Accessibility report

**Windows accessibility execution: PARTIAL on Phase 0.7D binaries; current Phase 0.7E binaries: NOT RUN. Mandatory accessibility gate remains unresolved.** Earlier Windows UI Automation trees were captured through the computer-use helper. Narrator, complete keyboard focus order, and several important states remain NOT RUN. Do not interpret framework features, imports, tests, or source properties as verified accessibility. The owner procedure is [`OWNER_GUI_REVIEW.md`](OWNER_GUI_REVIEW.md).

## Framework mechanisms to inspect

| Area | eframe implementation intent | Slint implementation intent | Verified outcome |
|---|---|---|---|
| Platform accessibility | `accesskit` eframe feature enabled; egui widget metadata for custom text input | Slint `accessibility` feature and explicit row role/label/selection/index/count, dialog form labels, live regions | NOT RUN |
| Row semantics | egui table row/selectable widgets; row names need inspection | `list-item`, explicit accessible name, selectable/selected state, index/count | NOT RUN; Slint `ListView` row semantics are source declarations only |
| Operation feedback | visible progress bar/terminal status | visible progress bar, accessible label, polite announcement status | NOT RUN |
| Validation | visible inline error in dialog | visible error with assertive live-region intent | NOT RUN |
| Focus | egui table/search/dialog flags and requested focus | standard controls plus custom FocusScope; row focus and event propagation require scrutiny | NOT RUN |

The exact exposed accessible roles for eframe’s `TableBuilder`, `row.response()` selection, custom dialog, status banner and synthetic context window are unknown. Enabling AccessKit does not prove every custom-drawn widget exposes useful names, roles, states, or actions. The Slint rows set explicit selection metadata; accessibility actions and multi-select narration were not inspected. The Slint dialog uses a `form` region because the current generic role list does not establish a dialog role in this source; inspect the Windows provider tree before deciding whether to replace it with a standard dialog implementation.

## Phase 0.7D live UI Automation observations — 2026-10-18

The computer-use helper captured UIA trees from both visible applications before the user stopped the helper with physical Escape:

| Observation | eframe | Slint |
|---|---|---|
| Window and main controls | Window title and region, appearance/settings, Files/Trash, navigation, search, New Folder/Import/Export/Rename/Delete, sortable Name/Size/Modified, feedback controls | Window title, Settings, Files/Trash, offline account status, navigation, Search, New Folder/Import/Export/Rename/Delete, Name/Size/Modified, list-size and appearance controls |
| File rows | Exposed as named `button` elements, e.g. the long synthetic name and Unicode fixture | Exposed beneath a `list` as selectable `option` rows with file/folder name, size and modified timestamp |
| Selection | No selected-row state was captured | A later snapshot after a row action showed one selected item and a selected annotation on the row. The immediately returned snapshot was stale and still said 0 selected; repeat this check. |
| Dialog and validation | NOT INSPECTED | Create Folder appeared as a `form` region with a named edit and Cancel/Create buttons. Submitting empty input surfaced the visible and tree-exposed text “Enter a name.” |
| Focus | NOT INSPECTED | At initial dialog open the helper still reported Search as focused. Clicking the edit exposed its value. Source now focuses the edit at creation, but the rebuilt binary was not visually rechecked. |
| Narrator / keyboard order | NOT RUN | NOT RUN |

This is direct UIA-tree evidence, not a Narrator pass or accessibility score. The Slint UIA tree includes useful names and selected state for at least one row; multi-select announcements, all actions, dialog focus/return, progress, error live regions, and Narrator speech remain unverified. eframe row buttons did not provide evidence of a selected state or table/list semantics in the captured tree.

The Windows SDK x64 Inspect.exe is present at `C:\Program Files (x86)\Windows Kits\10\bin\10.0.19041.0\x64\inspect.exe`; it was not launched after the helper reported that the user stopped Computer Use. The exact stop was: “Computer Use was stopped by the user with the physical Escape key.”

## Required manual validation

Run scenario 17 in `../test-scenarios/GUI_SCENARIOS.md` for both Release binaries with Narrator and Accessibility Insights for Windows or Inspect.exe. Capture application/window names, landmarks, column header/row roles and labels, selected states, focus order, validation messages, progress changes, and context menu behavior. Confirm keyboard-only access to every action and meaningful focus visibility. Until that is done, Accessibility is a mandatory unresolved gate for both frameworks.

## Phase 0.7E source and test update — 2026-10-18

The current eframe source enables eframe's `system_fonts` feature and passes a unit test that shapes the Japanese fixture name with `SystemFontProvider` and a panic-on-missing-glyph policy on this Windows host. This verifies font fallback shaping in the test harness, not visible runtime text, accessible roles/names, or Narrator behavior. No additional font file is bundled.

Slint's row focus border now uses a dedicated semantic `focus-ring` token mapped from the shared token set. The Slint update compiles and its unit test passes. The corrected Light palette and focus ring have not been viewed; no current Phase 0.7E UIA or Narrator inspection was run. Earlier UIA observations remain Phase 0.7D evidence only. The current owner run is detailed in [`OWNER_GUI_REVIEW.md`](OWNER_GUI_REVIEW.md).

## Phase 0.7B execution update — 2026-10-18

No GUI executable could be produced: Cargo, rustc and the MSVC compiler/linker are absent. Consequently no AccessKit/UI Automation tree, Narrator speech, focus order, control name, role, selected-state announcement, or dialog announcement was observed. Source declarations and the enabled AccessKit/Slint accessibility features remain implementation intent only. The accessibility gate is BLOCKED for both frameworks; manual Narrator validation is NOT RUN.

## Phase 0.7C pre-install snapshot — historical, superseded later on 2026-10-18

Rust and Cargo are installed and both lockfiles resolve, but GUI compilation stops at missing `link.exe`; no application binary is available to inspect. The Windows Computer Use helper was stopped by the user with Escape, so no further UI inspection was attempted. AccessKit and Slint accessibility declarations remain source-level intent only. UIA, Narrator, focus order, accessible names/roles/states, validation announcements and progress announcements are all NOT RUN; the accessibility gate remains BLOCKED for both candidates.

## Phase 0.7C authorized build follow-up — 2026-10-18

Both Release applications were built and process-launched. The eframe binary uses its AccessKit-enabled feature configuration; Slint has the `accessibility` feature enabled and its list rows declare labels, indices/counts, and selection state. Slint notices now declare an accessible `region` role with a polite live region. These are source/build observations only. PE import inspection confirms `uiautomationcore.dll` in both executables but does not prove the tree is populated or useful to Narrator.

| Inspection | eframe | Slint | Result |
|---|---|---|---|
| Windows UIA tree / Accessibility Insights or Inspect.exe | NOT RUN | NOT RUN | NOT RUN |
| Narrator names, roles, state, selection, progress and validation speech | NOT RUN | NOT RUN | NOT RUN |
| Tab/Shift+Tab order, visible focus and keyboard-only operation | NOT RUN | NOT RUN | NOT RUN |
| Screen-reader row/table semantics and context actions | NOT RUN | NOT RUN | NOT RUN |

No accessibility score is assigned. The UI automation helper was stopped by the user with Escape and no further UI input/inspection was performed. The app binaries were launched hidden through shell-only commands, which verified process/window creation but supplied no accessibility evidence. The manual procedure in this report remains required before the mandatory gate can pass.
