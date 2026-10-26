# UX6.2 — Remediation of the two P1 audit findings

**Start:** `main` at `2a67d9b1dccb4405ebeb22b7ac4e3571fd4ffce5`, clean working tree, remote `origin`.
**Scope:** the two P1 findings from the independent Codex audit of UX6.1. This is not a redesign, and it is not owner approval.

## P1-01 — Minimum window size versus the usable work area

**Root cause.**
- The window was created with a fixed minimum inner size of 880×600 points.
- At high scaling, the usable work area is smaller. For example, 1920×1080 at 200 % with a bottom taskbar leaves 960×510 points, or about 960×474 of client area once the 35.5-point frame is removed.
- Windows enforces the minimum through `WM_GETMINMAXINFO`. So the UX6.1 first-frame shrink was overridden, and the window stayed partly behind the taskbar.
- eframe also shows the window after the first paint, before that frame's resize commands run. A fix that only acted on the first frame would still show one oversized frame.

**Units used** (`src/ui/work_area.rs`):

| Unit | Meaning |
|---|---|
| Physical pixels | What Windows reports, in virtual-screen coordinates (negative left of or above the primary monitor). |
| Points | Physical pixels ÷ (DPI / 96 × egui zoom). |
| Inner size | The client area. |
| Outer size | Inner size plus the title bar and the invisible resize borders, which Windows includes in window rectangles. |

**Policy after the fix:**

| | Before | After |
|---|---|---|
| Minimum inner size | Always 880×600 | `min(880×600, usable inner area)` |
| Initial size | 1200×760 (or `--window-size`), clamped on the first frame | `max(desired, minimum)` capped by the usable area, set **before the window exists** |
| Position | eframe centred on the monitor, ignoring the taskbar | Centred in the work area |
| First frame | Shrink only, blocked by the minimum | Send the compatible minimum first, then shrink or move if needed (once) |
| Later moves and resizes | Never touched | Never touched |

**How it works:**
- **Before the window exists.** The eframe `window_builder` hook runs after winit has made the process per-monitor DPI aware.
  - It reads the primary monitor's `rcWork` (`GetMonitorInfoW`), its effective DPI (`GetDpiForMonitor`) and the decoration size for that DPI (`AdjustWindowRectExForDpi`, `WS_OVERLAPPEDWINDOW`).
  - From these it sets the size, minimum and position. eframe opens new windows on the primary monitor.
- **First frame.** The fit repeats the calculation from the real window geometry, using the monitor that contains the window. This covers windows that open elsewhere.
- **No native data.** The builder is left unchanged, and the first-frame fit falls back to the monitor size minus a 48-point taskbar.
- **No new dependencies.** It uses a few read-only `user32`/`shcore` calls with `SAFETY` notes, and changes no display configuration.

**Small work areas.**
- **Already compact:** the existing layout (sidebar, command overflow, scrolling table) and the dialogs.
- **Fixed:** the auth screens (create or unlock a vault, sign in, register, account list) had their card buttons pushed below the window, with no way to reach them. They now scroll vertically when the card is taller than the window. When it fits, the look is unchanged.

**Known limits:**
- **Monitor changes after startup.** The minimum is applied in physical pixels at startup DPI. If the window is later dragged to a monitor with a different DPI or a smaller work area, the minimum is not recomputed. Recomputing it would mean resizing the window after the user moved it, which the brief rules out.
- **Pre-sizing uses the primary monitor.** It is the monitor eframe opens new windows on. The first-frame fit corrects the window if it ends up elsewhere.

**Regression scenarios** (pure calculations with the decoration sizes measured on this machine: 16×39, 18×47, 22×56 and 26×71 physical pixels at 96, 120, 144 and 192 DPI):

| Scenario | Checked |
|---|---|
| 1920×1080 at 100 %, 125 %, 150 % and 200 % | Window inside the work area, and min ≤ size ≤ usable |
| 1920×1020 physical work area at 200 % | Minimum height drops to 474.5 points; width stays 880 |
| Work area under 600 points high (1366×768 at 150 %) | Compatible minimum, window inside |
| Work area under 880 points wide (1280×720 at 150 %) | Both axes of the minimum shrink |
| Taskbar at the top, left or right | Window inside the work area |
| Secondary monitor at negative x, and one above the primary | Window stays on that monitor |
| Requested window smaller than preferred | 1000×650 kept; 700×500 raised to the 880×600 minimum |
| Empty or unavailable work area | No negative sizes; the first-frame fit makes no changes |
| Every control reachable at 960×474 and 853×410 (all 22 preview states, all 6 dialogs, Settings) | Headless AccessKit test: a control fails if it is off-screen both at the top and after scrolling to the end. It fails without the auth-screen scroll fix. |

**Native verification performed** (Windows 11; the display was at **100 %, 96 DPI**, with a 1920×1032 work area). For each case I used `tools/measure-window.ps1` to read `GetWindowRect`, `GetClientRect`, `WM_GETMINMAXINFO` and the monitor's `rcWork` of the app's own window. No input was sent.

