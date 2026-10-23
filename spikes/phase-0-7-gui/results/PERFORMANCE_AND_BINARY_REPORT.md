# Performance and binary report

**Phase 0.7E Release builds and PE inspection: PASS. Current interactive performance measurements: NOT RUN.** Current binaries were built under a separate target root and inspected with `dumpbin /DEPENDENTS`; they were not launched or interacted with. Earlier memory snapshots remain preliminary Phase 0.7C shell-launch evidence.

## Phase 0.7E current Release artifacts — 2026-10-18

| Prototype | Executable | Bytes | MiB (binary) | SHA-256 |
|---|---|---:|---:|---|
| eframe | `target/phase-0-7e/release/encryptdrive-eframe-spike.exe` | 14,498,816 | 13.83 | `0C92A52585FBBAEAFFE3D010A590CB51DE4A6AD1376160B9D063B61D6FD91E34` |
| Slint | `target/phase-0-7e/release/encryptdrive-slint-prototype.exe` | 15,790,592 | 15.06 | `36418729B502457AE29B89C46111EC56CA9158C0D251521300624FFA4DAFF704` |

`dumpbin /DEPENDENTS` on both current executables reports direct imports of `VCRUNTIME140.dll` and Windows CRT API-set DLLs (`api-ms-win-crt-*`). eframe additionally imports `dxgi.dll`, `opengl32.dll`, `dwrite.dll`, and `uiautomationcore.dll`; Slint imports `dwrite.dll`, `opengl32.dll`, and `uiautomationcore.dll`, among other Windows system libraries. These are static PE imports, not a complete list of dynamically loaded modules. Runtime DLL availability, visual behavior, and disconnected launch are not verified by this inspection.

Current Phase 0.7E cold/warm launch, first paint, working set/private bytes, CPU, 10,000-row scroll/search/sort latency, resizing and GPU/software fallback: **NOT RUN**. The 0.7C values below were collected from earlier binaries in short shell process snapshots and must not be used as a current-build or framework ranking.

## Phase 0.7C release executable evidence — historical

| Prototype | Executable | Bytes | MiB (binary) | SHA-256 |
|---|---|---:|---:|---|
| eframe | `egui-prototype/target/release/encryptdrive-eframe-spike.exe` | 13,754,880 | 13.12 | `772296B910F550BF0092CAA154BF344093ED434D6F14A41DC08C7AEC265562DA` |
| Slint | `slint-prototype/target/release/encryptdrive-slint-prototype.exe` | 15,777,280 | 15.05 | `569FC4E8B60B300D9CCDB65AB7143AC1FD921C18C84B5D1E8F72428FC16207D0` |

## Launch-time memory snapshots

Windows Process working set and private bytes were read after 1.5 seconds. Each executable was started separately with the same `--entries` argument and then terminated by PID after measurement. All six processes were alive and had a non-zero native window handle.

| Prototype | Entries | Working set (bytes) | Private bytes |
|---|---:|---:|---:|
| eframe | 100 | 358,547,456 | 289,734,656 |
| eframe | 1,000 | 351,551,488 | 290,148,352 |
| eframe | 10,000 | 354,488,320 | 293,306,368 |
| Slint | 100 | 122,810,368 | 47,767,552 |
| Slint | 1,000 | 109,715,456 | 47,181,824 |
| Slint | 10,000 | 116,449,280 | 53,702,656 |

These samples show the GUI/runtime baseline dominates the list-size delta in this short run, especially for eframe; they do not establish peak memory or steady-state behavior. A controlled benchmark should use repeat runs, warm-up, an identical visible window, fixed display scale/GPU, and recorded OS/driver details.

## PE imports and deployment implications

`dumpbin /DEPENDENTS` was run on both Release files. Both import `VCRUNTIME140.dll` and Windows API-set CRT DLLs (`api-ms-win-crt-*`), plus operating-system UI/runtime DLLs. The eframe binary also imports `dxgi.dll` and `opengl32.dll`; Slint imports `dwrite.dll` and `opengl32.dll`. Both import `uiautomationcore.dll` for accessibility integration. The MSVC redistributable is therefore a deployment dependency unless the target environment already provides the required runtime. These are static PE imports, not a complete list of DLLs dynamically loaded at runtime.

