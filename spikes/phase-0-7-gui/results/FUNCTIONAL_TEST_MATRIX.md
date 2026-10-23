# Functional test matrix

**Phase 0.7E execution state: shared model tests PASS (18/18); eframe test PASS (1/1); Slint test PASS (1/1); fmt/check/strict Clippy PASS. Current-build GUI scenarios: NOT RUN.** Earlier visible observations and their build boundary are preserved under Phase 0.7D below. The computer-use helper was stopped with physical Escape; this turn did not send GUI input. Run [`OWNER_GUI_REVIEW.md`](OWNER_GUI_REVIEW.md) against the current Phase 0.7E Release binaries before treating any interaction as a GUI pass.

| Scenario | eframe source coverage | Slint source coverage | Result in this environment |
|---|---|---|---|
| Deterministic 100 / 1,000 / 10,000 fixtures | Shared fixture + Settings size control | Shared fixture + Settings size control | NOT RUN |
| Files / Trash, folder navigation, parent/back, breadcrumbs | Shared state and eframe controls | Shared state and breadcrumb Buttons | NOT RUN |
| Search and sort by Name, Size, Modified | Model + three TableBuilder header actions | Model + three header Buttons | NOT RUN |
| Single, Ctrl-click, Shift-range, Ctrl+A | Model stable ID selection; eframe click/key handlers | Model stable ID selection; Slint pointer modifier callback/key handler | NOT RUN; Slint focus routing also unverified |
| Selection after sort/filter | Shared model tests | Same shared model | Model PASS; GUI widget event paths NOT RUN |
| Enter / Escape / Ctrl+A / Ctrl+F / F2 / Delete / arrows | eframe key handler | Slint FocusScope event mapping | NOT RUN; Tab/Shift+Tab and Slint event propagation unverified |
| Shift+F10 / Alt+Left / Alt+Up | eframe keyboard context window and navigation | Slint context panel and keyboard routing | NOT RUN |
| Folder/file double click | Common activate logic and table response | Common activate logic and ListView double-click callback | NOT RUN |
| Context actions | Native egui response context menu; keyboard menu window | In-app custom context action panel | NOT RUN |
| Create / rename validation | Shared model + dialog | Shared model + dialog | Model PASS (18-test suite); current GUI dialogs NOT RUN |
| Delete to Trash / permanent-delete confirmation | Dialogs and queued operations | Dialogs and queued operations | NOT RUN |
| Simulated Import / Export plaintext warning | Dialogs and synthetic-only operation | Dialogs and synthetic-only operation | NOT RUN |
| Recovery notification | Status notification and control | Confirmation dialog and status notification | NOT RUN |
| Loading/error/retry | Settings controls | Settings controls | NOT RUN |
| Long operation, progress, cancel, success, failure | repaint-driven deterministic ticks | Slint timer-driven deterministic ticks | NOT RUN; responsiveness unknown |
| Light/Dark | shared semantic tokens, egui visuals | same semantic token values applied to properties | NOT RUN |
| CJK fixture text | eframe `Fonts` test with installed system-font provider | standard Slint text rendering | eframe unit test PASS on this host; current GUI visual rendering NOT RUN |

The 18 common model tests cover exact fixture sizes/repeatable IDs, stable selection after sorting/filtering, visible-only select-all, folder-first name sorting, range selection after sorting, Files/Trash location preservation, folder navigation/file activation, Unicode/long names, Windows reserved-name validation, rename validation, trash restore/collision resolution, cancel/failure states, and semantic-token contrast. They verify shared model logic, not either GUI event path. Test names and assertions are in `../common/tests/state_scenarios.rs`.

### Modified timestamp and sorting reconciliation

VF1 stores `createdAt` and `modifiedAt` (see Java `docs/VAULT_FORMAT.md`, manifest entry schema). The Java `ManifestEntry` model initializes both to the same instant; the inspected Java operations do not update `modifiedAt`. Java `FilesController` displays a MODIFIED column, but `files.fxml` explicitly sets `sortable="false"` on it and every other Files column. The Trash columns are also explicitly non-sortable. The inspected controller tests do not assert sorting behavior.

Phase 0.7's requested Name, Size and Modified sorting is therefore new spike UX behavior, not Java compatibility behavior. The deterministic fixture provides synthetic modified timestamps solely to exercise table sorting. Do not treat these values or the Modified column as production semantics. Resolve whether to expose this column only after the product owner defines a trustworthy meaning and compatibility rules.

## Phase 0.7B execution update — 2026-10-18

All shared model and framework checks remain BLOCKED before Cargo starts. This turn attempted the five requested commands in common and both app directories; all 15 failed command discovery because no Cargo executable exists. No source-level compile result can be inferred. The three manifests now declare Rust 1.95, but formatting, tests, Clippy, dependency resolution and release builds remain NOT RUN. Every GUI scenario remains NOT RUN because there is no executable.

The Modified column discrepancy is corrected above: VF1 stores modifiedAt; Java displays it; Java does not update it from createdAt and sets every Files/Trash table column non-sortable in FXML. The Phase 0.7 Modified sort is a synthetic-only enhancement request and is not verified compatibility behavior.

## Phase 0.7C pre-install snapshot — historical, superseded later on 2026-10-18

**Source/build checks:** `cargo fmt --check` passed for common, eframe, and Slint. `cargo check --locked` and `cargo clippy --all-targets --locked -- -D warnings` passed for common. `cargo check --locked` for each GUI stopped at the missing `link.exe`; `cargo test --locked` for common also stopped at that linker error before executing a test. All 18 GUI scenarios below remain NOT RUN.

