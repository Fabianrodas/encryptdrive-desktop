# Phase 2A-2 — Strict VF1 Header Parser

**Status:** R1 correction recorded; CI evidence is commit-specific; independent re-review pending
**Recorded:** 2026-11-05
**Starting baseline:** `f651632f931d1cce035f6b227596c95cc7840156` on `dev`
**Implementation commit:** `c2930f69a6b2a77fc1fb96096eb3b7d9da55e851` (parent: `f651632f931d1cce035f6b227596c95cc7840156`)
**Authorization:** Owner-approved Phase 2A-2 parser implementation. This record does not authorize main integration or Phase 2A-3.

## Result and boundary

`vault-format` now parses caller-provided `vault.json` bytes into `ParsedUnauthenticatedVaultHeader` through `parse_vault_header`. The parser validates a bounded, strict JSON document and the currently supported header's structural and semantic fields. It reuses `LengthCheckedInput` and `check_input_length` for the inclusive 262,144-byte complete-file limit before parsing.

The result is explicitly **unauthenticated**. Parsing does not prove that a password is correct, that the wrapped key decrypts, that the vault is authentic, that encrypted data is recoverable, or that Java and Rust cryptographic operations interoperate. No filesystem or network access, KDF execution, AES-GCM operation, authentication, or real-vault use was added.

## Parsing and compatibility policy

The parser uses a bounded structural scanner rather than building a complete JSON DOM. It validates the full document before making schema or version decisions, retains spans only for known root fields, and discards validated unknown root values. Unknown root fields are allowed. The `kdf` and `wrappedRegistryKey` objects have exact schemas and reject additional fields.

The scanner rejects a BOM, invalid UTF-8, malformed or trailing JSON, and duplicate property names after JSON string decoding in every object, including objects under unknown values and arrays. It counts nested objects and arrays with the root at depth 1; depth 32 is accepted and depth 33 is rejected. Strings are checked for valid escape and surrogate-pair structure. No extra member-count or unknown-string-size limit was added beyond the complete-file bound.

Known fields are validated as follows:

- `formatVersion` accepts only the supported version `1`; zero and unsupported future versions have distinct errors.
- `vaultId` must be canonical lowercase 8-4-4-4-12 UUID text. The parser does not require UUID version 4.
- `createdAt` must be a real UTC timestamp in the approved canonical form, with uppercase `T`/`Z`, seconds `00`–`59`, and zero through nine fractional digits. Seconds are range-checked before parsing so leap seconds cannot be normalized by the time library.
- Stored KDF metadata must name Argon2id and satisfy the approved stored ranges: parallelism 1–8, iterations 1–10, memory from `8 × parallelism` through 1,048,576 KiB, and a 16-byte salt. No KDF is run.
- `wrappedRegistryKey` requires version 1 and `AES/GCM/NoPadding`, with a 12-byte nonce and 48-byte ciphertext-plus-tag. No GCM operation is run.
- Base64 uses the standard alphabet and canonical required padding; whitespace, URL-safe forms, unpadded forms, and noncanonical pad bits are rejected.

The numeric path preserves mathematical integer semantics for integer, decimal, and exponent JSON representations without conversion through `f64`. The strict scanner first identifies a number token; `serde_json::Number` with `arbitrary_precision` is used only on that scalar token span, and the exact decimal/exponent value is range-checked. This avoids interpreting an ordinary user property named `$serde_json::private::Number` as serde's internal number representation. Numeric values in unknown subtrees are still syntax-validated by the scanner.

This implements the approved strict Rust policy, not every permissive behavior of the historical Java reader. In particular, Rust rejects duplicate keys and extra KDF/envelope fields and requires canonical Base64. The contract was preflighted against synthetic cases and Java's documented/writer contract; Java reader differential testing with shared fixtures remains pending. No Java fixtures or private documentation were copied into this repository.

## API, errors, and data exposure

