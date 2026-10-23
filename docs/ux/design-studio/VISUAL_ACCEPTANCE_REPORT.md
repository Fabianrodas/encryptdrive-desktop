# Phase 1A-UX visual acceptance report

> **Superseded status:** UX3 is source-reviewed but still lacks live visual evidence. Use [UX3 final visual report](UX3_FINAL_VISUAL_REPORT.md) for the current status and [UX3 owner review guide](UX3_OWNER_REVIEW_GUIDE.md) to run the remaining review.

**Status: SOURCE CANDIDATE READY FOR OWNER REVIEW; VISUAL ACCEPTANCE NOT RUN.** The implemented egui preview builds successfully and has source-level and automated interaction evidence. It has not been launched, inspected visually, or exercised with UI Automation/Narrator in this turn. No screenshot or visual score is claimed.

## Scope and evidence boundary

The default application remains an honest no-vault engineering shell. The isolated `design-preview` feature plus `--design-preview` argument displays deterministic in-memory fixtures and illustrative account/vault screens. No real files, vaults, credentials, network requests, crypto, or persistence are used by the preview.

The desktop UI was not launched because the previous explicit physical-Escape stop remains in force. The task explicitly says not to circumvent that stop with another input route. Consequently, all observations in this report are limited to source, automated model/unit tests, Cargo builds, and a fixture/projection microbenchmark. The owner-assisted procedure is in [Owner Review Guide](OWNER_REVIEW_GUIDE.md).

**Screenshot evidence:** none captured; no screenshot files were generated or claimed.

## Implementation coverage

- Native eframe/egui shell with compact header, Files/Trash navigation, resizable sidebar, breadcrumbs, search, command area, virtualized table, settings, and semantic Light/Dark tokens.
- Feature-and-argument-gated design preview with 22 named representative states and deterministic 100, 1,000, and 10,000-entry fixtures.
- Stable entry IDs, nested and empty folders, Unicode and long names, synthetic file type/size/timestamps, Trash, search/sort, selection, dialogs, notices, and operation progress/cancel/failure states.
- Row shortcuts are scoped to table focus. Plain Delete in Files and Shift+Delete in Trash follow the documented route. The model preserves selection by stable IDs across query and sort changes.
- Destructive dialogs request Cancel focus and ask egui to return focus to the last focused row or Files navigation. This code path is not live-verified.
- Dialogs record their invoking widget for cancellation. After a confirmed Move to Trash or Permanent Delete removes rows, focus returns to the stable Files or Trash navigation tab instead of a deleted row.

Detailed implementation status and state coverage: [Product Visual Direction](PRODUCT_VISUAL_DIRECTION.md), [Screen and State Matrix](SCREEN_STATE_MATRIX.md), [Component Inventory](COMPONENT_INVENTORY.md), and [Interaction and Accessibility Review](INTERACTION_AND_ACCESSIBILITY_REVIEW.md).

## Final automated verification — 2026-10-19

| Check | Result | Evidence boundary |
|---|---|---|
| `cargo fmt --all -- --check` | PASS | Source formatting |
| `cargo check --workspace --locked` | PASS | Default feature set |
| `cargo test --workspace --locked` | PASS — 10 tests | 3 core + 7 desktop library tests |
| Preview-feature desktop tests | PASS — 25 tests | 7 library + 3 UI unit + 15 preview-model integration tests |
| Strict workspace Clippy | PASS | `--all-targets -- -D warnings` |
| Strict preview-feature Clippy | PASS | Desktop `design-preview`, all targets, warnings denied |
| Workspace Release build | PASS | Windows MSVC target |
| Preview Release build | PASS | Windows MSVC target |
| `scripts/verify-architecture.ps1` | PASS | Stable toolchain override used |
| Interactive application behavior | NOT RUN | Prior physical-Escape stop |

Cargo warned that hard links were unavailable for incremental cache files and copied them instead. All listed commands completed with exit code 0. One initial architecture-script attempt without the stable override hit a rustup temporary-file permission error; rerunning with `RUSTUP_TOOLCHAIN=stable` passed. No repository or global toolchain configuration was changed.

## Release artifacts and bounded model benchmark

Rust `1.99.1 (58cf8f9c3 2026-10-13)`, Cargo `1.99.1 (5f94df478 2026-08-27)`, target `x86_64-pc-windows-msvc`. Resolved GUI dependencies include eframe/egui 0.37.0, egui_extras 0.37.0, AccessKit 0.25.1, and wgpu 30.0.1.

| Build | Bytes | SHA-256 |
|---|---:|---|
| Default no-vault shell | 14,145,536 | `B23F2D449346AE54F0E7B5DCD8548B02B0618DF09EF7B5716EF361AF43D17D05` |
| `design-preview` build | 14,691,840 | `35BAF8B5877934C8A540D7C1C6E0F70F006C9C21118284B7A3CCBB6FC5C90424` |

The fixture/projection benchmark was run in Release mode. It measures deterministic fixture construction, one cold visible-ID projection, and 1,000 cached projection lookups; it does not measure UI render time, scrolling, memory, GPU use, or responsiveness.

| Entries | Fixture generation | Cold projection | 1,000 cached projections |
|---:|---:|---:|---:|
| 100 | 0.128 ms | 0.082 ms | 0.020 ms |
| 1,000 | 0.785 ms | 0.391 ms | 0.020 ms |
| 10,000 | 6.880 ms | 4.597 ms | 0.020 ms |

PE import inspection and clean-host/native-runtime dependency checks were **NOT RUN**; no `dumpbin`, `llvm-readobj`, `objdump`, or equivalent inspection tool was available. The executable size does not establish portable execution.

## Critic score and defect status

The independent critic's final source-only assessment found **0 P0, 0 P1, and 0 P2** and recommends the candidate for owner visual review. Its subtotal is **58/85 source-supported nonvisual points**; **overall professional visual quality (15 points) remains unverified** because there are no screenshots or live rendering observations. The subtotal is not normalized, does not meet the 90/100 target, and does not certify the rendered result. See [Critic Review](CRITIC_REVIEW.md) for category-level evidence limits.

The final critic review is being recorded after the last source corrections. Live visual, contrast, responsive layout, DPI, UIA, Narrator, dialog focus, and owner preference checks remain separate approval gates. The numerical source score cannot substitute for these requirements or the owner's 90/100 target.

## Required owner review

1. Build and launch the preview using [Owner Review Guide](OWNER_REVIEW_GUIDE.md) from the repository root. Use only sample data.
2. Review 100/1,000/10,000 entries, 960×640 through 1920×1080, both themes, resize, search/sort, selection, actions, dialogs, and progress states.
3. Run the keyboard-only flows, inspect Windows UI Automation, then test with Narrator. Record observed role/name/state and actual spoken output.
4. Verify display/text scaling and High Contrast behavior on an available Windows configuration.
5. Capture genuine synthetic screenshots and ask the independent critic for a rendered-evidence review. Do not call this visual candidate approved until the owner accepts it.

No Phase 2, vault format, cryptography, or real vault/file operations were started.
