# UX6.1 — Responsive refinement and audit closure

**Start:** `main` at `28bc8b6`, clean working tree.
**Input:** the UX6.1 brief from the owner. The Codex UX6 audit verdict was "conditionally ready, no P0/P1". The audit document itself is not in this repository, so this work follows the brief.
**Status:** candidate for independent review. This is not owner approval of the design.

## 1. Responsive command bar

**Problem.** Below fixed content-width breakpoints (840 and 600 px), UX6 removed labels from every secondary command at once, even when there was room.

**Change** (`260d83a`):
- Each command's labelled width is measured with the same fonts and padding the button paints with. A test checks that the measured width equals the painted width.
- Commands stay inline and labelled, in priority order, while they fit:
  - Files: New folder, Import, Export, Rename, Move to Trash.
  - Trash: Restore, Delete permanently.
- Search gives up its spare width first, down to 160 px. Only then do the lowest-priority commands move into a **More commands** (⋯) menu.
- No command is ever dropped: every command is either inline or in the menu.
- Menu rows show:
  - the icon, the full label and the Windows ellipsis for commands that open a dialog;
  - the same shortcut hints as the row context menu (F2, Del, Shift+Del);
  - the same enabled rules as the bar;
  - "Delete permanently" in the danger colour.
- Bar and menu call one `run_command` function, so the dialogs, confirmations and the Restore notice are the same whichever way a command is chosen.

**Keyboard and focus:**
- More is a normal focusable button with a tooltip and the accessible name "More commands".
- Opening it from the keyboard moves focus to the first enabled item.
- Escape, or choosing Restore, returns focus to More.
- Commands that open a dialog use More as the dialog's invoker, so focus returns there when the dialog closes.
- While the menu is open, the existing global handler ignores table shortcuts and Escape.

**What overflows at each size** (Files, 125 %, synthetic multi-selection):

| Window | Inline | In More |
|---|---|---|
| 880×600 (minimum) | New folder, Import, Export, Rename | Move to Trash |
| 960×640 | New folder, Import, Export, Rename | Move to Trash |
| 1180×792 and wider | All five | — (no More button) |

Trash keeps both commands labelled at 880×600.

All rows come from captures of the final build (section 6).

## 2. Monitor work area

**Problem.** `fit_window_to_monitor` used the monitor size minus fixed margins. It assumed a bottom taskbar and a primary monitor at the origin.

**Change** (`59aa866`), in `src/ui/work_area.rs`:
- **Monitor lookup.** Take the window centre and find its monitor with `MonitorFromPoint(MONITOR_DEFAULTTONEAREST)`, then read `rcWork` with `GetMonitorInfoW`.
  - These are read-only calls; nothing in the display configuration changes.
  - No new dependency: it is a small `extern "system"` block with a `SAFETY` note.
- **Coordinate conversion.** egui reports window rects as physical pixels ÷ `pixels_per_point`, and the work area is converted with the same factor. Nothing is scaled twice, and negative virtual coordinates on secondary monitors are preserved.
- **Fitting rules:**
  - It runs once, on the first frame where egui reports the window geometry. A window the user later moves or resizes is never touched.
  - It only shrinks, never grows.
  - It centres the window in the work area when it has to shrink. If the window only overlaps a taskbar, it is nudged inside without resizing.
- **Fallback** when the query is unavailable, including non-Windows: monitor size minus a 48-point bottom taskbar.
- **Tests:**
  - 125 % conversion;
  - top taskbar;
  - secondary monitor at negative x;
  - a window that already fits is left alone;
  - an oversized window shrinks only on the overflowing axis;
  - an empty work area changes nothing;
  - a read-only call of the real API on this machine, which returned 1536×816.

**Live check** (final build, 125 %, bottom taskbar, physical work area 1920×1020). Window rects are as reported by `GetWindowRect` and include the invisible resize borders:

| Requested inner size | Window rect | Result |
|---|---|---|
| 880×600 | 1118×797 at 410,165 | Fits, left where Windows centred it |
| 960×640 | 1218×847 at 360,140 | Fits, left where Windows centred it |
| 1180×792 | 1493×1020 at 213,0 | Height shrunk to exactly the work area, centred horizontally |
| 1920×1080 | 1920×1020 at 0,0 | Shrunk to exactly the work area; nothing behind the taskbar |

**Not verified:** a taskbar on another edge, a second monitor, and the 100 %, 150 % and 200 % scale factors.

## 3. Accessibility

**Changes** (`06f78a8`):
- **Rows** announce what the columns show, for example "Annual report.txt, Text document, 73 KB, modified 2024-02-13 16:40". Folders omit the size. The selected state was already exposed.
- **Column headers** include the sort state ("Name, sorted ascending"); before, it was only a chevron.
- **The list** name includes its count ("Files, 100 items").
- **Decorative icons:**
  - 11 icons were added as text labels, so their private-use glyph appeared as text in the accessibility tree.
  - They are now painted and create no node: table rows, breadcrumb chevrons, status and feedback icons, the vault chip, "Local only", empty states and dialog warnings.
  - Each sits next to text that carries its meaning.
