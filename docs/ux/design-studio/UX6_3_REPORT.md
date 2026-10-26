# UX6.3 — DPI-safe startup window sizing

**Start:** `main` at `f29d69e352dad5cb8217610a078338efcae0ead5`, clean, level with `origin/main`.
**Scope:** the one remaining Codex finding on UX6.2, the DPI query in the pre-creation window hook. P1-02 (recursive scope) is not touched. This is not owner approval of the design.

## Original API problem

- **The UX6.2 call.** UX6.2's eframe `window_builder` hook called `GetDpiForMonitor(MDT_EFFECTIVE_DPI)` before the window existed.
- **Microsoft's documentation.** The [GetDpiForMonitor](https://learn.microsoft.com/en-us/windows/win32/api/shellscalingapi/nf-shellscalingapi-getdpiformonitor) remarks say the API "should not be used if the calling thread is per-monitor DPI aware" and point to [GetDpiForWindow](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getdpiforwindow). The same page's table says a per-monitor-aware process receives "the actual DPI value set by the user for that display". `GetDpiForWindow` needs an HWND, which does not exist in the hook.
- **The real risk in UX6.2.** The estimated DPI also set the native **minimum** size. A wrong value could therefore reintroduce the P1-01 failure: a minimum larger than the work area, which Windows enforces.

## Lifecycle and calling-thread awareness

These are verified in the pinned sources (eframe 0.37.0, winit 0.30.13) and at runtime.

| Step | What happens |
|---|---|
| Event loop creation | winit's `become_dpi_aware()` calls `SetProcessDpiAwarenessContext(PER_MONITOR_AWARE_V2)`, with fallbacks for older Windows. It is process-wide; winit never changes a thread's context. |
| `window_builder` hook | Signature `FnOnce(ViewportBuilder) -> ViewportBuilder`: no event loop and no `MonitorHandle`. eframe has just centred the builder using winit's `MonitorHandle::scale_factor()`. |
| winit `MonitorHandle::scale_factor()` | Implemented as `GetDpiForMonitor(MDT_EFFECTIVE_DPI)`, with a silent fallback to 96. |
| Window creation | Hidden. winit converts our point sizes with the new window's DPI (`GetDpiForWindow`). |
| First frame | Our UI runs (the HWND exists); egui's `pixels_per_point` = winit's window scale × egui zoom. eframe paints, **shows** the window, then applies that frame's viewport commands. |

**Calling-thread awareness, measured.**
- The new code reads `GetAwarenessFromDpiAwarenessContext(GetThreadDpiAwarenessContext())` in the hook and applies the estimate only when it returns 2 (per-monitor aware).
- **Real app:** native runs show the estimate applied, so the hook thread is per-monitor aware.
- **Test process:** cargo's test runner is not DPI aware. It reports 0, and the guard skips the estimate (`Skipped(NotPerMonitorAware(0))`).

## Alternatives considered

| Option | Verdict |
|---|---|
| A. winit's monitor scale before the window exists | **Rejected.** It cannot be reached from the hook (verified signature). Reaching it would mean replacing `eframe::run_native` with our own event loop, adding winit as a direct dependency, which the architecture check forbids. And `MonitorHandle::scale_factor()` *is* `GetDpiForMonitor`, so the documentation concern would only be hidden. |
| `GetDpiForWindow` in the hook | **Impossible.** No HWND exists yet. |
| `GetDpiForSystem` | **Rejected** (as the brief says). It is wrong for secondary monitors with a different scale. |
| Change the thread DPI awareness context | **Rejected.** It would change the units every later call sees, and winit expects the process context. |
| Hide the window until it is sized | **Not possible with eframe 0.37.** eframe always shows the window after the first paint, before applying that frame's commands. |
| B. Provisional estimate, authoritative per-window correction once the HWND exists | **Chosen** (below). |

## Selected solution (`src/ui/work_area.rs`, commit `58aac6a`)

1. **Estimate, before the window exists.**
   - It applies only when the hook thread is per-monitor aware and all native data is valid: work area present, DPI between 96 and 768 (100–800 %), frame size present.
   - It sets **only** the initial inner size and outer position. It never sets the minimum, so a wrong DPI cannot make Windows enforce a minimum larger than the work area.
   - Any failure is recorded as a typed reason (`EstimateSkipped`) and printed to stderr. The builder is then left as eframe made it.
