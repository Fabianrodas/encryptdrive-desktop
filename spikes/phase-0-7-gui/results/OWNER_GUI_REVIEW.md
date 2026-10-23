# Owner GUI Review — Phase 0.7E

**Status: NOT RUN.** The updated executables are built for direct owner review. This procedure is intentionally manual because desktop UI automation was stopped with the physical Escape key. No new screenshots, UIA observations, Narrator results, or visual claims are implied by a successful build.

## Direct launch

Run these commands from the repository root in PowerShell. They launch the already-built Windows executables; Cargo and the Rust toolchain are not needed at launch time. The binaries use synthetic in-memory data only.

```powershell
& '.\spikes\phase-0-7-gui\target\phase-0-7e\release\encryptdrive-eframe-spike.exe' --entries 100
& '.\spikes\phase-0-7-gui\target\phase-0-7e\release\encryptdrive-slint-prototype.exe' --entries 100
```

Repeat each launch with `--entries 1000` and `--entries 10000`. Exit one app before starting the paired comparison. Use the same monitor, Windows display scale, and window size for each comparison. Record the actual display scale and window dimensions below. A direct launch on this development host is not a clean-host/offline deployment test.

## Run record

| Field | eframe | Slint |
|---|---|---|
| Date / reviewer | NOT RUN | NOT RUN |
| Monitor resolution | NOT RUN | NOT RUN |
| Windows display scale | NOT RUN | NOT RUN |
| Window client size | NOT RUN | NOT RUN |
| Fixture size(s) | NOT RUN | NOT RUN |
| Screenshot path(s) | none captured | none captured |
| UIA tool and version | NOT RUN | NOT RUN |
| Narrator version / speech result | NOT RUN | NOT RUN |

For each row below, mark **PASS**, **FAIL**, **BLOCKED**, or **NOT RUN** and note the observed result. Do not infer a pass from source code, compilation, or a prior build.

## Paired explorer scenarios

Run the same action in each app before moving to the next scenario. Keep screenshots genuine and save them only when an actual application state has been captured.

| Scenario | eframe | Slint | Observation / evidence path |
|---|---|---|---|
| Initial launch, compact header, sidebar, breadcrumbs, table density | NOT RUN | NOT RUN | |
| 100 / 1,000 / 10,000 entries; scroll to beginning, middle, and end | NOT RUN | NOT RUN | |
| Resize to the minimum window and restore; check clipping and focus | NOT RUN | NOT RUN | |
| Search results, clear search, sort Name / Size / Modified | NOT RUN | NOT RUN | |
| Single select; Ctrl+click; Shift+click; Ctrl+A; sort/filter with selection | NOT RUN | NOT RUN | |
| Arrow-key focus and Enter on folder/file; Escape behavior | NOT RUN | NOT RUN | |
| Tab and Shift+Tab through all actions; confirm visible focus | NOT RUN | NOT RUN | |
| Ctrl+F, F2, Delete, Shift+F10, Alt+Left, Alt+Up | NOT RUN | NOT RUN | |
| Context menu actions and cancellation | NOT RUN | NOT RUN | |
| Create Folder and Rename: blank, invalid, collision, Unicode, success | NOT RUN | NOT RUN | |
| Delete to Trash; switch Files/Trash; permanent-delete confirmation | NOT RUN | NOT RUN | |
| Simulated Import; simulated Export plaintext warning; no file picker/open | NOT RUN | NOT RUN | |
| Light and Dark themes; text, controls, selected and focused rows | NOT RUN | NOT RUN | |
| Loading and error/retry states | NOT RUN | NOT RUN | |
| Long operation progress, disabled controls, cancel, success, failure | NOT RUN | NOT RUN | |
| Recovery notification | NOT RUN | NOT RUN | |
| 100%, 125%, 150%, and 200% Windows scaling where available | NOT RUN | NOT RUN | |

Do not use a real vault or real files. “Import” and “Export” in these prototypes must remain simulated. Cancel any operation before exiting if its state is unclear.

## Accessibility pass

With Accessibility Insights for Windows or Inspect.exe, inspect the application and file rows at initial state, after selecting one row, after multi-select, inside a validation dialog, and during operation progress. Record each visible control's UIA role, name, focus, enabled state, selected state, and actions. Then use Windows Narrator with keyboard only to navigate the same states. Verify that:

- Files, Trash, breadcrumb navigation, search, table headers, rows, actions, appearance/settings, and dialogs have understandable names and roles.
- The focused row and selected rows are distinguished both visually and by spoken state.
- Create/Rename validation and operation progress/cancel/success/failure are announced at useful times.
- Dialog focus starts in the name input when present, Escape closes appropriately, and closing returns focus to a meaningful control.
- Every required action is reachable without a pointer, with no focus trap or silent keyboard action.

Record Narrator speech excerpts as observations, not as a score. If any important item is missing or ambiguous, mark that accessibility gate **FAIL** or **BLOCKED**; do not assign an accessibility pass based only on the UIA tree.

## Offline and installation follow-up

Run each application from a clean Windows VM with network disconnected and no Rust/MSVC development tools. Record startup result, any runtime installer prompt, loaded non-system modules, and the PE imports listed in `PERFORMANCE_AND_BINARY_REPORT.md`. This is **NOT RUN** by this direct-launch procedure on the development host. The ordinary binaries currently import the MSVC runtime; test whether the deployment environment supplies it or evaluate the separately documented static-CRT build before packaging.

## Owner conclusion

| Gate | eframe | Slint | Notes |
|---|---|---|---|
| Visual quality and density | NOT RUN | NOT RUN | |
| Basic and keyboard interactions | NOT RUN | NOT RUN | |
| Accessibility / Narrator | NOT RUN | NOT RUN | |
| DPI and resize | NOT RUN | NOT RUN | |
| Offline clean-host execution | NOT RUN | NOT RUN | |
| Dependency/license approval | OPEN | OPEN | See dependency and license review. |
| Candidate selection | NO SELECTION | NO SELECTION | Select only after mandatory gates are resolved. |

