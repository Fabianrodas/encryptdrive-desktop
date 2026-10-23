# EncryptDrive Phase 0.7 — Native Rust GUI framework spike

This isolated spike contains two independent native Windows GUI prototype crates and one shared GUI-independent synthetic model. Phase 0.7E source checks, 18 shared model tests, one eframe Unicode font-shaping test and one Slint size-format test pass. Current Release rebuilds are kept under `target/phase-0-7e/`. The current binaries have not been opened or visually verified; prior Phase 0.7D UIA/visual evidence is for older builds. Most interaction, accessibility, DPI and clean-host/offline scenarios remain NOT RUN, so no framework is selected. This spike has no production vault, crypto, storage or application-core code.

## Contents

- `common/` — stable synthetic IDs, deterministic fixtures, logical selection/navigation/sort/dialog/operation state, shared semantic tokens and state tests.
- `egui-prototype/` — eframe 0.37.0 with `egui_extras::TableBuilder`, WGPU and AccessKit feature enabled.
- `slint-prototype/` — Slint 1.18.1 declarative UI with custom virtualized `ListView` delegates and stable-ID selection in the shared model.
- `test-scenarios/GUI_SCENARIOS.md` — shared manual run script.
- `results/` — build, functional, accessibility, DPI, binary, dependency and comparison evidence.

## Launch current Phase 0.7E binaries on Windows

From the repository root, run the prebuilt applications directly. Cargo and Rust are not needed at launch time. These commands use synthetic in-memory data only:

```powershell
& '.\spikes\phase-0-7-gui\target\phase-0-7e\release\encryptdrive-eframe-spike.exe' --entries 100
& '.\spikes\phase-0-7-gui\target\phase-0-7e\release\encryptdrive-slint-prototype.exe' --entries 100
```

Change `100` to `1000` or `10000` to set the fixture size. Use [`results/OWNER_GUI_REVIEW.md`](results/OWNER_GUI_REVIEW.md) to record paired UIA, Narrator, DPI and interaction results. A direct run on the developer host does not prove clean-host or disconnected-network behavior.

## Rebuild on Windows

Use Rust 1.95 or later for `x86_64-pc-windows-msvc`, Visual Studio C++ Build Tools and a Windows SDK. Start an x64 Developer PowerShell so `link.exe` is available. In each crate directory, the reproducible verification sequence is:

```powershell
cargo +stable generate-lockfile
cargo +stable fmt --check
cargo +stable check --locked
cargo +stable clippy --locked --all-targets -- -D warnings
cargo +stable test --locked
cargo +stable build --locked --release
```

To preserve older binaries, build into a separate target directory:

```powershell
cargo build --locked --release --target-dir ..\target\phase-0-7e
```

The command above is run separately from each prototype directory. It writes the executable to `spikes/phase-0-7-gui/target/phase-0-7e/release/`. Phase 0.7D static-CRT comparisons are under `target/phase-0-7d-crt-static/` and were not launched.

The following is the initial Phase 0.7C pre-install snapshot, retained for the audit trail and superseded by the current verification below: Rustup/Cargo were installed but the first Visual Studio modification failed with Windows Installer 1622; at that time no linker or release executable was available.

**Current verification:** Rust/Cargo `1.99.1`, rustfmt `1.10.0`, Clippy `0.1.99`, target `x86_64-pc-windows-msvc`, MSVC toolset `14.51.36231`/compiler `19.51.36260`, SDK `10.0.19041.0`. Phase 0.7E format, locked check, strict Clippy, tests, and Release rebuilds pass. eframe enables AccessKit, WGPU and native `system_fonts`; Slint uses its custom `ListView` and explicit stable-ID selection model. The current GUI binaries were not opened after the Escape stop. See `results/` for exact binary hashes/imports and remaining evidence gaps.

The Windows Computer Use helper was stopped by the user with the physical Escape key. Before the stop, both earlier Phase 0.7D builds were visibly opened at 100 entries, Slint single-row selection was observed in UIA, and Slint Create Folder validation and synthetic creation were exercised. Initial screenshots were displayed in the session but not saved. Current 0.7E visual checks, most GUI interactions, DPI/renderer behavior, Narrator, disconnected offline launch and clean-host installer checks remain NOT RUN. Do not select a framework or proceed to Phase 1 on this evidence alone.

Current Release binaries: `target/phase-0-7e/release/encryptdrive-eframe-spike.exe` and `target/phase-0-7e/release/encryptdrive-slint-prototype.exe`. Phase 0.7D static-CRT variants are in `target/phase-0-7d-crt-static/release/`. Ordinary PE imports include the Microsoft Visual C++ runtime and Windows system DLL dependencies. Neither static-CRT variant was launched. See the dependency report before planning redistribution.

Synthetic state remains in memory. The prototypes do not access real user files, vaults, credentials, secret keys, encrypted data or network services. Simulated Import and Export do not open a picker, read, decrypt, write or launch files.

The project’s nine Phase 0.5 UX documents and both reference repositories were treated as read-only. No Git history was initialized or changed.
