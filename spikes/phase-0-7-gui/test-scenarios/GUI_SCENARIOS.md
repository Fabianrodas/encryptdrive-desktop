# Shared GUI verification script

Run against each Release executable with the same Windows account, display, logical window size (1180×760), OS appearance, DPI mode and fixture size. Record actual values and retain screenshots from the live application; do not infer successful behavior from source code.

1. Launch with `--entries 100`; capture initial Files view and note first paint, visible rows, window size and any runtime dependency prompt.
2. Count or inspect the 100-entry root list; verify folders, files, size, Modified, Unicode and elided long names.
3. Select 1,000 entries from Settings and repeat; note resize/scroll behavior.
4. Select 10,000 entries; note first-paint latency, scroll latency, memory working set, CPU and responsiveness while resizing. Repeat after a cold launch and warm launch.
5. Resize to the minimum and restore 1180×760; verify columns and controls remain reachable and no horizontal clipping hides required actions.
6. Search `Synthetic document 000`, clear search, then sort Name ascending/descending, Size ascending/descending, Modified ascending/descending. Check that selected stable IDs remain selected after sorting and are pruned when filtered out.
7. Select one item, Ctrl-click a second, Shift-click a range, then Ctrl+A. Confirm Ctrl+A affects displayed results only. Sort after selecting and confirm selected identity does not change.
8. With mouse disconnected, Tab/Shift+Tab across the UI. Focus a file row, use Up/Down, Enter, Escape, Ctrl+A, Ctrl+F, F2, Delete and Shift+F10. Record focus indicators and any key not handled. Also try Alt+Left and Alt+Up.
9. Open a pointer context menu and Shift+F10 menu; use Open / simulated Export, Rename, Move to Trash/Restore and dismiss.
10. Create a blank folder name, a valid folder, and a case-insensitive colliding name in Collision Lab. Rename with blank, reserved Windows characters, >255 characters, valid, and colliding names. Capture accessible validation.
11. Verify Delete to Trash and separate permanent-delete confirmation from Trash. Verify simulated Import validates a name and inserts only synthetic state after success.
12. Double-click Projects, then a nested file. Confirm folder navigation and the explicit plaintext Export warning. Continue the simulation and verify no file picker, file I/O, plaintext output or process launch.
13. Switch Light↔Dark repeatedly. Capture both themes at the same dimensions. Verify contrast, selected/hover/focus/disabled/error states.
14. Start the long operation, observe progress for at least two seconds, cancel, and confirm the queued synthetic mutation did not occur. Start a controlled failure, record the error state. Start and complete a success path.
15. In Settings, simulate Loading then completion; simulate Error then retry; show and acknowledge Recovery notification.
16. Test at Windows 100%, 125%, 150% and 200% scaling; repeat after moving between monitors with different scaling. Record effective size and text clipping.
17. Inspect the Windows UI Automation/AccessKit tree with Inspect.exe or Accessibility Insights for Windows. Check app/window name, navigation landmarks, search label, table/list role, row names/selection state, headers, buttons, dialogs, errors and progress announcements. Then repeat with Narrator and record speech/focus order.
18. Turn off network access and launch both existing Release executables from their final delivery folder. Record missing-runtime prompts and confirm no download attempt or network dependency.

## Evidence capture form

For every run record prototype, Git/source revision or archive hash, Rust/Cargo versions, Windows build, GPU and driver, scaling, renderer, fixture count, cold/warm state, measured memory and timings, scenario result (`PASS`, `FAIL`, `BLOCKED`, `NOT RUN`), issue, and screenshot path. Keep screenshots original; do not recreate or annotate them as if captured from the application.

## Modified timestamp test note

The Modified header/sort tests in scenarios 2 and 6 exercise deterministic synthetic fixture values only. Java VF1 stores modifiedAt and displays it, but the Java model initializes it equal to createdAt and the inspected operations do not update it. The Java Files and Trash FXML disables sorting for all listed columns. This spike behavior is a proposed UX enhancement, not verified Java compatibility or production date semantics.