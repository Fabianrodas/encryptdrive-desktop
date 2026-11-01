# Phase 1B-4 — Production Shell and Synthetic Preview Isolation

**Status:** Local implementation candidate on `dev`; hosted CI and interactive Windows validation are pending.
**Source revision for Release artifacts:** `a2ea3de88e89298369b0790b026be10f560ef752` (`dirty=false`)
**Validation date:** 2026-11-01
**Scope:** Automated source/feature isolation checks, two isolated Windows Release builds, and a repeatable owner-run smoke protocol. No real vault behavior is implemented.

## 1. Verified implementation boundary

The production shell starts with `ShellState::new()`: Files selected, System appearance preference, `NoVaultOpen`, and Settings closed. It renders the app header, Files/Trash navigation, Settings, status bar and empty states. Trash navigation can display an explicit unavailable state; there is no Trash operation or file listing. The shell has no synthetic entries, operational file table, sorting, import/export, rename, delete, restore, authentication, vault creation, storage, network, or telemetry behavior. Settings and appearance live in the current process state; no preference persistence path was found.

The Desktop manifest declares `encryptdrive-core`, but the production shell does not call `readiness()` or another core API. This distinction is checked by source inspection and the isolation gate; the existing architecture validator checks the package dependency boundary, not API call sites.

The `design-preview` Cargo feature is empty and opt-in. `main.rs` declares the preview-only UI module tree (`ui/mod.rs`: app, dialogs, explorer, navigation, screens) and the preview entry point only behind that feature. `src/ui/` is **not** preview-only as a whole: the ungated `design.rs` module uses `#[path]` to compile seven shared files — `ui/components.rs`, `ui/copy.rs`, `ui/geometry.rs`, `ui/icons.rs`, `ui/theme.rs`, `ui/typography.rs`, and `ui/work_area.rs` — into the production shell binary as well as the preview binary. `lib.rs` also includes the preview model under `cfg(test)` so unit tests can compile it in a default test build; that test-only compilation is not production release enablement. Preview fixtures and operations are in-memory.

Both executables call `design::icons::install_font` at startup. It reads exactly three files, `segoeui.ttf`, `seguisb.ttf`, and `segoeuib.ttf`, from `%WINDIR%\Fonts` with one read-only `std::fs::read`, and falls back to the bundled fonts if they are unavailable. This is the only filesystem read the source gate permits; `ui/work_area.rs` additionally contains an `#[allow(unsafe_code)]` FFI module that calls `user32`/`shcore` for monitor work-area and DPI queries.

**Source gate design.** `scripts/ShellIsolation.psm1` enumerates every `*.rs` file under `apps/encryptdrive-desktop/src` recursively and fails closed if the tree, the Desktop manifest, or any of `lib.rs`, `main.rs`, `shell.rs`, `design.rs`, `preview.rs` is missing, if no Rust files are found, if a file cannot be read, or if the tree contains any reparse point (symlink or junction). The snapshot is the manifest plus a list of `{RelativePath, Content}` entries, so tests can mutate any file or add a new one. Two policy classes apply:

- **UI policy** — every file under `ui/` (shared production design files and preview-only modules). `std::net`, `std::process` and `std::os` are forbidden everywhere; `std::fs` and `std::path` are allowed only in `ui/icons.rs`, and there only as the single bounded font read described above with exactly the three approved filenames.
- **Strict policy** — every other file: root files including `design.rs`, `lib.rs`, `main.rs`, `shell.rs`, `preview.rs`, everything under `preview/`, and any new file anywhere in the tree. Any `std::fs`, `std::net`, `std::process`, `std::path` or `std::os` reference is forbidden.

