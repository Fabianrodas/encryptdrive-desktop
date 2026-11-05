# Phase 2A-3 — External Encrypted Envelope Parser

**Status:** Implementation and hosted CI passed for `ecd1bf7771ff96bbb751bc93de7fa739907c421e`; this post-run evidence update requires its own exact-SHA CI run
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
| D6 — nesting policy | The existing maximum of 32 nested JSON containers remains unchanged for the general/header parser. The external exact-schema scanner rejects forbidden or wrongly typed fields before traversing their values, so it never recursively walks those rejected values. |
| D7 — shared parser | Reuse the existing structural scanner and exact-number support, while keeping external-envelope early rejection separate from `vault.json`'s allowed unknown root fields. |
| D8 — category-safe APIs | `parse_encrypted_user_registry` and `parse_encrypted_account_manifest` select their own fixed limits and delegate to one private implementation. |
| D9 — ownership | The returned type owns one decoded ciphertext-plus-tag `Vec`, exposes borrowed accessors, and is not `Clone`. Nonce is a fixed 12-byte value. |
| D10 — errors | Typed errors are sanitized and never include source bytes, attacker-supplied field names, nonce or Base64 fragments. Debug identifies the result as unauthenticated and reports lengths only. |
| D11 — dependencies | No dependency, manifest, feature, lockfile, or architecture allowlist changes. Existing dependency configuration is retained. |
| D12 — compatibility/testing | Synthetic contract tests and the complete Phase 2A-2 regression suite are required. Java writer behavior is the compatibility target; intentional strict-reader divergences and unverified interoperability are stated below. |

Two additional owner-approved engineering conditions are applied: (1) safely reject forbidden fields after validating the minimum key/colon syntax, without materializing their names or traversing their values; and (2) avoid input-sized encoded-string copies during large Base64 decoding.

### Specification traceability note

The available Phase 2A-3 PREP document groups its proposed acceptance cases under lettered headings A–H, but does not define a literal `A01–J14` identifier catalog. The implementation prompt refers to that catalog and its D6 label also calls early rejection “D6,” while PREP uses D6 for JSON nesting. To avoid inventing a supposedly pre-approved mapping, the test matrix below cites the published rule groups and exact in-tree test names. The approved behavior itself is implemented and tested; the ID-numbering discrepancy is an audit/documentation follow-up, not a parser behavior change.

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
- `UserRegistryRepository.MAX_REGISTRY_BYTES` is 16 MiB and `ManifestRepository.MAX_MANIFEST_BYTES` is 64 MiB; the matching backup bytes use those repository limits.

These source observations support compatibility with canonical Java writer output. Rust deliberately rejects BOMs, invalid UTF-8, and noncanonical Base64 forms even if some Java reader paths may be more permissive. There was no Java app execution, fixture import, real-vault parsing, or differential Java/Rust run. Full interoperability remains **UNVERIFIED**. PREP questions OQ-03 (writer-reader content rules), OQ-04 (metadata UUID compatibility), OQ-06 (fixture redistribution), and OQ-07 (AAD belongs to `vault-crypto`) remain unchanged and deferred.

## Synthetic test traceability

All tests use deterministic synthetic bytes; no Java fixtures or stored files are used. Large boundary cases are consolidated in one integration test to avoid concurrent 64 MiB fixture pressure.

| Contract group | Executed test(s) | Coverage/result |
|---|---|---|
| A — canonical envelopes | `a01_a08_accepts_both_categories_and_variable_synthetic_ciphertexts`; `a03_a04_b17_accepts_reordered_fields_whitespace_escapes_and_exact_numeric_one` | Both category wrappers, ciphertext lengths 16/17/48/257, field order, JSON whitespace, escaped known key, exact numeric forms: passed. |
| B — schema and values | `b01_b16_rejects_schema_and_value_violations_with_typed_errors`; `b09_b10_b14_rejects_forbidden_fields_before_walking_their_values` | Missing/duplicate/escaped duplicate, unknown fields, wrong types, null, versions, algorithm, early unknown-field rejection, long plain/escaped keys, malformed/deep rejected values: passed. |
| C — JSON syntax | `c01_c15_rejects_non_strict_json_and_invalid_text` | Empty/whitespace input, root array, trailing comma, single quotes, unquoted key, comments, nonstandard whitespace, trailing text/document, BOM, invalid/truncated UTF-8, and isolated surrogate: passed. |
| D — canonical Base64 | `d01_d10_e01_e10_enforces_canonical_base64_and_decoded_lengths`; `d09_e_scanner_decodes_escaped_base64_without_changing_its_value`; `d_and_g_reject_invalid_base64_near_the_end_without_panicking`; unit `escaped_base64_quartets_survive_the_staging_carry_boundary` | Padding, pad bits, URL-safe, whitespace, invalid symbols, escaped ASCII/slash/padding, invalid near-end large data, chunk carry and no-panic repeated malformed input: passed. |
| E — nonce/ciphertext | `d01_d10_e01_e10_enforces_canonical_base64_and_decoded_lengths`; `a01_a08_accepts_both_categories_and_variable_synthetic_ciphertexts` | Nonce 11/12/13 and invalid encoding; ciphertext 0/1/15 rejected, 16/17/48 and larger variable values accepted structurally: passed. |
| F — complete-file bounds | `f01_f09_checks_full_file_limits_before_json_or_output_allocation` | Both wrappers accept maximum-minus-one and exact maximum valid envelopes, reject maximum-plus-one, prove registry wrapper rejects input accepted by manifest wrapper, and exercise the larger category: passed. Backup behavior is category reuse only; no recovery is implemented. |
| G — robustness/resources | `b09_b10_b14_rejects_forbidden_fields_before_walking_their_values`; `f01_f09_checks_full_file_limits_before_json_or_output_allocation`; `d_and_g_reject_invalid_base64_near_the_end_without_panicking` | Million-byte unknown key, 100k escaped key, deeply nested/malformed forbidden value, 100k-digit exact number, 1M escaped Base64, near-end invalid Base64, overflow-safe paths and repeat/no panic: passed. |
| H — confidentiality | `h_and_diagnostics_are_redacted_and_accessors_borrow` | Parsed `Debug` omits nonce/ciphertext; errors omit unknown and invalid-algorithm sentinels; ciphertext accessor returns a borrowed slice: passed. |
| I — Phase 2A-2 regression | `cargo test -p vault-format --locked` including all existing `header_parser.rs` cases | Existing header parser suite passed unchanged, including depth, unknown root extensions, strict syntax, exact numbers, timestamp, redaction and fixed 48-byte wrapped key requirements. |
| J — API/trust boundary | `a01_a08_accepts_both_categories_and_variable_synthetic_ciphertexts`; source/API review | Public wrappers bind category limits; result type is explicitly unauthenticated and non-`Clone`; no path, I/O, crypto, or plaintext API: passed by compile/tests and source review. |