- **Menu rows** (overflow and context menu) use the MenuItem role. Their names never contain icon glyphs.
- **Dialogs** use the grouped count the status bar uses ("10,000 items", not "10000 items").

**Ctrl+A and hidden selections:**
- Ctrl+A selects every row in the current view, including rows scrolled out of sight. It does not select only the rows on screen.
- The status bar and the Trash and Delete confirmations count `selected_ids()`, so they report the full scope.
- Selection is cleared whenever the visible set changes: search text, folder navigation, Back, or a destination change. A selection can therefore never include rows that are filtered out or in another folder.

**Scope test.** A headless test loads the 10,000-entry fixture and presses Ctrl+A with the table focused. It checks that every row in the view is selected, the status bar reads "10,000 items · 10,000 selected" and the Move to Trash dialog says "10,000 items will be moved to Trash."

**Test.** An AccessKit tree test at the 880×600 minimum checks:
- the header sort name;
- the list count;
- the More commands button;
- New folder;
- that no label or value contains a private-use character.

**Limitations.** These need Narrator or a richer AccessKit mapping and are not claimed:
- Rows are list items, not a grid with column cells. There is no "row N of M" position.
- The sort state is in the name, not a sort property.
- More does not expose expanded or collapsed state.
- The parent–child nesting of rows under the list node has not been verified.
- Inspecting the AccessKit tree does not prove compatibility with Narrator or other screen readers.
- Neither Narrator nor High Contrast was run.

## 4. Build identity

**Experiment.** I built the same commit (`06f78a8`) twice per variant from a clean package state, with the same command, toolchain and directory.

The raw hashes differ, but only 20 bytes change, all link metadata:

| Field | Where |
|---|---|
| COFF `TimeDateStamp` | PE header |
| Debug-directory timestamps (3 entries) | `.rdata` |
| CodeView RSDS PDB GUID (16 bytes) | `.rdata` |

The generated code is byte-identical. With those fields zeroed, both builds of each variant hash the same.

**Why the auditor's rebuilt hashes differ.** In order of certainty:

1. **Every MSVC link writes a new timestamp and PDB GUID.** No rebuild reproduces a recorded raw hash, even on the same machine. The UX6 hashes cannot be re-derived by anyone, by construction.
2. **Absolute Cargo registry paths.** The binary contains the absolute registry path 935 times (panic locations in dependencies), for example `C:\Users\<user>\.cargo\registry\src\index.crates.io-…\egui-0.37.0\src\context.rs`.
   - A different user profile or `CARGO_HOME` changes those strings, and with them the size, layout and hash.
   - It also means release binaries carry the builder's Windows user name. Workspace sources are recorded as relative paths.
3. **Both variants share one output path.** The shell and preview builds both write `target/release/encryptdrive-desktop.exe`, so the file there is whichever was built last. UX6 recorded the preview as final; building the shell afterwards overwrites it.
4. **Inputs to match.** Toolchain `1.99.1` (pinned in `rust-toolchain.toml`), `--locked`, the default `release` profile (no overrides) and the `design-preview` feature. A build without `--locked` or with another toolchain would differ in substance.

**UX6.1 artifact manifest** (commit `06f78a8`; copies kept outside version control in `target/ux61-artifacts/`):

| Variant | Command | Size | Raw SHA-256 | Link-normalised SHA-256 |
|---|---|---|---|---|
| Shell | `cargo build -p encryptdrive-desktop --release --locked` | 14,868,992 | `1B75083A5F8FE3D8F17C3DE0B99CBFFE85791A97C7334486E536EA64C38F7452` | `F6D60DBF68712607BB5A71D3807C5676524D0D28AFE683AFF20A7BFE13FF49A7` |
| Preview | same command plus `--features design-preview` | 15,492,096 | `53A7F27FD2CD95040EB0FA62E7717F3BD4C49B390EC8762A0FBFA0D3875D1341` | `A8E9A2B25E98BAF596AE6E202F2BA8E28774AB26AE6C0465A706D1CDA668471F` |

The second link of each variant had raw hashes `99E4121B…` (shell) and `CA00ECF4…` (preview), with identical normalised hashes. The normalised hash zeroes only the fields in the table above. It is comparable only between builds from the same registry path.

**Recommendations** (not applied; each changes the build configuration and needs an owner decision):
- Link with `/Brepro` for deterministic timestamps and GUIDs.
- Use `--remap-path-prefix` for the registry and home paths. This makes builds comparable across machines and removes the user name.
- Give each variant its own `--target-dir` or artifact name, so one cannot overwrite the other.

