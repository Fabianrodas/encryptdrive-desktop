# Phase 1A implementation report

**Status:** Workspace, core readiness API, and native shell implemented; review required  
**Date:** 2026-10-19

## Repository state

At start, the Rust repository was on unborn branch `main` with no `HEAD` commit. `.gitignore`, `docs/`, and `spikes/` were already untracked. The nine Phase 0.5 UX documents and Phase 0.7 evaluation were preserved. At finish, `main` remains unborn with no commits; all production additions are visible as untracked files under the same repository. No staging, commits, branch changes, tags, pushes, or visibility changes were made.

The private documentation repository retained its pre-existing uncommitted Phase 0.6A work and was read-only. The Java repository was read-only. Neither repository was written by this implementation.

## Toolchain and dependencies

- Rustup 1.29.1; Rust/Cargo 1.99.1 (MSVC host), rustfmt 1.10.0, Clippy 0.1.99.
- Installed target: `x86_64-pc-windows-msvc`; MSVC compiler/linker and Windows SDK were available.
- Root `rust-toolchain.toml` and Windows CI pin Rust 1.99.1. For local validation, `RUSTUP_TOOLCHAIN=stable` selected the already-installed alias, which reported the same exact Rust 1.99.1 compiler; invoking the absent exact alias would attempt a denied Rustup temp-file write in this sandbox. No global toolchain was installed or changed.
- Workspace package metadata declares Rust 1.95 minimum. Root lockfile contains 367 package records. Direct GUI dependencies are eframe 0.37.0 and egui_extras 0.37.0; WGPU 30.0.1 and AccessKit 0.25.1 are resolved transitively.
- Windows-filtered Cargo metadata found 159 registry packages with license metadata present for all 159. This is not a complete license-text or redistribution audit. See the [dependency and runtime review](../architecture/DEPENDENCY_AND_RUNTIME_REVIEW.md).

## Build and test results

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace --locked` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --locked` | PASS — 7 tests (3 core readiness, 4 shell state/theme) |
| `cargo build --workspace --release --locked` | PASS — Windows MSVC Release build |
| `pwsh -File scripts/verify-architecture.ps1` | PASS — exact workspace membership and direct dependency edges |
| Independent checks of vault-format, vault-crypto, vault-storage, and encryptdrive-core | PASS |
| Phase 0.7 common, eframe, and Slint `cargo check --locked --offline` | PASS for all three isolated manifests |

The local Cargo filesystem emits a hard-link warning for incremental compilation and copies the cache files instead. Checks and strict Clippy still pass. An unfiltered all-platform `cargo metadata` request attempted to reach `static.crates.io`, which the environment could not resolve; the Windows-filtered `--offline` metadata query succeeded and is the relevant product target.

## Release artifact

| Artifact | Size | SHA-256 |
|---|---:|---|
| `target/release/encryptdrive-desktop.exe` | 14,286,848 bytes (13.625 MiB) | `D34B5ADF69BE5C2588432AB1C4BF6779A740EF35D6651744136E778737F6AAB0` |

`dumpbin /DEPENDENTS` confirms imports of `VCRUNTIME140.dll`, Windows CRT API-set DLLs, UI Automation Core, DirectWrite, DXGI, OpenGL, and ordinary Windows system DLLs. No installer, portable package, CRT-static build, clean-host test, or disconnected launch was produced.

## Implemented shell and boundaries

- Compact Explorer-style native window with Files/Trash navigation, breadcrumb/status, disabled search and action controls, Name/Type/Size table, and Settings entry.
- Light/Dark themes derive from the Phase 0.5 semantic token values; Settings changes appearance in memory only.
- Empty startup state says no vault is open. All declared core capabilities are unavailable. No user-file, vault, account, secret, network, or telemetry operation is implemented.
- `encryptdrive-core` has no dependencies. `vault-format`, `vault-crypto`, and `vault-storage` contain documented crate boundaries only. No parser, crypto placeholder, storage shim, or vault I/O was added.
- The app uses Name, Type, and Size. The Phase 0.7 synthetic Modified field is excluded because the approved Phase 0.5 product UX does not define a trustworthy modification time; see [ADR-012](../architecture/ADR-012-EGUI-EFRAME-DESKTOP.md).

## Not run / remaining gates

The GUI was not opened, controlled, or screenshotted because the earlier explicit desktop automation stop remains in force. Therefore visual polish, focus states, window resizing, Narrator/UI Automation, keyboard navigation, display scaling, High Contrast, and offline cold launch are **NOT RUN**. The exact manual steps are in the [validation checklist](PHASE_1A_MANUAL_VALIDATION.md).

The Phase 1A shell is not a functional vault manager. File selection, custom file-manager shortcuts, context menus, dialogs, operation progress, authentication, vault create/open, Format 1 parsing, cryptography, storage, import/export, trash/recovery, and Phase 1B remain out of scope. The Rust product license and Windows runtime redistribution plan remain open. eframe is approved for this shell implementation only; accessibility and distribution acceptance remain mandatory release gates.
