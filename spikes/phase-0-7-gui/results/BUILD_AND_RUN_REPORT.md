# Build and run report

**Phase 0.7E source checks and Release builds: PASS; current GUI validation: NOT RUN; no framework selected.** The shared model passes 18 scenario tests; eframe passes one Unicode font-fallback test; Slint passes one size-format test. `cargo fmt --check`, locked checks, and strict Clippy pass for common and both apps. The 0.7D visible/UIA observations below belong to earlier binaries. Current Phase 0.7E executables were built and PE imports inspected but were not launched. The direct owner review is in [`OWNER_GUI_REVIEW.md`](OWNER_GUI_REVIEW.md). DPI, Narrator, clean-host/offline launch and owner/legal approval remain open.

## Phase 0.7E current build verification — 2026-10-18

| Crate / check | Result |
|---|---|
| common `cargo fmt --check`, `cargo check --locked`, strict Clippy, `cargo test --locked` | PASS; 18/18 state scenarios |
| eframe `cargo fmt --check`, `cargo check --locked`, strict Clippy, `cargo test --locked` | PASS; 1/1 Unicode fixture/system-font shaping test |
| Slint `cargo fmt --check`, `cargo check --locked`, strict Clippy, `cargo test --locked` | PASS; 1/1 binary size-format test |
| eframe and Slint `cargo build --locked --release --target-dir ..\target\phase-0-7e` | PASS; output under `target/phase-0-7e/release/` |
| Current GUI interaction, screenshot, Narrator, DPI and disconnected clean-host tests | NOT RUN |

Toolchain: rustup `1.29.1`; Rust/Cargo `1.99.1`; rustfmt `1.10.0`; Clippy `0.1.99`; `x86_64-pc-windows-msvc`; MSVC toolset `14.51.36231`, compiler `19.51.36260`, Windows SDK `10.0.19041.0`. The eframe manifest now enables `system_fonts`; its focused test passes for the Japanese fixture with this host's installed fonts. Slint focus color now maps through a `focus-ring` property after `focus` was rejected as a reserved member name. Shared validation rejects Windows device names, reserved characters, trailing dots and controls. None of these checks substitutes for visual or UIA validation.

## Starting repository state

| Repository | Starting branch / HEAD | Starting worktree | Treatment |
|---|---|---|---|
| `encryptdrive-desktop` | `main`; unborn branch, no HEAD commit | Untracked `.gitignore`, `docs/` with the nine Phase 0.5 UX documents, and `spikes/`; no tracked changes | Added the Phase 0.7 spike under `spikes/phase-0-7-gui/` and the new Phase 0.7 evaluation document under `docs/ux/`. The nine Phase 0.5 documents were preserved. No history initialization, staging or branch switch. |
| `encryptdrive-docs` | `docs/rust-transition-phase-0-6a`; HEAD `4b853627c51228fbc7fc53bdca859ee78dfc2303` | 12 modified tracked files and 10 untracked private files | Read-only; preserved. |
| Java `EncryptDrive` | `codex/phase4-dialog-notification-inline-form-ux`; HEAD `1f069478362eb68cf587206e86c9f520f393399f` | Tracked files clean; pre-existing `target_test-classes/` untracked | Read-only; preserved. |

## Toolchain inventory at implementation time

- OS registry reports Windows 10 Pro, NT version `10.0.26200.9711`, x64 (DisplayVersion `25H2`); Windows SDK `10.0.19041.0` was present.
- `rustc`, `cargo`, `rustup`, `rust-analyzer`, `cl`, `link`, `clang`, `lld-link`, GCC/MinGW, CMake, Ninja and MSBuild were not found on PATH.
- `vswhere.exe` found Visual Studio 18 Community at `C:\Program Files\Microsoft Visual Studio\18\Community`, but found no installation containing `Microsoft.VisualStudio.Component.VC.Tools.x86.x64`; no `cl.exe` or `link.exe` was found under the Visual Studio roots.
- `winget` could not be launched because the system denied access to its executable. No global tools were installed.
- Rust target and installed Rust version: **NOT AVAILABLE**.
- No GUI execution/accessibility automation surface was available in this task session.

## Prototype manifests

| App | Direct GUI crates | Rendering / accessibility setup | State |
|---|---|---|---|
| eframe | `eframe = 0.37.0`, `egui_extras = 0.37.0` (exact pins) | WGPU, AccessKit, default fonts, native system-font fallback; virtualized `TableBuilder` rows | `Cargo.lock`; fmt/check/strict Clippy/tests/release PASS; current launch NOT RUN |
| Slint | `slint = 1.18.1`, `slint-build = 1.18.1` (exact pins) | Winit + FemtoVG, software renderer feature, accessibility feature; custom virtualized `ListView` rows | `Cargo.lock`; fmt/check/strict Clippy/tests/release PASS; current launch NOT RUN |

