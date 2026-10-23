# DPI and rendering report

**Phase 0.7D build/window observation: PASS; current Phase 0.7E visual comparison: NOT RUN; DPI/resize/renderer fallback: NOT RUN.** Earlier Release applications were opened visibly and captured at the same 1180×792 outer-window image dimensions. Those captures predate the current source corrections. No 100–200% scale change, live resize, renderer fallback, or GPU compatibility test was run. Repeat with the current binaries using [`OWNER_GUI_REVIEW.md`](OWNER_GUI_REVIEW.md).

## Source configuration (not runtime evidence)

- Both app windows request an initial size of 1180×760 logical pixels and a minimum of 780×520. eframe uses its viewport size; Slint uses `preferred-width` / `preferred-height` plus the minimum constraints.
- eframe explicitly selects its WGPU renderer. The eframe feature set contains WGPU and AccessKit; there is no automatic proof of fallback behavior in this spike.
- Slint uses `backend-winit`, `renderer-femtovg`, `renderer-software`, and accessibility features. The intended default path is FemtoVG/OpenGL under Winit; software renderer availability is included but fallback selection has not been exercised.
- Both UIs use shared semantic token values for Light and Dark. Screenshots and color/contrast checks were not captured.

Phase 0.7E retains the same renderer configuration. The Slint row focus border now consumes a dedicated semantic focus-ring property, while eframe enables native system-font fallback. Both source corrections compile; neither current executable has been visibly launched for Light/Dark, CJK glyph, resize or scale inspection. Renderer fallback and DPI results remain **NOT RUN**.

Run GUI scenarios 5, 13, 16 and 18 in `../test-scenarios/GUI_SCENARIOS.md`. Include Windows display scale, GPU/driver, selected renderer, live resize behavior and screenshots at 100%, 125%, 150% and 200%. Test unsupported or remote desktop graphics contexts and document whether a usable software fallback exists. Do not award DPI or Rendering Compatibility points before those runs.

## Phase 0.7B execution update — 2026-10-18

The Windows SDK 10.0.19041.0 is installed, but neither prototype could be compiled or launched because Rust/Cargo and MSVC cl/link are absent. Resizing, display scaling, GPU/driver selection, renderer fallback and screenshots remain NOT RUN. The SDK presence is not a rendering result.

## Phase 0.7C pre-install snapshot — historical, superseded later on 2026-10-18

The Rust toolchain and Cargo dependency locks are now available, but eframe and Slint checks stop because Visual Studio's x86/x64 MSVC linker is still absent. The existing SDK 10.0.19041.0 was left selected as-is. No GUI launched, no screenshot was captured, and resize, display scaling, GPU/driver selection, renderer fallback and 100/125/150/200% DPI scenarios remain NOT RUN.

## Phase 0.7C authorized build follow-up — 2026-10-18

Both release executables were launched hidden with `--entries 100`, `1000`, and `10000`; all six created a non-zero native window handle and remained alive at the 1.5-second check. This confirms only window/process creation, not rendering or window behavior. No visual control was performed after the user stopped desktop automation with Escape.

| Scenario | eframe | Slint | Result |
|---|---|---|---|
| Initial logical size/minimum constraints compile | PASS | PASS | PASS |
| Visible content, scroll and resize | NOT RUN | NOT RUN | NOT RUN |
| DPI 100%, 125%, 150%, 200% | NOT RUN | NOT RUN | NOT RUN |
| GPU/driver choice and renderer fallback | NOT RUN | NOT RUN | NOT RUN |
| Remote desktop/unsupported renderer behavior | NOT RUN | NOT RUN | NOT RUN |

The configured Windows SDK `10.0.19041.0` and MSVC `14.51.36231` toolset built both binaries. No rendering score is awarded from successful compilation or native window creation.

## Phase 0.7D visible rendering update — 2026-10-18

- Both visible window captures were made at 1180×792 pixels including the outer frame; source defaults are 1180×760 logical client dimensions. The screenshots were displayed during the desktop session but were not persisted under `results/`.
- eframe's initial Light explorer had legible table/header/toolbar text and a dense file list. The Japanese characters in `résumé — 東京.txt` appeared as missing-glyph boxes. No font fallback correction was attempted because no GUI recheck remained available after the user stopped Computer Use.
- Slint's initial Light appearance showed near-white text on standard buttons, headers, and input controls against light surfaces. Switching Slint to Dark produced a coherent, legible dark appearance in the live view. The confirmed cause was a mismatch between the custom app token mode and the Slint standard widget `Palette.color-scheme`, which still followed the Windows dark scheme.
- Slint now binds the widget palette mode to the same Light/Dark appearance as the prototype tokens. The updated app passes `cargo check` and the Release build, but the Light correction has not been visibly retested.
- The computer-use helper reported a physical Escape stop; no further input was sent. Window resizing, min-size layout, popup placement, focus visibility after the correction, DPI at 100/125/150/200%, GPU selection, and FemtoVG/software renderer fallback remain NOT RUN.
