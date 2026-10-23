# EncryptDrive Desktop — Phase 0.5 Master Plan

**Status:** Prepared for owner review  
**Prepared:** 2026-10-17  
**Product:** New Rust-based EncryptDrive Desktop  
**Approved experience direction:** Option C — a hybrid of a familiar native file explorer and a polished premium desktop application

## Purpose

Define the product experience and visual rules for Desktop 1.0.0 before implementation. The Java application is a behavior and Vault Format 1 reference only. Its screen composition and visual identity are not design templates.

The experience should let people create or open a local encrypted vault, unlock that vault, authenticate to an account, and get directly to files. The workspace should feel like a desktop file manager: compact, predictable, keyboard accessible, and clear about operations that move plaintext outside the vault.

## Approved constraints

- Preserve the existing two-level vault and account security model and Vault Format 1 compatibility.
- Do not change cryptographic parameters or format during UX redesign.
- Do not persist passwords, keys, or sensitive recent-file history.
- Do not add telemetry, cloud synchronization, password recovery, previews, editors, or content indexing without owner approval.
- Keep vault selection explicit on each fresh launch unless the owner later approves a non-sensitive location preference.
- Keep file and account information out of operating-system notifications and diagnostic logs.
- Support light and dark appearance; use desktop conventions and restrained security status.
- Do not create application code, Rust scaffolding, framework dependencies, or prototype UI in Phase 0.5.
- The new repository is the only future development target. Do not copy private vaults, credentials, signing material, or private release evidence into it.

## Product experience summary

### Entry and authentication

1. First launch opens a simple vault picker with two clear actions: Create vault and Open existing vault. There is no dashboard, remembered vault list, or automatic sign-in.
2. Creating a vault collects the destination, folder name, and a new vault password. State the no-password-recovery consequence before creation. Show the final destination and password rule.
3. Opening an existing vault uses the system folder picker, then asks for the vault password. Never persist it.
4. After the vault password succeeds, show available accounts. If there are none, offer registration. If there are accounts, allow selection and account-password entry.
5. Registration collects the existing account fields and explains that account password recovery is unavailable.
6. Successful account login opens Files at the account’s root. No statistics page interrupts the main workflow.

### Workspace and file operations

7. The workspace has primary Files and Trash destinations. Files opens to a root folder table with a toolbar, breadcrumb, inline name search, and optional details pane.
8. Folder navigation is in place; the breadcrumb supports moving to an ancestor. File contents are not previewed or opened inside the app.
9. Import uses a system picker, shows the selected source and destination, explains name-conflict behavior, and reports truthful progress. Export warns that the chosen destination receives plaintext before the destination picker is used.
10. Search is by name in the active account’s tree only. It does not search file contents.
11. Trash supports restore, permanent deletion, and emptying with explicit confirmation. Do not claim permanent deletion securely erases physical media.
12. Account settings expose profile details and account-password change. Vault settings expose vault metadata and vault-password change.
13. Log out wipes the account session and returns to account selection while leaving the vault open. Lock/close vault destroys the vault session, releases the local lock, and returns to vault selection.

### Failure and recovery

14. Validation errors appear beside the relevant field or action. Operation errors explain what completed, what did not, and the safe next action.
15. Corruption and interrupted-write states explain whether a valid metadata backup was restored, whether some blob cleanup remains pending, and whether the vault can safely be used. Never imply account-password or vault-password recovery.
16. Long operations keep the window responsive, expose progress, and prevent unsafe close. Offer cancellation only where the operation can stop at a safe, defined boundary.

## Scope boundaries

| Category | Included direction |
|---|---|
| A. Required to reproduce Desktop 1.0.0 safely | Vault create/open/unlock; account registration/login and isolation; file/folder browsing and operations; import/export; name search; trash/restore/permanent deletion; explicit logout and vault lock; existing metadata recovery; account and vault password changes; local/offline behavior; clear failure handling. |
| B. Redesign improvements that preserve Vault Format 1 | Direct-to-Files landing; explorer-style navigation and selection; consistent keyboard behavior; light/dark themes; visible focus and assistive-technology names; compact status and details; clear operation progress; accessible confirmations and errors; no unnecessary dashboard or decorative card layout. |
| C. Requires explicit approval or a later release | Password/key recovery; remembered secrets; persistent recent-vault/file history; telemetry; cloud or multi-device sync; content search/indexing; previews/editors; account deletion; unattended auto-unlock; drag/drop if it changes import/move behavior; any vault-format extension. |

The current Java product has bounded local metadata-backup recovery. Keep that data-safety behavior under review, but do not broaden it into password, key, or account recovery. The user-facing notice and whether the user must confirm a backup restore remain an owner decision; see the decision log.

## Information architecture and screen specification

The detailed screen inventory and workflows are in [Information Architecture](INFORMATION_ARCHITECTURE.md) and [Screen Specifications](SCREEN_SPECIFICATIONS.md). The app has two primary workspace destinations, Files and Trash. Account and vault management are secondary destinations reachable from the account menu and app menu. No dashboard is the default landing screen.

## Visual and interaction system

