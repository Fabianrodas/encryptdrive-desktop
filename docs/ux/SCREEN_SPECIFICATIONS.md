# EncryptDrive Desktop — Screen Specifications

**Status:** Proposed for review  
**Shared accessibility rules:** Every screen follows the Windows keyboard order, has a visible focus indicator, exposes meaningful control names/roles to assistive technology, and supports 100%, 150%, and 200% display scaling. Validation is associated with its field. Dialogs have a clear Cancel/close route. Focus returns to the invoking control when a dialog closes. Color is never the only status signal.

## 1. Vault picker and first launch

- **Purpose / hierarchy:** Let the user create a vault or choose an existing one. Keep the EncryptDrive identity small, the two main actions prominent and equally legible, and explanatory privacy copy secondary. This is the first-launch screen; there is no dashboard or recent-vault list.
- **Components:** Create vault, Open existing vault, short local-storage explanation, optional Help/About.
- **Actions:** Create opens the creation flow; Open uses a native folder picker. Canceling the picker returns without changing the screen.
- **Keyboard/accessibility:** Create and Open are the first two controls in tab order; Enter activates the focused action; Escape cancels the native picker. Buttons have visible text, not icon-only labels.
- **States/data/privacy:** Initial, picker open, invalid/unsupported folder, permission denied, vault not found, and existing vault selected. Do not enumerate accounts or contents before successful unlock. No path is remembered.
- **Responsive behavior:** Center a compact, fixed-width form region in the native window; no illustration panel. At the proposed minimum size, both actions and explanatory text remain visible without scrolling.

## 2. Create vault

- **Purpose / hierarchy:** Collect a destination, folder name, and vault password before creating a new vault.
- **Components:** Destination path with Browse button; vault folder name; password and confirmation fields; concise password rule; explicit no-recovery notice; Create and Cancel.
- **Actions:** Browse uses a native folder picker. Create shows the final destination and starts creation. Prevent accidental overwrite. Cancel returns to the picker.
- **Keyboard/accessibility:** Each field has a persistent label. Enter submits only when valid; Escape cancels. Password visibility control, if offered, has an accessible name and never copies the value.
- **States/data/privacy:** Missing path/name, invalid name, existing destination, password mismatch, weak/short password, permission failure, creation progress, success, and partial cleanup failure. Do not retain field values after leaving the flow.
- **Responsive behavior:** One-column layout; labels remain visible at narrow widths. Long paths wrap or ellipsize with a way to inspect the full path.

## 3. Vault unlock

- **Purpose / hierarchy:** Unlock the explicitly selected vault.
- **Components:** Generic app title, selected folder display (user-invoked context only), vault password, Unlock, Choose another vault.
- **Actions:** Unlock submits the password; choose another returns to the folder picker.
- **Keyboard/accessibility:** Password field receives initial focus; Enter submits; failed submission returns focus to the field and gives a concise message. Screen reader announces failure without exposing key or KDF internals.
- **States/data/privacy:** Ready, unlocking, incorrect password, malformed header, unsupported format, corrupt header, permission/I/O error, and success. Do not persist the password. Do not show account names until vault unlock succeeds.
- **Responsive behavior:** Keep the password and Unlock action together; do not shrink text or hide the alternate-vault action.

## 4. Account selection and login

- **Purpose / hierarchy:** After the vault opens, select an account and authenticate to it.
- **Components:** Account list when accounts exist; selected account; account password; Login; Register account; Log out/Lock vault in secondary menu.
- **Actions:** Select account then log in; choose Register if permitted. Account switching after logout returns here without closing the vault.
- **Keyboard/accessibility:** Account list supports arrow-key navigation and clear selected state. Password field follows the selected account. Enter submits. Focus moves to the password field after account selection.
- **States/data/privacy:** No accounts, accounts available, login in progress, incorrect password, account unavailable, registry recovery notice, and vault-level failure. Account names appear only after vault unlock and are not persisted outside the vault.
- **Responsive behavior:** Use a short list rather than account cards. Long names truncate visually but remain fully available to accessibility APIs.