Under both policies the gate blanks comments and literals, extracts every complete `use` statement (including `pub`/`pub(crate)`, multiline, and leading `::` forms) and expands its use-tree — nested braces, `self`, `as` aliases and globs — into flat paths. It rejects imports of the forbidden `std` modules, of the production crates (`encryptdrive_core`, `vault_format`, `vault_crypto`, `vault_storage`), and of the `std` root itself (`use std as s;`, `use std::{self as s};`, `use std::*;`). Backstops reject fully qualified `std::` module paths (tolerating whitespace and comments between segments), real network/process/file constructors (`TcpStream::connect`, `TcpStream::connect_timeout`, `TcpListener::bind`, `UdpSocket::bind`, `Command::new`, `File::open`/`create`/`create_new`/`options`, `OpenOptions::new`), and any production-crate name. Every `#[path = "..."]` literal is resolved relative to the declaring file; absolute paths, resolutions outside `apps/encryptdrive-desktop/src`, non-literal path attributes, and targets that are not scanned `.rs` files are rejected. `include!` and `extern crate` are rejected; `include_bytes!`/`include_str!` remain allowed for compile-time assets.

The source gate also keeps the earlier semantic checks: empty default features, an explicit empty `design-preview` feature, the preview gate in `lib.rs`, the `ui` and `run_preview` gates and default `main` in `main.rs`, the production shell restrictions, all 22 preview scenarios, and explicit design-preview/synthetic-data wording. It is a targeted regression check, not a complete Rust parser or a security boundary (see *Scanner limitations* below). Production isolation is supported by the combined source/feature check, the unchanged architecture boundary check, and the separately verified Release manifest feature fingerprints; absence of a string in an EXE is not used as proof.

**Independent-review follow-up (2026-11-01):** Review found that the original scan omitted root application modules, could miss grouped or aliased imports, and did not constrain the Windows font loader to fixed filenames. Commit `cf56640` adds root entry/shell/preview sources, rejects direct and common grouped/aliased imports in the scanned scopes, and restricts the font exception to exactly `segoeui.ttf`, `seguisb.ttf`, and `segoeuib.ttf`.

**Second independent-review follow-up (2026-11-01):** A later audit reproduced two P1 gaps in the `cf56640` gate. (1) Coverage was a fixed list (`lib.rs`, `main.rs`, `shell.rs`, `preview.rs`, `preview/**`, `ui/**`), so file I/O added to `design.rs`, to a new root module, or to an in-tree `#[path]` target passed while compiling into the default production configuration. (2) The grouped-import pattern stopped at the first inner `}`, so nested imports such as `use std::{io::{self, Write}, net::TcpStream};` passed, and the network backstop matched only nonexistent `TcpStream::new`/`UdpSocket::new` constructors. The remediation commits following `8277721` replace the fixed list with recursive enumeration and the policy classes above, add use-tree expansion, `#[path]` resolution, `include!`/`extern crate` rejection and real network constructors, and correct scanner wording that called `ui/*` preview-only. The regression suite now has 41 negative cases (the original 13 plus 28 new), including a dedicated `lib.rs` file-I/O case and fail-closed enumeration cases. In scratch copies, weakening each new protection (fixed-list coverage, old import regex, no `#[path]` check, old network constructors, old `std::x::` backstop form, `lib.rs` excluded, no `include!`/`extern crate`/`std`-root/reparse/empty-tree check) made the suite fail. This remediation was implemented and validated by the implementing agent; it has not been independently reviewed.

### Scanner limitations

- It is text scanning with a small comment/literal blanker and use-tree expander, not a Rust parser, type checker, or macro expander. Unusual but valid syntax it cannot tokenize fails closed; semantics it cannot see are not checked.
- FFI is not inspected: `extern "system"` blocks, `#[link]` attributes and Win32 calls (such as the existing `user32`/`shcore` work-area queries) could reach files or the network without any `std` path.
- `extern crate` (including aliases) is rejected, but crate roots can still be reached by other indirect means the gate does not model, for example re-exports from a dependency or a local module alias chain whose individual statements look innocuous.
- Macro-generated behavior (`macro_rules!`, procedural macros from dependencies, `concat!`-built identifiers) is not expanded.
- Runtime dynamic loading (`LoadLibrary`/`GetProcAddress`, COM activation) is invisible to source scanning.
- File enumeration is not the compiled module graph: every `.rs` file under `src` is checked, including files no `mod` declaration reaches. That over-coverage is intentional and conservative.
- Runtime filesystem, registry and network behavior can only be established by manual observation (section 4F); source checks cannot prove absence of behavior.
- The gate is a regression guard for this boundary, not a formal security boundary.

