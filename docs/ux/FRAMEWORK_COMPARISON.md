# EncryptDrive Desktop — GUI Framework Comparison Protocol

**Status:** Protocol prepared; comparison not run; no framework selected  
**Candidates:** egui/eframe and Slint  
**Reason for second candidate:** Slint is the strongest bounded alternative to compare against eframe for this product because it is a Rust-capable desktop UI toolkit with explicit accessibility properties and no WebView frontend. This head-to-head tests the tradeoff between egui’s flexible custom-painted UI and Slint’s declarative desktop UI. Tauri remains a documented alternative from Phase 0, but is outside this two-candidate Phase 0.5 comparison.

The pair follows the product criteria rather than implementation convenience: both keep the UI in a native Rust application without a WebView frontend; eframe offers custom-painted flexibility and an MIT OR Apache-2.0 upstream license; Slint offers a declarative UI model and explicit accessibility properties, with a separate license/attribution decision. Tauri is not the bounded alternative because its web frontend and WebView runtime add a different security and packaging boundary. Iced is not selected for this spike because the Phase 0 research found its maturity and Windows screen-reader support insufficiently established for the current release gate.

| Comparison dimension | egui/eframe | Slint |
|---|---|---|
| UI model | Custom-painted immediate-mode widgets; theme and table behavior need deliberate product styling. | Declarative UI language and component model; confirm the control semantics needed for a file table. |
| Accessibility evidence | AccessKit tree is exposed by eframe on native platforms where an adapter is available. | Accessible role/name/landmark properties are available; validate the resulting Windows Narrator behavior. |
| Renderer and runtime | wgpu is the default; Glow is an option. Test device compatibility and fallback. | Renderer and build backend affect packaging; test the chosen configuration, including MSVC/Skia runtime dependencies if used. |
| License starting point | MIT OR Apache-2.0 upstream; audit all selected dependencies. | GPLv3, royalty-free attribution, or paid terms depending on distribution choice. Owner must approve the exact option. |
| Main question | Can custom-drawn controls feel and behave like a polished Windows file manager while preserving accessibility? | Can the declarative toolkit deliver the same density, table behavior, and packaging without unacceptable licensing or runtime cost? |

## Evidence checked

Official framework sources checked on 2026-10-17:

