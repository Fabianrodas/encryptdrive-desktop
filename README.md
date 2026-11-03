# EncryptDrive Desktop

EncryptDrive Desktop is a Windows-first Rust desktop application. The current workspace contains a native engineering shell and deliberately empty architecture boundaries for future, separately authorized vault work.

## Current application status

### Default production shell

Running the application without the `design-preview` feature opens the default engineering shell. It contains the compact application header, Files and Trash navigation, Settings, a status bar, and empty states for the unavailable vault and Trash views. It has **no operational file table, table columns, sorting, command bar, search control, or file-action shortcuts**. It does not provide real vault functionality.

### Opt-in synthetic design preview

The opt-in design preview is a separate review build. It uses synthetic in-memory fixtures and provides an Explorer-style table, search, sorting, breadcrumb navigation, simulated file actions, representative dialogs, sample data, and simulated progress and error states. These controls do not import, export, mutate, or open real files or vaults. Password-form screens are demonstrations only; do not enter a real password. The preview can receive text typed by the user, so it should not be described as accepting no user input.

No real-vault operations or production credential-handling workflow are implemented. This describes product capabilities visible in the source; it is not a claim of zero runtime I/O. The app uses operating-system resources such as fonts and Windows monitor/work-area information, and eframe or Windows graphics drivers may perform framework activity. Filesystem, registry, network, and child-process behavior have not been instrumented at runtime; see the open B04 gate in [the Phase 1B-4 validation record](docs/roadmap/PHASE_1B_SHELL_ISOLATION_VALIDATION.md).

## Build and run on Windows

Use a Windows x64 Visual Studio Developer PowerShell with the MSVC C++ tools and Windows SDK installed. The repository's `rust-toolchain.toml` pins Rust 1.99.1; `Cargo.lock` pins dependency resolution.

Run the default engineering shell:

```powershell
cargo run --locked --package encryptdrive-desktop
```

Run the separate synthetic design preview:

```powershell
cargo run --locked --package encryptdrive-desktop --features design-preview -- --design-preview
```

See [the developer guide](docs/development.md) for checks and architecture rules.

UX3 design-system, brand provenance, implementation evidence, and the remaining owner visual/accessibility gate are documented in [the UX3 design-studio report](docs/ux/design-studio/UX3_FINAL_VISUAL_REPORT.md).

## Workspace

- `apps/encryptdrive-desktop/` — native eframe application, with a default engineering shell and opt-in synthetic design preview.
- `crates/encryptdrive-core/` — GUI-independent product identity and readiness API boundary; the current desktop source does not call its readiness API, although the desktop manifest declares the dependency.
- `crates/vault-format/`, `crates/vault-crypto/`, `crates/vault-storage/` — reserved, API-empty boundaries; they do not implement VF1, cryptography, or real storage.
- `spikes/phase-0-7-gui/` — isolated eframe/Slint evaluation; not a production workspace member.
- `docs/ux/` — Phase 0.5 baseline, Phase 0.7 framework evaluation, and Phase 1A UX implementation/review records.

The Rust product license is undecided. No product `LICENSE` file is included. Dependency and Windows runtime redistribution reviews remain release gates.

Phase 1A build evidence and the outstanding visual/manual validation steps are in the [implementation report](docs/roadmap/PHASE_1A_IMPLEMENTATION_REPORT.md) and [manual checklist](docs/roadmap/PHASE_1A_MANUAL_VALIDATION.md).