2. **Authoritative, on the first frame.**
   - With a real window, it uses egui's `pixels_per_point`: winit's per-window scale (`GetDpiForWindow`) times egui zoom. That is the same factor egui used for the viewport rects and will use for the commands, so Windows DPI and egui zoom are applied exactly once.
   - It also uses the real decoration size (`outer − inner`) and the `rcWork` of the monitor that contains the window. Those coordinates are physical pixels, and negative coordinates are kept.
   - It sends `MinInnerSize = min(880×600, usable)`, then acts on the window:

     | Window state | Action |
     |---|---|
     | Already the planned size and inside the work area (estimate correct) | Untouched |
     | Planned size, but overlapping an edge | Nudged inside |
     | Anything else (estimate missing or wrong) | Resized and centred to the plan |

   - It runs once and never again after the user moves or resizes the window. The comparison with the estimate is printed to stderr.

**Can a wrong frame be seen?** Only when the estimate is missing or wrong. Then the window shows at the estimated (or eframe default) size for about one frame, because eframe makes it visible before applying the first frame's commands, and is then corrected. With a correct estimate, which is the normal case and was verified natively at 96 DPI, nothing changes after the window appears.

**Still unchanged:**
- Where there is room: 1200×760 preferred, 880×600 minimum.
- The work area is respected and nothing sits behind the taskbar.
- Auth screens scroll.
- No new dependencies. The bounded FFI adds `GetThreadDpiAwarenessContext` and `GetAwarenessFromDpiAwarenessContext` to the existing read-only calls; each has a `SAFETY` note.

## Testing

**Simulated, against the production functions** (`work_area.rs`, 19 tests):
- **Real decorations.** The `AdjustWindowRectExForDpi` frames at 96, 120, 144 and 192 DPI are asserted against this machine, so the fixture cannot drift.
- **Full startup.** `estimate_metrics`, then `plan_startup`, then a simulation of winit's rounding to whole physical pixels at the window's DPI with the real frame, then `first_frame_fit`.
  - A correct estimate needs no correction: 100/125/150/200 %, a 1020 px work area at 200 %, a small vertical work area, taskbars at the top, left and right, and monitors left of and above the primary (negative coordinates).
  - A too-low estimate (96 for a 200 % window) is shrunk inside the work area. A too-high one (192 for a 100 % window) grows to the plan. A missing estimate is corrected at every DPI.
- **The first-frame minimum** never exceeds the usable area, including a 1024×600 screen at 200 %.
- **Invalid native data skips the estimate:** a system-aware or unaware thread, no work area, a failed DPI query, 0 DPI, 4000 DPI, no frame.
- **The real hook** in the test process: it never sets a minimum, and the awareness guard skips the estimate.
- **Preserved from UX6.2:**
  - the plan invariants at every scale;
  - the 200 % minimum;
  - work areas under 600 points;
  - taskbars on every edge;
  - the smaller-request case;
  - unit conversion;
  - the five UX6.1 first-frame cases, now on `first_frame_fit` with the same expectations.