## 5. Validation (at the docs commit; code as in `06f78a8` plus one test)

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace --locked` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` (default and `design-preview`) | 0 warnings |
| `cargo test --workspace --locked` | 37 passed (3 core, 10 library, 24 desktop) |
| Tests with `--features encryptdrive-desktop/design-preview` | 108 passed (3 core, 10 library, 77 desktop, 18 model) |
| Release builds, shell and preview | PASS (manifest above) |
| `cargo metadata --no-deps --format-version 1` | PASS |
| `scripts/verify-architecture.ps1` | PASS; no new direct dependencies |

UX6 behaviour has no source changes and is still covered by the existing tests:
- dialog activation gating;
- Cancel as initial focus;
- Enter never confirming Trash, Delete, Import or Export;
- key-repeat filtering;
- the reentrancy click test;
- keyboard navigation and stable selection.

The visual design is unchanged: husky mark, density, typography, inputs, blue folders, themes, dialog layouts and the export and delete warnings. There is no UI-scale setting and no real vault functionality.

## 6. Captures and build attribution

All captures use `PrintWindow` on the app's own window only, with no input sent. The owner confirmed the desktop was idle.

**Build used for every capture:** `target/ux61-artifacts/encryptdrive-desktop-preview-06f78a8.exe`, raw SHA-256 `53A7F27FD2CD95040EB0FA62E7717F3BD4C49B390EC8762A0FBFA0D3875D1341` (the manifest's preview artifact). I re-hashed it immediately before capturing.

`captures/ux61/` holds 16 files named `<view>_<width>_<theme>_06f78a8.png`:
- Views: `files` (multi-selection, 4 selected) and `trash`.
- Widths: 880, 960, 1180 and 1920, requested as 880×600, 960×640, 1180×792 and 1920×1080.
- Themes: `light` and `dark`.

What they show:
- **880 and 960:** Files keeps New folder, Import, Export and Rename labelled, and Move to Trash moves into ⋯. Trash keeps both commands labelled.
- **1180 and 1920:** every command is inline and there is no More button. Search grows to 320 px.
- **Both themes:** same layout; disabled, selected and destructive states are visible.
- `trash_880_light` shows a hover highlight on one row, because the idle mouse pointer happened to rest there. It is incidental; no input was sent.

**Earlier captures.** The three captures taken earlier from the `260d83a` build were removed. Their content is pixel-identical to the final build's; only the title bar differs, which is the active or inactive window state.

**Not captured:** the overflow menu open, because opening it needs input. It is on the owner checklist.

**Windows minimized during the earlier attempt.** Firefox was open and in the foreground before each launch. In several launches the app window took the foreground and was then minimized after about two seconds, as Firefox took the foreground back. In one launch the window disappeared. Those launches produced the failed captures. Whether this was owner activity or Firefox taking focus on its own could not be determined. I had treated it as owner activity, stopped launching windows, and waited for the owner's go-ahead.

**Re-check with Firefox closed and the owner not touching the PC.** I launched the final build 12 times: 4 times at 960×640, and twice each at 880×600, 1180×792, 1600×900 and 1920×1080. Its window state and the foreground app were logged every 250 ms for 6–8 s. The window was never minimized or closed, and the app kept the foreground until it was terminated. The app therefore does not minimize itself.

## 7. Commits

| Commit | Subject |
|---|---|
| `260d83a` | fix(ui): improve responsive command overflow |
| `59aa866` | fix(windows): respect monitor work area |
| `06f78a8` | fix(a11y): improve explorer accessible semantics |
| `3692f76` | test(ui): cover select-all scope with 10,000 entries (test code only; release binaries unchanged) |
| `8339d47` | docs(ux): reconcile UX6 audit and build evidence |
| this commit | docs(ux): add final-build captures to the UX6.1 report |

## 8. Owner-assisted checklist

| Item | State |
|---|---|
| Light/Dark captures of the final `06f78a8` preview at 880, 960, 1180 and 1920 (Files and Trash) | **Verified** (16 captures, section 6) |
| More menu: open with mouse, Enter and Space; arrow and Tab through items; Escape returns focus to More; disabled items when nothing is selected; danger colour on Delete permanently | **Pending.** Needs real input. |
| Window fit at startup | **Verified** with a bottom taskbar (section 2). **Pending** with the taskbar on a side or the top. |
| Second monitor, including one left of or above the primary | **Blocked.** No second display here. |
| 100 %, 150 % and 200 % scaling | **Blocked.** It is a system setting, so I did not change it. |
| High Contrast | **Pending.** Owner. |
| Narrator: row names, header sort names, list count, More menu, dialogs | **Pending.** Owner. AccessKit names verified headlessly only. |
| Ctrl+A on 10,000 rows, then Move to Trash: dialog says "10,000 items" | **Verified headlessly** (test). On-screen check pending. |
