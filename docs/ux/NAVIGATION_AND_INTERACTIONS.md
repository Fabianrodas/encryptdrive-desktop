# EncryptDrive Desktop — Navigation and Interaction Specification

**Status:** Proposed for review  
**Related:** [Information Architecture](INFORMATION_ARCHITECTURE.md), [Screen Specifications](SCREEN_SPECIFICATIONS.md), [Design System](DESIGN_SYSTEM.md)

## Window shell

### Title bar

Keep Windows title-bar controls, resizing, taskbar behavior, and standard keyboard conventions. The title is “EncryptDrive — Files”, “EncryptDrive — Trash”, or the active settings page. Never include the vault path, account name, or selected filename.

### Sidebar

- Primary: Files and Trash, both with text labels and active-location state.
- Secondary: Account/Profile, Vault settings, About/Help, theme.
- Lock vault and Log out are separately named actions and placed away from routine file actions.
- Sidebar remains visible in the minimum supported window. At future compact widths, any icon rail must keep programmatic names and keyboard-reachable text alternatives.

### Files toolbar

Left-to-right grouping:

1. Back and Up.
2. Breadcrumb for the current folder.
3. Name search.
4. New folder, Import, Export.
5. Overflow for Rename, Move, and less frequent actions when selection context makes them available.

Primary actions remain visible at the proposed minimum window. The optional Details pane opens from a labeled command and is off by default.

### Main table and status

- Single click selects; keyboard arrows move row focus; Enter opens folders.
- Double-clicking a folder opens it. Double-clicking a file does not decrypt it or launch an external application; Export is the explicit plaintext operation.
- Table columns are Name, Type, Size unless the data contract is extended through an approved format decision. Modified time is not assumed.
- Status area shows current item count, selection count when applicable, and active operation state. Do not display an inaccurate “space saved/used” estimate.
- Trash status includes selected count and pending operation state only when those values are available.
- Details pane shows metadata only and never previews content.

## Keyboard map

| Shortcut | Action | Notes |
|---|---|---|
| Ctrl+F | Focus name search | Search stays within the active account tree. |
| Ctrl+Shift+N | Create folder | Use only when the current view supports creation. |
| F2 | Rename selected item | Require a selection; keep validation inline. |
| Delete | Move selected item(s) to Trash | Preserve the verified selection model; confirmation policy must be consistent. |
| Shift+Delete | Permanently delete trashed selection | Always confirm and state item count. |
| Enter | Open focused folder | No implicit file decryption or external launch. |
| Alt+Left | Return to prior folder location | Preserve search/location state where possible. |
| Alt+Up | Navigate to parent folder | Disabled at root; accessible name explains why. |
| Shift+F10 | Open item context menu | Same actions as toolbar/overflow. |
| Escape | Close menu/dialog or clear search focus | Does not silently cancel an unsafe storage operation. |
| F6 | Move focus among sidebar, toolbar/search, table, and open Details pane | The order is announced and visually apparent. |
| Ctrl+A | Select all visible rows | Include only if legacy multi-selection is verified; otherwise omit. |

Before implementation, audit conflicts with Windows, screen readers, and the selected framework. Every shortcut must also be reachable through a labeled menu or control. Do not intercept system/accessibility shortcuts.

## Context menus

- Folder menu: Open, Rename, Move, Move to Trash, and permitted creation/import commands.
- File menu: Export, Rename, Move, Move to Trash.
- Trash menu: Restore, Permanently delete.
- Multi-selection menu is present only if legacy behavior supports multi-selection.
- Dangerous actions are grouped and visually separated. Context menus never bypass confirmation or validation.
- Do not add drag-and-drop in Desktop 1.0 until its move/import semantics, cancellation, collision rules, and accidental-drop recovery are explicitly reviewed.

## Search behavior

- Ctrl+F moves keyboard focus to the visible search field.
- Search matches names only within the active account’s tree.
- Enter starts the search; result rows use the same table semantics as Files.
- A result provides its parent location. Activating a result navigates to that parent and focuses the item.
- Clear restores the prior folder view. Do not persist query history or suggestions.

## Dialog and focus behavior

- Opening a dialog moves focus to its title or first actionable control as appropriate.
- Modal dialogs keep Tab navigation inside the dialog and expose dialog/title roles.
- Escape and Cancel have consistent behavior and do not trigger the destructive action.
- After dismissal, focus returns to the control that opened the dialog.
- For destructive confirmations, initial focus is on Cancel. The primary action states the action and target, such as “Permanently delete 3 items.”
- For validation failure, focus moves to the first invalid field and the screen reader announces the specific correction.

## Progress and operation lifecycle

- Long import, export, password-change, recovery, and delete operations run without freezing the window.
- Keep the current view visible and show a compact in-window progress region with operation name and truthful progress.
- A cancellation control is offered only if cancellation leaves storage in a defined consistent state. Otherwise explain that the operation must finish safely.
- Prevent vault lock/logout/close while the current operation owns state that cannot be safely released. Explain the reason and expected next action.
- On completion, announce success, partial skips, or failure; keep actionable errors in the current view.
- On recovery, show a non-modal but persistent notice until acknowledged. Its exact wording and whether restore requires confirmation is an owner decision.

## Notifications

- Use in-window status/toasts for brief success and informational messages.
- Errors, partial imports, recovery notices, and pending deletion warnings remain visible until the user can understand them.
- Do not send filenames, account names, or vault paths through Windows notification center.
- Do not announce every file chunk or progress tick to screen readers; announce meaningful milestones and completion.

## Operation semantics

- Import collision and symlink behavior must match the reference implementation.
- Export always explains that the destination contains plaintext and rejects paths inside the vault.
- Trash is reversible metadata state. Permanent deletion removes encrypted blob references and data according to the existing deletion journal; never label it “secure erase.”
- Logging out clears the account session; locking/closing the vault clears the vault session. The actions must not be visually conflated.

