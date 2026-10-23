# EncryptDrive Desktop — Security and Privacy UX Requirements

**Status:** Proposed for review  
**Boundary:** UX requirements only; Vault Format 1 and cryptographic implementation remain unchanged.

## User-facing security model

Explain the two passwords in plain language:

- The **vault password** opens the selected vault.
- The **account password** opens one account within that vault.

Do not call the account password a second vault password. Do not imply that logging out locks the vault. The interface should show the current state plainly: account signed in, vault open, or vault locked/closed.

## Secrets and session data

- Never save passwords, derived keys, master keys, or recovery answers in settings, logs, crash reports, clipboard history, or recent-item lists.
- Password fields are masked by default. A show/hide option, if implemented, changes only the visible field state and has an accessible name.
- Do not offer “remember password,” automatic unlock, or password autofill owned by the app.
- Clear the submitted secret from UI state after successful authentication and after leaving the authentication flow. On failed authentication, clear or retain it only according to a reviewed usability/security decision; never write it to disk.
- Logout clears the active account session. Vault lock/close clears the vault session and releases the local lock.
- Do not show account names until the vault password has opened the encrypted registry.

## Data minimization

- No telemetry or network calls.
- No persistent recent vault, file, folder, search-query, or account history unless explicitly approved.
- A theme preference may be persisted only as a non-sensitive setting; never store vault path, username, or filenames with it.
- Do not put vault paths, usernames, filenames, search queries, or operation details into OS toast notifications.
- Diagnostic logs may record operation category and error class only. Redact user paths and names; never log password text, keys, decrypted metadata, or file content.
- Do not expose encrypted blob UUIDs or cryptographic internals in the normal workspace.
- When showing the selected vault path in Vault Settings, display it only in the user-invoked view.

## Import, export, and deletion language

- Import: identify source and vault destination; show conflict and skip behavior before the operation when possible.
- Export: state before selecting the destination that the exported copy is plaintext outside the vault. Reject a destination inside the vault. On failure, remove provisional plaintext before presenting the result.
- Trash: describe as reversible removal from the active file tree, not encryption or deletion of blob bytes.
- Permanent delete: describe removal of the item and encrypted blob data according to the app’s deletion process. Do not say “secure erase,” “unrecoverable from disk,” or similar claims about storage media.
- Partial operation: show completed/skipped/failed counts separately. Never report success for items whose storage transaction did not complete.

## Corruption and recovery language

Distinguish these situations:

- **Incorrect password:** explain that the password did not unlock this vault. Do not label the vault corrupt based only on a wrong password.
- **Unsupported format:** explain the format version and advise closing the vault; do not attempt a write.
- **Malformed or corrupt metadata:** state which metadata could not be read and whether a valid backup exists, without printing secrets or ciphertext.
- **Backup restore:** Java reference behavior includes bounded local metadata-backup recovery. Notify the user when a prior valid registry/manifest backup is used and indicate that the newest metadata changes may be absent. Whether to require confirmation before restoration is an unresolved owner decision.
- **Pending blob cleanup:** say that the item’s metadata deletion is recorded but one or more encrypted blobs still await cleanup, only when that state is known.
- **Password/key recovery:** no account-password or vault-password recovery is part of Desktop 1.0. Do not promise support or ask for a recovery phrase that does not exist.

Do not introduce cloud backup, reset links, recovery keys, or automated account recovery in this UX plan.

## Operation and window lifecycle

- Keep the UI responsive during file and metadata operations.
- Show progress only when a trustworthy unit or state is available.
- Prevent logout, vault lock, or app close only while required to preserve a well-defined storage operation. Explain why and when the user can continue.
- Offer Cancel only if a tested cancellation boundary prevents inconsistent metadata and partial plaintext exposure.
- On shutdown, clear session keys and transient credentials according to the core’s lifecycle; the UX must not claim memory is perfectly erased.
- Do not imply that closing the app closes the vault successfully if lock release failed. Present a clear retry/exit choice.

## Security indicators and copy

- Use one small, state-based status label such as “Vault open” or “Vault locked.” Do not repeat padlock icons beside every file.
- Avoid absolute claims such as “unbreakable,” “military-grade,” or “100% secure.”
- Describe local/offline behavior only when the built application has no network dependency and packaging supports the claim.
- Explain password loss plainly at vault/account creation and in password-change flows.
- Do not use color alone for locked, warning, error, or recovery status.

## Review checks

- Inspect screen text, accessibility announcements, title bar, notification behavior, and error details for path/name/secret leakage.
- Check every export, delete, lock, and recovery dialog against actual operation semantics.
- Confirm no UI preference stores vault/account identifiers.
- Verify that the displayed state matches the current session and local-lock state.

