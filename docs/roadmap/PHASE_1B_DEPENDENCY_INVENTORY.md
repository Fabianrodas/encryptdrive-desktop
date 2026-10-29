# Phase 1B-2 — Dependency, Licensing Metadata, and Supply-Chain Inventory

**Status:** Implemented locally on `dev`; evidence and review gates are recorded below. This phase does not complete Phase 1B, approve licensing, authorize distribution, or start Phase 1B-3.

## Scope and boundary

Phase 1B-2 records the Windows application’s locked dependencies, Windows-resolved graphs for the default shell and synthetic `design-preview`, Cargo license metadata, isolated Release artifact identity, compiler-log observations, and static PE import names. The inventory is generated from `Cargo.lock`, locked Cargo metadata, the build manifest, build logs, and `dumpbin /DEPENDENTS`.

The implementation commit is `894aeb116fe685dee9864be6dbea469c32d2b15c`, on local `dev`. It has not been pushed or merged. No Cargo dependency, `Cargo.toml`, `Cargo.lock`, Rust application source, Java repository, or private documentation file was changed. No artifact is uploaded or published. No real vault, plaintext, secret, or user file is opened.

## Implemented files

- [`DependencyInventory.psm1`](../../scripts/DependencyInventory.psm1) parses lock package identities, joins locked packages with Cargo metadata, constructs target-filtered reachable graphs, redacts absolute source/license paths, validates inventory consistency, and records verified build/import evidence.
- [`verify-dependency-inventory.ps1`](../../scripts/verify-dependency-inventory.ps1) generates a deterministic JSON inventory under ignored `target/phase-1b/dependency-inventory/`, validates its serialized output, and supports read-only validation of an existing inventory.
- [`verify-dependency-inventory.tests.ps1`](../../scripts/tests/verify-dependency-inventory.tests.ps1) exercises malformed JSON, missing fields, count and identity inconsistencies, missing lock references/configurations/checksums, license metadata contradictions, workspace classification, absolute-path redaction, and a real Cargo metadata generation from a working directory outside the repository.
- [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) pins both external actions by full commit SHA, disables checkout credential persistence, runs the new validator suite, then generates an inventory from the separately verified Windows Release manifest with native imports required.
- [`DEPENDENCY_AND_RUNTIME_REVIEW.md`](../architecture/DEPENDENCY_AND_RUNTIME_REVIEW.md) records measured counts, raw metadata exceptions, artifact identity, imports, and remaining approval gates.

The generated inventory JSON, executable copies, manifest, build logs, and Cargo target directories remain ignored beneath `target/`; they are local evidence and are not part of the commit.

## Measured inventory

The local full lock set contains 367 packages: 362 crates.io registry packages and 5 workspace packages. Registry packages have Cargo.lock checksums; there are no Git-sourced or external path packages in this lock set. All 362 registry packages declare a raw Cargo `license` expression; none declares `license_file`. The five workspace packages have no license expression or license file metadata and are explicitly reported as missing rather than assigned a presumed license.

For `x86_64-pc-windows-msvc`, both the shell and preview graph contain 161 packages reachable from the desktop application’s normal/build dependencies: one application root, three direct dependencies, and 157 transitive dependencies. Both configurations resolve the same package identities. The preview activates the root `design-preview` feature; the default shell does not request it. These are metadata-resolved packages, not a claim that every package compiled or linked into the executable.

The exact local build source was `894aeb116fe685dee9864be6dbea469c32d2b15c` with `dirty=false`; Rust was `1.99.1` (`58cf8f9c35b355199727627d07ae34a4e2846d67`), Cargo was `1.99.1` (`5f94df478`), and the target was Windows x64/MSVC. See the architecture review for the lockfile SHA-256, both raw executable hashes and sizes, all 27 imports per variant, and the build-log completeness limitation.

## CI changes

The workflow uses `actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1` (upstream release v7.0.1) and `dtolnay/rust-toolchain@d124f86f28ff42855784ff3319f48ac182c248f8` (Rust 1.99.1 action source). The checkout action has `persist-credentials: false`; workflow permission remains `contents: read`. The workflow keeps the existing format, locked build, strict Clippy, default/preview tests, artifact checks, isolated build variants, and architecture boundary checks. No artifact upload step was added.

