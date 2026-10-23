# Iteration log

> UX3 follow-up work is recorded in [UX3 iteration log](UX3_ITERATION_LOG.md). This earlier log remains archival and does not describe all current fixes.

**Scope:** Source-level implementation and review iterations for the design candidate. The owner's requested screenshot-driven cycle includes live visual evaluation; the earlier stop on GUI input prevents that step. Therefore no cycle below is claimed as a complete visual cycle.

## Research and design inputs

- Read the approved Phase 1A-UX brief, Phase 0.5 UX materials, ADR-012, and the current workspace. The default shell remains an honest no-vault state; synthetic preview is isolated from product capabilities.
- Agent A researched Windows conventions, egui/AccessKit, Linear, Raycast, 1Password, and Proton Drive using official/public documentation. Its findings were delivered as read-only research drafts and incorporated into [Design Research](DESIGN_RESEARCH.md) and [Competitive Analysis](COMPETITIVE_ANALYSIS.md). No proprietary artwork was copied.
- Selected the owner-approved Native Explorer × Premium Desktop direction. Decisions and alternatives are recorded in [Decision Register](DESIGN_DECISION_REGISTER.md).

## Pass 1 — Baseline critique and shell correction

**Critic evidence:** Agent C inspected source and approved UX requirements without opening the GUI. It found no P0 and identified three P1 concerns: no explicit first-launch Create/Open affordance, disabled reasons discoverable only by hover, and selected navigation with no separate non-color cue. It also identified repeated no-vault/readiness copy and an “ED” mark that did not carry the approved husky direction.

**Changes:** Added visible but honestly disabled Create/Open controls and their reason in the default shell; exposed unavailable-action copy outside hover tooltips; added a non-color selected cue to navigation; reduced repeated readiness copy; replaced the monogram with a temporary original geometric husky placeholder.

**Acceptance evidence:** Source inspection only. These changes were not rendered or tested with a screen reader.

## Pass 2 — Synthetic explorer and component implementation

**Changes:** Added a feature-and-argument-gated design preview; deterministic 100/1,000/10,000-item fixtures; stable entry IDs; nested and empty folders; Unicode/long names; synthetic size/date/type data; Trash; search/sort; selection; dialogs; named account/vault/recovery screens; and repaint-driven operation states. Built modular UI files for application, components, dialogs, explorer, navigation, screens, and theme. Default launch remains the no-vault shell.

**Automated checks at the initial implementation point:** Workspace tests passed (10 tests); strict workspace Clippy passed; default and preview checks/builds passed. The current final verification totals and executable measurements are recorded in [Visual Acceptance Report](VISUAL_ACCEPTANCE_REPORT.md). No live visual review occurred.

## Pass 3 — Critic corrections and state-model hardening

**Changes:** Cached the sorted visible-ID projection by stable query/folder/sort/model revision; fixed ArrowDown's initial focus to start on the first visible row without selecting it; made restoring a nested trashed item also restore deleted ancestors; added regression tests. Changed the selected navigation cue and documented disabled actions in the default shell. Fixed a Rust borrow conflict caused by the table closure reading the outer UI painter/context by retaining owned clones before building the table. Set the minimum window size to 960×640 to match the smallest requested design target.

**Build evidence at the iteration point:** Preview feature checks, 15 model tests, strict Clippy, architecture checks, and release builds passed. The final verification pass below records the rerun against the latest code.

## Pass 4 — Critic-driven keyboard, context, and boundary corrections

**Changes:** Scoped row shortcuts to table focus; made Escape clear focused search; implemented the approved Delete versus Shift+Delete routing; restricted Trash context actions; disabled workspace navigation on representative authentication screens; clarified the account boundary in empty states; clarified selected-folder descendant effects; added Cancel-first focus requests and focus return; preserved group selection for context actions; returned focus to stable destination tabs after destructive row removal; and replaced deprecated egui IDs with stable `Id::unique` IDs. Simplified the disabled navigation icon branch after strict Clippy identified identical branches.

**Acceptance evidence:** Source/model review and automated tests only. The independent final source critic review is recorded separately. No screenshots or runtime GUI checks were produced.

## Final automated verification — 2026-10-19

- `cargo fmt --all -- --check`: PASS.
- `cargo check --workspace --locked`: PASS.
- `cargo test --workspace --locked`: PASS, 10 tests total (3 core + 7 desktop library tests; preview-only tests are feature-gated).
- `cargo test -p encryptdrive-desktop --features design-preview --locked`: PASS, 25 tests total (7 library + 3 UI unit + 15 model integration tests after critic corrections).
- Strict Clippy for the workspace and feature-enabled desktop targets: PASS.
- Workspace and feature-enabled Release builds: PASS.
- `scripts/verify-architecture.ps1` with `RUSTUP_TOOLCHAIN=stable`: PASS.
- Deterministic fixture/projection benchmark: PASS as a model microbenchmark only; it does not measure rendered frames, scrolling, or memory.

Cargo emitted environment-only warnings that it could not hard-link its incremental cache and copied those files instead. They did not affect command exit status or generated binaries.

## Independent review status

- **Agent C baseline review:** completed; source-only; corrections listed in Pass 1.
- **Agent C implementation review:** final read-only, source-only review completed after the focus-return correction. It found 0 P0, 0 P1, and 0 P2; its evidence-bounded subtotal is 58/85 nonvisual points, with the 15-point visual category unverified. See [Critic Review](CRITIC_REVIEW.md).
- **Live visual review:** NOT RUN. This includes screenshots, theme comparisons, window resizing, DPI scaling, UI Automation, Narrator, focus announcements, and High Contrast. This is not replaced by source review or model tests.

## Cycle count and stopping condition

Four substantive source/design implementation passes are recorded, with automated and source-review corrections applied. **Complete screenshot-driven design cycles: 0.** Each full cycle in the owner's brief requires live visual evaluation; that step is blocked by the prior explicit GUI stop and has not been bypassed. Further visual changes would be speculative without owner-visible rendering. The smallest next step is owner-assisted launch and execution of [Owner Review Guide](OWNER_REVIEW_GUIDE.md), then a new critic pass against captured synthetic evidence.
