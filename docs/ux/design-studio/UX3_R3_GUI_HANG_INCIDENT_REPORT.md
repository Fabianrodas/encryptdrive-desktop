# Phase 1A-UX.4 — GUI Hang Incident and Stabilization Report

**Date:** 2026-10-20  
**Status:** Stability source fix verified by headless tests; native interaction still pending. Visual refinement was not started. No approval or product-finalization claim is made.

## Executive status

- **CONFIRMED CAUSE (source-level):** a custom button helper queried egui click state while the same egui input lock was held. The pointer-click accessors re-enter egui context input/write access, producing a reentrant lock deadlock on the ordinary button path.
- **POSSIBLE CAUSE (owner’s native incident):** this defect explains the reported “Not Responding” symptom and was reproduced through the complete headless app UI. The owner’s exact native interaction was not reproduced independently, so attribution to that individual occurrence remains unconfirmed.
- **FIX VERIFIED BY TEST:** the pointer click accessors were moved outside the input closure. A debug egui lock-detector reproducer failed before the fix and passed after it. The full workspace and preview suites subsequently passed.
- **LIVE VERIFICATION PENDING:** the rebuilt Release preview window was launched and sampled passively five times. Its own Windows process reported Responding each time. No mouse or keyboard input was injected. Owner interaction with buttons remains required.
- **STILL OPEN:** a one-run wall-clock timeout outlier occurred in the parallel preview test suite; both serial and repeat parallel runs passed. The scheduling explanation is plausible but not proven. See “Timing-test anomaly.”
- **Visual refinement:** intentionally not performed because the owner’s live interaction is still the remaining stability gate.

## Incident baseline and safety

At investigation start, the Release artifact at target/release/encryptdrive-desktop.exe was 15,580,672 bytes and had SHA-256:

0230F8EBFFF2717BB7F8FB64E2AFB8BFD359D32A00EBD6F703C72412468E62BD

That exactly matched the historical candidate hash supplied in the task. This identifies the tested binary; it does not prove which native interaction caused the owner’s report.

Before source edits, a local 19-file UI source snapshot was saved beneath target/incident-baseline-phase1a-ux4. The repository remained on an unborn main branch, with origin/main reported gone. The Phase 0.7 spike source manifest was captured before changes and compared afterward: all 29 files are unchanged. No Git history, staging area, commits, tags, or remotes were changed. Git status was read with a per-command safe.directory option; global Git configuration was not changed.

Only synthetic preview data and synthetic operations were exercised. No vault, password, encrypted file, real import/export, secret, or user data was accessed. No input was injected into the desktop or another process. The Java and private documentation repositories were not modified.

## Root cause and fix

The shared button activation helper in ../../../apps/encryptdrive-desktop/src/ui/components.rs previously read response.clicked_by(...) and response.clicked() from inside ui.input(|input| ...). egui’s input closure owns a context lock. The response accessors themselves query egui context state, and the observed debug stack re-entered egui Context input/write while that lock was held. The debug lock detector reported the lock failure after the helper was invoked by a full-app synthetic pointer click.

The fix evaluates both response click accessors before entering ui.input. The input closure now reads only Enter/Space key state. This removes the nested context access at its source and leaves pointer and keyboard activation decisions intact. Relevant source lines are currently 169–188 in components.rs.

The new regression test is custom_button_activation_can_read_input_without_reentrant_locking at components.rs:678. It was observed failing with egui’s lock detector before the correction and passing afterward. The actual design-preview UI uses the same helper for ordinary buttons, dialogs, and selectors.

No renderer, framework, dependency, or timeout behavior was changed to mask the lock failure.

## Reproduction and action matrix

The original report concerns a native window. Headless synthetic frames can establish state transitions and catch egui lock re-entry, but cannot establish real Windows input behavior. Therefore, each native row below remains owner-verifiable.

| Action | Expected | Evidence and observed result | Native status |
|---|---|---|---|
| Settings | Open and close without blocking | Full-app synthetic click test opens Settings; theme test switches to Light and resolves it on the next frame. | NOT RUN — owner must click. |
| Theme | System default remains; Light/Dark overrides respond | Full-app test covers Light; existing semantic theme tests cover Light/Dark token resolution. | NOT RUN — owner must click both. |
| Preview-state selector | Open and change synthetic state | Full-app click opens the picker and exposes accessible options. Selecting the offscreen “Error state” option was not reached in the 792 px headless viewport. | NOT RUN — owner must verify selection. |
| List-size selector | Change fixture size without a hang | Full-app mouse events select 1,000 items and assert the model size changes. | NOT RUN — owner must click. |
| Files / Trash / Back / breadcrumbs | Navigate to the requested synthetic destination | Existing navigation/state tests pass; the full-app tests do not exhaust every native navigation control. | NOT RUN — owner must click. |
| Search | Accept text and update visible results | Full-app synthetic click/text test changes the query and verifies matching visible entries. | NOT RUN — owner must type. |
| Table row selection | Select by filename and update toolbar/focus | Full-app synthetic click test verifies stable ID, selection toolbar, and table focus. Explorer regression tests cover filename hit targets and stable selection. | NOT RUN — owner must click. |
| Create Folder | Open, validate empty input, cancel safely | Full-app click and Enter test keeps the dialog open and shows “Enter a name.”; Escape closes it. | NOT RUN — owner must verify. |
| Export warning | Opening the warning must not start/export plaintext | Double-click and keyboard regression tests preserve the warning; repeated/held Enter tests require a distinct confirmation input. | NOT RUN — owner must verify. |
| Trash/permanent-delete confirmation | Enter on Cancel must not delete or move an entry | Dialog regression test verifies both destructive actions are not performed. Full-app repeated-Enter test leaves the selected synthetic item active. | NOT RUN — owner must verify. |
| Repeated keyboard event | No accidental confirmation after a dialog opens | Full-app repeated-Enter test and dialog key-repeat regressions pass. | NOT RUN — owner must verify. |
| Import/export operation | Synthetic progress/cancel/error only | Preview-model operation tests cover bounded progress, cancellation, and failure without external work. | NOT RUN — owner must verify UI presentation. |

