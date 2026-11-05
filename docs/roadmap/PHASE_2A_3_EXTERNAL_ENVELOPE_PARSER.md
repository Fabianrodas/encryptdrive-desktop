# Phase 2A-3 — External Encrypted Envelope Parser

**Status:** Phase 2A-3 R1 correction is recorded below; its exact-SHA CI result is in the R1 task handoff. Independent review and owner disposition remain required.
**Recorded:** 2026-11-05
**Starting baseline:** `438b17a168eef88e067613796821c991086901f9` on `main` and `dev`
**Implementation commit:** `e13e5ed22475e22c2641a86c8d994878427868c7` (parent: `438b17a168eef88e067613796821c991086901f9`)
**Authorization:** Owner-approved implementation of structural parsing for encrypted user-registry and account-manifest envelopes. This record does not authorize integration into `main`, Phase 2A-4, or Phase 2A-5.

## Scope and trust boundary

This milestone parses caller-provided bytes for the external JSON envelopes used by `users.enc`, its three backup generations, account-manifest files, and their three backup generations. Backups use the same category parser as their corresponding primary file. There is no path-based API, file access, backup selection, or recovery behavior.

The result is `UnauthenticatedExternalEnvelope`. Success means only that the bytes are structurally valid under the approved envelope contract and the ciphertext string is canonical Base64. It does not mean the GCM tag is valid, that the ciphertext can be decrypted, or that any plaintext is well formed. No keys, passwords, real vault data, Java fixtures, or private documentation were used or copied.

No cryptography, AAD construction, key derivation, plaintext registry or manifest parsing, filesystem or storage behavior, authentication, backup recovery, or GUI integration was added. `vault-format` remains free of unsafe code and I/O.

## Starting state and approved contract

The implementation began on existing branch `dev` at the approved baseline above. At preflight, local `main`, local `dev`, `origin/main`, and `origin/dev` were all at that SHA, and the tracked and untracked working tree was clean. The Rust repository remote was `git@github.com:Fabianrodas/encryptdrive-desktop.git`. The Phase 2A-2 header parser and its tests were present; no Phase 2A-3 implementation existed.

The owner-approved contract is:

| Decision | Implemented rule |
|---|---|
| D1 — exact schema | One object with exactly `version`, `algorithm`, `nonce`, and `ciphertext`, each once; property names are compared after JSON escape decoding. Unknown properties are forbidden. |
| D2 — JSON policy | Strict UTF-8, no BOM, one complete JSON document, standard JSON whitespace and escapes, no comments, trailing comma, invalid surrogate, or trailing document. |
| D3 — Base64 | Existing `base64` 0.23.1 standard engine; standard alphabet, required canonical padding and pad bits; no whitespace or URL-safe form. Valid JSON escapes are decoded before Base64 validation. |
| D4 — encrypted bytes | Nonce decodes to exactly 12 bytes. Ciphertext plus tag is variable length and at least 16 bytes; no fixed 48-byte restriction is applied to external envelopes. |
| D5 — input limits | Complete-file inclusive limits are 16,777,216 bytes for registry envelopes and 67,108,864 bytes for manifest envelopes. Existing `InputKind` and `LengthCheckedInput` enforce the correct category before parsing. |
| D6 — JSON nesting policy | The existing maximum of 32 nested JSON containers remains unchanged for the general/header parser. The separately approved early-rejection condition for external-envelope fields does not rename or replace D6. |
| D7 — shared parser | Reuse the existing structural scanner and exact-number support, while keeping external-envelope early rejection separate from `vault.json`'s allowed unknown root fields. |
| D8 — category-safe APIs | `parse_encrypted_user_registry` and `parse_encrypted_account_manifest` select their own fixed limits and delegate to one private implementation. |
| D9 — ownership | The returned type owns one decoded ciphertext-plus-tag `Vec`, exposes borrowed accessors, and is not `Clone`. Nonce is a fixed 12-byte value. |
| D10 — errors | Typed errors are sanitized and never include source bytes, attacker-supplied field names, nonce or Base64 fragments. Debug identifies the result as unauthenticated and reports lengths only. |
| D11 — dependencies | No dependency, manifest, feature, lockfile, or architecture allowlist changes. Existing dependency configuration is retained. |
| D12 — compatibility/testing | Synthetic contract tests and the complete Phase 2A-2 regression suite are required. Java writer behavior is the compatibility target; intentional strict-reader divergences and unverified interoperability are stated below. |