## 5. Account registration

- **Purpose / hierarchy:** Create a new account inside the open vault.
- **Components:** Full name, username, password, confirmation, visible field rules, no-password-recovery notice, Register and Cancel/Back.
- **Actions:** Register validates and creates the account; Cancel returns to account selection.
- **Keyboard/accessibility:** Persistent labels, field-associated errors, expected username/password focus order, Enter submit only when valid. Password is masked by default.
- **States/data/privacy:** Empty required fields, short/duplicate username, short/mismatched password, registration progress, success, and storage failure. Apply existing username normalization/uniqueness behavior. Do not store credentials in app preferences.
- **Responsive behavior:** Single column with scroll only if required. Keep the no-recovery notice adjacent to the password fields.

## 6. Main file explorer

- **Purpose / hierarchy:** Provide the daily workspace and open directly at the account’s root after login.
- **Components:** Files/Trash sidebar; Back/Up and breadcrumb; name search; New folder, Import, Export; overflow; file table with Name, Type, Size; optional Details pane; status/progress area.
- **Actions:** Open folders, navigate ancestors, create/rename/move items, import/export, search names, move items to Trash. Do not open file contents. The exact multi-selection and sorting behavior must match verified legacy behavior.
- **Keyboard/accessibility:** Arrow keys move through rows; Enter opens a folder; F2 renames; Delete moves the selection to Trash; Shift+F10 opens the context menu; Ctrl+A selects visible rows only if multi-select is supported. Shortcuts are also listed in the app menu. Table exposes row/column roles, sort state, selection state, and item name/type/size.
- **States/data/privacy:** Loading; empty vault; empty folder; populated folder; selection; search results; import/export running; operation error; inaccessible or malformed manifest. Display only metadata present in the account manifest. No content preview, recent files, or modified date without a trustworthy source field.
- **Responsive behavior:** Hide the optional Details pane first. Keep Name visible; reduce or hide secondary columns only at narrower sizes and expose their values in row details. At minimum size keep both Files and Trash navigation available.

## 7. File/folder import

- **Purpose / hierarchy:** Select plaintext source files/folders and a destination folder inside the active account.
- **Components:** Native source picker; destination breadcrumb; preflight summary; conflict explanation; progress and result.
- **Actions:** Add files or folders; confirm the target; start import. Nested name conflicts follow the existing skip behavior. A destination-root name conflict aborts before import. Do not follow symbolic links or junctions.
- **Keyboard/accessibility:** Native picker is keyboard operable; destination is announced; progress changes are announced at meaningful intervals, not every chunk. Close is disabled or explained while an unsafe operation is active.
- **States/data/privacy:** Empty selection; invalid/unreadable source; conflict; scanning; importing; partial skip; success; disk full; source disappeared; encrypted write/recovery failure. Show counts and the item currently being processed only inside the app.
- **Responsive behavior:** Keep destination and conflict summary visible; collapse detailed counts behind an accessible disclosure at narrow widths.

## 8. Export

- **Purpose / hierarchy:** Write chosen file contents as plaintext to an external user-selected location.
- **Components:** Selected item summary; plain-language warning that exported files are no longer protected by the vault; native destination picker; progress and result.
- **Actions:** Cancel or choose an external destination; confirm export; open destination only by explicit user action if supported. Reject a destination inside the vault.
- **Keyboard/accessibility:** Warning is read before the final action; Cancel is initial focus for destructive/privacy-sensitive confirmation. Progress and failure are announced without repeated alerts.
- **States/data/privacy:** No selection; destination inside vault; name collision; export in progress; authentication/tag failure; disk full; partial failure; complete. Do not leave unauthenticated partial plaintext after failure.
- **Responsive behavior:** The warning cannot be hidden below the action buttons. Long destination paths remain inspectable.

## 9. Name search