No Desktop or lower-crate runtime dependency, production API, authentication path, or real vault operation was added.

## 2. Automated evidence

### Build identity

Both Release variants were built with the existing PowerShell 7 driver into isolated Cargo target directories. The manifest was independently verified and the dependency inventory was generated with native PE imports required.

| Variant | Features in verified manifest | Release EXE | Size | Raw SHA-256 |
|---|---|---|---:|---|
| Production shell | `[]` | `target/phase-1b/artifacts/phase1b4-20261101-02/shell/EncryptDrive-shell-a2ea3de88e89-phase1b4-20261101-02.exe` | 14,875,648 bytes | `378651CE7B8CECD18694335C620245F7D60D0F6AD99BDEAA5A914D51A55236D7` |
| Synthetic design preview | `["design-preview"]` | `target/phase-1b/artifacts/phase1b4-20261101-02/design-preview/EncryptDrive-design-preview-a2ea3de88e89-phase1b4-20261101-02.exe` | 15,534,592 bytes | `2DF126C0A601F4C294B53323D886FB33686BF92128639E8F2A74CD38DAE1266F` |

These are raw SHA-256 hashes of the Release executables, not normalized hashes. The manifest source identity is the exact clean candidate revision used for the builds; only this report's artifact-evidence rows changed in the following documentation commit, with no executable-affecting source changes afterward.

The artifacts were **not rebuilt** for the scanner remediation. The remediation commits following `8277721` change only `scripts/ShellIsolation.psm1`, `scripts/tests/verify-shell-isolation.tests.ps1` and this report; `git diff a2ea3de HEAD -- apps crates Cargo.toml Cargo.lock rust-toolchain.toml scripts/build-windows.ps1 scripts/BuildArtifacts.psm1 .github` is empty, so no build input changed since the artifact source revision.

