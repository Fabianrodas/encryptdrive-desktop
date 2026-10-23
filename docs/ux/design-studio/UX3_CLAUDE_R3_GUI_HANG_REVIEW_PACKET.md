# Claude Code — Read-only R3 GUI Hang Review Packet

**Review requested:** independent, read-only assessment of the Phase 1A-UX.4 stabilization evidence.  
**No verdict is prewritten.** Do not modify, stage, commit, or clean repository files. Do not send input to unrelated windows. No design approval is requested by this packet.

## Candidate

- Repository: C:\Users\Asus Vivobook\Desktop\encryptdrive-desktop
- Candidate executable: target/release/encryptdrive-desktop.exe
- Launch: & 'C:\Users\Asus Vivobook\Desktop\encryptdrive-desktop\target\release\encryptdrive-desktop.exe' --design-preview
- Candidate SHA-256: 606D24A4946899EC833A3AD5E2265220A6B2307FE81B3602F52CB0E008510568
- Size: 15,585,792 bytes
- Lockfile SHA-256: 3CB09A9815F937F30137C3B9169F33834B1412CEAD428406613A65C99A64D754
- Rust/Cargo: 1.99.1, Windows MSVC x86_64.
- A native idle window was sampled five times as Responding=True and initially left open for owner interaction. A final process check later found the PID absent; whether the owner closed it or it exited is unknown. No button or keyboard input was injected by the implementation agent.

Read the complete incident report in UX3_R3_GUI_HANG_INCIDENT_REPORT.md before reaching conclusions.

## Root-cause claim to review

In ../../../apps/encryptdrive-desktop/src/ui/components.rs, button_activated_when_allowed previously called response.clicked_by(...) and response.clicked() inside a ui.input closure. The debug egui lock detector caught nested Context input/write access after a synthetic pointer click in the full-app UI. The correction evaluates the response click accessors before taking the input closure; the closure now reads only Enter/Space state.

Please independently inspect the old/current flow as available from the incident report and saved baseline snapshot beneath target/incident-baseline-phase1a-ux4. Determine whether the change removes reentrant context access on the click path without changing pointer or keyboard semantics. The debug test custom_button_activation_can_read_input_without_reentrant_locking is at components.rs:678.

The owner’s exact native “Not Responding” occurrence was not reproduced by this review run. Treat the code-level deadlock and attribution to that native occurrence as distinct claims.

## Runtime tests and outcomes

The tests exercise actual eframe App::ui frames through synthetic egui input at a fixed 1180×792 screen size. They do not start the native winit event loop.

Key tests are in ../../../apps/encryptdrive-desktop/src/ui/app.rs:
- complete_preview_app_frames_remain_bounded_with_ten_thousand_synthetic_entries
- synthetic_full_app_click_opens_settings_and_switches_theme_without_blocking
- synthetic_full_app_click_opens_and_validates_folder_dialog
- synthetic_full_app_row_click_updates_selection_toolbar_and_keyboard_focus
- synthetic_full_app_preview_state_selector_opens_with_accessible_options
- synthetic_full_app_list_size_selector_changes_fixture_size
- synthetic_full_app_scenario_transition_and_search_update_visible_results
- repeated_enter_on_open_trash_dialog_never_moves_the_selected_sample_item

A focused 10,000-entry run passed with 24 complete headless frames, median 2.10 ms and max 158.97 ms. A first full parallel preview-suite run failed two 2-second wall-clock assertions; its 10,000-entry sample showed median 9.04 ms and a single 8,103.72 ms outlier. A serial 92-test run and a subsequent parallel 92-test run both passed. The reason for the outlier is not proven; assess whether the watchdog strategy or test isolation warrants a follow-up rather than treating the single pass as conclusive.

Validation evidence:
- Default workspace: fmt, check, strict Clippy, 27 tests, release build passed.
- Preview feature: check and strict Clippy passed; serial 92-test run passed; repeated parallel 92-test run passed after one parallel run with the timing failures; release build passed.
- scripts/verify-architecture.ps1 passed.
- encryptdrive-core’s normal dependency tree contained no egui, eframe, winit, wgpu, or Slint packages.
- The current candidate was rebuilt after validation.
- Incremental cache hard-link warnings occurred; Cargo copied files and commands exited successfully.
- Windows Application Error/WER query was denied. Binary inspection tools dumpbin, objdump, and llvm-objdump were unavailable on PATH.

## Security and behavior that must not regress

The 61-test preview binary suite passed on the successful runs. Relevant regression tests verify:
- Enter on a file cannot open and confirm Export in one event.
- Held/repeated Enter cannot immediately confirm a newly opened dialog.
- Enter on initial Cancel focus cannot move to Trash or permanently delete.
- Export confirmation requires a distinct Enter on the confirm control.
- Table shortcuts remain available when non-text controls have focus and do not consume search text.
- Filename clicks select stable IDs; folder double-click navigates; file double-click opens the warning only.
- Selection and feedback do not move table geometry.
- Operations remain synthetic and do not access real files.

Review these guards alongside the changed helper; do not infer from a green test alone that native behavior is verified.

## Review focus

1. Confirm whether the nested egui context-lock path is a credible root cause and whether the patch removes the re-entry at the right layer.
2. Check for other response accessors, callbacks, or theme/font reconfiguration executed while holding egui input locks.
3. Examine the new full-app frame tests for genuine action coverage, false positives, timing flakiness, and assertions that could pass without a click taking effect.
4. Assess the 10,000-entry timing evidence separately from native event-loop/GPU responsiveness.
5. Review export-warning, destructive-confirmation, text-input routing, filename-click, and table-geometry regression tests for security or input-routing regressions.
6. Identify native verifications that remain necessary. Do not declare native interaction PASS from idle Responding samples.
7. Verify source-scope and repository safety: only three Rust source files differed from the pre-edit snapshot; 29 Phase 0.7 spike files remained unchanged; Java and encryptdrive-docs were not modified.

If you perform a review, report evidence with file and line references, distinguish verified findings from hypotheses, and provide a severity for any remaining defect. Do not edit the workspace or manufacture an approval verdict.


