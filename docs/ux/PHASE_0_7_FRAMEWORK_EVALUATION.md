# Phase 0.7 — Native Rust GUI framework evaluation

**Phase 0.7E status: source/test checks and current Release rebuilds PASS; current GUI validation NOT RUN. Recommendation C — no framework selection.** Rust 1.99.1/MSVC remains verified. The shared model has 18 passing tests; eframe has one passing Windows system-font shaping test; Slint has one passing size-format test. Current eframe font-fallback and Slint focus-token corrections have not been visually checked. Previous visible/UIA evidence applies to Phase 0.7D binaries only. Keyboard interactions, full dialog/operation behavior, Narrator, DPI, clean-host/offline execution and owner/legal approval remain open. Phase 1 is not authorized.

## Evidence

- Spike overview and launch/build commands: [`spikes/phase-0-7-gui/README.md`](../../spikes/phase-0-7-gui/README.md)
- Build environment and attempted command outcomes: [`BUILD_AND_RUN_REPORT.md`](../../spikes/phase-0-7-gui/results/BUILD_AND_RUN_REPORT.md)
- Shared GUI scenario script: [`GUI_SCENARIOS.md`](../../spikes/phase-0-7-gui/test-scenarios/GUI_SCENARIOS.md)
- Functional execution matrix: [`FUNCTIONAL_TEST_MATRIX.md`](../../spikes/phase-0-7-gui/results/FUNCTIONAL_TEST_MATRIX.md)
- Accessibility and Narrator status: [`ACCESSIBILITY_REPORT.md`](../../spikes/phase-0-7-gui/results/ACCESSIBILITY_REPORT.md)
- DPI and renderer status: [`DPI_RENDERING_REPORT.md`](../../spikes/phase-0-7-gui/results/DPI_RENDERING_REPORT.md)
- Performance, executable size and runtime status: [`PERFORMANCE_AND_BINARY_REPORT.md`](../../spikes/phase-0-7-gui/results/PERFORMANCE_AND_BINARY_REPORT.md)
- Dependency and Slint license options: [`DEPENDENCY_AND_LICENSE_REVIEW.md`](../../spikes/phase-0-7-gui/results/DEPENDENCY_AND_LICENSE_REVIEW.md)
- Weighted score and critical gates: [`FRAMEWORK_COMPARISON_RESULT.md`](../../spikes/phase-0-7-gui/results/FRAMEWORK_COMPARISON_RESULT.md)
- Reproducible manual follow-up: [`MANUAL_VALIDATION_CHECKLIST.md`](../../spikes/phase-0-7-gui/results/MANUAL_VALIDATION_CHECKLIST.md)
- Direct launch and paired owner run sheet for the current binaries: [`OWNER_GUI_REVIEW.md`](../../spikes/phase-0-7-gui/results/OWNER_GUI_REVIEW.md)

## Current evaluation outcome

The comparison report retains the last Phase 0.7D partial evidence totals (21/100 each; 79 unevaluated) as historical; it does not claim them as scores for the current binaries. Mandatory gates remain open and no framework is recommended. The next verification is the paired owner GUI review of the Phase 0.7E Release binaries, followed by UIA/Narrator, disconnected clean-host testing and owner/legal review.

## Phase 0.7D visible execution — 2026-10-18

Both original Release binaries were visibly launched at the same 100-entry default and captured at matching 1180×792 outer-window dimensions. Slint row selection and Create Folder empty validation/synthetic creation were exercised. UIA trees exposed named controls and rows; Slint exposed selectable list items and selected state. Initial Slint Light controls had very low contrast; Dark was legible. A source correction synchronizes the Slint standard-widget palette with Light/Dark mode, but it was not visibly retested. eframe's Japanese fixture characters rendered as tofu. See the Functional Test Matrix, Accessibility Report and DPI Rendering Report for specific evidence.

The computer-use helper stopped with “Computer Use was stopped by the user with the physical Escape key.” No further app input was sent. Updated dynamic-CRT and `crt-static` Release builds pass; static variants remove VCRUNTIME imports at a small binary-size increase but were not launched. Clean-host/offline, DPI, Narrator and full keyboard behavior remain NOT RUN. Recommendation: **C — Selection remains blocked** until mandatory checks and owner/legal approval complete.

## Recorded discrepancies and approval gate

The prototype follows the current Phase 0.7 request by displaying synthetic Modified values and allowing sorting. The reviewed Java code and approved Rust baseline indicate that Modified is not a reliable maintained field and that the Java table does not sort. These prototype features are comparison affordances only; resolve the product behavior before implementation.

Slint licensing remains a product-owner/legal decision. Review current GPLv3, attribution-based royalty-free and paid commercial options, costs, distribution rights, notices and attribution location before selecting Slint for any distributable product. The report lists the current official terms reviewed for this spike; re-check them at the approval gate.

The nine existing Phase 0.5 UX documents, private documentation repository and Java reference repository were left untouched. The experiment remains under `spikes/phase-0-7-gui/`; it is not integrated into the production application architecture. Phase 1 and Vault Format 1 are out of scope and were not started.

