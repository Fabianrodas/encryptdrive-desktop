# Phase 1A-UX Design Studio Implementation Plan

> **For agentic workers:** Three independent roles are established: Agent A researches, Agent B (the root implementer) alone changes product code, and Agent C critiques without editing. No commits are made per owner instruction.

**Goal:** Implement a substantially improved, cohesive egui/eframe desktop shell with a safe, deterministic synthetic preview and independent design review.

**Architecture:** Keep the production default as an honest, empty engineering shell. Put all fictional data and state transitions in an app-local preview model, compile-gated by `design-preview`; keep core and vault crates untouched. Divide UI into theme, components, shell/screens, table, and dialogs, with selection, filtering, and fixture rules unit-tested outside widgets.

**Tech Stack:** Rust 1.99.1 MSVC, Rust 2024, eframe/egui 0.37.0, egui_extras 0.37.0 `TableBuilder`, AccessKit.

**Spec:** `docs/ux/design-studio/PRODUCT_VISUAL_DIRECTION.md`

## Global Constraints

- Modify only `encryptdrive-desktop`; preserve all pre-existing untracked work and the unborn `main` branch.
- Keep preview behind both the `design-preview` Cargo feature and explicit `--design-preview` argument.
- Preview state is synthetic, deterministic, in-memory only; no filesystem, vault, crypto, authentication, network, secrets, or persistence behavior.
- Preserve Files/Trash, stable-ID selection, explicit file Export warning, and no real file opening.
- Keep `encryptdrive-core` GUI-independent and leave vault-format, vault-crypto, and vault-storage empty.
- Do not copy the Java logo or other non-redistribution-cleared artwork.
- Do not launch or control a GUI, take screenshots, or use alternate desktop-input tooling; the earlier physical Escape stop remains in force.
- Do not commit, stage, push, reset, clean, publish, or alter either read-only repository.
- Do not implement Phase 2, Vault Format 1, or cryptography.

## Review Focus

1. Fixture counts, stable IDs, Unicode, collision, and empty-folder behavior — test the generator at 100, 1,000, and 10,000 entries.
2. Selection correctness after query/sort/navigation changes — test ctrl-toggle, shift-range, filtered Ctrl+A, and stable-ID retention.
3. Accidental real side effects through preview controls — test that all simulated operations mutate only the in-memory model and no operation can create/export/open a real file.
4. Form input and destructive confirmation — test blank/invalid/duplicate names, unsafe export confirmation, Cancel-first permanent deletion, and focus restoration at the state-model boundary.
5. Large-list render cost — test visible-row projection/virtualized table path and avoid rebuilding/sorting the entire fixture each frame.

---

### Task 1: Synthetic preview model and interaction rules

**Files:**
- Create: `apps/encryptdrive-desktop/src/preview.rs`
- Create: `apps/encryptdrive-desktop/src/preview/model.rs`
- Create: `apps/encryptdrive-desktop/src/preview/state.rs`
- Modify: `apps/encryptdrive-desktop/src/lib.rs`

**Interfaces:**
- `EntryId(u64)`, `EntryKind`, `Entry`, `DatasetSize::{OneHundred, OneThousand, TenThousand}`, `PreviewModel::fixture(size)`.
- `ExplorerState` owns destination, folder ID/history, query, sort, selected IDs, anchor/focused IDs, dialog, and operation state.
- Model methods validate and mutate only synthetic entries: create folder, rename, move to Trash, restore, permanently delete, and projected visible IDs.

- [ ] Write tests for exact fixture counts, determinism/stable IDs, nested/empty folders, Unicode/long names, collisions, deleted timestamps, and sorted/filter projections.
- [ ] Run the new tests with the local `stable` alias to verify expected missing API failures.
- [ ] Implement deterministic fixture generation and GUI-independent state/validation.
- [ ] Test Ctrl/Shift selection math and prove sort/filter preserve IDs while Ctrl+A selects displayed IDs only.
- [ ] Run preview-module tests and confirm default features still build without preview data exposed.

### Task 2: Semantic design system and reusable controls

**Files:**
- Create: `apps/encryptdrive-desktop/src/ui/theme.rs`
- Create: `apps/encryptdrive-desktop/src/ui/components.rs`
- Create: `apps/encryptdrive-desktop/src/ui/mod.rs`
- Modify: `apps/encryptdrive-desktop/src/lib.rs`

**Interfaces:**
- `ThemeChoice::{Light, Dark}`, `DesignTokens`, `apply_theme(ctx, choice)`; default Light, no persistence or System-following until a product decision is approved.
- Reusable primary/secondary/destructive buttons, icon-with-name controls, status badge, breadcrumb segment, focus-aware navigation item, inline field/error, and dialog frame.
- Original vector husky/drive-bay placeholder with a non-asset implementation.