- **The reachability test** (all 22 preview states, 6 dialogs and Settings at 960×474 and 853×410, including the auth screens' scrolling) still passes.

**Native, at the current system scaling.** Windows 11, primary monitor 1920×1080 at **100 % (96 DPI)**, work area 1920×1032 (bottom taskbar).

*Startup diagnostics* (debug build, stderr):

| Arguments | Estimate | Window DPI on the first frame | Correction |
|---|---|---|---|
| `--design-preview` | Applied, 96 DPI, 1200×760 at 352,116 | 96 | None |
| `--window-size=1920x1080` | Applied, 96 DPI, 1904×993 at 0,0 | 96 | None |
| `--window-size=700x500` | Applied, 96 DPI, 880×600 at 512,196 | 96 | None |

*Release candidates* (`measure-window.ps1`: `GetWindowRect`, `GetClientRect`, `WM_GETMINMAXINFO`, `rcWork`, `GetDpiForWindow` of the app's own window; no input):

| Build and arguments | Window (physical) | Client | Native minimum (outer) | Inside the work area |
|---|---|---|---|---|
| Shell | 1216×799 at 352,116 | 1200×760 | 896×639 | Yes |
| Preview | 1216×799 at 352,116 | 1200×760 | 896×639 | Yes |
| Preview, `--window-size=1920x1080` | 1920×1032 at 0,0 | 1904×993 | 896×639 | Yes (exactly the work area) |
| Preview, `--window-size=700x500 --scenario=1` | 896×639 at 512,196 | 880×600 | 896×639 | Yes |

The client size and position equal the plan. The native minimum (880×600 plus the 16×39 frame) is now set by the first frame alone, because the hook no longer sets one.

**NOT RUN natively:**
- 125 %, 150 % and 200 % scaling;
- a work area under 600 points;
- a taskbar at the top or on a side;
- a secondary monitor, including one at negative coordinates;
- mixed-DPI monitors.

I did not change the display configuration, and no pre-configured VM or second display was available. The simulated tests do not prove real behaviour at those settings.

## How to run the remaining native checks

1. **Change the scaling** in Settings › System › Display › Scale (125 %, 150 % or 200 %). Sign out and back in if Windows asks.
2. **Measure the release candidate:**
   `pwsh -File docs/ux/design-studio/tools/measure-window.ps1 -Exe target/ux63-artifacts/encryptdrive-desktop-preview-58aac6a.exe -AppArgs --design-preview`
3. **Expect** `inside_work_area = True`, a client area no larger than the work area minus the frame, and a native minimum no larger than the window.
4. **Expected values** for this 1920×1080 monitor, from the production policy, assuming the taskbar stays 48 points high:

   | Scale | Work area (physical) | Client (physical) | Window position (physical) | Native minimum (outer, physical) |
   |---|---|---|---|---|
   | 125 % | 1920×1020 | 1500×950 | 200,11 | 1118×797 |
   | 150 % | 1920×1008 | 1800×952 | 48,0 | 1342×956 |
   | 200 % | 1920×984 | 1894×913 | 0,0 | 1786×984 |

5. **Read the diagnostics** (optional): run a debug build (`cargo build -p encryptdrive-desktop --features design-preview`) from a console. It prints the estimate, the first-frame window DPI and any correction. A `correction=Some(..)` line means the estimate and the window DPI disagreed.
6. **Taskbar and monitors:** repeat with the taskbar at the top or on a side, and with the window starting on a second monitor.

## Validation (at `58aac6a`)

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace --locked` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --locked` | PASS: 49 (3 core, 10 library, 36 desktop) |
| `cargo check --workspace --features design-preview --locked` | PASS |
| `cargo clippy --workspace --features design-preview --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --features design-preview --locked` | PASS: 137 (3 core, 10 library, 94 desktop, 30 model). The same result in 3 separate runs, with no intermittent failures. |
| `cargo build --workspace --release --locked` | PASS |
| `cargo build -p encryptdrive-desktop --features design-preview --release --locked` | PASS |
| `scripts/verify-architecture.ps1` | PASS; no new dependencies |

The only "warning" lines in the log are rustc incremental-cache notices (hard-linking failed, files copied), not code warnings.

**Candidate executables** (build directories are ignored by Git):

| Variant | Path | Raw SHA-256 |
|---|---|---|
| Preview | `target/ux63-artifacts/encryptdrive-desktop-preview-58aac6a.exe` (15,532,032 bytes) | `2C2E8D782E311DB60D0FF503BD658E503A8D687729CF961BD4A7C7A94AF75CFA` |
| Shell | `target/ux63-artifacts/encryptdrive-desktop-shell-58aac6a.exe` (14,873,600 bytes) | `00CDAAEC7F31DD151495FCBC0B23155F8687E6F4ADDDDE31EC82C1CFABB897CA` |

The link-normalised hashes are `9E308712…` (preview) and `E71F1093…` (shell). Rebuilt raw hashes always differ by the link timestamp and PDB GUID; see the UX6.1 report.

## Commits

| Commit | Subject |
|---|---|
| `58aac6a` | fix(windows): use DPI-safe initial window sizing (implementation and its tests) |
| this commit | docs(ux): record UX6.3 verification and limitations |

## For the targeted review

- `src/ui/work_area.rs`: the module docs (policy, units, authority of each step), `estimate_metrics`, `startup_viewport`, `first_frame_fit`, and the `windows` FFI block.
- The tests named above.
- The NOT RUN configurations, which need the owner or an environment already set to another scale.
