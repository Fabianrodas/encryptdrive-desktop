# Phase 1B-0 / 1B-1 — Build Identity and Artifact Isolation

**Status:** Limited implementation authorized; Phase 1B remains in progress and is not complete.

## Authorized scope

The owner authorized only:

- **Phase 1B-0:** establish the engineering baseline and create the functional integration branch from the approved `main` baseline.
- **Phase 1B-1:** isolate default-shell and synthetic `design-preview` Windows builds, record build identity, and measure reproducibility.

This authorization does not include Phase 1B-2 or later, Vault Format 1, cryptography, storage, real-vault operations, application command/event APIs, packaging, signing, distribution, upload, publication, or merge to `main`.

## Verified starting baseline

| Item | Recorded value |
|---|---|
| Rust repository | `encryptdrive-desktop` |
| Starting branch | `main` |
| Starting commit | `50ab6adf1a0e3a4cea2a2d2f676136fe1e0f0cbc` |
| Local worktree | Clean before branch creation |
| Local `origin/main` tracking ref | `50ab6adf1a0e3a4cea2a2d2f676136fe1e0f0cbc` |
| New integration branch | `dev`, created from the starting commit |
| Windows target | `x86_64-pc-windows-msvc` |
| Rust toolchain | `1.99.1-x86_64-pc-windows-msvc` |
| Framework | eframe/egui `0.37.0` |

The local Git remote uses the expected EncryptDrive Desktop repository identity. A live `git ls-remote` check could not complete because Windows denied creation of an SSH IPC object (`NtCreateDirectoryObject`, `0xC0000022`); this record therefore confirms the local tracking ref, not the current server-side ref.

The approved UX6.3 design and its existing accessibility, High Contrast, DPI, multi-monitor, runtime, clean-host, licensing, and distribution gates remain unchanged. The synthetic preview remains synthetic-only.

## Build-identity evidence

### Implemented boundary

The 1B-1 candidate adds a Windows/MSVC PowerShell build driver, a manifest schema and verifier, negative-path verifier tests, and CI coverage for both existing desktop build variants. The driver invokes Cargo with explicit feature selection and separate persistent target directories for `shell` and `design-preview`. It places each run's copied release executables and `manifest.json` beneath a unique `target/phase-1b/artifacts/<RunId>/` directory and refuses to reuse an existing run ID/output directory. It cleans only the desktop package's release output within that variant's target directory before rebuilding; dependency caches remain available.

The versioned manifest records the source revision and dirty state, target, Rust/Cargo versions, release profile, lockfile SHA-256, normalized Rust flags, linker identity/version, and each executable's variant, Cargo features, relative output and target paths, byte size, raw SHA-256, PE timestamp and CodeView PDB identity. The verifier checks the expected two variants, features, profile and target; variant/output/target separation; path containment; artifact presence, size and raw hash; and recorded PE metadata. It does not publish artifacts. No Cargo dependency, lockfile, Rust source, UI, or application API was changed.

Implementation references: [Windows build driver](../../scripts/build-windows.ps1), [manifest verifier](../../scripts/verify-build-artifacts.ps1), [manifest schema](build-artifact-manifest.schema.json), [verifier rejection tests](../../scripts/tests/verify-build-artifacts.tests.ps1), and [Windows CI workflow](../../.github/workflows/ci.yml).

The default build mode remains an unmodified release build. Optional experimental modes apply MSVC `/Brepro`, Rust path remapping, or both; only baseline mode is wired into CI. These flags are not enabled by default.

### Toolchain and build inputs

The local Windows build used:

| Input | Observed value |
|---|---|
| Target | `x86_64-pc-windows-msvc` |
| Rust | `rustc 1.99.1 (58cf8f9c3 2026-10-13)`; commit `58cf8f9c35b355199727627d07ae34a4e2846d67` |
| Cargo | `cargo 1.99.1 (5f94df478 2026-08-27)` |
| Linker | `link.exe (MSVC linker) 14.51.36260.0` |
| Profile | `release` |
| Cargo.lock SHA-256 | `3CB09A9815F937F30137C3B9169F33834B1412CEAD428406613A65C99A64D754` |
| Build source revision | `b667cddbd286555f2f5ea2f70e9ea018fe935a76`, clean (`dirty=false`) |