- eframe builds native Windows/macOS/Linux apps; its official README describes wgpu as the default renderer and Glow as an option: [eframe README](https://github.com/emilk/egui/blob/main/crates/eframe/README.md).
- The current egui/eframe workspace declares MIT OR Apache-2.0: [workspace manifest](https://github.com/emilk/egui/blob/main/Cargo.toml). Audit each selected crate and transitive dependency at the pinned release, rather than assuming the workspace license covers the full application.
- eframe enables AccessKit on native platforms where an adapter is available; built-in widgets expose roles/names and egui_kittest can inspect the accessibility tree: [egui accessibility guide](https://github.com/emilk/egui/blob/main/docs/accessibility.md).
- Slint exposes accessible roles, names, and landmarks through accessibility properties: [Slint accessibility reference](https://docs.slint.dev/latest/docs/slint/reference/common/).
- Slint supports Windows, macOS, Linux, and other targets from a shared UI codebase: [Slint overview](https://docs.slint.dev/latest/docs/slint/).
- Slint’s license options include GPLv3, a royalty-free desktop/mobile/web license with attribution, and paid commercial terms: [official Slint FAQ](https://slint.dev/faqs), [license terms](https://slint.dev/terms-and-conditions).
- Slint’s Windows packaging guide notes that MSVC builds use the Visual C++ runtime and the Skia renderer adds the C++ standard library: [Slint Windows packaging](https://docs.slint.dev/latest/docs/slint/guide/platforms/desktop/windows/packaging/).

These facts establish the comparison criteria, not that either framework passes. Recheck versions, licenses, dependencies, and current documentation when the comparison is approved. eframe renderer fallback and Slint renderer/runtime choices must be determined empirically.

## Bounded comparison scope

The eventual spike evaluates one representative Main File Explorer screen only. It does not implement vault creation, authentication, crypto, persistence, file import/export, or product workflows. It uses synthetic, non-sensitive rows and in-memory state only. The spike must not access any vault or real file content.

The spike is not part of Phase 0.5 and must wait until the owner reviews the planning package. It must not become the Rust app skeleton or a full prototype.

## Identical screen and test data

Both candidates render the same 960×640 logical-pixel window at the same operating-system scale, with the same tokens, font policy, icons, labels, spacing, and accessibility descriptions.

The screen contains:

- Native window title bar, app menu, Files/Trash sidebar, and active Files state.
- Back/Up controls, breadcrumb, name search, New folder, Import, Export, and overflow.
- File table with Name, Type, and Size.
- Optional Details pane in its closed and open states.
- Selection, hover, focus, empty folder, search results, and an in-window operation-progress state.
- A synthetic dataset with a nested folder, representative file names/types/sizes, a long filename, and an empty folder. No real user or vault data.

No framework may use more content, animation, or visual effects than the other.

## Identical interaction task script

1. Navigate keyboard focus to search and find a synthetic file by name.
2. Move from a result to its parent folder and confirm breadcrumb/current-location behavior.
3. Navigate the table with arrows, open a folder with Enter, and return with Alt+Up.
4. Open and dismiss a context menu using Shift+F10 and Escape.
5. Open/close the Details pane using keyboard and mouse, then verify focus and screen-reader order.
6. Resize the window and move it between displays with different scaling.
7. Read the same UI using Windows Narrator and inspect the UI Automation/accessibility tree.
8. Launch the packaged ZIP with the network disconnected on a clean supported Windows image.

The spike simulates progress in memory. It does not read or write any vault data.

## Test environment and evidence

At minimum, capture:

- Windows baseline and architecture selected by the owner.
- 100%, 150%, and 200% display scaling; one mixed-DPI monitor move if hardware is available.
- Keyboard-only execution of every step.
- Windows Narrator reading order, names, roles, selected state, and progress announcements.
- Accessibility tree export/snapshot.
- Screenshot at the same window size and state for each candidate, in light and dark themes.
- Renderer selection, startup failures, fallback behavior, and graphics-device/runtime assumptions.
- ZIP contents, uncompressed size, imported runtime dependencies, and offline cold start.
- Installer build/install/uninstall behavior and required runtime disclosures.
- Dependency versions, license texts, feature flags, last stable release, upstream release activity, Windows CI coverage, MSRV, unresolved critical issues, and known API stability constraints at the selected versions.

For eframe, compare the selected wgpu configuration with a Glow option if the default cannot meet the GPU/driver target; do not assume the alternate renderer works as a software fallback. For Slint, record the selected renderer and whether Visual C++ runtime libraries are bundled or required. Use a clean machine/VM to verify, not only a developer workstation.

## Scoring rubric

Score each category from 0 to 4: 0 = blocked/unusable; 1 = major defect; 2 = partial or inconsistent; 3 = meets the criterion with manageable gaps; 4 = strong and independently verified. Weighted points equal score ÷ 4 × category weight.

| Category | Weight | Minimum evidence |
|---|---:|---|
| Visual fidelity and information density | 15 | Same layout/tokens; readable file rows; no clipping at test size. |
| Keyboard navigation and interaction | 15 | All scripted actions work with visible focus and predictable focus return. |
| Screen-reader/UI Automation support | 18 | Names, roles, reading order, selection, status, and progress are usable with Narrator. |
| DPI, resizing, and native window behavior | 12 | No overlap/clipping at 100/150/200%; correct title bar and resize behavior. |
| GPU compatibility and fallback | 10 | Startup and rendering behavior understood on supported graphics baseline; fallback is documented and tested. |
| Portable ZIP and offline operation | 10 | Cold start from ZIP with network disconnected; all required runtime files accounted for. |
| Installer packaging | 6 | Install/uninstall works and runtime prerequisites are disclosed/bundled. |
| License and dependency posture | 6 | Framework and dependency license inventory is complete at pinned versions. |
| Maintainability and cross-platform path | 8 | UI remains separable from Rust core; update cadence, API stability, Windows/macOS/Linux path documented. |
| **Total** | **100** | |

## Decision gates

A candidate is ineligible if it:

- Cannot complete the required flows using only keyboard.
- Cannot expose critical controls/state to Windows Narrator or the accessibility tree.
- Leaks synthetic screen contents into OS notifications or unexpected logs.
- Requires network access to launch in the approved offline distribution model.
- Has an undisclosed runtime or license condition unacceptable to the owner.
- Fails at an owner-approved DPI or Windows baseline.

Among eligible candidates, the preferred result is the higher total score, provided no accessibility category is below 3/4 and no other category is below 2/4. If scores are within 5 points, compare screenshot review, packaging size/runtime, licensing, and maintenance risk; the owner makes the final selection. Do not select on easiest implementation alone.

## Results record template

When run later, append a dated section containing candidate version/commit, Cargo features, renderer, test machine/OS, per-category score and evidence, package/runtime observations, license inventory, failures, and owner decision. Keep the synthetic app and data isolated; do not copy the spike into product code without separate review.

## Protocol acceptance

- Both candidates receive the same screen, text, sample rows, assets, and task script.
- Test dimensions and DPI states are identical.
- Accessibility and offline/packaging are release gates, not optional score bonuses.
- No actual framework result is claimed in Phase 0.5.

