# Dependency and license review

**Phase 0.7E dependency resolution, source checks, Release builds and current PE inspection: PASS; clean-host execution and license approval: OPEN.** The current eframe graph enables native system-font fallback. The refreshed Windows-targeted metadata inventory has 159 registry packages for eframe and 320 for Slint, and every registry package has `license` or `license_file` metadata. Current binary hashes, sizes and imports are recorded in the performance report. Historical static-CRT link/import evidence below applies to Phase 0.7D sources only; neither current nor historical static variants were clean-host/offline tested. Cargo metadata is not a full license audit, legal determination, or owner approval. Do not choose a Slint license from this spike.

| Prototype | Direct GUI pins | Configured feature intent | Lockfile | Runtime/dependency evidence |
|---|---|---|---|---|
| eframe | eframe 0.37.0, egui_extras 0.37.0; `egui_system_font_provider` is active through `system_fonts` and directly used by the Unicode shaping test | WGPU, AccessKit, default fonts, native system-font fallback | Generated with `--locked`; 365 lock entries; 159 registry packages / 161 metadata records including app + common | Phase 0.7E Release binary and PE imports are reported in `PERFORMANCE_AND_BINARY_REPORT.md`; current GUI launch/offline behavior NOT RUN |
| Slint | slint 1.18.1, slint-build 1.18.1; both declare `GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0` | Winit, FemtoVG, software renderer feature, accessibility; `compat-1-18` | Generated with `--locked`; 538 lock entries; 320 registry packages / 322 metadata records including app + common | Phase 0.7E Release binary and PE imports are reported in `PERFORMANCE_AND_BINARY_REPORT.md`; current GUI launch/offline behavior NOT RUN |
| Shared crate | local `encryptdrive-spike-common` | no GUI dependencies | Generated; no external dependencies | Source and checks only |

The manifests use edition 2024 and declare Rust 1.95 minimum. Stable Rust 1.99.1 is installed. `cargo tree --locked --offline --edges features --depth 1` confirmed the selected direct eframe features (AccessKit, default fonts, `system_fonts`, WGPU) and Slint features (accessibility, Winit, FemtoVG, software renderer and std); Slint build uses `slint-build` defaults. The refreshed CSV inventory contains 484 package records across the three metadata graphs, including 159 eframe and 320 Slint registry records; all registry records have license or license-file metadata. Each app graph additionally includes its local app and shared common package, neither of which declares package license metadata. Metadata presence does not validate license texts, compatibility, notices or distribution obligations.

Evidence file: [`DEPENDENCY_LICENSE_INVENTORY.csv`](DEPENDENCY_LICENSE_INVENTORY.csv). The independent Cargo.lock files are in `../common/`, `../egui-prototype/`, and `../slint-prototype/`.