- [ ] Write token contrast/state tests and small component layout/label tests where the egui test harness permits.
- [ ] Confirm failing tests identify absent token roles and theme resolution.
- [ ] Implement full semantic roles, spacing, type ramp, selection/focus/disabled/destructive states, and Light/Dark/System mappings.
- [ ] Run focused tests and Clippy; verify custom icon controls have visible text/tooltip and accessible widget names.

### Task 3: Application shell, navigation, table, and keyboard

**Files:**
- Create: `apps/encryptdrive-desktop/src/ui/app.rs`
- Create: `apps/encryptdrive-desktop/src/ui/explorer.rs`
- Create: `apps/encryptdrive-desktop/src/ui/navigation.rs`
- Modify: `apps/encryptdrive-desktop/src/main.rs`
- Modify: `apps/encryptdrive-desktop/Cargo.toml`

**Interfaces:**
- `DesktopApp` receives `ShellMode::{EngineeringPreview, DesignPreview(PreviewModel)}`.
- Main shell renders header, Files/Trash sidebar, title/breadcrumb/search, command groups, virtualized file table, and one status/preview region.
- Table consumes a cached projected `Vec<EntryId>` and calls stable-ID state methods; folder double-click navigates; file double-click opens Export warning.

- [ ] Write tests for feature/argument gating, default honest state, title/privacy behavior, action enablement, and key-to-action mapping.
- [ ] Confirm new tests fail before shell implementation.
- [ ] Implement compact Explorer layout, 960×640 minimum, responsive column priority, focus-aware state, keyboard navigation, search, sort, context menu, and synthetic dataset selector.
- [ ] Verify 10,000-row use goes through `TableBuilder::body.rows` virtualization and cached projections.

### Task 4: Preview screens, forms, dialogs, and simulated operations

**Files:**
- Create: `apps/encryptdrive-desktop/src/ui/screens.rs`
- Create: `apps/encryptdrive-desktop/src/ui/dialogs.rs`
- Create: `apps/encryptdrive-desktop/src/ui/operation.rs`
- Modify: `apps/encryptdrive-desktop/src/ui/app.rs`
- Modify: `apps/encryptdrive-desktop/src/preview/state.rs`

**Interfaces:**
- `PreviewScenario` selects vault picker/create/unlock, account selection/login/register, Files empty/populated/nested/search/multi-selection, Trash/restore, account/vault settings, recovery/corruption, busy/error/disabled states.
- `DialogKind` selects create-folder, rename, move-to-trash, import, export plaintext warning, permanent-delete, recovery notice, and representative create/unlock/login forms.
- `OperationState` is deterministic in-memory progress with explicit cancel/success/failure states; no workers, file handles, or external side effects.

- [ ] Test scenario mapping, form validation, dialog consequence copy, progress cancellation, and synthetic-only mutations.
- [ ] Verify failures are model-level before implementing widgets.
- [ ] Implement representative accessible states with persistent labels, inline errors, safe default/cancel focus, focus restoration, and no fake success wording.
- [ ] Build preview and use the scenario selector to expose every matrix state without writing user data.

### Task 5: Research/decision documents, critic review, and correction cycle

**Files:**
- Create: `docs/ux/design-studio/{DESIGN_RESEARCH,COMPETITIVE_ANALYSIS,PRODUCT_VISUAL_DIRECTION,DESIGN_DECISION_REGISTER,DESIGN_SYSTEM_V2,SCREEN_STATE_MATRIX,COMPONENT_INVENTORY,INTERACTION_AND_ACCESSIBILITY_REVIEW,ITERATION_LOG,CRITIC_REVIEW,VISUAL_ACCEPTANCE_REPORT,OWNER_REVIEW_GUIDE}.md`
- Modify: source files called out in actionable Agent C findings (Agent B only).

- [ ] Integrate verified research and sources without copying product artwork/screens.
- [ ] Complete Iteration 1: layout/design system → build/test → source-level evaluation; record evidence and blocker for GUI capture.
- [ ] Complete Iteration 2: implement preview data/workflows → build/test → independent Agent C review.
- [ ] Complete Iteration 3: targeted corrections → build/test → final adversarial Agent C review.
- [ ] Resolve P0/P1 code/design findings; document P2 and unverified subjective/platform issues without claiming owner approval.

### Task 6: Full verification and owner handoff

**Files:**
- Modify: `docs/ux/design-studio/ITERATION_LOG.md`, `CRITIC_REVIEW.md`, `VISUAL_ACCEPTANCE_REPORT.md`, `OWNER_REVIEW_GUIDE.md`

- [ ] Run format, locked check, strict Clippy, workspace tests, locked Release build, and architecture boundary check using installed compiler.
- [ ] Recheck isolated Phase 0.7 manifests without changing their contents or integration status.
- [ ] Record tool versions, test counts, release size/hash, and exact unrun GUI/Narrator/DPI/offline checks.
- [ ] Preserve current Git history and report unstaged files accurately.
- [ ] Deliver candidate for owner review; do not call it owner-approved or final.