| Build input | Observed value |
|---|---|
| Target/profile | `x86_64-pc-windows-msvc` / Release |
| Rust | `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, commit `58cf8f9c35b355199727627d07ae34a4e2846d67` |
| Cargo | `cargo 1.99.1 (5f94df478 2026-08-27)` |
| PowerShell | `7.6.5` |
| Cargo.lock SHA-256 | `3CB09A9815F937F30137C3B9169F33834B1412CEAD428406613A65C99A64D754` |
| Linker probe | `link.exe (MSVC linker) 14.51.36260.0` detected; actual linker invocation is not independently proven by the manifest |
| Build source | `a2ea3de88e89298369b0790b026be10f560ef752`, clean |
| Native imports | 27 observed for each executable using `dumpbin /DEPENDENTS`; this is a static import view, not runtime `LoadLibrary` tracing |

The local Windows registry reports ProductName `Windows 10 Pro`, DisplayVersion `25H2`, and build `26200.9711`. CIM queries were unavailable in this agent environment, so GPU model/driver could not be collected. The per-monitor registry value was `DpiValue=0`; its effective scaling was not established. No visual scale is inferred from that registry value.

### Quality and boundary checks

| Check | Result | Evidence |
|---|---|---|
| `cargo fmt --all -- --check` | PASS | No formatting changes required. |
| `cargo check --workspace --locked` | PASS | Default feature configuration compiled. |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS | Default feature configuration, strict warnings. |
| `cargo test --workspace --locked` | PASS | 49 passed, 0 failed. This test configuration compiles preview modules through `cfg(test)`. |
| `cargo check --workspace --features design-preview --locked` | PASS | Explicit preview feature configuration compiled. |
| `cargo clippy --workspace --all-targets --features design-preview --locked -- -D warnings` | PASS | Preview feature configuration, strict warnings. |
| `cargo test --workspace --features design-preview --locked` | PASS | 137 passed, 0 failed; includes preview model, operation, keyboard, dialog and UI-state tests. |
| `scripts/verify-architecture.ps1` | PASS | Five workspace packages and direct dependency boundaries verified. |
| `scripts/verify-shell-isolation.ps1` | PASS | All 20 Rust files under `apps/encryptdrive-desktop/src` scanned; production-shell defaults, feature gates, scenario catalog, imports, fully qualified calls, `#[path]` targets and preview labeling checked. Re-run after the scanner remediation. |
| `scripts/tests/verify-shell-isolation.tests.ps1` | PASS | 41 negative cases rejected (re-run after the scanner remediation). The original 13: default-feature enablement, unguarded UI module, direct/grouped/aliased file and core imports, I/O in root preview/shell sources, an unbounded font path, a fixture in the production shell, and a shortened scenario catalog. The 28 added: file I/O in `design.rs`, in a new root module, in `lib.rs` (dedicated case), and in an in-tree `#[path]` target; `#[path]` escaping `src`, absolute, or pointing at a non-Rust file; `include!`; `extern crate`; nested, aliased, multiline, leading-`::`, `std`-root alias/self-alias/glob, and `std::os` imports; unqualified `TcpStream::connect` and `TcpListener::bind`; leading-`::` and comment-split fully qualified `std` paths; `std::path` in a non-font UI file; a second filesystem call in the font loader; a snapshot missing `design.rs`; and fail-closed enumeration of a missing source root, an empty tree, and a junction inside the tree (the junction case prints `BLOCKED` instead where Windows denies junction creation; it ran on this host). Also passed through a GitHub-style `pwsh -command` wrapper (exit 0 on real source, nonzero on a violating scratch copy). |
| `scripts/tests/verify-build-artifacts.tests.ps1` | PASS with one BLOCKED subcase | All executable and manifest rejection cases passed; Windows denied junction creation for the reparse-point case. |
| `scripts/tests/verify-powershell-wrapper.tests.ps1` | PASS | Standalone and GitHub Actions wrapper exit propagation, including intentional failures. |
| `scripts/tests/verify-dependency-inventory.tests.ps1` | PASS | Malformed and inconsistent inventories, lock/configuration, checksums, classification, and redaction cases. |
| `scripts/verify-build-artifacts.ps1` on this run's manifest | PASS | Both expected variant directories, features, names, sizes, and raw hashes verified. |
| `scripts/verify-dependency-inventory.ps1 -RequireNativeImports` | PASS | Both variants: 367 lock entries, 161 Windows-resolved packages, 27 static imports each. |
| Hosted CI for this candidate | NOT RUN | No push was authorized. The new workflow step is local-only until a separately authorized publication. |

Cargo emitted non-fatal Windows incremental-cache hard-link warnings during test compilation and fell back to copying cache files.

After the scanner remediation, `git diff --check`, the seven Cargo commands above (49 and 137 tests, unchanged), `scripts/verify-architecture.ps1`, `scripts/verify-shell-isolation.ps1` and the shell-isolation regression suite were re-run and passed. The build-artifact, PowerShell-wrapper and dependency-inventory suites and verifiers were not re-run because none of their inputs changed.

## 3. Production shell and preview behavior checks

| Invariant | Result | Basis and limit |
|---|---|---|
| Default state is Files, System theme, and no open vault | PASS | Existing Rust unit tests verify initial `ShellState`. |
| No fabricated entries in production shell | PASS | Default `main` constructs `DesktopShell`; preview UI is feature-gated; source check rejects preview fixture references in `shell.rs`. |
| Trash has no real vault behavior | PASS | The shell renders an unavailable state; navigation is not a Trash data operation. No manual UI run was performed. |
| No real file commands enabled in default shell | PASS | The production shell has no operation handlers; `encryptdrive-core` readiness is not called. Source/feature evidence only. |
| Appearance preference is process-local | PASS by source review | State is stored in `ShellState`; no settings persistence API or storage call exists. Restart behavior still needs manual confirmation. |
| Preview fixture and operation state are in-memory | PASS by source and tests | Model contains deterministic entries and state changes; preview/model/UI sources contain no file mutation, network, process, core or lower-vault calls. |
| Simulated import/export/delete/restore/progress/cancel | PASS in automated tests, manual UI NOT RUN | Existing preview tests exercise model, dialogs, synthetic status and cancel/error transitions. No real files or vaults were accessed. |
| Preview cannot establish authenticated production session | PASS by source boundary | Feature-gated preview has no core/authentication call path. This is not a test of a future authentication implementation. |
| 100/1,000/10,000 fixtures and stable IDs | PASS | Existing preview tests verify counts, deterministic fixtures, stable IDs, filtering/sorting and exact recursive operation scope. |
| Preview scenario list | PASS | Isolation gate found exactly 22 listed scenarios. |
| Release variant identity | PASS | Clean manifest declares no features for shell and only `design-preview` for preview; independent verifier passed both. |

