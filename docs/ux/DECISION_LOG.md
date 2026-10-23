# EncryptDrive Desktop — Decision Log

**Status:** Prepared for owner review  
**Purpose:** Separate approved direction from proposals and choices that still require an owner decision.

## Approved

| ID | Decision | Status / basis |
|---|---|---|
| D-001 | Use Option C: a hybrid between a native file explorer and a polished premium desktop application. | **Approved** in the Phase 0.5 owner brief. |
| D-002 | New Rust Desktop repository is the only future development target; Java and private docs are references. | **Approved** in the owner brief. |
| D-003 | Preserve the two-level vault/account model and Vault Format 1 compatibility. | **Approved** in the owner brief. |
| D-004 | Do not introduce remembered secrets, telemetry, sensitive persistent recent-file history, cloud sync, password recovery, previews, editors, or content search without explicit approval. | **Approved constraint** in the owner brief. |
| D-005 | Phase 0.5 is planning only. No Rust code, Cargo workspace, package installation, or full prototype. | **Approved constraint** in the owner brief. |
| D-006 | Do not copy confidential vaults, credentials, signing keys, or internal release evidence into the new repository. | **Approved constraint** in the owner brief. |

## Proposals awaiting review

| ID | Proposal | Why it is proposed | Owner choice / review |
|---|---|---|---|
| D-007 | Land on Files after login; keep old Overview statistics in compact vault information rather than as the default screen. | Matches the approved direct file-manager direction while retaining useful information. | Confirm this relocation preserves the product behavior the owner values. |
| D-008 | Keep Files and Trash as primary navigation; put profile and vault settings in secondary menus. | Keeps daily file operations prominent without hiding account/vault controls. | Review labels and placement. |
| D-009 | Follow Windows System theme by default; offer Light, Dark, and System. | Satisfies the requested themes and desktop conventions. | Confirm whether the selected non-sensitive theme preference may persist in local app settings. |
| D-010 | Use the proposed neutral/blue palette and system UI typography in DESIGN_SYSTEM.md. | Restrained, readable, and framework-independent. | Approve palette, identity direction, and contrast targets before visual implementation. |
| D-011 | Use 960×640 logical px as the initial minimum comparison window. | Gives the file table, navigation, and toolbar enough space for a compact layout. | Validate or change during framework spike. |
| D-012 | Keep file double-click limited to folder navigation; file export is explicit. | Avoids surprising plaintext creation or external app launch. | Confirm this is acceptable despite file-explorer convention. |
| D-013 | Include a bounded one-screen eframe-vs-Slint comparison after this package review. | Evaluates the preferred candidate and strongest pure-Rust alternative on equal evidence. | Authorize separately; final framework remains unselected. |

## Owner decisions required before implementation or framework lock

| ID | Question | Current evidence / options | Needed by |
|---|---|---|---|
| D-014 | What Windows versions and architectures are Desktop 1.0 release targets? | Frameworks can target multiple OSes, but packaging, native behavior, and test coverage depend on the exact baseline. | Before framework/package acceptance. |
| D-015 | What distribution is required: installer, portable ZIP, both, and what is the acceptable size/runtime policy? | Phase 0 references Windows installer and Portable ZIP behavior. Framework renderers may add native runtime dependencies. | Before package spike. |
| D-016 | What license will cover the new app and shared Rust crates? | Java’s historical licensing decision does not decide the Rust app or its dependencies. Slint has separate GPLv3, attribution, and commercial terms; the selected version and mode require owner/legal review. | Before dependency selection or public release. |
| D-017 | Should existing Java local metadata-backup recovery remain automatic, notify-after-restore, or require confirmation before restoring? | Existing behavior attempts valid local metadata backups. This is not password or key recovery. The new brief forbids introducing automatic recovery without approval, so the scope of that restriction must be explicit. | Before recovery UX implementation. |
| D-018 | What are the exact legacy table semantics for multi-select, sorting, double-click, context menus, and operation cancellation? | Phase 0 confirms file/folder workflows but these interaction details need source-level confirmation before they are frozen. | Before explorer implementation. |
| D-019 | Should theme preference be remembered across launches? | A theme is non-sensitive; remembering vaults, accounts, filenames, or searches remains out of scope. | Before settings implementation. |
| D-020 | Which file metadata may appear in the table? | VF1 contract confirms encrypted names and nonnegative file size; the discovery did not establish a reliable modified-time field. Type may be inferred from the name. | Before table columns are finalized. |

## Items deliberately not decided

- Final GUI framework, renderer, and UI component library.
- New crate boundaries, dependency versions, or Rust architecture.
- Vault format changes, password recovery, content index, preview/editor, cloud features, or account deletion.
- Any setting that persists vault path, account name, filenames, search queries, passwords, or keys.
- A promise of secure physical erasure or absolute security.

## Resolution record

After review, update each pending row with one of: Approved, Rejected, Deferred, or Needs evidence. Record the date and a short rationale. Do not treat silence as approval for any new capability.

