# EncryptDrive Desktop — Information Architecture

**Status:** Proposed for review  
**Audience:** Product, design, and engineering  
**Scope:** Desktop 1.0.0 experience; Vault Format 1 remains unchanged

## Product model

EncryptDrive has two authentication boundaries:

1. **Vault boundary:** the user chooses a local vault folder and enters its vault password. The password unwraps the vault root key.
2. **Account boundary:** after the vault opens, the user selects or registers an account and enters that account’s password. Each account has its own user key and encrypted manifest.

The interface must make it clear which boundary is active. Logging out of an account is not the same as locking the vault.

## Application states

- **No vault open:** vault picker and create/open actions.
- **Vault selected, locked:** vault password entry for the selected folder.
- **Vault open, no account session:** account selection/login or account registration.
- **Vault open, account active:** Files or Trash workspace and account/vault settings.
- **Operation active:** current workspace remains visible; operation progress and safe-close state are announced.
- **Vault locked or closed:** account and vault keys are cleared, the local lock is released, and the app returns to the vault picker.

Session transitions:

- Fresh launch → vault picker.
- Create vault → account registration for the new vault.
- Open vault → vault password → account selection/login.
- Register → account session → Files root.
- Logout → account selection/login; vault stays open.
- Lock/close vault → vault picker; account and vault session state are discarded.
- Close app while an unsafe operation is active → explain that it must finish or reach a safe cancellation point first.

No state automatically restores a remembered vault path, account, or password.

## Window structure

### Top-level window

- Use the operating-system title bar, move/resize behavior, minimize/maximize/close controls, and system window icon.
- Window title stays generic: EncryptDrive plus the current page label. Do not put the vault folder name, username, filename, or path in the title bar.
- Keep native window chrome separate from app content. Do not simulate a web dashboard header.

### Unlocked workspace

- **Primary navigation:** Files and Trash in a compact left sidebar with visible labels and a clear current-location indicator.
- **Secondary management:** account and vault actions in the lower sidebar/account menu and a conventional app menu. Keep Lock vault and Log out visibly distinct.
- **Main region:** command toolbar, breadcrumb/location, optional name search, file table or trash table, optional details pane, and a compact status/progress area.
- **Optional details pane:** off by default. Shows metadata only; it does not preview or open file contents.
- **Notifications:** in-window only. Do not surface filenames, account names, or vault paths in Windows toast notifications.
- **Dialogs:** native folder/file pickers for selection; focused app dialogs for warnings, confirmations, and form flows.

## Navigation hierarchy

- **Files**
  - Vault root
  - Nested folders
  - Search results within the active account tree
  - Item details pane (optional, not a navigation destination)
- **Trash**
  - Deleted items and subtrees
  - Restore
  - Permanently delete
  - Empty Trash
- **Account menu**
  - Profile and account password
  - Log out
- **Vault menu**
  - Vault details
  - Vault password
  - Lock/close vault
- **App menu**
  - Theme preference
  - About and license notices
  - Help / keyboard shortcuts

Files and Trash are primary destinations. Profile, vault metadata, password changes, and help are secondary. The app opens directly into Files after authentication. The old Overview counts should remain available in a compact vault-information view or status summary, not as a default dashboard.

## Explorer layout and data hierarchy

The Files toolbar contains navigation history/up, breadcrumb, search, New folder, Import, Export, and an overflow menu. The main table uses one row per file or folder. Name is primary and left aligned. Type and size are secondary. Do not show a Modified column unless the Rust/format contract has a trustworthy modification field; the discovered VF1 contract does not establish one.

The Trash view uses deleted item names and available deletion/original-parent metadata. Grouping or sorting should follow confirmed existing behavior; do not invent dates or status fields absent from the manifest.

The details pane may show only data that exists in the active vault metadata, such as name, item type, size, and current location. It must not reveal content, file previews, external file history, or data from other accounts.

## Action hierarchy

### Primary actions

- Create or open a vault.
- Unlock the vault and log in/register.
- Navigate folders.
- Create a folder.
- Import files/folders.
- Export selected files.
- Search names.
- Move an item to Trash.
- Restore an item.

### Secondary actions

- Rename and move items.
- Permanently delete one or more trashed items.
- Empty Trash.
- Change account or vault password.
- View profile/vault details.
- Log out or lock the vault.
- Choose Light, Dark, or System theme if the owner approves persistence behavior.

Destructive actions are separated from routine navigation and require clear target counts and confirmation. The exact multi-selection model and whether drag/drop is supported must be confirmed against the Java reference before implementation.

## Information visibility rules

- Before vault unlock: show no account list, usernames, filenames, or manifest-derived details.
- After vault unlock: show account names needed for account selection. Do not copy them to app configuration or logs.
- During an active account session: show only that account’s file tree.
- In the title bar, taskbar notification, or crash message: do not include private vault/account/file identifiers.
- Show full vault path only in a user-invoked vault details screen.
- Search history is not persisted.
- Do not expose encrypted storage UUIDs as normal user-facing names.

## Back and context behavior

- Breadcrumb segments navigate to ancestors; the root segment returns to the vault root.
- Alt+Left returns to the previous folder location if history is available; Alt+Up moves to the parent.
- Dialog Cancel/Escape returns to the screen that launched it and restores focus to the invoking control.
- After opening a folder, focus moves to the table heading or first row predictably.
- Context menu actions are the same actions and confirmations as toolbar/menu actions; there is no hidden destructive behavior.
- Search results preserve query and prior folder context when the user exits search.