## 4. Owner-run Windows smoke protocol

### Before starting

1. Use the exact two Release executables and matching manifest listed above. Do not replace them with a debug build or a different revision.
2. Record Windows edition/version/build, display scale, monitor arrangement, GPU/driver, and date. Record the shell and preview raw hashes from the manifest. The current agent record above does not substitute for the owner's display configuration.
3. Close other EncryptDrive windows. Do not choose a real vault or point the app at personal files. Keep the preview run synthetic.
4. Record each step as PASS, FAIL, BLOCKED, or NOT RUN. Attach a screenshot, UI Automation tree, or Narrator observation only if captured during that step.

### A. Production shell start and empty state

1. Launch the shell executable without arguments.
2. Confirm the title is `EncryptDrive`; Files is initially selected; the header says no vault is open; the engineering status is visible; and no entries, file table, command bar, or active vault operation is shown.
3. Confirm Files and Trash destinations and Settings are visible. Select Trash and confirm it shows an unavailable explanation with no data operations. Return to Files.
4. Mark any unexpected account, vault, filename, file table, import/export control, or operation as FAIL and close the application without selecting a real location.

**Recorded status:** NOT RUN — the application was not launched to avoid controlling the owner's desktop.

### B. Appearance and in-memory settings

1. Open Settings with the mouse, then repeat using keyboard navigation. Verify System, Light, and Dark choices, visible focus, and an explanation that the choice lasts only for the current session.
2. Change each appearance and confirm shell surfaces, text, focus, and disabled states remain readable. Close and reopen Settings; confirm the selected value remains while the process is open.
3. Close and relaunch the app. Confirm appearance returns to the default System preference; this is the manual check for absence of persisted settings.

**Recorded status:** NOT RUN — no interactive session was launched.

### C. Keyboard and responsive layout

1. With keyboard only, traverse the header, Files/Trash navigation, Settings, and any explanatory disabled controls using Tab and Shift+Tab. Confirm focus order is logical and every focused control has a visible focus indicator.
2. Confirm unsupported actions are not enabled and their explanations are reachable. Check Escape closes Settings without changing vault state.
3. Resize the window through a wide layout, the preferred review size, and the smallest supported layout. Check that the sidebar remains usable, the central empty state does not overlap, and the status bar remains visible.
4. Repeat with `--window-size=880x600` and `--window-size=1180x792`; capture only the application window if an owner chooses to save screenshots.

**Recorded status:** NOT RUN — resizing, keyboard focus, and screenshots were not exercised.

### D. Windows display behavior

On a supported display or test machine, repeat shell launch, Settings, resize, and focus checks at 100%, 125%, 150%, and 200% scaling. Also check:

- Windows High Contrast enabled and disabled.
- Text scaling increased from the default.
- A taskbar on each edge and reduced work area.
- A mixed-DPI second monitor; move the window between monitors and reopen it after the move.
- Startup when the primary display is smaller than the preferred window size.

Record the actual monitor and scale for each run. Do not infer DPI coverage from unit tests, monitor-work-area source inspection, or the registry value alone.

**Recorded status:** NOT RUN — no DPI, High Contrast, taskbar, text-scale, or mixed-monitor UI session was performed.

### E. UI Automation and Narrator