Two additional owner-approved engineering conditions are applied: (1) safely reject forbidden fields after validating the minimum key/colon syntax, without materializing their names or traversing their values; and (2) avoid input-sized encoded-string copies during large Base64 decoding.

### Specification traceability note

The complete Phase 2A-3 PREP report was supplied by the owner, but its item-by-item text was not present in the implementation agent's local working context when this document was first written. The earlier statement that the numbered matrix did not exist was incorrect. This R1 record restores the owner-supplied ranges and family descriptions without inventing individual case wording. The PREP's D6 is the JSON nesting policy; early rejection is a separate, additional owner-approved engineering condition.

## Schema, parsing order, and error precedence

The accepted logical schema is:

```json
{"version":1,"algorithm":"AES/GCM/NoPadding","nonce":"<canonical standard Base64>","ciphertext":"<canonical standard Base64>"}
```

Field order and standard JSON whitespace are flexible. `version` must be mathematically exactly 1; `1`, `1.0`, `1e0`, and `1.00e+0` are accepted without floating-point conversion. Zero/negative values are invalid, fractions are non-integral, out-of-range values are rejected, and other valid integer versions are unsupported. `algorithm` must decode exactly to the case-sensitive string `AES/GCM/NoPadding`.

The parser's deterministic precedence is:

1. Enforce the complete-input category limit.
2. Validate the whole input as UTF-8; reject a leading UTF-8 BOM.
3. Require an object root and parse each key and the colon needed to identify its member.
4. Reject an unknown decoded property name immediately as `ForbiddenField`; do not allocate that name or traverse its value. Reject a repeated known name after the key and colon are recognized.
5. Reject known fields of the wrong JSON type without walking a nested value. For correctly typed values, validate their full scalar syntax, the complete object, and document termination before semantic checks.
6. Report missing required fields after a complete candidate object is validated.
7. Validate exact version semantics and algorithm value.
8. Validate canonical Base64 for nonce and ciphertext, then enforce decoded lengths.
9. Fallibly reserve and construct the one owned ciphertext output, then return the unauthenticated value.

An input with a forbidden field may be rejected even when the unexamined value is malformed. Such input is already invalid. A successful result is never produced from a partial scan: all accepted fields, object punctuation, trailing whitespace, and end-of-document are checked before decoding or construction.

## Implementation and allocation strategy

`strict_json.rs` now provides an external-schema scanner, bounded decoded-key comparison, zero-allocation ASCII string visitors, and exact `u64` parsing from the existing validated number span. `header.rs` reuses that exact-number helper; the header parser's existing behavioral contract remains covered by its unchanged regression tests. The external scanner does not change `StrictDocument`'s unknown-field or depth semantics.

The unescaped Base64 fast path borrows the source span directly. Validation uses fixed 3,072-byte scratch storage over 4,092-byte chunks; decoding writes directly into the single output allocation. The escaped path decodes JSON escapes into a fixed 4,093-byte staging buffer, retaining any incomplete quartet across chunks. The same standard Base64 engine validates and decodes both paths. It does not build a JSON DOM or a full encoded `String`.

Decoded length is counted with checked addition during validation. Only after the entire value is validated does the parser call `try_reserve_exact`, resize the output once, and decode into it. The nonce uses a fixed stack array. The test input slice remains caller-owned and is borrowed; the parser cannot prevent a future file-reading caller from loading an oversized file before calling it, so future bounded read infrastructure must enforce the same complete-file ceiling before an unbounded read.

For a maximum-size valid manifest, the input is 67,108,864 bytes and the decoded ciphertext is 50,331,582 bytes. The parser's additional large allocation is that one owned output buffer; chunk scratch/staging are fixed-size and independent of input length. No output clone is made. Allocation reservation failures map to a sanitized `AllocationFailed` error. OS allocation behavior and allocator metadata are outside this structural allocation accounting.

## Java compatibility evidence

Source-level inspection of the read-only Java reference confirmed:

