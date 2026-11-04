# Phase 2A-1 — Structural Types and Input Boundaries

**Status:** Local implementation and validation complete; hosted `dev` CI and independent review pending
**Recorded:** 2026-11-03
**Starting baseline:** `392c0283849ea7952f2c7d835a5333b8990fdda6` on `dev`
**Authorization:** The owner explicitly authorized Phase 2A-1. This record covers only its bounded structural foundation.

## Objective and result

Provide a small, zero-I/O foundation in `vault-format` for later structural parsing: the known VF1 version constant, source-backed complete-file size limits, a typed size error, and a borrowed representation that records only that a size check passed. No JSON parser, metadata object model, cryptographic behavior, or vault operation is implemented.

## Implemented public API

The only functional source change is `crates/vault-format/src/lib.rs`.

- `VF1_FORMAT_VERSION: u8` is `1`. It identifies the currently specified version; it does not parse input or classify unsupported versions.
- `InputKind` distinguishes `VaultHeader`, `EncryptedUserRegistry`, and `EncryptedAccountManifest`. `max_bytes()` returns that category's confirmed complete-file ceiling.
- `check_input_length(kind, actual_bytes)` compares a caller-supplied `usize` directly to the selected limit. It does not allocate, inspect bytes, perform I/O, or use arithmetic that can overflow.
- `LengthCheckedInput::new(kind, bytes)` borrows the original slice and checks `bytes.len()`. Its accessors expose the category, borrowed bytes, length, and emptiness without copying.
- `FormatError::InputTooLarge { kind, actual_bytes }` implements `std::error::Error` and formats a deterministic message using only the input category and size metadata. It never includes input contents.

These names deliberately say **length checked**. Success does not mean nonempty, valid UTF-8, valid JSON, structurally valid VF1 metadata, or cryptographically authenticated. Zero bytes pass a size-only check and are not thereby a valid VF1 file. No `TrustedVault`, authenticated, or structurally validated type is produced.

No `FormatVersion` type is introduced: a constant is sufficient for the known version and avoids implying that unsupported-version parsing already exists. That classification belongs to a later parser milestone.

## Confirmed complete-file limits

The limits below apply to each complete file, including an encrypted JSON envelope where applicable. They are Java VF1 contract limits, not plaintext metadata budgets.

| Public constant | Category | Maximum |
|---|---|---:|
| `VAULT_JSON_MAX_BYTES` | complete `vault.json` | 262,144 bytes (256 KiB) |
| `ENCRYPTED_USER_REGISTRY_MAX_BYTES` | complete encrypted user registry, including envelope | 16,777,216 bytes (16 MiB) |
| `ENCRYPTED_ACCOUNT_MANIFEST_MAX_BYTES` | complete encrypted account manifest, including envelope | 67,108,864 bytes (64 MiB) |

No plaintext limit is inferred from an encrypted-file limit. No JSON-depth, collection-count, or string-size budget is defined here.

## Errors and deferred taxonomy

Implemented error taxonomy: only `InputTooLarge`, because it is the only failure this milestone can produce. Future parser-specific distinctions may include invalid UTF-8/JSON, missing or mistyped fields, duplicate/forbidden unknown properties, unsupported version, invalid identifiers/KDF/Base64/nonces, short ciphertext, invalid names/tree relationships, integer overflow, and parser resource exhaustion. Those variants are not present until an implemented operation needs them.

## Zero-I/O and security boundary

`vault-format` operates only on caller-provided values, lengths, and borrowed byte slices. It performs no filesystem, network, Windows API, process, environment, clock, randomness, session, credential, key, encryption, or decryption work. `#![forbid(unsafe_code)]` remains in force. A caller may later provide bytes it obtained elsewhere; this crate does not obtain them itself.

There is no JSON parser, Serde integration, Base64 decoder, UUID parser, Unicode engine, AAD construction, KDF, AES-GCM, key wrapping, authentication, storage, GUI integration, or real-vault functionality. Java fixtures remain outside this public repository; none were copied or referenced by symlink.

## OQ-01 through OQ-08 policy and verification dependencies

The owner approved these as Phase 2A design directions. That approval does not claim their compatibility verification is complete.