The new CI validator exercises both the complete lock inventory and the two Windows app graphs. After the build manifest verifier succeeds, CI requires `dumpbin` collection and writes the inventory only to ignored `target/`. The pinned action revisions were checked against their official upstream release/action files; links and SHAs are recorded in the architecture review. Hosted CI has **NOT RUN** for this local commit because it has not been pushed.

## Verification record

| Check | Local result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace --locked` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --locked` | PASS; non-fatal incremental-cache hard-link warnings fell back to copying |
| Preview `cargo check`, strict Clippy, and tests with `--features design-preview --locked` | PASS; same non-fatal cache fallback warning on tests |
| `scripts/verify-architecture.ps1` | PASS |
| New dependency-inventory regression suite | PASS, including full Cargo metadata generation and malformed/inconsistent inputs |
| Existing build-artifact rejection suite | PASS for executable cases; Windows junction creation remained BLOCKED by host access denial |
| Existing PowerShell wrapper regression suite | PASS |
| `scripts/verify-build-artifacts.ps1` on both clean local variants | PASS |
| Release build of shell and design-preview | PASS; both variants built from the same clean commit |
| Inventory generator with `-RequireNativeImports` | PASS; both variants checked with `dumpbin /DEPENDENTS` |
| CI workflow YAML and PowerShell parser validation | PASS |
| Hosted GitHub Actions for commit `894aeb1…` | NOT RUN; commit remains local |

The generated evidence file is `target/phase-1b/dependency-inventory/phase1b2-894aeb1-clean.json`. It is ignored and is not a committed report. Recreate the evidence with a new, unique run ID:

```powershell
pwsh -NoProfile -File scripts/build-windows.ps1 -RunId '<unique-run-id>'
pwsh -NoProfile -File scripts/verify-build-artifacts.ps1 -ManifestPath 'target/phase-1b/artifacts/<unique-run-id>/manifest.json'
pwsh -NoProfile -File scripts/verify-dependency-inventory.ps1 -ManifestPath 'target/phase-1b/artifacts/<unique-run-id>/manifest.json' -RequireNativeImports
```

Do not reuse an existing run ID; the build and inventory scripts refuse to overwrite prior evidence.

## Remaining gates and limitations

- This inventory is not a vulnerability scan, full SBOM attestation, source-code audit, legal opinion, or review of crate `LICENSE`/`NOTICE` texts. The slash-delimited expressions, `Ubuntu-font-1.0`, `self_cell`’s `Apache-2.0 OR GPL-2.0-only`, and the five unlicensed workspace package manifests need owner/legal review before distribution.
- Rust product licensing, third-party notice obligations, GitHub Action bundle licensing, Microsoft runtime redistribution, and any package bundling decision remain open. No license is inferred from Java.
- The linker path/version is an environment probe recorded with `usage_proven=false`; Cargo’s actual linker invocation was not independently established.
- `dumpbin /DEPENDENTS` sees static PE imports only; it does not enumerate runtime `LoadLibrary` activity. One `Compiling` line per variant reflects incremental-cache observations and is not a complete compiler/linker package set.
- The junction negative test remains BLOCKED by Windows access denial. It is a pre-existing build-artifact path-hardening test limitation, not a passing result.
- No clean-host offline launch, runtime installation, Narrator/UI Automation, High Contrast, Windows support-floor, installer, signing, reproducible cross-host build, or distribution test was performed by Phase 1B-2.
- This work does not implement typed application commands/events, session orchestration, Vault Format 1 parsing, cryptography, storage, or real vault operations. Those require their separately authorized phase and conformance gates.

## Next review gate

Request an independent review of this local `dev` commit, then run its hosted Windows CI after a separately authorized publication. Any license expression, runtime, or host-path risk must be dispositioned by the owner/relevant reviewer. Keep Phase 1B-3, Vault Format 1, cryptography, storage, packaging, and publication out of scope until separately authorized.
