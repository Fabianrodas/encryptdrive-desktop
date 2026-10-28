# Phase 1B-0 / 1B-1 — Build Identity and Artifact Isolation

**Status:** Limited implementation authorized; Phase 1B remains in progress and is not complete.

## Authorized scope

The owner authorized only:

- **Phase 1B-0:** establish the engineering baseline and create the functional integration branch from the approved `main` baseline.
- **Phase 1B-1:** isolate default-shell and synthetic `design-preview` Windows builds, record build identity, and measure reproducibility.

This authorization does not include Phase 1B-2 or later, Vault Format 1, cryptography, storage, real-vault operations, application command/event APIs, packaging, signing, distribution, upload, publication, or merge to `main`.

## Verified starting baseline

| Item | Recorded value |
|---|---|
| Rust repository | `encryptdrive-desktop` |
| Starting branch | `main` |
| Starting commit | `50ab6adf1a0e3a4cea2a2d2f676136fe1e0f0cbc` |
| Local worktree | Clean before branch creation |
| Local `origin/main` tracking ref | `50ab6adf1a0e3a4cea2a2d2f676136fe1e0f0cbc` |
| New integration branch | `dev`, created from the starting commit |
| Windows target | `x86_64-pc-windows-msvc` |
| Rust toolchain | `1.99.1-x86_64-pc-windows-msvc` |
| Framework | eframe/egui `0.37.0` |

The local Git remote uses the expected EncryptDrive Desktop repository identity. A live `git ls-remote` check could not complete because Windows denied creation of an SSH IPC object (`NtCreateDirectoryObject`, `0xC0000022`); this record therefore confirms the local tracking ref, not the current server-side ref.

The approved UX6.3 design and its existing accessibility, High Contrast, DPI, multi-monitor, runtime, clean-host, licensing, and distribution gates remain unchanged. The synthetic preview remains synthetic-only.

## Build-identity evidence

Implementation, measured artifacts, validation results, reproducibility observations, and independent-review items will be added below after the 1B-1 candidate is built and verified. Local executables and generated manifests remain under ignored `target/` paths and are not committed.