| OQ | Approved direction | Still required before the relevant parser or dependency work |
|---|---|---|
| OQ-01 | Rust follows the documented strict nested-envelope contract; Java may accept extra properties. | Differentially verify and preserve the known reader divergence before parser implementation. |
| OQ-02 | Reject duplicate keys, BOM, invalid UTF-8, and trailing JSON; keep allowed unknown top-level header fields; reject unknown KDF/envelope fields unless verified otherwise. | Reconcile unverified Java acceptance details before finalizing parser behavior. No JSON behavior was implemented here. |
| OQ-03 | Separate existing-data structural validity from Java writer rules and Java's actual reader checks. | Gather differential evidence before registry/manifest rejection rules are implemented. |
| OQ-04 | Enforce canonical lowercase UUID text where VF1 explicitly requires it; do not impose UUID v4 as a general reader restriction. | Verify manifest UUID text acceptance from primary sources before implementing those fields. |
| OQ-05 | Preserve the three confirmed complete-file ceilings. | Obtain exact numerical approval for depth, collection, string, and plaintext metadata budgets. |
| OQ-06 | Keep Java reference fixtures outside this public repository. | Complete provenance and redistribution-permission review before any fixture sharing. |
| OQ-07 | AAD ownership remains with `vault-crypto`. | No AAD work belongs in this crate; changing ownership requires separate architecture approval. |
| OQ-08 | Add no third-party dependencies in Phase 2A-1. | Assess JSON, Base64, UUID, and Unicode library choices and Unicode-version behavior in a separate review. |

## Validation performed on Windows

Toolchain: `rustc 1.99.1`, `cargo 1.99.1`, `x86_64-pc-windows-msvc`, PowerShell 7.6.5. `CARGO_INCREMENTAL=0` was set only in individual Cargo command processes to avoid host cache hard-link warnings; no persistent environment setting was changed.

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace --locked` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --locked` | PASS, 60 tests total; 11 in `vault-format` |
| `cargo check --workspace --features design-preview --locked` | PASS |
| `cargo clippy --workspace --features design-preview --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --features design-preview --locked` | PASS, 151 tests total; 11 in `vault-format` |
| `scripts/verify-architecture.ps1` | PASS; five workspace members and existing dependency boundaries verified |
| `scripts/verify-shell-isolation.ps1` | PASS; 20 Desktop Rust source files scanned |
| `scripts/tests/verify-shell-isolation.tests.ps1` | PASS, 42 negative cases rejected; Windows junction/reparse subcheck BLOCKED by host `NewItemCreateIOError` |
| `scripts/tests/verify-dependency-inventory.tests.ps1` | PASS |
| `scripts/tests/verify-powershell-wrapper.tests.ps1` | PASS |
| `scripts/tests/verify-build-artifacts.tests.ps1` | PASS; Windows junction/reparse subcheck BLOCKED by host `NewItemCreateIOError` |

Cargo manifests and `Cargo.lock` are unchanged; the architecture validator confirms `vault-format` still has no direct dependencies. The existing script output reports 42 shell-isolation negative cases; this milestone does not change that suite or resolve the separately recorded P2-F02 observation. The hosted CI result must be recorded in the implementation handoff against the exact published `dev` SHA; this local record does not claim hosted CI success.

## Phase boundary and next work

Phase 2A-2 remains unimplemented and requires its own authorization. It may add the JSON parsing and structural behavior defined by the approved direction only after the relevant compatibility questions above are resolved. It must preserve the distinction between length-checked, structurally validated, and authenticated data. Cryptography, storage, Java fixture integration, and real vault operations remain later work.

This milestone does not modify the historical Phase 1A architecture snapshot or claim Phase 1B Windows gates are closed. The following Phase 1B follow-ups remain open and unchanged: P2-F01 raw-identifier coverage for the font-loader exception; P2-F02 scanner negative-case count; P2-F03 preview password-form feedback/state assertions; P2-F04 exact SHA attribution for historical CI runs; and the non-reproduced compact-layout observation. Windows validation N01–N09 remain **NOT RUN / UNVERIFIED**; B01–B04 remain **BLOCKED**. No accessibility, Windows release, runtime-isolation, or distribution certification is claimed.