The public entry point is `parse_vault_header(&[u8]) -> Result<ParsedUnauthenticatedVaultHeader, HeaderParseError>`. The returned type exposes the supported version, canonical `VaultId`, timestamp, stored KDF parameters, and unauthenticated encrypted-envelope bytes through typed accessors. The error enum distinguishes size, UTF-8/BOM/JSON, duplicate/schema, version, number, identifier, timestamp, KDF, Base64, envelope, and nesting failures. Error diagnostics do not embed raw JSON. Parsed-header `Debug` redacts the vault identifier and timestamp; nested KDF/envelope `Debug` reports only fixed structural metadata and lengths, not salt, nonce, or ciphertext bytes. `VaultId`'s explicit `Display` and the typed accessors continue to expose their intended values; callers that explicitly format or log those values can disclose them.

## Dependencies and architecture validation

Direct dependencies are exactly:

| Crate | Version | Features / defaults |
|---|---:|---|
| `serde` | 1.0.229 | default features disabled |
| `serde_json` | 1.0.154 | default features disabled; `std`, `arbitrary_precision` |
| `base64` | 0.23.1 | default features disabled; `std` |
| `time` | 0.3.55 | default features disabled; `std`, `parsing` |

No `unbounded_depth`, `preserve_order`, Base64 `simd-unsafe`, UUID, crypto, storage, or I/O dependency was introduced. The PowerShell architecture validator reads `cargo metadata --no-deps` and enforces the declared direct dependency versions, default-feature settings, feature lists, and crate boundaries. It does not calculate effective feature unification. The resolved `vault-format` graph was separately inspected with `cargo tree -p vault-format -e features --locked`; that inspection confirmed the approved effective features and absence of the prohibited features. Locked CI checks compile and test the resolved lockfile, while its architecture regression suite checks the validator's declared-manifest enforcement. These checks have distinct roles.

`Cargo.lock` adds the approved direct dependencies and their resolved transitive packages (including `serde_json`, `base64`, and `time` dependency trees); it does not add an unrelated direct dependency. Declared dependency license metadata was inventoried. This is not a legal approval or redistribution decision. The repository MSRV remains Rust 1.95; validation used Rust/Cargo 1.99.1 because 1.95 was not installed, so direct execution on the declared MSRV remains unverified. Dependency metadata reports no new package with a declared MSRV above 1.88.

## Validation evidence