| Build | Arguments | Window (physical) | Client | Native minimum (outer) | Inside the work area |
|---|---|---|---|---|---|
| Shell | — | 1216×799 at 352,116 | 1200×760 | 896×639 | Yes |
| Preview | `--design-preview` | 1216×799 at 352,116 | 1200×760 | 896×639 | Yes |
| Preview | `--window-size=1920x1080` | 1920×1032 at 0,0 | 1904×993 | 896×639 | Yes, exactly the work area |
| Preview | `--window-size=700x500 --scenario=2` | 896×639 | 880×600 | 896×639 | Yes |

**Not verified natively:**
- 125 %, 150 % and 200 % scaling, and work areas under 600 points. Only the pure calculations and the headless layout test cover these. Changing the display scale is a system setting, so I did not change it.
- A taskbar on another edge, and a second monitor.

The display was at 125 % during UX6.1 (24 October) and at 100 % for UX6.2. I did not change it.

## P1-02 — Recursive operation scope

**Root cause.**
- The Trash and Delete confirmations counted `selected_ids()`.
- The synthetic operation then expanded descendants with its own recursive walk.
- With Ctrl+A on the 10,000-entry fixture, the dialog said "10,000 items" and the operation changed 10,005: five entries sit inside Documents and Photos.
- The expansion also scanned every entry at each step and used a `Vec` membership check, O(n²) on the UI thread.
- It ignored destinations, so an item already in Trash inside a folder was counted again and given a new deletion time.

**One authoritative scope** (`src/preview/model.rs`):
- `PreviewModel::plan(operation, selection)` returns an `OperationPlan`:
  - the operation;
  - the distinct selected IDs that exist in the operation's destination (Files for Move to Trash, Trash for Delete permanently);
  - every affected ID: the selection plus descendants in the same destination;
  - the fixture revision.
- **The walk** is iterative over a parent-to-children index, with a visited set. That makes it O(n log n), with no recursion, no double counting of a selected descendant and no infinite loop on a malformed cycle. Planning the 10,000-entry fixture is asserted to take under 1 s even in debug builds.
- **`execute_plan`** changes exactly the plan's affected set. It refuses with `PlanError::Stale` and changes nothing if the fixture changed after the plan was made.
- **`move_to_trash` and `permanently_delete`** now call `plan` and `execute_plan`, so the scope has a single interpretation.
- **Boundaries.** One preview model is one synthetic vault, and there is no account state in it. The destination boundary is explicit in the operation.

**Dialog flow** (`src/ui/dialogs.rs`):
- The plan is made once, when a Trash or Delete dialog opens. Every entry point goes through it: command bar, More menu, context menu, Del / Shift+Del, and the scenario preset.
- The body text and the completion notice come from the plan.
- **At confirmation**, the plan is recomputed from the current selection and fixture. If the result differs, nothing runs, the dialog closes, and the status bar says: "The items changed after the dialog opened, so nothing was moved or deleted. Check the selection and try again."
- **For future production work:** a change to the affected set must require a new confirmation, never a silent widening. This is documented on `OperationPlan`.

**Selected versus affected:** the selection count stays in the status bar ("10,000 selected"). The confirmation leads with the affected total whenever folders bring in unselected items.

| Case | Confirmation text |
|---|---|
| One file | “Annual report.txt” will be moved to Trash. |
| One folder with contents | 4 items will be moved to Trash: “Documents” and 3 items inside it. |
| Ctrl+A on 10,000 entries | 10,005 items will be moved to Trash: the 10,000 items you selected and 5 items inside them. |
| Permanent deletion of a folder | 4 items will be permanently deleted: “Documents” and 3 items inside it. This can't be undone. |
| Permanent deletion without contents | “Deleted sample 05.zip” will be permanently deleted. This can't be undone. |
| After confirming | Moved 10,005 items to Trash. / Permanently deleted 4 items. |

The captions are unchanged ("Contents of selected folders move with them…" and "Sample data — no real file or vault is affected."), and so are the dialog layout and buttons.

**Exact affected-set tests** (`tests/preview_model.rs`, `ui/dialogs.rs`, `ui/app.rs`):

| Case | Expected affected IDs |
|---|---|
| One selected file (4) | {4} |
| One empty folder (3) | {3} |
| Folder with children (2) | {2, 201, 202} |
| Nested folders, five levels | {1, 101, 102, 103, A, B, C} |
| Two independent folders (1, 2) | 7 IDs, no overlap |
| Parent plus selected descendants (1, 101, 103) | {1, 101, 102, 103}: 3 selected, 1 included |
| Duplicate and unknown IDs | Ignored safely |
| 10,000 roots | All 10,000 roots plus {101, 102, 103, 201, 202} = 10,005, both at the model level and end to end (Ctrl+A, dialog, real click on the confirm button, then compare the trashed set) |
| Permanent deletion of a folder | Exactly {1, 101, 102, 103} removed; other Trash and active items remain |
| Stale plan (rename after planning) | `Err(Stale)`, revision and entries unchanged |
| Selection changed before confirmation | Nothing moves; error message shown |
| Cancellation (Escape) | Plan discarded; fixture revision unchanged |
| Item already in Trash inside a moved folder | Not counted or re-trashed; keeps its original deletion time |

