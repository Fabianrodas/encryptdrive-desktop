# Framework comparison result

**Recommendation C — no selection.** Current Phase 0.7E source passes formatting, locked checks, strict Clippy and tests: 18 shared model tests, one eframe Unicode font-fallback test and one Slint size-format test. Current Release rebuilds are isolated from the earlier visible executables; current-build visual and interaction validation was not run. The 21/100 entries below are the last partial evidence totals from Phase 0.7D, not scores for the current binaries. The mandatory basic-interaction, accessibility, offline-execution and acceptable-license gates remain unresolved; no winner can be justified.

## Phase 0.5 weighted rubric

The following score entries are the last partial assessment from Phase 0.7D's visible builds. They are retained as historical evidence only; no fresh numerical score is assigned to the current Phase 0.7E binaries because their GUI behavior has not been re-run.

Points below are provisional evidence points, not an overall product recommendation. A `0` means the category was not verified in the running UI or isolated deployment context; it does not mean the framework failed that category. Partial credit is limited to the evidence identified in each row.

| Category | Weight | eframe | Slint | Evidence boundary |
|---|---:|---:|---:|---|
| Visual quality and density | 15 | 7/15 | 3/15 | Phase 0.7D visible binaries: eframe's Explorer-like table was legible but CJK rendered as tofu; Slint Dark was legible, while Light standard controls had very low contrast. Current eframe system-font fallback and Slint palette/focus changes have not been visibly retested. |
| Keyboard and interactions | 15 | 0/15 | 3/15 | No keyboard input was tested. Slint UIA-indexed row selection and Create Folder empty validation/synthetic creation were exercised. eframe controls were not exercised. |
| Accessibility and Narrator | 18 | 3/18 | 5/18 | Direct UIA tree observation exposed eframe control/file names and Slint buttons, list items, row names/selected state, edit and validation text. Narrator, keyboard order and broad state/action checks remain unverified. |
| DPI and window behavior | 12 | 0/12 | 0/12 | Initial window creation verified; visible resize and 100–200% scale scenarios not run. |
| Rendering compatibility | 10 | 0/10 | 0/10 | Renderer path, GPU behavior and fallback were not observed. |
| Offline portable execution | 10 | 0/10 | 0/10 | Local process launch passed on the development host; no disconnected clean-host run. |
| Installer feasibility | 6 | 2/6 | 2/6 | Native windows launch on this host and PE imports identify runtime requirements; clean target install/package test not run. |
| Licensing and dependencies | 6 | 3/6 | 2/6 | Locked graphs and license metadata inventory exist; full transitive license audit and owner approval are absent. Slint terms need an explicit product decision. |
| Maintainability | 8 | 6/8 | 6/8 | Current format, locked check, strict Clippy and test commands pass; 18 common tests, 1 eframe font-shaping test, and 1 Slint size-format test pass. These are not GUI interaction tests; no independent code review was run. |
| **Verified points** | **100** | **21/100** | **21/100** | **Only partial evidence points are counted.** |
| **Unevaluated points** | **100** | **79/100** | **79/100** | **These criteria remain unverified; they are not implied passes.** |

These evidence totals do not measure product readiness. The visual row contains known failures; most other categories have untested requirements. Equal verified totals do not imply equivalent behavior, and neither candidate satisfies the mandatory gates.

## Verified evidence

- Rust `1.99.1`; eframe/egui_extras `0.37.0`; Slint/slint-build `1.18.1`; target `x86_64-pc-windows-msvc`.
- Current `cargo fmt --check`, `cargo check --locked`, strict Clippy and tests pass after corrections. Common has 18 state tests; eframe has one Unicode shaping unit test; Slint has one size-format unit test. Neither GUI has interaction automation.
- Current Phase 0.7E Release executables are 14,498,816 bytes (eframe) and 15,790,592 bytes (Slint). Current `dumpbin /DEPENDENTS` output and SHA-256 values are in `PERFORMANCE_AND_BINARY_REPORT.md`. The current executables were not visually launched.
- In Phase 0.7D, both original Release executables were opened visibly at 100 entries before the helper stop. A Slint row-selection interaction and folder dialog empty validation/success were exercised. The UIA tree exposed controls and row semantics; Narrator was not tested.
- `VCRUNTIME140.dll` and Windows API-set CRT DLLs are PE dependencies for both. This development host has the authorized MSVC workload and existing SDK; clean-machine distribution remains untested.

## Critical unresolved gates

1. **Basic interactions:** only partial Slint pointer/dialog coverage exists. Keyboard-only operation, range/Ctrl multi-selection, search/sort, context menus, Trash, import/export and operation feedback remain unverified. Delete-in-Trash routing is source/model-test corrected but not GUI-retested.
2. **Accessibility:** UIA trees show meaningful names and Slint row selection metadata, but keyboard order, full action/state semantics, dialog focus/return, progress/error announcements and Windows Narrator remain unverified.
3. **Offline execution:** launch from a clean Windows environment with network disconnected and no developer toolchain; identify prompts and loaded runtime modules.
4. **License acceptability:** owner/legal review must approve eframe's dependency/notices and decide whether any Slint license, attribution and cost terms fit the product.

## Specification reconciliation and framework notes

- The Java reference displays `modifiedAt` but initializes it equal to `createdAt`, does not update it in the inspected operations, and marks Files/Trash columns non-sortable. The Phase 0.7 Modified values and sort are synthetic comparison affordances only; product semantics remain an approval question.
- Slint's documented `StandardTableView` current-row API does not establish stable multi-selection. This spike uses custom `ListView` row delegates and the shared stable-ID selection model. No real Ctrl/Shift multi-selection or screen-reader selection behavior was exercised.
- eframe uses `egui_extras::TableBuilder` virtualized rows and WGPU, with AccessKit enabled. Slint uses a custom `ListView`, Winit, FemtoVG plus the software-renderer feature. Compilation does not establish a fallback renderer or DPI behavior.
- Current Slint license options and the conflict between the terms page and pricing comparison on attribution are recorded in `DEPENDENCY_AND_LICENSE_REVIEW.md`. No option has been selected for the future product.

## Recommendation and next verification

Do not select a candidate yet. The narrow next step is an owner-assisted visible run of the remaining shared scenarios using the updated binaries, followed by Accessibility Insights/Inspect and Narrator for both. Then test disconnected clean-host launch and obtain owner/legal license review. Phase 1 and Vault Format 1 remain out of scope.

## Phase 0.7D status

The computer-use helper returned “Computer Use was stopped by the user with the physical Escape key.” No further app input was sent. Equivalent initial Phase 0.7D window captures were displayed at 1180×792 outer dimensions but were not saved. The Slint palette was synchronized and its row focus token now uses a dedicated focus-ring property; these source corrections require visible retesting. eframe's native system-font fallback was enabled and a CJK font shaping test passes on this host; runtime visual verification across Windows configurations remains open. See `FUNCTIONAL_TEST_MATRIX.md`, `ACCESSIBILITY_REPORT.md`, `DPI_RENDERING_REPORT.md`, `PERFORMANCE_AND_BINARY_REPORT.md`, and `OWNER_GUI_REVIEW.md`.