1. Inspect the shell's UI Automation tree. Record accessible names and roles for the application header, Files, Trash, Settings, theme choices, status bar, and empty-state message.
2. Check enabled/disabled state exposure and ensure decorative icons are not announced as separate actionable controls.
3. Start Windows Narrator and traverse the shell without a mouse. Record the spoken name, role, selected/disabled state, focus order, and any duplicate or missing announcement.
4. Repeat in Light and Dark appearance and High Contrast if available.

**Recorded status:** NOT RUN — UI Automation inspection and Narrator were not started.

### F. Production isolation and offline start

1. Launch only the verified shell executable while offline or with network access disabled by the owner.
2. Confirm the shell reaches its empty state without runtime downloads or an unexpected network prompt.
3. Confirm no design-preview selector, sample vault, sample rows, simulated-operation banner, or synthetic action is present.
4. Confirm no real vault path or file is requested. Do not select or inspect any real user data.

**Recorded status:** NOT RUN — the executable was not launched; offline behavior has not been observed.

#### F.1 Process Monitor and network observation procedure

Use this to observe what the production shell actually touches at startup. It complements, and cannot be replaced by, the source gate.

1. Start Sysinternals Process Monitor (`Procmon64.exe`) before launching the shell. In **Filter**, add `Process Name` `is` `<exact test EXE name>` (for example `EncryptDrive-shell-a2ea3de88e89-phase1b4-20261101-02.exe`) → `Include`, and remove other Include rules so only that process is captured. Keep Process Monitor's default Exclude rules for its own and System activity.
2. Clear the display, start capture, then cold-launch the shell (no other EncryptDrive process running, ideally after a reboot or at least after closing every EncryptDrive window). Capture from before launch until the empty state is shown and idle, roughly 30–60 seconds, then stop capture.
3. Classify every file and registry event. Expected Windows resources include: the EXE itself and its directory; system DLLs under `%WINDIR%\System32`/`SysWOW64` and WinSxS; GPU driver files and driver/shader caches (for example under `%LOCALAPPDATA%` vendor or `D3DSCache` folders); fonts, including exactly `segoeui.ttf`, `seguisb.ttf` and `segoeuib.ttf` under `%WINDIR%\Fonts`; and registry reads for DPI, theme, accessibility, input and graphics settings. Any write, create, rename or delete outside OS/driver-managed caches, any read of user documents or vault-like paths, and any font file other than the three approved names is **unexpected**.
4. Record unexpected file operations sanitized: operation, result, category, and a redacted path (for example `%USERPROFILE%\…\<file>.ext`). Do not record personal paths, usernames or file contents, and do not commit PML/CSV captures to the repository.
5. Repeat the cold launch offline (network disabled by the owner) and confirm the shell still reaches its empty state.
6. For network, open Resource Monitor (`resmon.exe`) → **Network** tab, tick the test EXE's process, and check *Network Activity* and *TCP Connections* during launch and idle; an equivalent per-process tool (for example a Process Monitor `Network` event class filter or TCPView) is acceptable. Any connection or listener is **unexpected** for this shell.
7. Limitations: observation covers only the captured window and the code paths exercised (launch, empty state, and any interaction performed); background OS components may act on the process's behalf; and the absence of events is not proof that untriggered code paths cannot touch files or the network.

**Recorded status:** NOT RUN — no Process Monitor or Resource Monitor capture was taken.

### G. Synthetic preview-only demonstration

1. Launch the separately hashed design-preview executable with `--design-preview --scenario=6` for the synthetic explorer.
2. Check the explicit design-preview/synthetic label and sample-data warning. Change list size, navigate, search, sort, select, and use preview-only dialogs.
3. Exercise scenario 12 for simulated Import progress and cancellation, scenario 13 for the plaintext Export warning, scenario 15 for permanent-delete confirmation, scenario 18 for the synthetic recovery notice, and scenarios 19–20 for progress/error states.
4. Confirm only synthetic in-memory state changes and that all dialogs continue to identify sample data. Never select a real file, vault, or destination.

**Recorded status:** NOT RUN — all state and dialog evidence is from automated/headless tests, not a live preview window.