- `MetadataJson.envelope` reads the exact four envelope members with `JsonReader` strict mode, requires exact JSON types, rejects duplicates/extra members/trailing content, and uses exact `BigDecimal` integer conversion.
- `AesGcmService` emits the case-sensitive `AES/GCM/NoPadding` algorithm and standard Java `Base64.getEncoder()` strings for nonce and ciphertext; it enforces the 12-byte nonce and a minimum 16-byte ciphertext/tag on crypto use.
- Both repositories serialize the encrypted envelope with their field-initialized `new Gson()` instances (`UserRegistryRepository` and `ManifestRepository`) via `gson.toJson(aes.encrypt(...))`. Gson's default HTML-safe JSON writer can render Base64 padding `=` as lowercase `\u003d`. The JSON-decoded value is still standard canonical Base64. Rust therefore interprets valid JSON escapes before checking Base64 canonicality; unescaped `=` and escaped `\u003d` representations of the same value must decode identically. No AES-GCM operation is needed to validate this structural behavior.
- `UserRegistryRepository.MAX_REGISTRY_BYTES` is 16 MiB and `ManifestRepository.MAX_MANIFEST_BYTES` is 64 MiB; the matching backup bytes use those repository limits.

These source observations support compatibility with canonical Java writer output. Rust deliberately rejects BOMs, invalid UTF-8, and noncanonical Base64 forms even if some Java reader paths may be more permissive. There was no Java app execution, fixture import, real-vault parsing, or differential Java/Rust run. Full interoperability remains **UNVERIFIED**. PREP questions OQ-03 (writer-reader content rules), OQ-04 (metadata UUID compatibility), OQ-06 (fixture redistribution), and OQ-07 (AAD belongs to `vault-crypto`) remain unchanged and deferred.

## Synthetic test traceability

All tests use deterministic synthetic bytes; no Java fixtures or stored files are used. Large boundary cases are consolidated in one integration test to avoid concurrent 64 MiB fixture pressure.

| Original PREP IDs and family | Committed evidence | Status and limits |
|---|---|---|
| A01–A09 — canonical valid envelopes | `a01_a08_accepts_both_categories_and_variable_synthetic_ciphertexts`; `a03_a04_b17_accepts_reordered_fields_whitespace_escapes_and_exact_numeric_one`; R1 Gson-padding and escaped/unescaped equivalence tests | **PARTIAL** — the listed canonical categories, lengths, ordering, whitespace, escapes, and numeric-one forms are asserted. The original A01–A09 item text was not available for per-ID confirmation. |
| B01–B17 — schema and value restrictions | `b01_b16_rejects_schema_and_value_violations_with_typed_errors`; `b09_b10_b14_rejects_forbidden_fields_before_walking_their_values`; `a03_a04_b17_accepts_reordered_fields_whitespace_escapes_and_exact_numeric_one`; R1 exact-number tests | **PARTIAL** — missing/duplicate/unknown/wrong-type fields, version/algorithm rules, and early rejection have concrete assertions; individual B-case descriptions are not reconstructed. |
| C01–C15 — JSON syntax and text | `c01_c15_rejects_non_strict_json_and_invalid_text` | **PARTIAL** — strict syntax, UTF-8/BOM, trailing content, and surrogate cases are asserted; exact original C-case-to-assertion mapping is unavailable. |
| D01–D10 — Base64 | `d01_d10_e01_e10_enforces_canonical_base64_and_decoded_lengths`; `d09_e_scanner_decodes_escaped_base64_without_changing_its_value`; `d_and_g_reject_invalid_base64_near_the_end_without_panicking`; `gson_style_lowercase_padding_escapes_decode_to_the_original_synthetic_bytes`; `unescaped_base64_round_trips_at_decoder_chunk_boundaries`; `unescaped_base64_rejects_invalid_padding_characters_and_pad_bits_at_boundaries`; `escaped_and_unescaped_gson_forms_decode_identically_across_staging_boundaries` | **PARTIAL** — canonicality, lower-case `\u003d`, both padding counts, staging/chunk boundaries, malformed padding, invalid symbols, pad bits, and decoded-byte equality are asserted. Per-ID PREP wording is not inferred from function names. |
| E01–E10 — nonce and ciphertext lengths | `d01_d10_e01_e10_enforces_canonical_base64_and_decoded_lengths`; `a01_a08_accepts_both_categories_and_variable_synthetic_ciphertexts`; R1 Gson-padding test | **PARTIAL** — nonce 11/12/13 and ciphertext below/at/above 16 bytes plus variable lengths have assertions; exact E-case mapping is unavailable. |
| F01–F09 — complete-file limits and backups | `f01_f09_checks_full_file_limits_before_json_or_output_allocation` | **PASS for file-size assertions** — registry and manifest maximum-minus-one, exact maximum, and maximum-plus-one execute. Backups reuse the corresponding category parser; recovery/selection is **NOT APPLICABLE** to this parser milestone. Individual F-case wording is unavailable. |
| G01–G10 — robustness and resources | `b09_b10_b14_rejects_forbidden_fields_before_walking_their_values`; `f01_f09_checks_full_file_limits_before_json_or_output_allocation`; `d_and_g_reject_invalid_base64_near_the_end_without_panicking`; R1 large-number and Base64-boundary tests | **PARTIAL** — large accepted-size values, early rejection, near-end malformed input, exponent cancellation, and bounded chunk paths are exercised; no per-ID claim or timing assertion is made. |
| H01–H07 — diagnostic confidentiality | `h_and_diagnostics_are_redacted_and_accessors_borrow` | **PARTIAL** — `Debug` and error redaction plus borrowed ciphertext access are tested; exact original H-case wording is unavailable. |
| I01–I06 — Phase 2A-2 regression and architecture | `cargo test -p vault-format --locked` including `header_parser.rs`; R1 `preserves_negative_zero_policy_for_header_version_and_kdf_values`; architecture regression tests | **PARTIAL** — existing header API/tests, negative-zero policy, and architecture checks pass; exact I-case mapping is unavailable. |
| J01–J14 — CI and independent-review gates | Exact-SHA hosted workflow steps in the R1 final report; independent Claude review and owner disposition | **PARTIAL** — executable CI gates are reported from the exact R1 run; independent review and owner approval remain **UNVERIFIED/PENDING**. The previous run 42102302758 belongs to defective pre-R1 SHA `102b482940d2edb44a88ad479b9599f9da6429f7` and does not validate this correction. |