[Design System](DESIGN_SYSTEM.md) defines the proposed visual tokens, density, theming, typography, icon treatment, window behavior, and accessibility gates. [Navigation and Interactions](NAVIGATION_AND_INTERACTIONS.md) defines window structure, shortcuts, menus, focus movement, progress, notifications, and operation feedback.

Use a system UI typeface, neutral surfaces, one restrained blue action color, clear text hierarchy, and small but comfortable controls. Keep the title bar and standard window controls. Avoid gradient hero panels, decorative shadows, excessive rounded cards, animation for its own sake, and repeated lock badges.

## Security and privacy UX

[Security and Privacy UX Requirements](SECURITY_PRIVACY_UX.md) makes privacy constraints testable. The main requirements are: secrets stay in the active process session only; no path/name/password telemetry; no OS notification content; explicit plaintext export warning; honest corruption and deletion language; and no claims stronger than the actual format or storage behavior.

## Framework comparison

Do not select the final GUI framework in Phase 0.5. [Framework Comparison Protocol](FRAMEWORK_COMPARISON.md) defines a later bounded comparison of egui/eframe and Slint on the same main explorer screen, synthetic data, screen dimensions, task script, and acceptance rubric. The comparison is not a full product prototype and has not been run.

## Phase 0.5 execution roadmap

| Milestone | Work and deliverable | Depends on | Review checkpoint / acceptance |
|---|---|---|---|
| M0 — Lock the brief | Record approved Option C, Rust-only target, VF1 compatibility, explicit exclusions, and unresolved owner choices. | Phase 0 discovery | Owner confirms that scope and constraints are accurate. |
| M1 — Map the product | Complete the journey, app states, primary/secondary navigation, and data visibility rules in the IA. | M0 | Every required journey has a start, success state, back path, and failure path. |
| M2 — Specify screens | Write purpose, hierarchy, actions, keyboard/accessibility, states, displayed data/privacy, and minimum-window behavior for each required screen. | M1 | Screen specs cover all flows in the brief with no unapproved capability. |
| M3 — Define visual and interaction rules | Set palette, type, control density, table, dialog, status, progress, theme, keyboard, and motion rules. | M1; can proceed alongside M2 | Tokens are consistent across light/dark; contrast and focus requirements are explicit. |
| M4 — Review security/privacy UX | Trace secrets, paths, filenames, recovery, export, lock, logs, notifications, and deletion messages. | M2 and M3 | Each sensitive action has accurate feedback and no new data persistence. |
| M5 — Freeze framework protocol | Select the two candidates and define the identical test screen, rubric, tools, test matrix, and package checks. | M2 and M3 | A reviewer could run the comparison later and reach an evidence-based result. |
| M6 — Owner review and handoff | Resolve scope decisions or explicitly defer them; approve the Phase 0.5 package. | M0–M5 | Owner accepts the plan before a framework spike or implementation roadmap is started. |

### Phase 0.5 deliverables

- [Master plan](PHASE_0_5_MASTER_PLAN.md)
- [Information architecture](INFORMATION_ARCHITECTURE.md)
- [Screen specifications](SCREEN_SPECIFICATIONS.md)
- [Design system](DESIGN_SYSTEM.md)
- [Navigation and interaction specification](NAVIGATION_AND_INTERACTIONS.md)
- [Framework comparison protocol](FRAMEWORK_COMPARISON.md)
- [Security and privacy UX requirements](SECURITY_PRIVACY_UX.md)
- [Acceptance checklist](PHASE_0_5_ACCEPTANCE.md)
- [Decision log](DECISION_LOG.md)

## Risks and controls

| Risk | Control in this plan |
|---|---|
| UI makes claims stronger than actual encryption, recovery, or deletion semantics | Use exact state language; review all security copy against the VF1 specification and implemented operation. |
| Framework comparison is biased by unequal screen scope or styling | Same screen, tokens, synthetic data, task script, DPI settings, and score rubric for both candidates. |
| Screen-reader support exists on paper but misses custom controls | Treat actual Windows Narrator/UI Automation workflows as a hard gate; inspect the accessibility tree. |
| Framework packaging adds an undisclosed runtime or network setup | Test ZIP and installer on a clean supported Windows image with network disconnected; inventory runtime dependencies. |
| UX design invents behavior absent from current product | Mark unknown legacy behavior and require evidence or owner decision before implementation. |
| VF1 data fields are assumed where they do not exist | Do not design a modified-date column or content preview until the data contract supports one. |
| Private material enters the public Rust repository | Keep source references bibliographic; copy only reviewed public synthetic vectors after provenance/license review. |

## Evidence basis

Behavior statements in this plan are drawn from the Phase 0 reconstruction and reference Java source/tests, especially the Java README, VAULT_FORMAT.md, COMPATIBILITY.md, and the service, repository, and UI test suites. Those repositories remain references and were not modified. This plan does not reproduce private strategy documents or internal release evidence.

The visual recommendations are proposals for owner review, not legacy requirements. The framework comparison sources are linked in [Framework Comparison Protocol](FRAMEWORK_COMPARISON.md).

## Next action

Review the nine planning documents and resolve the owner decisions in [Decision Log](DECISION_LOG.md). Phase 0.5 ends at this review gate; no prototype, Cargo workspace, or application implementation is part of this deliverable.