Native idle sampling is a separate, limited result: the current Release preview process (PID 17276, title “EncryptDrive — Design preview”) reported Responding=True in five samples about 700 ms apart. Its working set was about 339.3 MiB; reported CPU rose from 0.31 to 0.34 seconds. It was left open for owner interaction after those samples. A final process check later found PID 17276 no longer running; whether the owner closed it or it exited for another reason is unknown. This is an idle-window startup sample, not a button-interaction PASS.

## Runtime-oriented test coverage

The new complete-app frame helper invokes the actual eframe App::ui with a synthetic 1180×792 screen and egui input events. AccessKit is enabled where accessible controls are queried. The tests are headless and do not start or drive the native winit window.

Relevant tests in ../../../apps/encryptdrive-desktop/src/ui/app.rs:

- synthetic_full_app_click_opens_settings_and_switches_theme_without_blocking
- synthetic_full_app_click_opens_and_validates_folder_dialog
- synthetic_full_app_row_click_updates_selection_toolbar_and_keyboard_focus
- synthetic_full_app_preview_state_selector_opens_with_accessible_options
- synthetic_full_app_list_size_selector_changes_fixture_size
- synthetic_full_app_scenario_transition_and_search_update_visible_results
- repeated_enter_on_open_trash_dialog_never_moves_the_selected_sample_item
- complete_preview_app_frames_remain_bounded_with_ten_thousand_synthetic_entries

The 10,000-entry responsiveness test renders 24 complete egui app frames. An isolated fresh run measured median 2.10 ms and maximum 158.97 ms. This is headless CPU/frame-preparation timing; it is not a native frame-rate, GPU, Narrator, or event-loop measurement.

### Timing-test anomaly

One fresh parallel preview-suite run failed two wall-clock assertions: the 10,000-entry frame test recorded median 9.04 ms and one maximum of 8,103.72 ms, and the first frame of the Trash dialog test exceeded its 2-second limit. In that same run, 59 of 61 binary tests passed. No egui lock-detector failure or reproducible source defect was reported by those two assertions.

Follow-up evidence:
- Serial preview workspace test run: 92 passed, 0 failed.
- Repeat parallel preview workspace test run: 92 passed, 0 failed; the binary test group completed in 0.48 seconds.
- Separate 10,000-entry test run: 1 passed; median 2.10 ms, maximum 158.97 ms.

The run-to-run difference is consistent with machine/test scheduling contention, but that mechanism has not been independently proven. The wall-clock watchdog remains a test reliability concern and is recorded as STILL OPEN; it was not hidden or “fixed” by inflating the timeout.

## Security and interaction regressions

The following safeguards were re-run in the passing 61-test preview binary suite:

- Enter that opens a file’s Export warning cannot also confirm the newly opened warning.
- Held or repeated Enter cannot immediately dismiss or confirm a new dialog.
- Enter with Cancel initially focused cannot move an item to Trash or permanently delete it.
- Confirmation requires a distinct Enter on the actual confirmation control.
- Global shortcuts do not consume text input, and non-text widget focus does not disable explorer shortcuts.
- Clicking the rendered filename selects its stable entry ID; double-clicking a folder navigates, while double-clicking a file opens the simulated export warning.
- Selection-dependent toolbar appearance does not shift table geometry.
- Synthetic operations do not access the filesystem.

Named tests include entering_a_file_and_rendering_the_same_enter_keeps_export_warning_open, held_enter_cannot_dismiss_or_confirm_the_export_dialog_that_it_opened, enter_on_cancel_cannot_move_or_permanently_delete_selected_items, repeated_enter_cannot_confirm_after_focus_moves_to_a_destructive_action, clicking_the_rendered_filename_selects_its_stable_entry_id, and selecting_a_row_does_not_move_the_rendered_table.

## Source and evidence changes

The 19-file baseline comparison found exactly three changed Rust files:

- ../../../apps/encryptdrive-desktop/src/ui/components.rs — the root-cause fix and its lock regression test.
- ../../../apps/encryptdrive-desktop/src/ui/app.rs — full-app synthetic input tests and bounded 10,000-entry frame test.
- ../../../apps/encryptdrive-desktop/src/ui/navigation.rs — explicit AccessKit labels for the preview-state and list-size selectors; no visual redesign.

This report and the Claude review packet are new files beneath docs/ux/design-studio in encryptdrive-desktop. The existing nine Phase 0.5 UX documents were not modified. Phase 0.7’s 29 source files remain byte-identical to the pre-task manifest.

## Build and test verification

Toolchain used: Rust 1.99.1 (58cf8f9c3, 2026-10-13), Cargo 1.99.1 (5f94df478, 2026-08-27), x86_64-pc-windows-msvc, LLVM 23.1.1. The repository’s pinned toolchain is the same version. Invoking it without +stable was blocked because rustup could not create a temporary file under the user rustup directory; validation used cargo +stable/rustc +stable without changing global configuration.

Cargo.lock SHA-256: 3CB09A9815F937F30137C3B9169F33834B1412CEAD428406613A65C99A64D754. Locked direct GUI crates include eframe 0.37.0 and egui_extras 0.37.0.

| Check | Default workspace | design-preview feature |
|---|---|---|
| cargo fmt --all -- --check | PASS | Same formatted workspace |
| cargo check --workspace --locked | PASS | PASS |
| cargo clippy --workspace --all-targets --locked -- -D warnings | PASS | PASS |
| cargo test --workspace --locked | PASS — 27 tests | One parallel invocation had two timing assertion failures; serial rerun PASS — 92 tests; repeat parallel rerun PASS — 92 tests |
| cargo build --workspace --release --locked | PASS | PASS |
| scripts/verify-architecture.ps1 | PASS | PASS |
| GUI-framework dependency boundary in encryptdrive-core | PASS — no egui/eframe/winit/wgpu/Slint in its normal dependency graph | Same workspace result |

Cargo emitted warnings that incremental-cache hard links were unavailable and it copied cache files instead. Commands still completed with exit code zero. The warning does not identify a source or application failure.

The current Release design-preview candidate is 15,585,792 bytes:

- SHA-256: 606D24A4946899EC833A3AD5E2265220A6B2307FE81B3602F52CB0E008510568
- Path: target/release/encryptdrive-desktop.exe
- Launch from PowerShell: & 'C:\Users\Asus Vivobook\Desktop\encryptdrive-desktop\target\release\encryptdrive-desktop.exe' --design-preview

The candidate was launched using that command and left open as PID 17276 for the owner. No screenshot was captured. Windows event-log access was denied (“Attempted to perform an unauthorized operation”), so no application event record could be checked. dumpbin, objdump, and llvm-objdump were not available on PATH; native dependency enumeration was therefore NOT RUN. Successful native startup and idle window responsiveness were observed independently of that dependency inspection.

## Manual owner verification still required

PID 17276 was no longer present at the final process check; it may have been closed by the owner or exited, and the reason is unknown. Relaunch the candidate with the command above if needed, then use its own window and synthetic sample data:

1. Click Settings; switch Light, then Dark; close Settings. Confirm each action returns immediately.
2. Open the preview-state picker and choose an error or empty state. Switch list size between 100, 1,000, and 10,000.
3. Click Files, Trash, Back, and breadcrumbs.
4. Search for a visible sample name; clear the query. Click the filename itself, then use keyboard navigation.
5. Open New Folder, submit an empty name to see validation, then cancel.
6. Select a sample file and open Export. Confirm the plaintext warning stays visible. Cancel it; do not start an export.
7. Open Move to Trash or permanent-delete confirmation. Keep Cancel focused and press Enter once; confirm the dialog cancels and the synthetic item remains.
8. Exercise Import and the synthetic progress/cancel/error UI only. Do not select a real file or export plaintext.
9. If the window stops responding, note the last clicked control and approximate time; avoid sending input to other applications.

Narrator/UIA tree inspection, high-contrast mode, DPI scaling, and comparable screenshots were not repeated for this incident. These are not represented as PASS.

## Repository state and phase boundary

Final Git status remains an unborn main branch with origin/main [gone]. Existing top-level untracked entries remain .github/, .gitignore, Cargo.lock, Cargo.toml, README.md, apps/, crates/, docs/, rust-toolchain.toml, scripts/, and spikes/. No files were staged or committed. target/ contains the ignored source snapshot, manifests, validation logs, and Cargo artifacts.

Only encryptdrive-desktop was modified. The Java repository and encryptdrive-docs were not written. No production vault-format, vault-crypto, vault-storage, or encryptdrive-core implementation was introduced. Phase 1B and Vault Format 1 were not started.

**Gate decision:** stop at stability verification. The source-level deadlock fix is test-verified and a native idle candidate is available, but button interactions in the native window still need owner verification. Do not treat this report as final design approval.


