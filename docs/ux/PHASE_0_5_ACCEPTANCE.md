# EncryptDrive Desktop — Phase 0.5 Acceptance Checklist

**Status:** Artifacts prepared; owner review pending  
**Phase exit:** This planning package is accepted, or returned with requested changes. It does not authorize framework selection, code, or application scaffolding.

## Deliverables

| Requirement | Artifact | Prepared |
|---|---|---|
| Phase 0.5 master plan and execution milestones | [Master plan](PHASE_0_5_MASTER_PLAN.md) | Yes |
| Information architecture and full app states | [Information architecture](INFORMATION_ARCHITECTURE.md) | Yes |
| Required screens with purpose, hierarchy, actions, accessibility, states, data/privacy, and responsive behavior | [Screen specifications](SCREEN_SPECIFICATIONS.md) | Yes |
| Typography, light/dark tokens, geometry, components, density, focus, motion, window and DPI rules | [Design system](DESIGN_SYSTEM.md) | Yes |
| Navigation, shortcuts, menus, feedback, progress, and focus behavior | [Navigation and interactions](NAVIGATION_AND_INTERACTIONS.md) | Yes |
| Same-screen framework comparison plan, criteria, evidence, and decision gates | [Framework comparison](FRAMEWORK_COMPARISON.md) | Yes |
| Secret handling, local-data limits, export, delete, recovery and lock language | [Security and privacy UX](SECURITY_PRIVACY_UX.md) | Yes |
| Explicit decisions, status, and owner questions | [Decision log](DECISION_LOG.md) | Yes |

## Product behavior acceptance criteria

- [ ] First launch leads to explicit Create vault or Open existing vault actions.
- [ ] Vault unlock and account login are distinct states and screens.
- [ ] Registration and both password-change flows state password requirements and lack of recovery.
- [ ] Successful account login lands directly in Files.
- [ ] Files and Trash are primary workspace destinations.
- [ ] File/folder actions, import/export, name-only search, trash/restore/permanent deletion, logout, and vault lock are documented.
- [ ] Error, corruption, backup-recovery, pending-delete, and interrupted-operation states are documented.
- [ ] No flow silently persists a password, key, sensitive recent item, vault path, account name, or search query.
- [ ] No UI plan changes Vault Format 1 or cryptographic parameters.

## UX and accessibility acceptance criteria

- [ ] Every specified screen documents purpose, hierarchy, components, actions, keyboard/accessibility behavior, states, data/privacy, and responsive behavior.
- [ ] Keyboard-only use is specified for primary and destructive workflows.
- [ ] Focus order, focus restoration, visible focus, role/name/state exposure, and progress announcements are explicit.
- [ ] Light and dark themes have semantic tokens and contrast review requirements.
- [ ] High contrast, text scaling, and 100/150/200% DPI checks are in the plan.
- [ ] Native Windows window and picker behavior is included in the framework comparison.
- [ ] No content-first workflow is blocked behind a dashboard.
- [ ] No decorative effect or security badge makes an unsupported claim.

## Framework comparison acceptance criteria

- [ ] egui/eframe and Slint use the same explorer screen, data, labels, icons, tasks, window size, theme, and DPI matrix.
- [ ] Comparison scores visual fidelity, keyboard access, Narrator/UI Automation, DPI/window behavior, GPU fallback, offline ZIP, installer, dependencies/license, maintainability, and cross-platform path.
- [ ] Screen-reader and offline/packaging checks are decision gates.
- [ ] The comparison remains unrun and no final framework is selected in Phase 0.5.
- [ ] The future comparison is a single synthetic-screen spike, not a full application prototype.

## Privacy and scope acceptance criteria

- [ ] Export is described as plaintext leaving the vault.
- [ ] Permanent deletion is not described as secure physical erasure.
- [ ] Local metadata backup recovery is distinguished from password/account recovery.
- [ ] No telemetry, cloud sync, content indexing, preview/editor, or remembered secret is added.
- [ ] New behaviors not established in Phase 0 are marked for confirmation or owner approval.
- [ ] Private vaults, credentials, signing material, and private release evidence are absent from these artifacts.

## Phase completion review

- [ ] Owner confirms the approved Option C direction is represented accurately.
- [ ] Owner reviews the pending choices in the [Decision Log](DECISION_LOG.md).
- [ ] Any requested changes are recorded and the affected documents are updated.
- [ ] Framework comparison is authorized separately after the Phase 0.5 review.
- [ ] A later implementation roadmap is written only after the owner accepts this package.

## Scope audit

This checklist and the other Phase 0.5 documents are planning-only Markdown. No Rust source, Cargo manifest/lockfile, dependency, UI prototype, test, private vault data, or copied internal evidence is included.