The build source revision is the CI/build implementation commit immediately before this evidence-only documentation commit. Therefore the manifest's source identity is intentionally not the final documentation HEAD. Local runs and generated files are under ignored `target/`; no executable, PDB, or manifest is committed.

### Build and quality validation

All of the following local checks passed on the Windows MSVC host:

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace --locked` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --locked` | PASS — 49 tests |
| `cargo check --workspace --features design-preview --locked` | PASS |
| `cargo clippy --workspace --features design-preview --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --features design-preview --locked` | PASS — 137 tests |
| `scripts/verify-architecture.ps1` | PASS |
| PowerShell parser on new `.ps1`/`.psm1` scripts | PASS |
| Manifest schema JSON syntax parse | PASS |
| `scripts/tests/verify-build-artifacts.tests.ps1` | PASS — valid manifest plus 12 rejection/edge assertions |
| `scripts/verify-build-artifacts.ps1` on six recorded build manifests | PASS |
| End-to-end build driver, baseline, `/Brepro`, and remap runs | PASS |

Cargo emitted non-fatal environment/cache warnings during the work: one cold-build warning that the global cache's last-use SQLite database was read-only, and incremental-cache hard-link failures that fell back to copying. Builds and checks completed successfully; no unrelated global tooling or configuration was changed. An initial pre-fix smoke attempt stopped because the verifier function import order was wrong; the import was corrected and the final script tests and end-to-end runs passed.

The checked-in Windows CI workflow retains formatting, default and preview checks, strict Clippy, both test suites, manifest rejection tests, isolated release builds for both variants, manifest verification, and architecture validation. A GitHub Actions run was not observed in this task. The live remote query could not be completed because Windows denied SSH IPC-object creation (`NtCreateDirectoryObject`, `0xC0000022`), so remote CI status and current server-side branch state remain unverified.

### Artifact measurements and reproducibility

The following are raw SHA-256 values of the produced `.exe` files, not normalized hashes. Each A/B pair used the same recorded source revision, clean state, target, toolchain, linker, lockfile, profile, feature set, Rust flags, and per-variant Cargo target directory; run IDs and artifact output paths differed. Run IDs were `phase1b-baseline-a/b`, `phase1b-brepro-a/b`, and `phase1b-remap-a/b`.

| Mode / variant | Size (bytes) | Run A raw SHA-256 | Run B raw SHA-256 | Repeated PE timestamp / PDB GUID |
|---|---:|---|---|---|
| Baseline / shell | 14,875,648 | `DC634767E80AE0CC62D56E93024C902F84576034F09B50C7A041965451226018` | `A61DA2C4D583EECEEB5003EAFD8F296E248950B82E8391270B565923805B21FB` | Differed: `0x6AE2843A` / `9ffe3fff-2f8a-4bd4-b6e6-455e04091788`; `0x6AE2845A` / `9f10a288-bb15-4090-b604-1c548f54ffaf` |
| Baseline / design-preview | 15,534,592 | `D9EC14A21D14349B48C23FBBD1612823FEE255070B9155CB83EB3CB679AA7CFE` | `062B9A37215B19A74D0484154BA1FBD0BE33F93F90DE4B616222BF84969B4B26` | Differed: `0x6AE28444` / `b4f34d78-b90e-4b8d-89a5-61720e27ebf7`; `0x6AE28466` / `40f627ce-fbbe-4c2d-8d17-996de404e762` |
| `/Brepro` / shell | 14,875,648 | `9D0372A782BCFB2D826B0F0E10C2C76B23EEEB6EF8A075A113B11749D457355E` | `9D0372A782BCFB2D826B0F0E10C2C76B23EEEB6EF8A075A113B11749D457355E` | Identical: `0x69AEDF27` / `b516d6a6-ec9f-a958-f087-e8b56ab759cd` |
| `/Brepro` / design-preview | 15,535,104 | `A9C393687577651D30A40887630B7C312BE309BA3F9B14EC002D1F80ADDE7759` | `A9C393687577651D30A40887630B7C312BE309BA3F9B14EC002D1F80ADDE7759` | Identical: `0x7EBBC201` / `85e88131-27aa-5b91-197b-afd1a4786188` |
| Path-remap / shell | 14,855,680 | `DBE44FF6EFE838435912EEB38179047E5386234BE8C718F78CF4AC515F5798B0` | `7BFE50C74C9A556545163A1FCC013E2DAD50E2BF3E6B52F3CDEA26B2A127337F` | Differed: `0x6AE2861F` / `76e4465d-8aca-4fcf-aa90-4243c6eca413`; `0x6AE286AA` / `98fa1784-55f8-4a16-836c-b8fe29f6fa35` |
| Path-remap / design-preview | 15,514,112 | `C9EE2E6AE9E7B4F2B135FDB5A5C37C17B68AA7143DF513AF353D0E6C5EE52ACE` | `0B67C16D7D6842CAB77770915AE45D3C00176346A9D53A34C587E10E98726DDE` | Differed: `0x6AE2869C` / `6456b87c-83d7-41dd-b835-36c90bfed066`; `0x6AE286B6` / `2f887222-bc6a-483b-b44c-22c30834576f` |