- **Purpose / hierarchy:** Find items by name in the active account’s tree.
- **Components:** Search field in the Files toolbar; clear action; result list using normal table rows; result parent location.
- **Actions:** Enter starts search; Clear restores the prior folder context. Selecting a result can navigate to its parent and focus the item.
- **Keyboard/accessibility:** Ctrl+F focuses search; Escape clears or leaves search according to current focus; results announce count and name-only scope. Preserve query and prior location on return.
- **States/data/privacy:** Empty query, searching, no matches, results, and search failure. Search names only; do not search content or persist query history.
- **Responsive behavior:** Search remains reachable in toolbar overflow if it cannot fit inline; it must not become icon-only without a text alternative.

## 10. Trash

- **Purpose / hierarchy:** Restore or permanently remove deleted items.
- **Components:** Trash navigation state; table; Restore; Permanently delete; Empty Trash; available deletion/original-parent details.
- **Actions:** Restore an item/subtree; permanently delete selected items; empty trash with a separate confirmation. Restore returns to the original parent when available, otherwise the root, using existing conflict handling.
- **Keyboard/accessibility:** Same table navigation as Files. Destructive actions are not adjacent to Restore without separation and labels. Confirmations state the count and target.
- **States/data/privacy:** Empty; populated; restore success/conflict; pending blob deletion; permanent deletion failure; recovery warning. Do not claim secure physical erasure.
- **Responsive behavior:** Keep Name and action availability; show parent/date metadata only when it exists. Details are available without hover.

## 11. Account settings / profile

- **Purpose / hierarchy:** Show the active account’s profile and change its password.
- **Components:** Full name, username, Change password form, Save/Cancel, return to Files.
- **Actions:** Change password with current-password verification and new-password confirmation. No account deletion in Desktop 1.0 without approval.
- **Keyboard/accessibility:** Password fields have persistent labels and inline rules; validation is announced; focus returns to Save or the first invalid field.
- **States/data/privacy:** Read-only profile, change in progress, incorrect current password, validation error, success, and storage/backup failure. Do not show other accounts’ data here.
- **Responsive behavior:** A focused settings page, not a dashboard card grid; fields stay in one column.

## 12. Vault settings

- **Purpose / hierarchy:** Show vault-level metadata and change the vault password or close the vault.
- **Components:** User-invoked vault location; vault identifier/format/creation details if useful; Change vault password; Lock/Close vault.
- **Actions:** Change vault password; close/lock and return to vault selection. Clearly distinguish this from account logout.
- **Keyboard/accessibility:** Password dialogs identify the vault-level boundary. Destructive/closing actions use explicit labels and keyboard confirmation.
- **States/data/privacy:** Vault details loaded, password change in progress/failure/success, busy operation blocks close, lock release failure. Do not show the path in window title, notifications, or logs.
- **Responsive behavior:** Metadata uses a compact label/value list; full path can wrap or be copied only if an explicit safe copy action is approved.

## 13. Confirmations, errors, and operation status

- **Purpose / hierarchy:** Make irreversible, plaintext, corrupt, and interrupted states understandable and actionable.
- **Components:** Short title naming the action; item count and scope; one-sentence consequence; primary safe action; Cancel; optional details disclosure for diagnostics.
- **Actions:** Confirm only when the user understands the consequence. Permanent delete and Empty Trash are distinct. Export confirmation explains plaintext destination. No password-recovery promise.
- **Keyboard/accessibility:** Cancel receives initial focus for destructive actions; Escape cancels; focus is trapped only inside modal dialogs and restored afterward; controls have named roles. Screen reader receives one meaningful progress/state announcement per update interval.
- **States/data/privacy:** In-progress, success, partial success with skipped count, recoverable failure, corruption, and unrecoverable error. Keep technical details behind an optional disclosure; never include secret material.
- **Responsive behavior:** Dialog width is content-sized with readable wrapping; never truncate the consequence or primary/cancel labels.

## Shared verification checklist

- Every screen has a complete keyboard path and screen-reader role/name.
- All field errors are next to their field and announced.
- Dialog open/close returns focus predictably.
- At 960×640 logical units, no required action is clipped. This is a proposed test size, pending framework evaluation.
- At higher scaling, text reflows before it truncates; controls do not overlap.
- Files, account names, and vault paths are not placed in OS notifications or diagnostic messages.