Both depend on `../common`. The apps are separately invoked from their own directories. Independent Cargo.lock files now exist for both apps and the shared crate.

## Initial Phase 0.7B command snapshot — historical

In the initial Phase 0.7B environment, all five commands stopped before execution because PowerShell could not resolve `cargo`. That is historical evidence, superseded by the Phase 0.7C run below.

Release executable, runtime launch, offline launch, dependency inspection, binary size, screenshots and runtime measurements: **NOT RUN**.

## Original blocker follow-up — superseded by Phase 0.7C below

Use the existing `Cargo.lock` files and run the locked commands listed in the parent README. Fix compiler/lint/test failures before running `GUI_SCENARIOS.md`. Preserve lockfiles and record tool versions and exact outputs. This report does not mark either GUI prototype as built or working.

## Phase 0.7B toolchain re-audit — 2026-10-18

Direct rustc --version, cargo --version and rustup show commands were attempted and PowerShell could not resolve any of them. The current-user C:\Users\Asus Vivobook\.cargo\bin and .rustup\toolchains directories do not exist; CARGO_HOME and RUSTUP_HOME are unset, and no PATH entry contains Rust or Cargo. Recursive searches of the user's Tools, bin, AppData\Local\Programs and Codex runtime cache found no rustc.exe or cargo.exe. No Rust version or MSVC target is installed or can be reported.

MSVC prerequisites: cl and link are not resolvable and where.exe found neither. vswhere found Visual Studio 18 Community at C:\Program Files\Microsoft Visual Studio\18\Community but did not find the required Microsoft.VisualStudio.Component.VC.Tools.x86.x64 component; searches found no cl.exe or link.exe. Windows SDK 10.0.19041.0 is present, including x64 rc.exe. The SDK alone does not supply the MSVC compiler or linker. No target ABI can be built in this environment.

The manifest minimum was corrected to Rust 1.95 for the shared crate and both prototypes. This reflects the supplied egui 0.37.0 MSRV correction. It does not mean Rust 1.95 is installed or that the manifests were checked by Cargo.

Fresh attempts were made from common, egui-prototype and slint-prototype for cargo fmt --check, cargo check, cargo clippy --all-targets -- -D warnings, cargo test and cargo build --release. All 15 commands were BLOCKED before starting because cargo is not recognized. These are environment blockers, not compiler/test failures and not evidence that the Rust source compiles.

The Windows SDK was present; the host lacks Rustup/Rust/Cargo and the Visual Studio Community installation lacks the C++ x64 build tools. No toolchain was installed or changed. No generated lockfiles, executables, runtime launch evidence or build measurements exist.

To unblock on the Windows host, install the latest stable Rust toolchain with target x86_64-pc-windows-msvc and use Visual Studio Installer to add the Desktop development with C++ workload and x64 MSVC tools to the existing Visual Studio 18 Community installation (or install the equivalent Build Tools workload); retain or select the already installed Windows SDK 10.0.19041.0. Then open an x64 Native Tools or Developer PowerShell and verify rustc --version, cargo --version, rustup show, where.exe cl and where.exe link before running the commands in MANUAL_VALIDATION_CHECKLIST.md. This setup changes user/global tools and needs owner authorization; this agent did not perform it.

The six requested launch attempts (each prototype with --entries 100, 1000 and 10000) were also made from the app directories. Every attempt was BLOCKED before Cargo started because cargo is not recognized. No app process was launched.

## Phase 0.7C pre-install snapshot — historical, superseded later on 2026-10-18

### Installed Rust toolchain

