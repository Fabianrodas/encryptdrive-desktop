# Manual validation checklist

**Owner follow-up for the current build:** use [`OWNER_GUI_REVIEW.md`](OWNER_GUI_REVIEW.md) to directly launch both Phase 0.7E executables and record paired GUI, UIA, Narrator, scaling and offline outcomes. The current binaries have source/build checks only. Previous Phase 0.7D visual/UIA results below refer to earlier binaries and do not validate the current corrections.

Build prerequisites, source checks and updated Release builds are complete. Visible GUI validation is **PARTIAL**. The helper stopped on physical Escape; no further Windows app input was sent. Current performed/remaining items are separated below. Initial screenshots were displayed in the task session but not saved as files.

## Prepare

- [x] Install stable Rust 1.99.1 through official Rustup; verify host/target `x86_64-pc-windows-msvc`, components `rustfmt` and `clippy`, `rustc 1.99.1`, Cargo 1.99.1, rustfmt 1.10.0 and Clippy 0.1.99.
- [x] Add Desktop development with C++ and `Microsoft.VisualStudio.Component.VC.Tools.x86.x64` to the existing Visual Studio Community 2026 instance. Verified with `vswhere`; MSVC toolset `14.51.36231`, compiler `19.51.36260`, `cl.exe`, `link.exe`; existing SDK `10.0.19041.0` retained and not separately reinstalled. Initial redist 1622 error was recovered using process-only no-space `TEMP`/`TMP`.
- [x] Generate independent Cargo.lock files for common, eframe, and Slint; capture `cargo tree --locked --offline -e features --depth 1` and target-filtered Cargo metadata license inventory. Third-party license metadata is present; full license/legal review remains pending.
- [x] Finish locked `check`, strict `clippy`, `test`, and Release build commands for common and both GUIs. Current Phase 0.7E passes; common has 18 state tests, Slint has 1 size-format test, and eframe has 1 font-shaping test. Neither GUI has an interaction automation test.
- [x] Record updated Release executable byte lengths, SHA-256 and PE static imports in `PERFORMANCE_AND_BINARY_REPORT.md`. A bounded `crt-static` build/import experiment also passed; neither static-CRT variant was launched.

## GUI scenarios

- [x] Visible initial launch at the same default 100 entries for both apps; outer captures were 1180×792 including frame. Initial screenshots were rendered in the task UI but were not persisted.
- [x] Slint UIA row selection was observed across successive snapshots; one later tree marked one item selected. The immediate first tree was stale. Repeat after launching the updated binary.
- [x] Slint Create Folder dialog: empty submit produced “Enter a name”; a synthetic name submitted successfully and appeared in the list. No filesystem data was read or written.
- [x] Slint Dark appearance was visibly legible. Initial Slint Light standard controls had a contrast defect; palette/focus source corrections were built but NOT visibly retested. eframe Light showed tofu in an earlier build; system-font fallback and a shaping test were added in Phase 0.7E, but live rendering was NOT visibly retested.
- [ ] Repeat visible launch for each current Phase 0.7E app at fixtures 100, 1,000, 10,000; visible 100-entry launches were performed only on Phase 0.7D builds.
- [ ] Run search/sort, keyboard, Ctrl/Shift multi-selection, context actions, Trash, import/export and recovery in both updated apps. eframe dialogs and keyboard actions were not visibly exercised.
- [ ] Visually verify the corrected Slint Light palette and focused dialog input; verify operation success/cancel/failure, error/loading, DPI and resize in both updated apps.
- [ ] Run scenario 17 with Accessibility Insights for Windows or Inspect.exe, then Narrator. Capture role/name/selection/focus/progress/error evidence.
- [ ] Run scenario 18 offline from a clean Windows VM with no network and no developer toolchain installed. Record runtime prompts and loaded DLLs.
- [ ] Repeat cold and warm 10,000-entry list. Measure working set, CPU, first paint, search, sort and scroll latency using the same procedure for both apps.
- [ ] Save only genuine application screenshots and measurements under `results/evidence/` with metadata. Do not create illustrative screenshots and call them test results.
- [x] Prior Phase 0.7C hidden launch check for `--entries 100`, `1000`, and `10000`; all six windows were created. This is startup-only evidence, not visible list validation.

The live session stop message was: “Computer Use was stopped by the user with the physical Escape key.” The original builds remain open; do not send more GUI input in this stopped session. Inspect.exe is installed in the Windows SDK but was not launched. Do not mark the unchecked scenarios as source-verified passes.

## Approval gates

- [ ] Accessibility minimum is met and Narrator can reach/understand every required control and file row.
- [ ] Basic interactions, keyboard-only operation and stable selection pass.
- [ ] Both applications launch offline without runtime downloads.
- [ ] Slint license option, attribution placement, distribution rights, dependency licenses and cost are approved by product owner/legal reviewer.
- [ ] Modified timestamp/product semantics are resolved against the Java compatibility reference and approved Rust direction.
- [ ] Weighted scores use evidence from executed tests only; any critical issue is documented separately from score.

## Phase 0.7B prerequisite update — 2026-10-18

The inspected Windows host has Windows SDK 10.0.19041.0, but no Rust toolchain, MSVC compiler or MSVC linker. Install the latest stable Rust toolchain with x86_64-pc-windows-msvc and Visual Studio Build Tools with the Desktop development with C++ workload, retaining/selecting the installed SDK. This changes user/global development tools and requires owner authorization; no installation was performed here.

After setup, verify rustc --version, cargo --version, rustup show, where.exe cl and where.exe link. From common and then each prototype, run cargo fmt --check, cargo check, cargo clippy --all-targets -- -D warnings, cargo test and cargo build --release. In each app directory generate and retain its own Cargo.lock before rerunning the same checks with --locked. The shared fixture and the full 18-scenario UI script remain NOT RUN.

## Phase 0.7C pre-install status — historical, superseded later on 2026-10-18

Rustup setup, stable Rust 1.99.1, `rustfmt`, Clippy, the MSVC target, lockfiles, default-feature trees and package license metadata inventory are complete. The Visual Studio C++ modification remains incomplete; after its UAC/elevation step succeeds, verify `vswhere` reports the existing instance with `Microsoft.VisualStudio.Component.VC.Tools.x86.x64`, then confirm `cl.exe` and `link.exe` are present. Re-run all locked checks, tests and Release builds before beginning GUI scenarios. Do not treat current `cargo check` attempts as passing builds: eframe and Slint stopped at `link.exe`.

## Phase 0.7C authorized build follow-up — 2026-10-18

The authorized Rustup and existing-instance MSVC installations completed and were verified. All formatting, locked checks, strict Clippy, tests and Release builds passed. Static PE imports were recorded and each GUI process/window was verified for all three synthetic sizes. The visible GUI, input, Narrator/UIA, scaling, renderer fallback, clean-host packaging and disconnected-network checks above remain unchecked and NOT RUN. The one-step next action is an owner-performed manual UI pass using `../test-scenarios/GUI_SCENARIOS.md`; this is required because desktop UI automation was stopped by the user with Escape.