This traceability table uses only the owner-supplied identifier ranges and family descriptions. It does not assert that a test passes merely because its name contains an identifier, and it does not invent unavailable individual case descriptions. D6 continues to mean the PREP's JSON nesting policy; the early-rejection rule is traced separately as the additional engineering condition.

## Initial Phase 2A-3 local validation and resource observations (pre-R1)

Local validation completed on Windows MSVC with Rust/Cargo `1.99.1-x86_64-pc-windows-msvc`:

- Initial TDD red check: the focused integration target failed to compile because the new public API did not yet exist; this confirmed the test precondition.
- `cargo test -p vault-format --test external_envelope_parser --locked`: passed after implementation.
- `cargo test -p vault-format --locked`: passed 13 unit tests, 10 external-envelope integration tests, and 31 existing header-parser tests (54 total, plus doctests).
- `cargo clippy -p vault-format --all-targets --locked -- -D warnings`: passed after correcting the reported style findings.
- `cargo test --release -p vault-format --test external_envelope_parser --locked --no-run`: passed.
- A Release test-process measurement of the sequential large-boundary scenario passed in 0.39 seconds, with an approximately 116.5 MiB sampled peak working set (23 samples at 10 ms intervals). The process included maximum 16/64 MiB inputs and outputs, large escaped Base64, and large invalid Base64. This is an observational process-level sample, not a formal memory or timing bound.

The complete local gates passed as follows:

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace --locked` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --locked` | PASS — 103 tests across workspace, including 10 external-envelope and 31 header-parser tests |
| `cargo check --workspace --features design-preview --locked` | PASS |
| `cargo clippy --workspace --features design-preview --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --features design-preview --locked` | PASS — 194 tests across workspace, including 10 external-envelope and 31 header-parser tests |
| `pwsh ./scripts/verify-architecture.ps1` | PASS |
| Architecture regression suite | PASS |
| Build-manifest rejection suite | PASS, with the junction subcase BLOCKED by `NewItemCreateIOError` |
| Shell/preview isolation suite | PASS, 42 negative cases rejected; reparse-point junction subcase BLOCKED by `NewItemCreateIOError` |
| Dependency inventory suite | PASS |
| PowerShell wrapper suite | PASS |
| Windows Release shell and design-preview builds | PASS; separate target directories and artifacts verified |
| Native PE dependency inventory | PASS; 27 observed PE imports per executable |
| `git diff --check` | PASS |

