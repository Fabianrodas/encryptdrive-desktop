# Phase 1A Production Workspace Implementation Plan

> **For agentic workers:** Inline execution is authorized by the user's implementation request. Work task-by-task; keep the existing spike isolated and do not create commits.

**Goal:** Establish the approved Windows Rust Desktop production workspace, an honest non-vault eframe shell, documented crate boundaries, and repeatable validation.

**Architecture:** One Cargo workspace contains the Desktop shell and four GUI-independent boundary crates. The shell depends on `encryptdrive-core`; all lower crates remain deliberately API-empty until their authorized implementation phase. The Phase 0.7 spike remains a separate nested workspace.

**Tech Stack:** Rust 1.99.1 MSVC, edition 2024, eframe/egui and egui_extras 0.37.0, GitHub Actions on Windows.

**Spec:** User-provided EncryptDrive Desktop Phase 1A request; current product direction in `docs/ux/` and private reference baselines (read-only).

## Global Constraints

- Only modify `encryptdrive-desktop`; never modify the Java or private documentation repositories.
- Preserve `docs/ux/` and `spikes/phase-0-7-gui/`; keep the latter excluded from the production workspace.
- Do not implement Vault Format 1 parsing, cryptography, vault I/O, authentication, import/export, or production operations.
- No real vault, user files, secrets, network access, or telemetry.
- Do not create a license file; the Rust product license remains undecided.
- Do not commit, stage, push, tag, publish, or alter repository visibility.
- Do not open the GUI or use desktop automation in this run; the earlier explicit stop remains in force.

## Review Focus

- The preview must not imply an unlocked vault or implemented operations; test readiness and empty capabilities.
- The core crate must remain GUI-independent; inspect Cargo metadata and enforce dependency boundaries.
- Light/Dark appearances must derive from stable shared semantic tokens; test exact token resolution.
- The GUI must start without opening files, vaults, or network services; test initial shell state and inspect source.
- Adding the production root workspace must not absorb or alter Phase 0.7; run its existing locked checks independently.

## File Map

- Root: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.gitignore`, `.github/workflows/ci.yml`.
- Production packages: `apps/encryptdrive-desktop/` and `crates/{vault-format,vault-crypto,vault-storage,encryptdrive-core}/`.
- Public docs: `README.md`, `docs/development.md`, `docs/architecture/`, `docs/roadmap/`.
- Architecture boundary check: `scripts/verify-architecture.ps1`.
- Existing Phase 0.5 docs and Phase 0.7 spike are preserved.

## Tasks

### Task 1: Workspace and boundary crates

- Create the pinned root workspace and five package manifests.
- Ensure the root members contain exactly the Desktop app and four core boundaries; explicitly exclude the nested spike packages.
- Add crate-level documentation explaining that no format, crypto, storage, or domain APIs are implemented in Phase 1A.
- Verify `cargo metadata --no-deps` and independent package checks.

**Status:** Complete.

**Ruling:** Pin the repository and CI to exact Rust 1.99.1 for reproducibility. For local validation only, set `RUSTUP_TOOLCHAIN=stable`: the already-installed alias reports the same Rust 1.99.1 compiler, while invoking the absent exact alias would attempt a denied Rustup temp-file write outside the workspace. Cost if wrong: local validation must be repeated when the pinned toolchain is installed normally; the exact version was verified from the compiler output.

### Task 2: Core readiness API (TDD)

- First add tests for product identity, engineering-preview readiness, and every declared vault capability being unavailable; run them red.
- Implement a small GUI-independent `encryptdrive-core` API that makes those assertions pass without placeholder security functions.
- Run core tests, format, and strict Clippy.

**Status:** Complete; 3 readiness tests pass.

### Task 3: Native application shell (TDD for state/tokens)

- First add tests for initial no-vault state and deterministic Light/Dark token mapping; run red.
- Implement the eframe 0.37.0 native Explorer × Premium Desktop shell with Files/Trash navigation, hierarchical sidebar, breadcrumbs, search, action toolbars, Name/Type/Size table, explicit unavailable-operation state, and Settings theme switch.
- Keep mutating actions disabled and show `Engineering preview — vault operations unavailable`.
- Run application tests and strict Clippy; build a Windows release executable.

**Status:** Complete; 4 shell-state/theme tests pass and the Windows Release executable builds.

### Task 4: Architecture records and developer workflow

- Add public-safe ADR-012, Phase 1A crate boundary/roadmap notes, a developer guide, and a Windows CI workflow.
- Document the Phase 0.5/0.7 discrepancy over a Modified column and sort semantics; use only fields supported by the approved UX baseline in the shell.
- Add and run a metadata-based architecture boundary script; do not add a LICENSE file.

**Status:** Complete; boundary script passes.

### Task 5: Whole-workspace validation and report

- Run format, workspace check, strict Clippy, tests, and release build on the installed MSVC toolchain.
- Measure the release executable and inspect available PE/runtime dependencies.
- Re-run the isolated Phase 0.7 manifest checks with their own locks.
- Record exact repository states, versions, checks, binary evidence, GUI/manual limitations, license gate, and remaining Phase 1A boundaries in the docs.

**Status:** Complete; evidence is in `docs/roadmap/PHASE_1A_IMPLEMENTATION_REPORT.md`.

## Rulings

- **Toolchain pin:** pin Rust 1.99.1 in the repository and CI. Use the installed stable alias only for local checks because the exact-named alias is absent and Rustup cannot write its temporary sync file outside this workspace. Cost if wrong: the local compiler could differ from the pin; both report Rust 1.99.1 in this environment.
- **Modified field:** use Name, Type, and Size instead of Phase 0.7's synthetic Modified column because the approved UX baseline does not define a trustworthy modification field. Cost if wrong: a later owner-approved data contract may require the production shell to add a column.
- **GUI verification:** do not open or automate the new GUI because the earlier explicit desktop automation stop remains in force. Cost: no visual, keyboard, Narrator, DPI, or clean-host launch claim is available; the manual checklist remains open.
- **Git history:** leave all changes uncommitted because the repository is unborn and the owner explicitly requested no commit. Cost: the review state remains a visible set of untracked files until the owner chooses a later integration step.