Implementation commit `c2930f69a6b2a77fc1fb96096eb3b7d9da55e851` is the exact `dev` push SHA. Hosted [Windows Rust CI run 41971936229](https://github.com/Fabianrodas/encryptdrive-desktop/actions/runs/41971936229) was a `push` event on `dev`, completed with **SUCCESS** for that SHA. Its single `validate` job passed formatting, locked default and design-preview checks, strict Clippy, both test suites, PowerShell rejection/wrapper/dependency/isolation/architecture tests, both isolated Windows Release variants, artifact-manifest verification, native dependency inventory, and workspace-boundary validation.

Local validation used Rust/Cargo 1.99.1 (`x86_64-pc-windows-msvc`). The focused `vault-format` suite passed 40 tests, including 28 header-parser tests. Default workspace tests passed 89 tests; `design-preview` workspace tests passed 180 tests. Formatting, locked workspace checks, strict Clippy in both feature configurations, architecture validation, shell/preview isolation, dependency inventory, build-manifest and PowerShell wrapper suites passed. The local Windows junction/reparse fixture could not be created (`NewItemCreateIOError`), so that local subcheck remains **BLOCKED**; hosted CI's corresponding test steps passed.

The clean committed Windows Release verification used run identity `phase-2a-2-c2930f6` (`dirty=False`):

| Variant | Size | SHA-256 | Observed native PE imports |
|---|---:|---|---:|
| Default shell | 14,875,648 bytes | `3A9A579CF3FAC1BCE4ABCC42B72FE45DB2E4247BA8D67C2E8CFE82AD513E5684` | 27 |
| Design preview | 15,535,616 bytes | `0696A41D9C2CDCB7ECAB5B7416BF5BCFEF8FA286106F0316CFD15DBD11D8E091` | 27 |

The dependency inventory includes `vcruntime140.dll`; this evidence is not runtime-redistribution or clean-host certification.

Synthetic release measurements used 10 warmups and 300 iterations per case. They are approximate process-level observations, not parser-only bounds or timing guarantees:

| Scenario | Input | Mean | p95 | Observed working-set delta |
|---|---:|---:|---:|---:|
| Maximum valid header | 262,144 bytes | 169 µs | 196 µs | ~98 KB |
| Many distinct unknown properties | 205,281 bytes / 12,000 keys | 3,091 µs | 5,258 µs | ~1.06 MiB |
| Depth 32 | 475 bytes | 5 µs | 7 µs | ~106 KB |
| Malformed duplicate-heavy content | 205,296 bytes | 3,362 µs | 6,358 µs | ~209 KB |
| Long numeric token | 200,390 bytes | 1,026 µs | 1,397 µs | ~197 KB |

Unknown values are discarded rather than materialized as a tree. The duplicate-key sets still allocate in proportion to unique object keys; the many-property case was the largest observed allocation pattern. These measurements do not prove a universal upper memory bound and do not justify an additional arbitrary member limit.

## Deferred work and phase boundary

This milestone does not parse the registry or account manifest, implement read/write semantics, construct AAD, derive keys, decrypt, authenticate, store data, or open vaults. Those responsibilities remain outside 2A-2; any new parser, compatibility, or data-flow contract requires its authorized milestone and review. Java/Rust cryptographic interoperability and full VF1 conformance are not established here.

Independent review of the complete diff from `f651632f931d1cce035f6b227596c95cc7840156` remains pending. The following Phase 1B findings remain open and unchanged: P2-F01 through P2-F04 and the compact-layout residual observation. Windows gates N01–N09 remain **NOT RUN / UNVERIFIED**; B01–B04 remain **BLOCKED**. No Windows accessibility, release, clean-host, or distribution certification is claimed. Phase 2A-2 success does not authorize main integration or begin Phase 2A-3.

## Phase 2A-2 R1 — Focused parser correction

**Recorded:** 2026-11-05
**Scope:** `header.rs`, `header_parser.rs`, and this document only.

### Timestamp normalization correction (P1-01)

The original lexical shape check established that the seconds characters were decimal digits but did not restrict their value. As a result, `time` 0.3.55 accepted leap-second strings at valid month ends and normalized them to the final nanosecond of the preceding second. R1 now reads the already shape-checked seconds bytes and rejects values above 59 before calling the time parser; it does not normalize the input.

Public-API regression coverage rejects `2016-12-31T23:59:60Z`, `2016-12-31T23:59:60.5Z`, `2026-11-30T23:59:60Z`, `2026-11-30T23:59:60.5Z`, `2026-06-30T23:59:60Z`, `2026-11-04T23:59:60Z`, and seconds `61` and `99`. Adjacent valid values `2016-12-31T23:59:59Z`, `2016-12-31T23:59:59.999999999Z`, and `2026-11-30T23:59:59Z` remain accepted. Existing invalid-date, offset, separator, precision, missing-seconds, `24:00`, and missing-`Z` checks remain in place.

The new leap-second test was first run against the pre-correction implementation and failed with `Ok`; its parsed `created_at` was `2016-12-31 23:59:59.999999999 +00:00:00`. It then passed with the lexical range check. This public-API red/green reproduction demonstrates the regression test catches the original defect. No separate mutation harness was run.

### Diagnostic redaction (P2-01)

`VaultId`'s `Debug` now emits only a redaction marker; canonical `Display`, equality, and the byte accessor are unchanged. Parsed-header `Debug` redacts both `vault_id` and `created_at`, retaining format version and safe nested structural metadata. Synthetic regression coverage checks header, identifier, KDF, and envelope `Debug`, while confirming canonical UUID display and timestamp/UUID accessors still return the parsed values. This prevents automatic `Debug` disclosure; it does not prevent callers from explicitly logging values obtained from public accessors.

### Validation-scope and timestamp documentation (P2-02/P2-03)

The dependency section above now separates `cargo metadata --no-deps` manifest enforcement, the explicit `cargo tree -p vault-format -e features --locked` effective-feature inspection, and locked CI compilation/tests. The timestamp contract now records the `00`–`59` seconds range and pre-parse leap-second rejection.

The previously recorded hosted run `41974411830` belongs to the pre-correction SHA `924cd8254808823793a46ae324d86c799b4c7af7`; it is not evidence for R1. The R1 hosted result must be associated with the correction commit's exact SHA and run.