### Manual evidence matrix

| Area | Result | Evidence available |
|---|---|---|
| Production launch, title, selected Files destination, empty state | NOT RUN | Exact shell Release hash and manifest; no screenshot. |
| Trash unavailable explanation and no operations | NOT RUN | Source inspection and tests only; no live observation. |
| Settings keyboard use and session-only appearance | NOT RUN | State/unit tests; restart persistence not manually checked. |
| Tab/Shift+Tab focus order and visible focus | NOT RUN | Existing automated preview UIA/focus tests do not validate production shell in Narrator. |
| Resizing and compact layout | NOT RUN | Existing geometry tests; no live resize screenshot. |
| 100/125/150/200% scaling and mixed monitors | NOT RUN | No display-scale session. |
| High Contrast and increased text scaling | NOT RUN | No interactive validation. |
| UI Automation tree and Windows Narrator | NOT RUN | No accessibility client was opened. |
| Offline launch and no runtime download | NOT RUN | No app launch or network-isolated host test. |
| Process Monitor file/registry and Resource Monitor network observation (F.1) | NOT RUN | No capture taken. |
| Preview-only visual warnings and dialogs | NOT RUN | Automated state/dialog tests passed; no live screenshot. |

No screenshots, visual observations, Narrator scores, runtime-download observations, or GUI performance measurements were produced for this candidate.

## 5. CI and remaining gates

The new workflow step runs the shell/preview source isolation regression suite as a headless check. It does not claim GUI, DPI, accessibility, offline-host, or packaging validation. Existing SHA-pinned actions, `contents: read`, locked Cargo commands, manifest rejection suite, PowerShell wrapper suite, dependency inventory checks, separate Release variants, manifest verification, and architecture validation remain in place. No artifact-upload or release step was added.

Remaining limitations and follow-up:

- Hosted CI was not run for the local candidate — six commits on `dev` after `main` (`61e7d9e`) through `8277721`, plus the scanner-remediation commits following `8277721` (one changing the scanner module and its regression suite, one updating this report) — because no push was authorized.
- The manifest reparse-point negative test's junction subcase remains BLOCKED by the host's access denial; all other manifest rejection cases passed.
- No UI Automation tree, Narrator, High Contrast, display scale, mixed-monitor, or manual keyboard-only result has been verified.
- No clean-host/offline launch or runtime-redistributable check has been performed.
- Clean-host packaging, Microsoft runtime redistribution, installer/portable execution, signing, reproducible cross-host builds, support range, and licensing/release approval remain later gates.
- Vault Format 1, cryptography, storage, authentication, and real vault operations are outside this work and remain unimplemented.

## 6. Independent review handoff

Review the local commits on `dev` and verify:

1. `scripts/ShellIsolation.psm1` recursively scans every Rust file under `apps/encryptdrive-desktop/src` with the UI and strict policy classes, expands nested use-trees, resolves `#[path]` targets, and fails closed on missing inputs, empty trees, unreadable files and reparse points; its 41 negative cases cover the original 13 gaps and the second-review bypasses. It does not claim to be a complete Rust parser or runtime security boundary; the Windows font-read exception is limited to three fixed basenames under Fonts and to `ui/icons.rs`.
2. The source checker, default/preview Release manifest features, and existing architecture validator jointly support the isolation claims; `cfg(test)` preview compilation is described correctly.
3. The recorded shell behavior matches the source and does not treat Trash navigation as a real Trash operation or the core dependency declaration as readiness API usage.
4. The raw executable hashes match the ignored local files under `target/phase-1b/artifacts/phase1b4-20261101-02/`; the report distinguishes them from normalized hashes and names the exact build revision.
5. The report marks hosted CI and every live GUI/accessibility/display/offline check as NOT RUN rather than implying source/tests prove runtime behavior.
6. No executable, manifest, build log, user data, or private material is tracked.

Phase 1B-4 remains a local candidate pending independent review, owner-run manual validation, and separate owner disposition. No Phase 1B-5 or VF1 implementation is included.
