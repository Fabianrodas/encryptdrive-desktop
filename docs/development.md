# Development guide

## Prerequisites

- Windows x64.
- Rust 1.99.1 as pinned in `rust-toolchain.toml` (the workspace declares Rust 1.95 as its minimum).
- Visual Studio C++ Build Tools and Windows SDK for the `x86_64-pc-windows-msvc` target. Use a Developer PowerShell so `link.exe` is available.

## Workspace checks

Run these commands from the repository root:

```powershell
cargo fmt --all -- --check
cargo check --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --package encryptdrive-desktop --release --locked
cargo metadata --no-deps --format-version 1
pwsh -File scripts/verify-architecture.ps1
```

Launch the shell with:

```powershell
cargo run --locked --package encryptdrive-desktop
```

The shell is intentionally non-operational: it has no vault, file, account, or network access. Do not add a UI action that implies an unavailable operation succeeded.

## Architecture rules

- Desktop widgets depend on `encryptdrive-core`, not directly on vault-format, vault-crypto, or vault-storage.
- Core and the three lower crate boundaries stay GUI-independent.
- The Phase 1A lower crates expose no parser, serializer, crypto, or storage placeholder API. Add functionality only in an authorized phase with tests appropriate to its data and security boundaries.
- Keep `spikes/phase-0-7-gui/` independent. Its manifests and lockfiles are not part of the production workspace; validate them with their own `--manifest-path` and lockfiles.
- Keep private strategic history in the private documentation repository. Do not copy private evidence, Java source, credentials, vaults, or user files here.
- Do not add a root license until the owner decides the Rust product license. Re-check dependency licenses, attribution, and runtime redistribution for release artifacts.