The requirement refers to `A01–J14`, but the available PREP does not define those numeric IDs or a J-group numbering. The test function names retain the task's in-tree A–H trace labels, and I/J above are implementation traceability groups, not a claim that these identifiers appeared in the earlier PREP. This should be reconciled during independent review if exact historical numbering is required.

## Local validation and resource observations

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

## Hosted dev CI evidence

The first published implementation/evidence snapshot passed [Windows Rust CI run 42100435684](https://github.com/Fabianrodas/encryptdrive-desktop/actions/runs/42100435684). It was a `push` event on `dev`, workflow `.github/workflows/ci.yml`, exact head SHA `ecd1bf7771ff96bbb751bc93de7fa739907c421e`, run attempt 1, with job `validate` concluding **SUCCESS**. Every workflow step passed: format, locked default and preview checks, strict Clippy, both workspace test runs, PowerShell tests, isolation and architecture checks, both separate Release variants, artifact verification, and native dependency inventory.

Job logs confirm `f01_f09_checks_full_file_limits_before_json_or_output_allocation` passed in both default and `design-preview` test runs (the test includes maximum-minus-one, exact maximum, and maximum-plus-one cases for both file categories). The hosted isolation suite rejected 43 negative cases and passed its junction/reparse-point case. Hosted Release artifacts were built from `ecd1bf7771ff96bbb751bc93de7fa739907c421e` with `dirty=false`; both variants verified and the native inventory collected 27 PE imports per executable. The workflow reported 377 lockfile records and 161 Windows-resolved packages per variant. Hosted artifact hashes differ from the local build and are not reproduced as local identity claims here.

This CI run validates the code and roadmap at `ecd1bf...`. The current commit adds only this run evidence to the roadmap; it is a new final `dev` SHA and must receive its own hosted run before this milestone's exact-SHA publication gate is considered complete. No workflow change or manual rerun was made.

The direct dependency configuration remains `serde 1.0.229`, `serde_json 1.0.154` (including existing `std` and `arbitrary_precision` features), `base64 0.23.1`, and `time 0.3.55`; no dependency, feature, Cargo manifest, or Cargo.lock change was made. The effective `vault-format` feature graph was inspected offline and showed only the existing approved features. Cargo.lock's SHA-256 remained `8718BAF81D753669F35C348C3D5248489879C639EC60CA8639EF11FFD409F9A2`. Rust 1.95 has not been installed or run; direct MSRV validation remains **UNVERIFIED**.

## Deferred findings and later gates

The following historical statuses are preserved: Phase 1B P2-F01 through P2-F04 and its compact-layout residual observation remain open; Windows N01–N09 remain **NOT RUN / UNVERIFIED** and B01–B04 remain **BLOCKED**. Direct Rust 1.95 compilation, full differential Java/Rust interoperability, and complete VF1 conformance remain **UNVERIFIED**. No release readiness is claimed.

This milestone does not resolve OQ-03, OQ-04, or OQ-06. OQ-07 remains assigned to `vault-crypto`. Full VF1 conformance, cryptographic reviews, authentication, storage, Windows distribution, and release gates remain later work.

## Publication and review gate

The implementation and initial validation record are published on the existing `dev` branch. This post-run evidence update is the next documentation-only commit. After it is pushed, the exact new `dev` SHA must receive its own normal hosted Windows Rust CI run, including the large-boundary test; run 42100435684 is evidence for `ecd1bf...`, not for the later document commit. `main` remains at the starting baseline.

After hosted CI passes for the final documentation commit's exact SHA, this implementation is ready for independent Claude Code review from baseline `438b17a168eef88e067613796821c991086901f9` through that exact published `dev` SHA. Review should focus on early-rejection safety, complete accepted-document validation, exact number semantics, Base64 canonicality and escaped chunk carry, allocation ordering and overflow, category limits, sanitized diagnostics, and preservation of the Phase 2A-2 header contract. Independent review and owner approval are still required before any `main` integration. Phase 2A-4 and Phase 2A-5 remain unauthorized.
