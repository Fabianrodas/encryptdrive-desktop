# UX3 R3 Owner Manual Validation Checklist

**Candidate:** Release with design-preview, SHA-256  
0230F8EBFFF2717BB7F8FB64E2AFB8BFD359D32A00EBD6F703C72412468E62BD  
**Status:** All owner-operated GUI checks below are NOT RUN. No GUI was launched by Codex.

## Launch this exact candidate without rebuilding

Confirm the executable hash, then start the preview directly:

    Get-FileHash 'C:\Users\Asus Vivobook\Desktop\encryptdrive-desktop\target\release\encryptdrive-desktop.exe' -Algorithm SHA256
    & 'C:\Users\Asus Vivobook\Desktop\encryptdrive-desktop\target\release\encryptdrive-desktop.exe' --design-preview

The expected SHA-256 is 0230F8EBFFF2717BB7F8FB64E2AFB8BFD359D32A00EBD6F703C72412468E62BD. These commands do not rebuild. Stop if the hash differs. The preview uses only deterministic synthetic in-memory data. Do not open a real vault or enter private data.

For the production shell, run the same executable without --design-preview:

    & 'C:\Users\Asus Vivobook\Desktop\encryptdrive-desktop\target\release\encryptdrive-desktop.exe'

The preview and shell share the same executable path. Do not rebuild between checks unless you intend to replace the candidate.

## 12-step checklist

| # | Check | Procedure and expected result | Status |
|---:|---|---|---|
| 1 | Exact candidate launch | Confirm the SHA above, launch with --design-preview, and inspect the initial Files view. It opens without rebuilding, network/runtime download, or access to real user data. | NOT RUN |
| 2 | Enter on selected file | Select a synthetic file and press Enter once. The simulated Export dialog and plaintext warning stay open; no operation starts from the same Enter event. | NOT RUN |
| 3 | Held/repeated Enter | Hold Enter through dialog opening, then release and press again. Repeats must not confirm; a later distinct action on the focused confirmation control is required. | NOT RUN |
| 4 | Cancel-focused destructive dialog | Open Move to Trash and Permanent Delete. Cancel must visibly have initial focus; Enter cancels and leaves fixture state unchanged. Only deliberate focus movement and a new confirmation may act. | NOT RUN |
| 5 | Delete and Escape | In Files and Trash, Delete opens the appropriate confirmation. Escape closes/cancels without changing synthetic state; Delete must not target a row hidden by search. | NOT RUN |
| 6 | Table arrows and focus | With table focus, test Up/Down, Left/Right, Home/End, PageUp/PageDown, Enter, Escape, Tab and Shift+Tab. Row navigation stays in the table; Left/Right do not spatially escape; Escape clears selection but retains row focus; Tab traversal can leave. | NOT RUN |
| 7 | Ctrl+A and Ctrl+F | Ctrl+A selects only the displayed results. Ctrl+F moves focus to Search; typing and normal text editing continue to work without table shortcuts stealing input. | NOT RUN |
| 8 | Filename pointer behavior | Click directly on file-name text to select. Double-click a folder to navigate. Double-click a file to show the simulated Export warning, never opening or writing a real file. | NOT RUN |
| 9 | Themes, focus and geometry | Check Light and Dark. Hover, focus, selected, pressed and disabled states should be distinguishable. Compare table position before/after selection and notice/error/progress feedback; rows must not shift. Inspect header label and metadata alignment. | NOT RUN |
| 10 | Production shell | Launch the same binary without --design-preview. Inspect the real shell’s compact header/sidebar and no-vault wording; do not infer shell acceptance from the preview. | NOT RUN |
| 11 | Resize and DPI | Resize at 960×640, 1180×792 and 1600×900 if convenient. Repeat at 100%, 150% and 200% display scaling where possible. Record clipping, overflow, focus-ring rendering, and table movement. | NOT RUN |
| 12 | Narrator/UIA | Inspect the UI Automation tree with Accessibility Insights or an equivalent tool and traverse with Windows Narrator. Record roles, accessible names, focus/selection/enabled states, dialog errors, progress and status announcements. | NOT RUN |

## Evidence record

For every capture or observation, record the candidate SHA-256, process mode (preview/shell), window size, Windows build, display scale, theme, scenario, result, and any screenshot/UIA/Narrator artifact path. Capture only the EncryptDrive window and synthetic data. Mark unavailable checks NOT RUN with the tool/reason; do not infer success from the headless tests.

The source report matrix is [UX3_R3_REMEDIATION_MATRIX.md](UX3_R3_REMEDIATION_MATRIX.md). It records unresolved source-report reconciliation and open/pending checks. This checklist is a validation aid, not approval.
