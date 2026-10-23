# EncryptDrive Desktop

EncryptDrive Desktop is the Windows-first Rust desktop application. The current production workspace contains the Phase 1A native shell and deliberately empty architecture boundaries for future, separately authorized vault work.

## Current application status

The eframe shell is an **engineering preview**. It displays Files and Trash navigation, breadcrumbs, search and file actions, an Explorer-style table, a local/offline status, and System/Light/Dark appearance settings. The default shell has no active vault workflows; it explains unavailable vault support without presenting permanently disabled Create/Open buttons. The opt-in `design-preview` feature plus `--design-preview` argument uses only synthetic in-memory data. The application does not access user files, credentials, real vaults, network services, or telemetry.

The table shows Name, Type, and Size, following the approved UX baseline. Phase 0.7 used synthetic Modified values only for framework comparison; see the discrepancy and resolution in [ADR-012](docs/architecture/ADR-012-EGUI-EFRAME-DESKTOP.md).

## Build and run on Windows

Use a Windows x64 Visual Studio Developer PowerShell with the MSVC C++ tools and Windows SDK installed. The repository's `rust-toolchain.toml` pins Rust 1.99.1; `Cargo.lock` pins dependency resolution.

```powershell
cargo run --locked --package encryptdrive-desktop
```

The app starts empty and performs no data or network I/O. See [the developer guide](docs/development.md) for checks and architecture rules.

UX3 design-system, brand provenance, implementation evidence, and the remaining owner visual/accessibility gate are documented in [the UX3 design-studio report](docs/ux/design-studio/UX3_FINAL_VISUAL_REPORT.md).

## Workspace

- `apps/encryptdrive-desktop/` — native eframe application shell.
- `crates/encryptdrive-core/` — GUI-independent product identity and readiness API.
- `crates/vault-format/`, `crates/vault-crypto/`, `crates/vault-storage/` — reserved, API-empty boundaries.
- `spikes/phase-0-7-gui/` — isolated eframe/Slint evaluation; not a production workspace member.
- `docs/ux/` — Phase 0.5 baseline, Phase 0.7 framework evaluation, and Phase 1A UX implementation/review records.

The Rust product license is undecided. No product `LICENSE` file is included. Dependency and Windows runtime redistribution reviews remain release gates.

Phase 1A build evidence and the outstanding visual/manual validation steps are in the [implementation report](docs/roadmap/PHASE_1A_IMPLEMENTATION_REPORT.md) and [manual checklist](docs/roadmap/PHASE_1A_MANUAL_VALIDATION.md).