## Phase 0.7B status — 2026-10-18

The current-user Cargo/Rustup directories and all searched alternate Rust locations are absent; PATH and environment checks found no Rust/Cargo. The MSVC compiler and linker are absent. Windows SDK 10.0.19041.0 is installed. All 15 fresh Cargo format/check/Clippy/test/release-build commands across common, eframe and Slint were BLOCKED before execution. No lockfiles, executables, screenshots, GUI runs, Narrator/UI Automation observations or measurements exist.

The earlier Rust 1.92 recommendation was incorrect for eframe 0.37.0. The three Cargo manifests and launch instructions now require Rust 1.95 or later. No global tool installation was performed. All framework scores remain N/A, no candidate is recommended, and Phase 1 remains out of scope.

Modified compatibility correction: VF1 stores the timestamp and Java displays it, but Java initializes modifiedAt equal to createdAt, does not update it in the inspected operations, and explicitly disables sorting on its Files/Trash columns. The spike's deterministic Modified sort is test-only and should not be carried into the product without a separate owner-approved timestamp meaning.

Evidence:
- [Build and run report](../../spikes/phase-0-7-gui/results/BUILD_AND_RUN_REPORT.md)
- [Functional test matrix](../../spikes/phase-0-7-gui/results/FUNCTIONAL_TEST_MATRIX.md)
- [Accessibility report](../../spikes/phase-0-7-gui/results/ACCESSIBILITY_REPORT.md)
- [DPI and rendering report](../../spikes/phase-0-7-gui/results/DPI_RENDERING_REPORT.md)
- [Performance and binary report](../../spikes/phase-0-7-gui/results/PERFORMANCE_AND_BINARY_REPORT.md)
- [Dependency and license review](../../spikes/phase-0-7-gui/results/DEPENDENCY_AND_LICENSE_REVIEW.md)
- [Framework comparison](../../spikes/phase-0-7-gui/results/FRAMEWORK_COMPARISON_RESULT.md)
- [Manual validation checklist](../../spikes/phase-0-7-gui/results/MANUAL_VALIDATION_CHECKLIST.md)

## Phase 0.7C pre-install status — historical, superseded later on 2026-10-18

- Rustup 1.29.1 installed stable Rust/Cargo 1.99.1, `rustfmt`, Clippy, and `x86_64-pc-windows-msvc`.
- `cargo fmt --check` passed for common and both GUI crates. The common crate passed `cargo check --locked` and strict Clippy. Two fixture lint findings were corrected and the Clippy rerun passed.
- Cargo generated independent lockfiles for common, eframe and Slint. The eframe and Slint target-filtered dependency graphs contain 152 and 320 crates.io package records, respectively; every third-party record has license or license-file metadata. This is metadata evidence, not legal approval.
- Both GUI `cargo check` runs and common `cargo test` stopped because `link.exe` is unavailable. The authorized modification of the existing Visual Studio Community 2026 instance failed at its x86 VC redistributable step with Windows Installer 1622 (unquoted temp path containing a space). A process-only no-space-temp retry remains pending elevation; no `cl.exe` or `link.exe` has appeared.
- All GUI scenarios, DPI/rendering, release builds, screenshots, binary measurements, UIA/Narrator and offline launch remain NOT RUN. The Computer Use helper was stopped by the user with Escape; this turn performed no further UI inspection.
- **Recommendation:** none. Finish the authorized MSVC workload installation, then rerun compile, tests and the shared manual matrix. Slint's license option remains an owner/legal gate. Phase 1 and Vault Format 1 were not started.

## Phase 0.7C authorized toolchain and execution follow-up — 2026-10-18

- Rustup `1.29.1`; Rust/Cargo `1.99.1`; rustfmt `1.10.0`; Clippy `0.1.99`; target `x86_64-pc-windows-msvc`.
- Added the NativeDesktop workload and x64/x86 MSVC tools to the existing Visual Studio Community 2026 instance. Verified `cl.exe`, `link.exe`, toolset `14.51.36231`, compiler `19.51.36260`; existing SDK `10.0.19041.0` retained.
- Format, locked check, strict Clippy and tests passed for common, eframe and Slint. Common: 8/8 scenario tests passed; each GUI test target: 0 tests. Both Release builds passed.
- Hidden process launch checks for each prototype at 100, 1,000 and 10,000 entries all remained alive and produced a native window handle. Binary hashes/sizes, launch memory snapshots and PE static imports are in the performance report.
- No screenshot or app-control interaction was recorded. The user stopped desktop UI automation with Escape; keyboard, selection gestures, dialogs, context menus, themes, DPI, UIA/Narrator, rendering fallback and disconnected-network checks remain NOT RUN.
- **Recommendation:** none. The remaining gate is visible paired GUI validation plus UIA/Narrator, offline clean-host launch and owner/legal dependency/license approval. No Phase 1 or Vault Format 1 work began.