## Phase 2A-1H — Controlled API Safety Hardening Addendum

**Recorded:** 2026-11-04
**Original Phase 2A-1 integrated baseline:** `191084c952f8403c8efbf63df6b400f66498a458`
**Independent review:** Claude Code returned **CONDITIONALLY APPROVED — NONBLOCKING FOLLOW-UPS**, with no confirmed P0/P1 and four P2 findings. Follow-up review of this maintenance remains pending.

This dated addendum preserves the Phase 2A-1 status snapshot above. It records only the approved Phase 2A-1H maintenance; it does not authorize or begin Phase 2A-2.

### Scanner regression-count qualification

- The local shell-isolation suite rejected **42 negative cases**. The host could not create its required Windows junction/reparse fixture (`NewItemCreateIOError`), so that specific subtest was **BLOCKED** locally; it is not counted as a pass.
- Hosted Windows CI reported **43 negative cases rejected**, including successful execution of the junction-related case. The hosted result is distinct from the locally blocked fixture.
- The local build-artifact manifest suite also reported its junction/reparse subtest as **BLOCKED** for the same host limitation; its remaining rejection cases passed.

### Complete-file limits also cover corresponding backups

The confirmed limits apply to complete files, including the corresponding backups: `vault.json` remains limited to **256 KiB**; the encrypted user registry and its backup share the **16 MiB** complete-file ceiling; the encrypted account manifest and its backup share the **64 MiB** complete-file ceiling. These are existing category limits. No separate plaintext limits or backup-specific `InputKind` variants are introduced.

### P2 disposition

| Finding | Correction recorded |
|---|---|
| 2A1-P2-01 — constructor regression coverage | Direct `LengthCheckedInput::new` tests reject a header input one byte over its limit, prove the same borrowed input succeeds for the larger registry and manifest categories, retain each supplied category and borrow the same buffer, and check the inclusive header boundary through the constructor. Isolated mutation checks detect removal of the constructor check, a fixed wrong category, and changing `>` to `>=`. |
| 2A1-P2-02 — borrowed bytes in `Debug` | `LengthCheckedInput` retains `Clone` and `Copy` but implements `Debug` manually with only `kind` and `len`. Regression coverage uses a synthetic sentinel plus non-printable bytes and checks that neither the sentinel, a byte-array rendering, nor hexadecimal content appears. This does not claim memory erasure or prevent caller-side logging. |
| 2A1-P2-03 — weak or redundant tests | The display contract now has an exact deterministic assertion for category, actual length, and maximum. The unrelated sentinel assertion and duplicate numeric-allocation test were removed; the extreme-value numeric check and meaningful boundary coverage remain. The non-behavioral terminology test was removed; the boundary terminology remains documented above. |
| 2A1-P2-04 — evidence qualifications | The local/hosted scanner counts and junction limitation are distinguished above, and complete-file ceilings are explicitly applied to the corresponding metadata backups. |

### Validation and commit evidence

Local validation for this maintenance used Rust/Cargo `1.99.1` on `x86_64-pc-windows-msvc` and PowerShell `7.6.5`. The focused `vault-format` suite passed **12 tests**; the default workspace passed **61 tests** and the `design-preview` workspace passed **152 tests**. Formatting, locked checks, strict Clippy, architecture boundaries, shell/preview isolation, the existing PowerShell regression suites, isolated Windows release manifest verification for both variants, and dependency-inventory generation/validation passed. Three isolated mutation checks failed as expected, demonstrating that the constructor tests detect the specified regressions. The local junction subtests remain **BLOCKED** as described above.

The maintenance SHA is the Git identity of the single commit containing this addendum and its source changes; its exact value is recorded in the Phase 2A-1H handoff. The hosted `dev` result is recorded there only after the run for that exact SHA completes. No prior `main` CI run is used as evidence for this maintenance commit. At the time this addendum was authored, the new commit and its hosted run did not yet exist.

The four P2 items above are addressed in code/tests/documentation and await independent follow-up review. No parser, JSON/Base64/UUID/Unicode processing, cryptography, storage, I/O, new dependency, or real-vault operation was added. The deferred OQ-01 through OQ-08 verification dependencies and Phase 1B Windows gates remain unchanged.