Official technical references reviewed: [eframe crate documentation](https://docs.rs/crate/eframe/latest), [egui_extras TableBuilder](https://docs.rs/egui_extras/latest/egui_extras/struct.TableBuilder.html), [Slint releases and MSRV notes](https://github.com/slint-ui/slint/releases), [Slint crate features](https://docs.rs/crate/slint/latest/features), [Slint ListView](https://docs.slint.dev/latest/docs/slint/reference/std-widgets/views/listview/), and [Slint StandardTableView](https://docs.slint.dev/latest/docs/slint/reference/std-widgets/views/standardtableview/).

## Slint licensing owner gate

The Slint crate metadata names three options. The current royalty-free desktop/mobile/web license terms say distribution requires either an `AboutSlint` widget reachable from the top-level menu (or splash screen if there is no About screen), or a visible attribution badge on a public page; they exclude embedded use. GPLv3 carries copyleft obligations. Paid commercial plans have separate eligibility, subscription, support and fallback-license terms. There is a material inconsistency to resolve: the current [pricing comparison table](https://slint.dev/pricing) labels “Attribution to Slint is optional,” while the linked [Royalty-free License 2.0 terms](https://slint.dev/terms-and-conditions) state those two attribution routes as distribution conditions. Product owner/legal must confirm which license/version and terms control and document the required attribution policy before a product choice.

- GPLv3: copyleft obligations apply; determine whether it fits EncryptDrive distribution and linking architecture.
- Royalty-free Desktop/Mobile/Web option: the official terms describe an attribution requirement (AboutSlint entry in the top-level menu or a badge on the public binary download page) and exclude embedded use; confirm product distribution and attribution placement with the owner.
- Paid commercial option: review the current vendor license, developer/test seat terms, supported platforms, distribution rights, support and cost with the product owner/legal counsel.

This document records options only. No license has been selected, and no statement that Slint is acceptable for the eventual product is made. Re-check current official terms at the approval gate because pricing and terms can change. All 159 eframe and 320 Slint registry packages expose license metadata, but eframe/egui direct crate declarations and every resolved transitive license still require license-text, notice and compatibility review; this report does not certify them.

The official Slint sources reviewed for licensing are the [FAQ/license overview](https://slint.dev/faqs) and [Terms and Conditions](https://slint.dev/terms-and-conditions). Terms may change; legal review should use the terms in effect for the intended distribution.

## Runtime note — initial pre-build state (superseded by follow-up below)

The current source selects WGPU for eframe and Winit/FemtoVG with a software renderer feature for Slint. Actual DLLs, VC runtime need, OpenGL availability, fallback activation and offline portable execution are unknown without Release builds and clean-machine runs. Record runtime files, licenses/notices and installer implications before framework selection.

## Phase 0.7B toolchain update — 2026-10-18

The app manifests now declare Rust 1.95 minimum. No Rust/Cargo version is installed or discoverable, and no lockfiles were generated. Exact direct crate pins remain eframe/egui_extras 0.37.0 and Slint/slint-build 1.18.1. These are not resolved dependency graphs: transitive versions, features after resolution, licenses, notices, and native runtime components are all NOT RUN / BLOCKED. The installed Windows SDK is 10.0.19041.0; MSVC compiler and linker are missing. No package or installer feasibility conclusion is supported by this environment.

The prior statement that Rust 1.92 was sufficient for eframe was incorrect. The current requirement is Rust 1.95 or later. The installed version cannot be reported. Do not install or update a global toolchain without owner authorization.
## Static API reference spot-check — not a build result

The eframe startup uses the documented 0.37.0 run_native AppCreator result closure, NativeOptions viewport/renderer fields, and Renderer::Wgpu. The table uses TableBuilder and TableBody::rows, which the 0.37.0 egui_extras documentation describes as rendering visible rows for large tables. The Slint source uses the documented 1.18.1 build.rs compile plus include_modules! flow and Timer::start on the event-loop thread. These are narrow reference/API shape checks only; they do not validate Rust types, generated callback/property names, the full .slint file, linking or runtime behavior.

References: [eframe 0.37.0 run_native](https://docs.rs/eframe/0.37.0/eframe/fn.run_native.html), [eframe 0.37.0 NativeOptions](https://docs.rs/eframe/0.37.0/eframe/struct.NativeOptions.html), [eframe 0.37.0 Renderer](https://docs.rs/eframe/0.37.0/eframe/enum.Renderer.html), [egui_extras 0.37.0 TableBody](https://docs.rs/egui_extras/0.37.0/egui_extras/struct.TableBody.html), [Slint 1.18.1 crate guide](https://docs.rs/slint/1.18.1/slint/), [Slint 1.18.1 Timer](https://docs.rs/slint/1.18.1/slint/struct.Timer.html), and [slint-build 1.18.1](https://docs.rs/slint-build/1.18.1/slint_build/).

## Phase 0.7C pre-install verification snapshot — historical, superseded later on 2026-10-18

`cargo generate-lockfile` succeeded for the common crate and both independent app crates. Resolved GUI manifests remain pinned at eframe/egui_extras 0.37.0 and Slint/slint-build 1.18.1. Cargo metadata inventory was generated offline for `x86_64-pc-windows-msvc`; third-party license metadata is present for every selected crates.io package. No full license text scan or approval was performed. The installed toolchain is Rust/Cargo 1.99.1. Both Windows GUI dependency graphs have not passed compilation because `link.exe` is missing. Native DLL, Visual C++ redistributable, GPU/software fallback, binary licensing notices and offline deployment implications remain NOT RUN.

## Phase 0.7C authorized build and binary follow-up — 2026-10-18

Both app lockfiles and the common lockfile were consumed with `--locked`. Exact prototype versions: eframe/egui_extras `0.37.0`; Slint/slint-build `1.18.1`. The Windows target-filtered graph contains 152 crates.io packages for eframe and 320 for Slint; every third-party package record has `license` or `license-file` metadata. This does not validate the license text or compatibility of every transitive dependency. The Rustup version/hash, PE hashes, and byte lengths are recorded in `BUILD_AND_RUN_REPORT.md` and `PERFORMANCE_AND_BINARY_REPORT.md`.

`dumpbin /DEPENDENTS` found `VCRUNTIME140.dll` and Windows API-set CRT imports in both EXEs, so a clean deployment host may need the matching Microsoft Visual C++ Redistributable. The eframe EXE imports DXGI and OpenGL; Slint imports DirectWrite and OpenGL; both import UI Automation Core. These are static PE import tables, not the full runtime module set. Both apps created native windows on the configured development host; disconnected-network and clean-host runs remain NOT RUN. Runtime packaging feasibility is therefore partial, not passed.

No Slint license option was selected. The owner/legal gate remains open for GPLv3 compatibility, the royalty-free option's attribution condition, paid commercial terms, distribution rights, costs, and attribution placement. eframe direct crates declare MIT OR Apache-2.0, but transitive license compatibility and notice obligations also require review. See the current official Slint terms and pricing linked above; re-check them at approval time.

## Phase 0.7D CRT/link and launch-status update — 2026-10-18

The ordinary updated binaries in `target/phase-0-7d/release/` continue to import `VCRUNTIME140.dll` and the `api-ms-win-crt-*` DLLs. A bounded Release build experiment set process-only `RUSTFLAGS="-C target-feature=+crt-static"` and wrote to `target/phase-0-7d-crt-static/release/`. `dumpbin /DEPENDENTS` confirms that both static-CRT executables omit the VCRUNTIME and CRT API-set imports. The eframe binary grew by 186,880 bytes (1.36%); Slint grew by 160,256 bytes (1.02%). Other Windows system imports remain, including WGPU/OpenGL/DXGI-related imports for eframe, DirectWrite/OpenGL for Slint, and UI Automation Core for both.

This demonstrates a viable build-time CRT option, but not portable execution: the static variants were not launched after the user stopped Computer Use, and no clean-host or offline test was run. Before using `crt-static` for release, launch both variants, run the visible interactions, check UIA and graphics, and test packaging. Microsoft runtime redistribution and notices remain owner review items for the ordinary dynamic-CRT binaries.

License review is unchanged: no Slint option is selected; GPLv3, royalty-free attribution conditions, paid commercial terms, distribution scope and cost require owner/legal review. The dependency metadata inventory does not replace full license-text and notice review.