- The official Rustup Windows MSVC installer was downloaded from `static.rust-lang.org`; its `.sha256` sidecar matched the downloaded executable (`6f4bef66261261fcb43131be8720bab817d403a09edec7455c371974b90bdb7e`). Rust recommends Rustup for standard installs and lists Visual Studio MSVC tools as the Windows prerequisite ([Rust install guide](https://rust-lang.org/install.html)).
- `rustup 1.29.1 (d95a37b6a 2026-08-13)`; stable `rustc 1.99.1 (58cf8f9c3 2026-10-13)`; Cargo `1.99.1 (5f94df478 2026-08-27)`; host and installed target `x86_64-pc-windows-msvc`; `rustfmt 1.10.0-stable`; Clippy `0.1.99`.
- Installed stable components: cargo, clippy, rust-std, rustc, rustfmt. The x86_64 MSVC target is installed. The user profile Rustup/Cargo setup succeeded.

### Visual Studio MSVC workload status

The authorized change targeted the existing Visual Studio Community 2026 instance only and requested `Microsoft.VisualStudio.Workload.NativeDesktop` plus `Microsoft.VisualStudio.Component.VC.Tools.x86.x64`, without `--includeRecommended`; the existing Windows SDK 10.0.19041.0 was not separately selected for reinstallation. The first modification returned installer error 1603 because the x86 `Microsoft.VisualCpp.Redist.14` step failed with `0x80070656` / Windows Installer 1622. The installer log shows its wrapper passing a `%TEMP%` log path containing `Asus Vivobook` unquoted to the redistributable executable. A second attempt uses process-only `TEMP`/`TMP=C:\Windows\Temp`; at report time its shell session is still pending Windows elevation and no new installer log or MSVC compiler files have appeared. `vswhere` does not yet report the NativeDesktop workload or VC tools component, and `cl.exe`/`link.exe` remain absent. The installation is therefore **BLOCKED/incomplete**, not passed. Microsoft documents `setup.exe modify` for an existing instance and notes the operation may require administrator elevation ([modify guidance](https://learn.microsoft.com/en-us/visualstudio/install/modify-visual-studio?view=visualstudio), [command-line parameters](https://learn.microsoft.com/en-us/visualstudio/install/use-command-line-parameters-to-install-visual-studio?view=visualstudio)).

This is a retained pre-install snapshot, superseded by the verified installation and retry outcome later in this report.

### Actual command results

| Command | Result | Evidence |
|---|---|---|
| `cargo fmt --check` in common, eframe, and Slint crates | PASS (3/3) | Formatting discrepancies were corrected with rustfmt before the passing rerun. |
| `cargo check --locked` in common | PASS | `encryptdrive-spike-common` completed dev-profile check. |
| `cargo clippy --all-targets --locked -- -D warnings` in common | PASS | Two `manual_is_multiple_of` findings were corrected, then the strict rerun passed. |
| `cargo check --locked` in eframe | BLOCKED | Cargo/Rust started; a host build script failed because `link.exe` was not found. |
| `cargo check --locked` in Slint | BLOCKED | Cargo/Rust started; host build scripts failed because `link.exe` was not found. |
| `cargo test --locked` in common | BLOCKED | Test compilation stopped because `link.exe` was not found; no test case executed. |
| GUI `clippy`, GUI tests, release builds | NOT RUN / BLOCKED | Dependent on the missing MSVC linker; no executable exists. |

Cargo `generate-lockfile` succeeded for `common/`, `egui-prototype/`, and `slint-prototype/`. Resolution locked 355 packages for the eframe app and 537 for Slint, selecting the highest versions compatible with each manifest's Rust 1.95 MSRV. Each lockfile is Cargo format version 4. The 10,000-entry fixture and UI are still unmeasured because neither app has built or launched. No UI scenario was run; the Windows Computer Use helper was stopped by the user with Escape, so this turn performed no further desktop UI inspection.

## Phase 0.7C authorized toolchain and build follow-up — 2026-10-18

### Installation verified

- Official Rustup installer SHA-256: `6f4bef66261261fcb43131be8720bab817d403a09edec7455c371974b90bdb7e`; Rustup `1.29.1 (d95a37b6a 2026-08-13)`.
- Stable `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, Cargo `1.99.1 (5f94df478 2026-08-27)`, host/target `x86_64-pc-windows-msvc`, rustfmt `1.10.0-stable`, Clippy `0.1.99`.
- Added `Microsoft.VisualStudio.Workload.NativeDesktop` and `Microsoft.VisualStudio.Component.VC.Tools.x86.x64` to the existing Visual Studio Community 18 instance. Verified with `vswhere`; MSVC toolset `14.51.36231`, compiler `19.51.36260`; `cl.exe` and `link.exe` resolve after `vcvarsall.bat x64`.
- Existing Windows SDK `10.0.19041.0` was retained and not separately reinstalled. The redistributable installer’s initial 1622 failure was recovered by retrying with process-only `TEMP`/`TMP=C:\Windows\Temp`; no persistent environment override was added.

### Fresh validation

| Crate / command | Result |
|---|---|
| common `cargo fmt --check` | PASS |
| common `cargo check --locked` | PASS |
| common `cargo clippy --locked --all-targets -- -D warnings` | PASS |
| common `cargo test --locked` | PASS — 8 scenario tests; 0 failed |
| eframe `cargo fmt --check`, `cargo check --locked`, strict Clippy, `cargo test --locked` | PASS; app test target contains 0 GUI tests |
| Slint `cargo fmt --check`, `cargo check --locked`, strict Clippy, `cargo test --locked` | PASS; app test target contains 0 GUI tests |
| both `cargo build --locked --release` | PASS — eframe 4m 15s; Slint 3m 23s |

Cargo emitted a filesystem hard-link warning for incremental cache files and fell back to copying; it did not fail checks, tests, or builds. No filesystem settings were changed to suppress it.

Compiler/API defects corrected: egui 0.37 panel/input/table APIs; Slint Window preferred-size constraints and required accessibility role; a Slint theme-toggle borrow conflict; and strict Clippy findings in both GUIs. Slint’s unused operation/action parameters were removed and key modifiers grouped, with behavior preserved.

### Native process launch

Each release executable was started hidden with `--entries` values 100, 1,000 and 10,000. Every process was alive after 1.5 seconds and exposed a non-zero native window handle. This confirms native process/window creation only; it is not a visual or interaction test. Per-size startup memory snapshots are in `PERFORMANCE_AND_BINARY_REPORT.md`. No screenshot was captured because the user stopped desktop UI automation with Escape; no further UI automation/input was used.

The release binaries start on this configured host. A disconnected-network clean-machine launch is **NOT RUN**; runtime DLL inspection shows a dependency on `VCRUNTIME140.dll` and the Windows API-set CRT, so packaging/redistributable feasibility remains open. The two app test harnesses contain no GUI tests; only the shared model's 8 scenario tests exercised logical state.

## Phase 0.7D execution update — 2026-10-18

### Repository state and boundaries

- Starting Rust repo: branch `main`, unborn HEAD, `origin/main` absent, pre-existing untracked `.gitignore`, `docs/` (nine Phase 0.5 UX files), and `spikes/`. There were no pre-existing tracked modifications.
- Work was limited to `spikes/phase-0-7-gui/` and `docs/ux/PHASE_0_7_FRAMEWORK_EVALUATION.md`. No commit, branch switch, staging, cleanup or reset occurred.
- Final read-only Git status rechecks match the recorded pre-existing states. The private docs repo remains on `docs/rust-transition-phase-0-6a` with the same 12 modified tracked and 10 untracked files; Java remains on `codex/phase4-dialog-notification-inline-form-ux` with tracked files clean and the same untracked `target_test-classes/`. Neither repository was modified by this work.
- Rust/Cargo remain `1.99.1`; rustfmt `1.10.0`; Clippy `0.1.99`; target `x86_64-pc-windows-msvc`; MSVC toolset `14.51.36231`; compiler `19.51.36260`; SDK `10.0.19041.0`.

### Updated checks and corrections

| Check | common | eframe | Slint |
|---|---|---|---|
| `cargo fmt --check` | PASS | PASS | PASS |
| `cargo check --locked` | PASS | PASS | PASS |
| `cargo clippy --all-targets --locked -- -D warnings` | PASS | PASS | PASS |
| `cargo test --locked` | PASS, 9 scenario tests | PASS, 0 GUI tests | PASS, 1 format test |
| `cargo build --locked --release` | N/A library | PASS | PASS |

The Slint size-parity test was observed failing before the formatter correction and passes now. The new shared page-aware Delete destination test was observed failing before the model method existed and passes now. The fixes and visual verification limit are described in `FUNCTIONAL_TEST_MATRIX.md`.

### Visible GUI evidence and stop condition

Both original Release binaries were visibly opened at 100 entries. Equivalent outer-window captures were 1180×792, including frame; no screenshot files were persisted. Slint single-row selection was observed across successive UIA snapshots, and Create Folder empty validation plus successful synthetic folder creation were exercised. Its Dark appearance was legible. The initial Slint Light palette had low-contrast default controls; a source correction now synchronizes its palette, but that new build was not visibly launched. eframe displayed the Explorer-like table with dense rows, but the Japanese fixture characters were tofu. More details and statuses are in `FUNCTIONAL_TEST_MATRIX.md` and `ACCESSIBILITY_REPORT.md`.

The helper returned the exact stop message “Computer Use was stopped by the user with the physical Escape key.” No additional Windows app input was sent. Updated optimized builds were produced in `target/phase-0-7d/`; `crt-static` variants were built in `target/phase-0-7d-crt-static/`. Static CRT removes the MSVC CRT imports at +186,880 bytes (eframe) and +160,256 bytes (Slint), but these variants were not launched.

### Remaining verification

No visible 1,000/10,000-entry pass, search/sort/multi-select keyboard pass, context/dialog/operation pass in eframe, DPI/resize/renderer fallback, Narrator/Inspect run, clean host, disconnected-network launch, or owner/license approval was completed. Keep the decision at **C — Selection remains blocked**. Do not begin Phase 1 or VF1.
