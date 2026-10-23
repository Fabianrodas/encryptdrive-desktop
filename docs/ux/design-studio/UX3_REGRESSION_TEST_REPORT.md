# UX3 R3 Regression Test Report

**Run date:** 2026-10-20  
**Validation:** Locked workspace, default shell and design-preview feature  
**GUI execution:** NOT RUN; R3 requires owner-operated manual validation.

## Commands and results

All commands were run from the repository root with the installed stable MSVC toolchain:

| Command | Result |
|---|---|
| cargo +stable fmt --all -- --check | PASS |
| cargo +stable check --locked --workspace --all-targets | PASS |
| cargo +stable check --locked --workspace --all-targets --features encryptdrive-desktop/design-preview | PASS |
| cargo +stable clippy --locked --workspace --all-targets -- -D warnings | PASS |
| cargo +stable clippy --locked --workspace --all-targets --features encryptdrive-desktop/design-preview -- -D warnings | PASS |
| cargo +stable test --locked --workspace --all-targets | PASS — 27 tests, 0 failed |
| cargo +stable test --locked --workspace --all-targets --features encryptdrive-desktop/design-preview | PASS — 84 tests, 0 failed |
| cargo +stable build --locked --workspace --release | PASS |
| cargo +stable build --locked --package encryptdrive-desktop --release --features design-preview | PASS — final candidate |
| ./scripts/verify-architecture.ps1 | PASS — workspace membership and direct dependency boundaries verified |

Test totals: default 3 core + 10 desktop library + 14 UI/design tests. Preview 3 core + 10 desktop library + 53 UI/design/dialog tests + 18 preview-model integration tests. Additional example/library targets reported zero tests. Cargo emitted warnings that the sandbox filesystem could not hard-link incremental cache files and copied them instead; command exit codes remained zero. No warning was suppressed.

## Focused regression evidence

| Concern | Tests / assertion | Automated result | Boundary |
|---|---|---|---|
| Enter opening a file cannot skip Export warning | entering_a_file_and_rendering_the_same_enter_keeps_export_warning_open exercises the same egui input frame through the global handler and modal renderer. | PASS: warning stays open; operation remains idle. | Egui event path; physical key not run. |
| Fresh, repeated, held Enter and deliberate confirmation | held_enter_cannot_dismiss_or_confirm_the_export_dialog_that_it_opened, export_confirmation_requires_a_distinct_enter_on_the_confirm_control, enter_on_the_initial_cancel_focus_cancels_export_instead_of_starting_it, repeated_enter_cannot_confirm_after_focus_moves_to_a_destructive_action. | PASS: repeat cannot confirm; initial Cancel cancels; a released key followed by a fresh Enter on the focused confirm control can confirm. | Does not prove OS keyboard delivery or visible focus. |
| Destructive Cancel focus | enter_on_cancel_cannot_move_or_permanently_delete_selected_items. | PASS: Move to Trash and Permanent Delete cancel without changing synthetic item state. | Synthetic fixture only. |
| Forms and Escape | held_enter_cannot_submit_a_form_until_a_distinct_key_press, enter_only_submits_create_and_rename_forms_from_their_text_field, escape_closes_export_without_starting_an_operation. | PASS: form ownership/fresh confirmation and Escape cancellation are exercised. | No platform IME/manual focus testing. |
| Table focus policy | left_right_and_escape_keep_keyboard_focus_in_the_table, table_focus_keeps_arrows_and_escape_and_allows_tab_traversal, table_arrow_home_end_page_and_space_events_update_stable_selection. | PASS: Up/Down/Home/End/Page/Space navigation and focus policy are asserted; Tab is allowed to traverse. | Native Tab, Shift+Tab, Alt navigation, and Shift+F10 are pending. |
| Shortcut and text-edit routing | focused_non_text_widget_does_not_block_explorer_shortcuts, ctrl_f_is_routed_to_search_when_the_table_has_focus, focused_text_edit_keeps_global_shortcuts_out_of_its_input, delete_does_not_target_an_item_hidden_by_a_changed_query. | PASS: Ctrl+A targets displayed results, Ctrl+F requests Search, text editing keeps input ownership, hidden selections are not actionable. | Real Windows focus and keyboard are pending. |
| Filename and folder pointer routing | clicking_the_rendered_filename_selects_its_stable_entry_id, double_clicking_the_rendered_folder_name_navigates_into_that_folder, double_clicking_a_file_opens_the_export_warning_without_starting_it. | PASS: real rendered widget responses are driven through egui test input. | Not physical mouse interaction. |
| Selection and feedback geometry | selecting_a_row_does_not_move_the_rendered_table, selection_and_feedback_transitions_preserve_the_rendered_table_geometry, compact_commandbar_keeps_table_geometry_when_selection_actions_appear. | PASS: row origin stays identical through selection, notice, error, progress, and compact selection toolbar. Standard test screen row origin is 173 logical px; an earlier headless measurement before the layout refactor was 197 px. | Logical headless geometry, not live pixels. |
| Notification precedence | operation_error_and_notice_states_render_only_one_feedback_message, feedback_states_use_existing_semantic_status_colors. | PASS: one feedback message and semantic status colors. | Long-list visibility/clipping need screenshots. |
| State styling and Light focus | hover_pressed_focus_and_disabled_styles_use_distinct_semantic_treatments, focused_light_text_field_emits_a_semantic_focus_outline, theme contrast tests. | PASS: token relationships and 2 px Light focus stroke are emitted. | Shape output is not rasterized UI and does not establish Narrator/UIA support. |
| Table label and row alignment | sorted_header_keeps_its_text_separate_from_the_icon_glyph, metadata_cells_center_their_text_in_the_row_height. | PASS: full sort label has separate geometric marker; metadata center offset is bounded to 0.5 logical px. | Final Release glyph rendering still needs inspection. |

## Not run

- No executable launch, mouse/keyboard injection, screenshots, or foreground-window automation.
- No Windows Narrator, UI Automation/Accessibility Insights, High Contrast, or 100/150/200% DPI session.
- No offline disconnect/relaunch and no imported-DLL inspection.
- No owner review of Release state styling, sort marker, metadata baselines, black strip, or shell/preview visual consistency.

See the manual R3 validation matrix, Release candidate report, and R3 remediation matrix. Do not infer live GUI passes from headless tests.