**Dialog safety preserved, with tests:**
- Activation is ignored on the frame a dialog opens.
- Cancel has the initial focus.
- Enter never confirms Trash, Delete, Import or Export unless the confirm button is focused and the press is fresh.
- Held and repeated Enter cannot confirm.
- Escape cancels.

New tests cover Enter on the opening frame and on Cancel with a folder selected: no descendant is touched.

## Validation (at `c53c42e`)

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace --locked` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --locked` | PASS: 43 (3 core, 10 library, 30 desktop) |
| `cargo check -p encryptdrive-desktop --features design-preview --locked` | PASS |
| `cargo clippy -p encryptdrive-desktop --features design-preview --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --features design-preview --locked` | PASS: 131 (3 core, 10 library, 88 desktop, 30 model) |
| `cargo build --workspace --release --locked` | PASS |
| `cargo build -p encryptdrive-desktop --features design-preview --release --locked` | PASS |
| `scripts/verify-architecture.ps1` | PASS; no new dependencies |

**Note on warnings.** The only "warning" lines in the log are rustc incremental-cache notices: hard-linking failed in `target\debug\incremental`, so rustc copied the files instead. That is a filesystem condition, not a code warning. Clippy with `-D warnings` passed for both builds.

UX6.1 behaviour has no source changes and is covered by the existing tests:
- the reentrant-input hang fix;
- filename click selection;
- Ctrl+A and Ctrl+F;
- folder double-click;
- stable table geometry;
- 10,000-row frame timing;
- keyboard navigation;
- the command overflow and its accessibility names.

## Candidate artifacts (commit `c53c42e`)

The copies are kept outside version control, in `target/ux62-artifacts/`. The link-normalised hash zeroes only the link timestamps and the PDB GUID; see the UX6.1 report for why.

| Variant | Size | Raw SHA-256 | Link-normalised SHA-256 |
|---|---|---|---|
| Preview (`--features design-preview`) | 15,528,960 | `625852C5FAEB3E2CCC6582D79C79D011433AD4074670380A71F71EEF0A609B53` | `3425B191D844F38E1A1048ECFB1BDB56C66B5E3002AA2A227F8F55C166A35734` |
| Shell | 14,870,016 | `3F02D103893D144444887AAE85DE0B67F6825A34F4A6418E797FB12A9F945183` | `3BF5BE50CBE8CC59C3341C69C6AB5E96B70A24C79C3E5E0E0B3B6EEBD7EA06AC` |

**Captures** (`captures/ux62/`, `PrintWindow` of the app window only, from the preview artifact above): `permanent_delete_880_light_c53c42e.png` and `permanent_delete_880_dark_c53c42e.png`.
- They show the unchanged dialog design and the wording for an item without contents.
- The wording for folders with contents is verified by the headless tests, not by a capture. The preview has no flag that opens that state without input.

## Commits

| Commit | Subject |
|---|---|
| `ef43e08` | fix(windows): reconcile minimum window size with usable work area |
| `c53c42e` | fix(ui): align recursive deletion confirmation with affected scope |
| this commit | docs(ux): record UX6.2 audit remediation |

## Remaining manual checks

- **Higher scaling.** At 125 %, 150 % and 200 %, run `tools/measure-window.ps1` with the preview and shell artifacts. Expect `inside_work_area = True`, a client size of at most the work area minus the frame, and a native minimum no larger than the window.
- **Second monitor.** With a monitor left of or above the primary, launch the app and drag it there. The window must stay where it is put.
- **Taskbar on another edge.** With the taskbar at the top or on a side, launch the app and confirm nothing sits behind the taskbar.
- **10,000-entry flow by hand.** Ctrl+A on 10,000 entries, then Del. The dialog must say 10,005 items, and after confirming the status must say "Moved 10,005 items to Trash."
- **Small-window scrolling.** Scroll the auth screens at a small window size, with the mouse wheel and by tabbing to the buttons.
- **Still pending from UX6.1:** keyboard use of the More menu, Narrator and High Contrast.

## Handoff for independent verification

1. Check out `main` at the docs commit that follows `c53c42e`. There should be no local changes.
2. Run the validation table above.
3. Read `src/ui/work_area.rs`: the policy, the units and the native calls.
4. Read `src/preview/model.rs`: `plan`, `execute_plan`, `expand`.
5. Read the `RunPlan` branch in `src/ui/dialogs.rs`.
6. Compare the tests named in this report with the brief's mandatory scenarios.
7. If a different DPI or monitor layout is available, run `tools/measure-window.ps1` against freshly built artifacts.

Rebuilt raw hashes will differ by the link metadata; compare the link-normalised hashes instead.