The apps launched on the configured development host after the authorized Visual Studio workload installation. A clean-machine installer test and disconnected-network launch were **NOT RUN**. No runtime download was observed or requested, and source review found no application network client, but that is not a substitute for an offline VM run.

## Measurements not collected

| Metric | eframe | Slint |
|---|---|---|
| Cold/warm launch-to-first-paint | NOT RUN — no visual capture | NOT RUN — no visual capture |
| Peak/idle working set and CPU | NOT RUN | NOT RUN |
| Scroll, search, sort latency at 10,000 | NOT RUN | NOT RUN |
| Resize/DPI/GPU fallback performance | NOT RUN | NOT RUN |
| Full dynamically loaded module set | NOT RUN | NOT RUN |
| Clean install / disconnected launch | NOT RUN | NOT RUN |

No screenshot or frame-rate figure was manufactured. Desktop UI automation was stopped by the user with Escape; the process checks above used shell-only hidden launches and did not inspect or interact with application controls.

## Phase 0.7D Release rebuild and bounded CRT experiment — 2026-10-18

Because both original Release executables remained open in the visible desktop session, updated binaries were built in separate Cargo target roots instead of replacing those running files. The GUI corrections compile and link in Release mode.

### Updated dynamic-CRT Release artifacts

| Prototype | Output | Bytes | MiB | SHA-256 |
|---|---|---:|---:|---|
| eframe | `target/phase-0-7d/release/encryptdrive-eframe-spike.exe` | 13,754,880 | 13.12 | `65651CE77B98CA97B83D8F132C36F194F8105C1E13D0D7D79E255891F5E9776B` |
| Slint | `target/phase-0-7d/release/encryptdrive-slint-prototype.exe` | 15,788,544 | 15.05 | `F2C223D4A5ACAF9FE208FEC7BC75D30F0204491A1D7616477D2335CBA510641F` |

Measured optimized build times in this run were 6m25s (eframe) and 6m05s (Slint), each from a separate empty target root. These are build-time observations, not launch or runtime performance benchmarks.

### `crt-static` experiment

Both Release prototypes were rebuilt using the process-only `RUSTFLAGS="-C target-feature=+crt-static"` setting into `target/phase-0-7d-crt-static/`. `dumpbin /DEPENDENTS` confirms that both static-CRT variants no longer import `VCRUNTIME140.dll` or the `api-ms-win-crt-*` DLLs. Their ordinary Windows UI, graphics, and accessibility imports remain. This confirms the compiler/linker can statically include the MSVC CRT for these builds. The variants were **not launched**, so runtime behavior and functionality under this configuration remain unverified; do not treat them as release candidates yet.

| Prototype | Static-CRT bytes | Delta from dynamic-CRT | SHA-256 |
|---|---:|---:|---|
| eframe | 13,941,760 | +186,880 bytes (+1.36%) | `3F48376F8A6DC4FB0312CE643612D746188885153E9FE5DD12A9972873633E45` |
| Slint | 15,948,800 | +160,256 bytes (+1.02%) | `5EE0C2F0AC496F6B4FB4ABCBF129C8382CF191548D1659DB6805374F11B4822F` |

The ordinary updated binaries still directly import `VCRUNTIME140.dll` plus CRT API-set DLLs. The static-CRT experiment removes those imports at a modest size increase. Neither variant has been tested on a clean host or disconnected network; Windows system DLL imports, graphics behavior, UIA behavior, packaging and redistribution conditions still require review.

### Phase 0.7D runtime measurement status

No new cold/warm launch, 1,000/10,000 visible list, CPU, steady-state/peak memory, scrolling latency, or repeated-navigation measurements were taken. The computer-use helper stopped on physical Escape before these scenarios. The old 1.5-second snapshots remain clearly preliminary and are not extended into a comparative ranking. No frame rate, screenshot, DPI result, or responsiveness number was fabricated.