| Required run | eframe | Slint | Current status |
|---|---|---|---|
| Build and initial launch | no executable | no executable | BLOCKED: `link.exe` absent |
| 100 / 1,000 / 10,000 rows, scroll, resize | not launched | not launched | NOT RUN |
| Search, sorting, stable-ID selection, keyboard | not launched | not launched | NOT RUN |
| Context menu, dialogs, validation, simulated operations | not launched | not launched | NOT RUN |
| Light/Dark, loading/error, progress/cancel | not launched | not launched | NOT RUN |
| DPI, accessibility, Narrator, offline launch | not launched | not launched | NOT RUN |

The common scenario tests are authored but unexecuted because their test binary cannot be linked. `MANUAL_VALIDATION_CHECKLIST.md` remains the reproducible procedure. No interaction result is inferred from source or Clippy.

## Phase 0.7C authorized build follow-up — 2026-10-18

| Scenario / verification | eframe | Slint | Result |
|---|---|---|---|
| Deterministic fixture sizes and stable IDs | Shared model test PASS; process alive at 100/1,000/10,000 | Same | PASS for model/startup only; rows not visually inspected |
| Shared state selection, sorting/filtering, range, navigation, name validation, file activation, operation cancel/failure | 8 common scenarios PASS | Same shared model | PASS at model level; widget event routes NOT RUN |
| Formatting, locked check, strict Clippy, test build | PASS; 0 GUI-specific tests | PASS; 0 GUI-specific tests | PASS (GUI test count is zero) |
| Native launch and window creation | PASS at all 3 sizes; non-zero HWND | PASS at all 3 sizes; non-zero HWND | PASS; hidden launch only |
| Visible scroll/resize, search/sort controls, pointer/keyboard selection, context menu, dialogs | NOT RUN | NOT RUN | NOT RUN |
| Light/Dark visual parity, loading/error display, progress/cancel behavior | NOT RUN | NOT RUN | NOT RUN |
| DPI, UIA/Narrator, screenshot, disconnected-network launch | NOT RUN | NOT RUN | NOT RUN |

The GUI `cargo test --locked` targets both exited successfully but each reported 0 tests. The common crate reported 8 passed, 0 failed. No user interaction was simulated through the native controls; the user's Escape action had stopped desktop automation, so the launch/memory checks were shell-only and hidden.

## Phase 0.7D visible GUI update — 2026-10-18

The two already-built Release executables were launched visibly through the Windows computer-use helper. Both window captures were 1180×792 including the outer window frame; both applications request 1180×760 logical client dimensions. Initial screenshots were displayed in the task session but were not saved as files. Testing stopped when the helper reported: “Computer Use was stopped by the user with the physical Escape key.” No further app input was sent.

| Scenario | eframe | Slint | Result / evidence |
|---|---|---|---|
| Initial visible launch, 100 entries | PASS | PASS | Both native windows and their initial explorer views were captured. |
| UI Automation names and roles | PARTIAL PASS | PARTIAL PASS | eframe exposed named toolbar buttons and file rows as buttons. Slint exposed named buttons, a list, and file rows as selectable options. This is a UIA tree observation, not Narrator approval. |
| Slint single selection | NOT RUN | PARTIAL PASS | An indexed row action was followed by a subsequent accessibility snapshot with one selected item and the row marked selected. The first immediate snapshot still showed 0 selected, so repeat after the owner restarts the updated binary. |
| Create Folder dialog and empty-name validation | NOT RUN | PASS | The dialog appeared; empty submission showed “Enter a name.” A synthetic folder name was set through the UIA edit value and submitted; the folder appeared and the success notice was visible. No real file operation occurred. |
| Light and Dark appearance | PARTIAL FAIL | FAIL before correction; correction NOT RUN | eframe Light view was legible but showed tofu for the Japanese fixture characters. Slint Dark was legible. Slint Light had near-white standard-widget text on a light surface. A source correction synchronizes Slint `Palette.color-scheme` to Light/Dark, but the rebuilt binary was not visibly rechecked. |
| Long-name truncation, scroll, resize, 1,000 / 10,000 visible rows | NOT RUN | NOT RUN | The two visible runs remained at 100 entries; no resize action was performed. |
| Search, sort, Ctrl/Shift selection, keyboard-only navigation, context menu | NOT RUN | NOT RUN | The Escape stop interrupted this pass before these scenarios. |
| Import / Export warning / Trash / restore / permanent delete / recovery | NOT RUN | NOT RUN | Not exercised through either GUI. |
| Progress, cancel, success, failure, loading and error | NOT RUN | NOT RUN | Not exercised through either GUI. |
| DPI, Narrator, disconnected launch | NOT RUN | NOT RUN | See the DPI, Accessibility, and Deployment reports. |

### Corrections after the visible pass

- Slint now sets the standard widget palette to the same Light/Dark mode as the prototype tokens. Its initial Light mismatch was reproduced visually, but the correction has only passed Slint compilation and Release build; visible verification remains outstanding.
- Slint text dialogs now focus the LineEdit when created; non-text dialogs focus Cancel. The initial Create Folder dialog left the accessibility focus observation on Search. The focus correction is build-verified only.
- Slint file sizes now use the same binary units as eframe. A new unit test first failed (`82.4 KB` versus expected `80.5 KB`) and now passes for KB and MB examples.
- Delete now uses a shared page-aware model method, selecting Move to Trash in Files and Permanent Delete in Trash. The new common test first failed because the method was absent, then passed after implementation. Both UI crates compile and pass strict Clippy; GUI-key behavior was not visibly retested.

All remaining GUI scenarios are NOT RUN or BLOCKED by the stopped helper, not passed by source inspection. The shared test suite now has 9 tests; eframe has 0 GUI-target tests; Slint has 1 size-format test.