The first local Windows Release run used `phase-2a-3-local-precommit-20261105`; its build manifest correctly recorded the starting Git SHA and `dirty=true` because implementation files were uncommitted. It produced a verified manifest and dependency inventory, but those executables are not represented as a clean committed-build identity. After the implementation commit, clean Release run `phase-2a-3-e13e5ed-clean` recorded source SHA `e13e5ed22475e22c2641a86c8d994878427868c7` and `dirty=false`. Both isolated variants and the native dependency inventory verified:

| Variant | Size | Raw executable SHA-256 | Observed PE imports |
|---|---:|---|---:|
| Shell | 14,875,648 bytes | `E2254C5F67D933BF0E8016B88BF646AAD306F209F76FB288557A9B51F403E9EF` | 27 |
| Design preview | 15,535,616 bytes | `3146CC74A2F433CA8BA62B2FB4EAA6E8F827A101048035ED1C114BDF5FD31179` | 27 |

The inventory reported 377 Cargo.lock package records and 161 Windows-resolved packages for each configuration. The build manifest records linker detection but does not prove linker invocation (`usage_proven=false`). These are local build observations, not reproducibility, clean-host, runtime-redistribution, installer, or release certification. The release boundary test above was run separately and passed. The two junction/reparse-point subcases could not run locally because this Windows environment denied junction creation; they remain BLOCKED locally, not passed. Hosted Windows CI must provide separate evidence for them.

The repository's `.git` metadata required one-command `safe.directory` overrides due directory ownership; no global Git setting was changed. Git emitted line-ending normalization warnings on modified LF working copies, while `diff --numstat` showed localized edits rather than whole-file newline churn; no whole-file line-ending churn was observed.

## Initial hosted dev CI evidence (pre-R1)