Baseline output was not bit-for-bit reproducible in these same-host repetitions. `/Brepro` produced identical raw executable hashes, PE timestamps, and PDB GUIDs for the repeated runs using the same per-variant target directories. This is promising local evidence, not proof of reproducible builds across clean hosts, toolchain installations, or changed paths.

A separate `/Brepro` shell build into an alternate Cargo target directory produced the same byte length (`14,875,648`) but a different raw SHA-256 (`1A9D26EAB15843584528E1BB02B25EF88B6A20B3255269923E296FD2908B22B1`, compared with `9D0372A782BCFB2D826B0F0E10C2C76B23EEEB6EF8A075A113B11749D457355E`), PE timestamp (`0xAF3A66AC` versus `0x69AEDF27`), and PDB GUID (`e6c2454c-3aef-26b9-497a-37eb8a89fef4` versus `b516d6a6-ec9f-a958-f087-e8b56ab759cd`). Thus same-directory repetition does not establish location-independent reproducibility; the cause was not isolated.

Path remapping used normalized flags `--remap-path-prefix=<repository-root>=/encryptdrive` and `--remap-path-prefix=<cargo-home>=/.cargo`. A byte-string inspection confirmed the literal repository-root and Cargo-home paths appeared in baseline A executables and were absent from the corresponding path-remap A executables. The remap A/B hashes still differed, so remapping alone did not make these artifacts reproducible. The PDB files in the active variant Cargo target directories after the remap B run still contained both literal paths. This inspection only checks those exact strings; it does not prove that all sensitive path information is absent from executables or symbols.

Combined `/Brepro` plus path-remapping mode was not run. No timing, memory, frame-rate, GUI interaction, DPI, accessibility, packaging, signing, offline clean-host, or installer measurement was part of this build-identity scope. No prototype was launched and no input was injected.

### Independent review and remaining gates

This is an implementation candidate for independent Claude Code review, not an approval or completion declaration. Review should inspect:

- PowerShell argument quoting, MSVC discovery/`VsDevCmd` behavior, path validation, and failure cleanup.
- Manifest schema/verifier agreement, tampering resistance, uniqueness and stale-output handling.
- Whether preview feature selection and package-scoped clean behavior remain correct as workspace features evolve.
- CI behavior on the actual hosted Windows image and whether its selected linker is sufficiently pinned for the intended identity claims.
- Reproducibility on a second clean Windows host; `/Brepro` and path remapping remain experimental until then.
- PDB path disclosure and whether symbols should be retained, distributed, or handled separately in a later packaging decision.
- Whether build outputs should eventually be archived by CI; this change deliberately does not upload or publish them.

Phase 1B remains in progress. These changes do not start Phase 1B-2, implement production application boundaries, or authorize Vault Format 1, cryptography, storage, real-vault operations, packaging, signing, distribution, or release approval.
