# Competitive pattern analysis

This compares documented interaction principles, not live visual inspection. No product artwork or screen layout is copied. Source facts are separated from EncryptDrive-specific design judgments.

## Linear — focus, selection, contextual work

**Source fact:** Linear documents moving list focus, selecting one or multiple items, then invoking actions with shortcuts, a command menu, or a context menu. [Select issues](https://linear.app/docs/select-issues) · [Search](https://linear.app/docs/search)

**Design judgment:** Transfer the separation between row focus and selected rows, and keep contextual commands consistent with the toolbar. Keep EncryptDrive search local to the active synthetic tree rather than copying workspace-wide search.

**Accessibility/security consequence:** Expose the selected state in the accessible tree and retain stable item IDs through sorting and filtering. Do not create persistent search history or search outside the active vault.

**Alternative and cost:** Use familiar Windows shortcuts, a labeled context menu, and selection-aware actions. This is moderate implementation work. A command palette requires a shared command registry and additional focus/semantics testing.

## Raycast — discoverable keyboard actions

**Source fact:** Raycast documents an Action Panel with actions and keyboard shortcuts for the focused item. [Action Panel](https://manual.raycast.com/action-panel) · [Keyboard shortcuts](https://manual.raycast.com/keyboard-shortcuts) · [Quicklinks](https://manual.raycast.com/quicklinks)

**Design judgment:** Transfer the idea through a row context menu and explicit shortcut labels. Do not copy a global launcher, global hotkeys, or broad file indexing into a local vault manager.

**Accessibility/security consequence:** Keep actions reachable by keyboard and clearly attached to the selected item. Avoid global search or stored paths that broaden data scope.

**Alternative and cost:** A context menu and toolbar overflow are low-to-moderate effort; a searchable global action panel adds command discovery, focus, and accessibility complexity and is not required for this candidate.

## 1Password — plain-language security state

**Source fact:** 1Password documents account and collection organization, device unlock, auto-lock, and account-recovery mechanisms. [Sidebar](https://support.1password.com/sidebar/) · [Unlock and auto-lock settings](https://support.1password.com/unlock-auto-lock/) · [Windows Hello security](https://support.1password.com/windows-hello-security/)

**Design judgment:** Transfer the practice of clearly naming account/vault context and explaining the consequence of security settings. Do not import device unlock, account recovery, or cloud identity assumptions into EncryptDrive without separate product/security approval.

**Accessibility/security consequence:** Security state must be expressed in text, not a lock icon or color alone. Account and vault identity must remain distinct.

**Alternative and cost:** Keep explicit vault selection and unlock, with clear password consequences. That is lower cost than storing device-unlock credentials, which would require a separate threat model and owner decision.

## Proton Drive — truthful operation feedback

**Source fact:** Proton Drive’s Windows documentation describes cloud synchronization, Explorer integration, status, and activity feedback. [Windows app guide](https://proton.me/support/proton-drive-windows-app) · [Sync status](https://proton.me/support/proton-drive-windows-track-synced-folders)

**Design judgment:** Transfer concise, truthful in-window progress and clear completion/failure feedback. Do not copy cloud status, sync badges, storage-usage indicators, or tray sync controls into the offline EncryptDrive shell.

**Accessibility/security consequence:** Progress needs useful text and a screen-reader announcement. Keep file/vault identifiers out of OS notifications.

**Alternative and cost:** A compact in-window operation status is moderate effort and only appropriate when the underlying operation can report accurate results. In this preview, progress changes only in memory.

## Windows Fluent — familiar shell and keyboard behavior

**Source fact:** Microsoft documents navigation, command bars, spacing, keyboard interactions, dialogs, and accessibility for Windows apps. [Design](https://learn.microsoft.com/en-us/windows/apps/design/) · [Design basics](https://learn.microsoft.com/en-us/windows/apps/design/basics/) · [Keyboard](https://learn.microsoft.com/en-us/windows/apps/design/input/keyboard-interactions)

**Design judgment:** Use the familiar file-manager frame and command hierarchy while keeping the implementation in egui. The prototype’s semantics and focus behavior must be checked on Windows UI Automation; resembling Windows is not equivalent to native-control behavior.

## Overall direction

Use Windows structure and interaction conventions, with Linear’s focus/selection distinction and Raycast’s contextual-action discoverability. Use 1Password only as a reference for clear security settings and Proton only for honest operation feedback. Keep EncryptDrive’s own offline, local-vault, and no-recovery boundaries.