The first published implementation/evidence snapshot passed [Windows Rust CI run 42100435684](https://github.com/Fabianrodas/encryptdrive-desktop/actions/runs/42100435684). It was a `push` event on `dev`, workflow `.github/workflows/ci.yml`, exact head SHA `ecd1bf7771ff96bbb751bc93de7fa739907c421e`, run attempt 1, with job `validate` concluding **SUCCESS**. Every workflow step passed: format, locked default and preview checks, strict Clippy, both workspace test runs, PowerShell tests, isolation and architecture checks, both separate Release variants, artifact verification, and native dependency inventory.

Job logs confirm `f01_f09_checks_full_file_limits_before_json_or_output_allocation` passed in both default and `design-preview` test runs (the test includes maximum-minus-one, exact maximum, and maximum-plus-one cases for both file categories). The hosted isolation suite rejected 43 negative cases and passed its junction/reparse-point case. Hosted Release artifacts were built from `ecd1bf7771ff96bbb751bc93de7fa739907c421e` with `dirty=false`; both variants verified and the native inventory collected 27 PE imports per executable. The workflow reported 377 lockfile records and 161 Windows-resolved packages per variant. Hosted artifact hashes differ from the local build and are not reproduced as local identity claims here.

The subsequent pre-R1 evidence-only commit `102b482940d2edb44a88ad479b9599f9da6429f7` also passed [Windows Rust CI run 42102302758](https://github.com/Fabianrodas/encryptdrive-desktop/actions/runs/42102302758). That run was for the pre-R1 implementation later rejected by independent review. Neither run 42100435684 nor 42102302758 validates the R1 correction below. No workflow change or manual rerun was made for those runs.

The direct dependency configuration remains `serde 1.0.229`, `serde_json 1.0.154` (including existing `std` and `arbitrary_precision` features), `base64 0.23.1`, and `time 0.3.55`; no dependency, feature, Cargo manifest, or Cargo.lock change was made. The effective `vault-format` feature graph was inspected offline and showed only the existing approved features. Cargo.lock's SHA-256 remained `8718BAF81D753669F35C348C3D5248489879C639EC60CA8639EF11FFD409F9A2`. Rust 1.95 has not been installed or run; direct MSRV validation remains **UNVERIFIED**.

## R1 focused correction

Independent review of pre-R1 `dev` SHA `102b482940d2edb44a88ad479b9599f9da6429f7` returned **REJECTED — MATERIAL CORRECTION REQUIRED**. This section records the single authorized R1 remediation. It does not reopen D1–D12, redesign the parser, or authorize `main` integration or a later phase.

### P1-01 — exact exponent classification

The prior helper saturated every exponent at `1_000_000`, inherited from work whose complete input ceiling was 256 KiB. Reuse for 16 MiB registry and 64 MiB manifest envelopes invalidated that assumption: a bounded mantissa may itself exceed one million digits. Before the fix, the new public-API regression ran against the unchanged pre-R1 production helper and observed, sequentially, `(Ok(()), Ok(()), Err(VersionOutOfRange))` for the three cases below. The expected results are shown alongside them:

| Synthetic version token | Mathematical value | Pre-R1 observed | Required/corrected result |
|---|---:|---|---|
| `0.` + 999,999 zeroes + `1e2000000` | `10^1,000,000` | Accepted as version 1 | `VersionOutOfRange` |
| `1` + 1,000,000 zeroes + `e-2000000` | `10^-1,000,000` | Accepted as version 1 | `NonIntegralVersion` |
| `1` + 1,500,000 zeroes + `e-1500000` | `1` | Rejected as out of range | Accepted as version 1 |

The correction parses a signed exponent as a bounded magnitude derived from the actual mantissa digit count `D`: `D + 21`, using checked arithmetic. This bound is a classification threshold, not a claim about the true exponent. A positive magnitude at or beyond it must append at least 21 zeroes after subtracting the fractional scale (which is at most `D`); any nonzero mantissa then exceeds the 20-decimal-digit width of `u64`. A negative magnitude at or beyond it requires a scale greater than `D`, so a nonzero mantissa cannot be integral. Larger exponents have those same classifications; the helper still validates every exponent character and never treats the truncated magnitude as the mathematical value. Values below the threshold retain their full exact exponent and are classified without `f64`, rounding, BigInt, or an allocated decimal copy. The scale calculation uses `i128` and checked conversions/arithmetic; zero mantissas remain mathematically zero, while the external version policy still rejects zero and negative zero.

Permanent public-API tests now cover all three reproductions, accepted equivalents `1`, `1.0`, `1e0`, `1.00e+0`, `0.1e1`, and `100e-2`, positive/negative million and million-plus-one exponents, exponents beyond signed-integer range, cancelling and partially cancelling long mantissas, values at and just above `u64::MAX`, large zero with signed exponents, negative-zero policy, and malformed exponent JSON. The old behavior was observed before changing production code; the corrected implementation passes the same test.

The shared `parse_vault_header` API and `header.rs` were not modified. Existing header tests still pass, and a focused regression now records the prior negative-zero policy for format version and KDF iterations. Exact integer, fraction, overflow, KDF range, timestamps/leap-second and wrapped-key assertions remain in the existing Phase 2A-2 suite.

### P2-01 — PREP matrix traceability

The inaccurate statement that `A01–J14` was absent has been removed. The table above now records owner-supplied ranges A01–A09 through J01–J14, their approved family descriptions, committed assertion evidence, and a status. It does not infer individual case wording from function names. D6 remains JSON nesting; early rejection remains a separate additional implementation condition. Where the exact numbered PREP item text is unavailable to this implementation context, the table marks the mapping **PARTIAL** instead of claiming a complete per-ID audit.

### P2-02 and P2-03 — Gson padding and Base64 regressions

Java source inspection confirms `UserRegistryRepository` and `ManifestRepository` use default `new Gson()` instances to serialize `aes.encrypt(...)` envelopes. Gson's default HTML-safe writer can represent standard Base64 padding with lowercase JSON escapes `\u003d`. Rust must decode JSON escapes before validating canonical Base64; a literal `=` and its `\u003d` JSON spelling represent the same canonical decoded Base64. This structural check requires no GCM operation and does not establish full Java/Rust interoperability.

New committed synthetic tests cover 15, 16, 17, 1,000, 1,001, and 262,145 ciphertext bytes, including one- and two-padding-character output, exact decoded-byte and nonce equality, and a larger escaped value crossing staging chunks. Unescaped round trips cover encoded lengths 4,088, 4,092, 4,096, 8,184, 8,188, and 8,192 characters with one-, two-, and zero-padding cases. Separate assertions reject padding before remaining encoded characters, an invalid symbol at a chunk boundary, and noncanonical final pad bits. Escaped and unescaped forms of the same synthetic ciphertext are compared byte-for-byte. The production Base64 decoder was not changed; its borrowed fast path and bounded staging path remain subject to source review rather than an allocation-instrumentation claim.

### R1 validation and disposition

The R1 local validation is separate from the pre-R1 observations above. On Windows MSVC with Rust/Cargo `1.99.1-x86_64-pc-windows-msvc`, the focused `vault-format` suite passed 13 unit, 16 external-envelope, and 32 header-parser tests (61 total). The full workspace passed 110 tests in the default configuration and 201 tests with `design-preview`; both runs include the R1 exponent and Base64 regressions. Formatting, locked workspace checks, strict Clippy, and both test configurations passed.

| R1 check | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace --locked` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --locked` | PASS — 110 tests; 16 external-envelope and 32 header-parser tests |
| `cargo check --workspace --features design-preview --locked` | PASS |
| `cargo clippy --workspace --features design-preview --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --features design-preview --locked` | PASS — 201 tests; 16 external-envelope and 32 header-parser tests |
| Architecture validation and regression suites | PASS |
| Dependency inventory structure and native import collection | PASS — 377 lockfile records, 161 Windows-resolved packages per configuration, 27 observed PE imports per executable |
| Build-manifest verification | PASS — shell and design-preview variants |
| PowerShell wrapper and dependency-inventory regression suites | PASS |
| Build-manifest rejection and shell/preview isolation suites | PASS; local junction-creation subcases BLOCKED by `NewItemCreateIOError` |
| `git diff --check` | PASS |

The local Release run used ID `phase-2a-3-r1-local-20261105a`. Its manifest reports the starting revision `102b482940d2edb44a88ad479b9599f9da6429f7` and `dirty=true`, because it includes the uncommitted R1 working-tree changes. It is not a clean committed-build identity. The verified local outputs were shell 14,875,648 bytes (SHA-256 `39EEFEB57F15CDD0DB6D4D4AB397EE0C045A5B7C15DA48BF26FB3BE0F64B9A72`) and design-preview 15,535,616 bytes (SHA-256 `3EBEF3F83CD1DDE8AC8DE6F912D5A1992B72F477C49F16E7DD8E58F9F021FBC2`); each had 27 observed PE imports. These local measurements do not establish reproducible builds, clean-host behavior, runtime redistribution, or release readiness.

The previous hosted runs 42100435684 (`ecd1bf...`) and 42102302758 (`102b482...`) are pre-R1 and are not evidence for this correction. The exact R1 commit's hosted CI result is recorded in the task handoff; that workflow must match the final pushed R1 SHA. Independent Claude R1 review and owner approval remain pending. No integration into `main`, cryptography, storage, filesystem behavior, or Phase 2A-4/2A-5 work is authorized.

## Deferred findings and later gates

The following historical statuses are preserved: Phase 1B P2-F01 through P2-F04 and its compact-layout residual observation remain open; Windows N01–N09 remain **NOT RUN / UNVERIFIED** and B01–B04 remain **BLOCKED**. Direct Rust 1.95 compilation, full differential Java/Rust interoperability, and complete VF1 conformance remain **UNVERIFIED**. No release readiness is claimed.

This milestone does not resolve OQ-03, OQ-04, or OQ-06. OQ-07 remains assigned to `vault-crypto`. Full VF1 conformance, cryptographic reviews, authentication, storage, Windows distribution, and release gates remain later work.

## Publication and review gate

R1 is a single focused commit descending from pre-R1 `dev` SHA `102b482940d2edb44a88ad479b9599f9da6429f7`; its hosted CI must match the final published R1 SHA. `main` remains at `438b17a168eef88e067613796821c991086901f9`. The exact run URL, job, steps and large-boundary test result are in the R1 task handoff.

Claude's R1 review range is exactly `102b482940d2edb44a88ad479b9599f9da6429f7` through the new R1 `dev` SHA. Review should focus on the context-derived exponent threshold and three counterexamples, exact values around u64 and signed exponent boundaries, Phase 2A-2 semantics, Gson lowercase escaped padding, Base64 chunk/padding/pad-bit tests, PREP traceability, and scope/dependency boundaries. Independent review and owner approval are still required before any `main` integration. Phase 2A-4 and Phase 2A-5 remain unauthorized.
