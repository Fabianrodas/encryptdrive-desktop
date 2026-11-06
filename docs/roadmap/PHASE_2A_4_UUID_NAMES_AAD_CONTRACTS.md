# Phase 2A-4 — UUID, Logical-Name and AAD Contracts

**Status:** Implemented on `dev`; exact-SHA hosted CI, independent Claude review and owner disposition are pending. This record does not authorize integration into `main`.
**Recorded:** 2026-11-05
**Starting baseline:** `9beb0b7ebb3cd84b88e16b454f41e526304c29f1` on `main` and `dev`
**Authorization:** Owner-approved Phase 2A-4 implementation of canonical identifiers, Java-compatible logical-name rules and pure AAD byte construction. No Phase 2A-5/2A-6 scope is included.
**Implementation commits:** `8c800e366e978f78d948c45227219b5e0abdccef` (typed IDs and Java-compatible names); `2392128269bc9a1bb74da1fa3807dafd285f996a` (AAD and architecture boundary).

## Scope and trust boundary

This milestone adds canonical UUID types and creation/rename-compatible logical-name utilities to `vault-format`, plus six pure AAD byte constructors to `vault-crypto`. The name functions operate on caller-supplied UTF-8 text and do not parse or validate stored manifest data. The AAD functions return fixed-size byte arrays only; they do not hash, authenticate, encrypt, decrypt, derive keys or wrap keys.

No plaintext registry or manifest parser, metadata relationship validation, tree operation, filesystem or storage behavior, authentication, UUID generation, password/key handling, real vault operation, GUI change or cryptographic algorithm was added. No Java source, private documentation or Java fixture was modified, copied or committed. All vectors are synthetic.

## Frozen decision compliance

| Decision | Implemented rule |
|---|---|
| D1 — UUID text | Typed constructors accept exactly canonical lowercase 36-character UUID text. The existing strict `vault.json` parser continues to reject noncanonical spellings. Historical metadata-reader policy remains OQ-04. |
| D2 — UUID version | No version or variant restriction is applied. Parsing does not generate identifiers. |
| D3 — typed IDs | Existing public `VaultId` and its 16-byte representation remain. `UserId` and `EntryId` are additive wrappers with distinct Rust types. |
| D4 — name length | Length is measured as Unicode scalar/code points after stripping, from 1 through 255. Supplementary characters count once; combining marks count separately. |
| D5 — stripping | `strip_java_whitespace` follows Java 21 `Character.isWhitespace`, not Rust `str::trim`. The exact scalar set was differentially checked. |
| D6 — case comparison | `logical_name_eq_ignore_case` reproduces the tested Java 21 `String.equalsIgnoreCase` behavior: equal UTF-16 code-unit length followed by the Java simple case key `lower(upper(codePoint))`. No locale rules, expansion folding or normalization are used. |
| D7 — accepted names | Reject only post-strip empty, `.`, `..`, U+0000, and values over 255 code points. Other controls, separators, colon, reserved Windows names and trailing periods remain allowed. |
| D8 — creation versus reading | `validate_logical_name` is for creation/rename inputs only. It is not a stored-manifest validator. OQ-03 remains deferred. |
| D9 — root naming | `/` is accepted as an ordinary logical name, consistent with the Java creation source and synthetic probe. Special root-folder semantics and root mutations are separate and were not implemented. |
| D10 — AAD | Six exact templates, separators, purpose strings, version markers, identifier order and UTF-8 bytes are fixed by independent golden tests below. |
| D11 — AAD ownership | AAD construction is in `vault-crypto`. The single authorized internal edge is `vault-crypto → vault-format`; the architecture validator checks its exact workspace path, version and feature policy. |
| D12 — Unicode/dependencies | No new third-party dependency was added. The 1,456-entry case table is derived from observable Java 21 `Character` API results; no JDK implementation source or third-party Unicode table was copied. Its runtime/version provenance is recorded below. |
| D13 — test provenance | UUID, name and AAD cases use deterministic synthetic values. No private Java fixture or vector file was copied. OQ-06 remains deferred. |
| D14 — gates | New contract tests, Java differential checks, parser regressions, architecture tests, strict Rust quality checks and exact-SHA hosted CI are required. Independent review remains pending. |

The approved operational collision rules for later manifest integration remain: active siblings participate; deleted siblings do not reserve a name; files and folders share one sibling namespace; a rename excludes itself; equivalent names under different parents can coexist. This phase implements only the pure comparator and no sibling collection or mutation engine.

## Canonical identifiers

`VaultId`, `UserId` and `EntryId` each retain 16 UUID bytes and expose a canonical lowercase `Display` representation and `to_canonical_bytes()`. Their `from_canonical_str()` constructors reject uppercase or mixed-case text, missing/extra/misplaced separators, invalid length, non-hex input, braces and whitespace. Parsing errors do not echo the rejected value; `Debug` redacts IDs. `EntryId` is distinct from a future blob/content ID and is the only file identifier accepted by the AAD functions.

ID-01/ID-02 cover canonical round trips, nil, version 1, version 4, other version bits, nonstandard variant bits, malformed spellings and domain-specific types. The existing Phase 2A-2 header parser's canonical lowercase behavior remains covered by its regression suite. No UUID is generated by this implementation.

## Logical-name behavior and Java 21 differential evidence

`validate_logical_name(&str)` returns a borrowed slice after Java-compatible edge stripping. It allocates no normalized or copied name. The value is not asserted safe as an OS path or export filename. Valid Rust `str` cannot represent isolated UTF-16 surrogate code units; no surrogate-repair behavior is introduced. Historical manifest reading and its compatibility implications remain deferred to Phase 2A-5/OQ-03.

The Java reference's `ManifestService.requireAvailableName` applies `String.strip()`, then rejects empty, `.`, `..`, NUL and more than 255 code points. Its documented metadata-name policy permits separators and does not apply Windows filename restrictions. The Java test suite confirms slash-containing names, backslash, colon, `CON`, trailing periods, NBSP preservation and supplementary length behavior. The standalone oracle separately checks the exact ordinary name `/`.

### Reproducible oracle

Run `pwsh ./scripts/verify-java-unicode-compat.ps1` from the repository root. The script compiles `scripts/unicode-compat/JavaUnicodeOracle.java` in an isolated temporary directory, emits synthetic vectors from the installed Java API, checks the committed mapping table against the emitted mapping, and pipes those vectors into the Rust-only probe example. It removes only its unique child directory under the operating-system temp directory.

Recorded environment and result:

- JDK distribution/runtime: Oracle Corporation JDK `21.0.2+13-LTS-58`, Windows x64; the script verifies the exact runtime string.
- Unicode basis: Java SE 21 `Character` data, Unicode 15.0, as documented by the [Java 21 Character API](https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/lang/Character.html). `String.strip()` documentation specifies `Character.isWhitespace`; see the [Java 21 String API](https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/lang/String.html).
- Rust toolchain: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, `cargo 1.99.1 (5f94df478 2026-08-27)`, target `x86_64-pc-windows-msvc`.
- Whitespace corpus: all 1,112,064 valid Unicode scalar values; Java identifies 25 as `Character.isWhitespace`. Rust's `strip_java_whitespace` produced the identical set. Seven additional edge/mixed/supplementary strings matched `String.strip()` exactly.
- Case table: 1,456 non-identity `lower(upper(codePoint))` entries, sorted and checked into `java_case_fold_table.rs`. The generated mapping checksum is `42556AF8AEFC14AB` (FNV-1a 64-bit integrity checksum; not a cryptographic claim). The oracle tests every mapped scalar against its Java key and adjacent scalar, all members of each observed case-equivalence class pairwise, named edge cases and 10,000 seeded randomized pairs.
- Name corpus: 16 Java-generated validation cases including 0/1/254/255/256 boundaries, 254 ASCII plus a supplementary character, 255 ASCII plus a supplementary character, composition, combining marks, NUL, dot names, stripping and `/`.
- UUID corpus: 16 synthetic Java `UUID.toString()` canonical spellings; Rust parsing and round-trip output matched.
- Case-comparison corpus: 14,418 Java-generated `equalsIgnoreCase` pairs, seed `454E435259505444`; mismatches: **0**.
- Limit: this demonstrates the tested Java 21 API behavior for valid Rust Unicode strings and canonical UUID spellings. It does not establish full Java/Rust application interoperability or historical manifest-reader compatibility.

The generated map is 1,456 pairs (11,648 bytes of scalar values before static alignment). Lookup uses binary search. Name trimming borrows input; length detection stops after the 256th scalar once prior rejection rules are checked. Case comparison performs no normalized copy. The oracle example is verification-only and is not called by the desktop application.

### Original NAME-01 through NAME-06 traceability

| ID | Preserved acceptance meaning | Evidence |
|---|---|---|
| NAME-01 | 0/1/255/256 code points; supplementary scalars; 254 ASCII plus emoji accepted; 255 plus emoji rejected; stripping before counting. | `logical_name_phase_2a4::name_01_uses_code_point_lengths_after_stripping`; Java-generated `N` vectors. |
| NAME-02 | Java `Character.isWhitespace` edge stripping; NBSP and related nonbreaking spaces remain; Java/Rust whitespace differences; mixed sequences. | `name_02_strips_exact_java_whitespace_and_preserves_nonbreaking_spaces`; exhaustive scalar oracle plus seven `String.strip()` vectors. |
| NAME-03 | Empty, whitespace-only, `.`, `..` and NUL are rejected. | `name_03_rejects_only_the_approved_empty_dot_dotdot_and_nul_cases`; Java-generated name vectors. |
| NAME-04 | Other controls, interior tab, `/`, `\\`, colon, `CON`, reserved Windows examples and trailing period are permitted. | `name_04_preserves_accepted_characters_including_slashes_and_windows_names`; Java source and oracle confirm ordinary `/`. |
| NAME-05 | No NFC/NFD normalization; composed/decomposed strings remain distinct; combining marks count as code points. | `name_05_does_not_normalize_and_counts_combining_marks_as_code_points`; Java-generated cases. |
| NAME-06 | Java `equalsIgnoreCase`; Photos/name casing; `ß` versus `SS`; Turkish I, Greek sigma, Kelvin sign, supplementary case pairs, and Unicode differences. | `name_06_matches_java_equals_ignore_case_cases`; 14,418 Java-generated comparisons, zero mismatches. |

Supplemental tests use separate identifiers or descriptive test names; NAME-01 through NAME-06 have not been renumbered or repurposed.

## Pure AAD contracts

The six public functions return fixed-size byte arrays and require typed IDs. AAD-05/AAD-06 take `EntryId`, not a blob ID. Tests compare literal expected text, exact byte length, complete bytes and independent hexadecimal strings. These vectors are synthetic and non-secret.

| Test | Exact text | Bytes | Hex |
|---|---|---:|---|
| AAD-01 `vault_key_aad` | `EncryptDrive|vault-key|v1|550e8400-e29b-41d4-a716-446655440000` | 62 | `456e637279707444726976657c7661756c742d6b65797c76317c35353065383430302d653239622d343164342d613731362d343436363535343430303030` |
| AAD-02 `users_aad` | `EncryptDrive|users|v1|550e8400-e29b-41d4-a716-446655440000` | 58 | `456e637279707444726976657c75736572737c76317c35353065383430302d653239622d343164342d613731362d343436363535343430303030` |
| AAD-03 `user_key_aad` | `EncryptDrive|user-key|v1|550e8400-e29b-41d4-a716-446655440000|123e4567-e89b-42d3-a456-426614174000` | 98 | `456e637279707444726976657c757365722d6b65797c76317c35353065383430302d653239622d343164342d613731362d3434363635353434303030307c31323365343536372d653839622d343264332d613435362d343236363134313734303030` |
| AAD-04 `manifest_aad` | `EncryptDrive|manifest|v1|550e8400-e29b-41d4-a716-446655440000|123e4567-e89b-42d3-a456-426614174000` | 98 | `456e637279707444726976657c6d616e69666573747c76317c35353065383430302d653239622d343164342d613731362d3434363635353434303030307c31323365343536372d653839622d343264332d613435362d343236363134313734303030` |
| AAD-05 `file_key_aad` | `EncryptDrive|file-key|v1|550e8400-e29b-41d4-a716-446655440000|123e4567-e89b-42d3-a456-426614174000|f47ac10b-58cc-4372-a567-0e02b2c3d479` | 135 | `456e637279707444726976657c66696c652d6b65797c76317c35353065383430302d653239622d343164342d613731362d3434363635353434303030307c31323365343536372d653839622d343264332d613435362d3432363631343137343030307c66343761633130622d353863632d343337322d613536372d306530326232633364343739` |
| AAD-06 `file_content_aad` | `EncryptDrive|file|v1|550e8400-e29b-41d4-a716-446655440000|123e4567-e89b-42d3-a456-426614174000|f47ac10b-58cc-4372-a567-0e02b2c3d479` | 131 | `456e637279707444726976657c66696c657c76317c35353065383430302d653239622d343164342d613731362d3434363635353434303030307c31323365343536372d653839622d343264332d613435362d3432363631343137343030307c66343761633130622d353863632d343337322d613536372d306530326232633364343739` |

## Architecture and dependencies

The only added edge is the explicitly authorized internal path dependency `vault-crypto → vault-format`, pinned to workspace version `=0.1.0` with default features disabled. `scripts/ArchitectureValidation.psm1` requires the exact edge, workspace manifest path and dependency settings; regression tests reject missing, unpinned or redirected edges and any extra lower-crate dependency. `Cargo.lock` changes only to record this local edge. The external dependency graph remains unchanged; there are no new external crates.

## Local validation and build evidence

Local checks on Windows/MSVC used Rust/Cargo 1.99.1. The exact commands and current outcomes are:

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace --locked` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --locked` | PASS; includes Phase 2A-2/2A-3 parser regressions and new ID/name/AAD tests. |
| `cargo check --workspace --features design-preview --locked` | PASS |
| `cargo clippy --workspace --features design-preview --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --features design-preview --locked` | PASS; includes preview and parser regressions plus new contract tests. |
| `cargo test -p vault-format --locked` | PASS: 17 unit tests, 16 external-envelope tests, 32 header-parser tests, 3 ID tests and 7 logical-name tests. |
| `cargo test -p vault-crypto --locked` | PASS: 6 AAD golden tests. |
| `pwsh ./scripts/verify-java-unicode-compat.ps1` | PASS: corpus and mapping results recorded above. |
| `pwsh ./scripts/verify-architecture.ps1` and `pwsh ./scripts/tests/verify-architecture.tests.ps1` | PASS. |
| Build-manifest rejection suite | PASS; Windows junction/reparse subcase BLOCKED by `NewItemCreateIOError`. |
| Shell/preview isolation suite | PASS, 42 negative cases rejected; Windows junction/reparse subcase BLOCKED by `NewItemCreateIOError`. |
| Dependency-inventory regression suite | PASS. |
| PowerShell wrapper regression suite | PASS. |
| Windows Release build scripts | Both shell and design-preview built and their manifest/native import inventory verified. The first local run recorded the starting commit with `dirty=true`; exact final-commit artifact identity is not claimed from that run. |
| `git diff --check` | PASS before commits. |

The existing workflow runs formatting, locked workspace check, strict Clippy, default and design-preview tests, PowerShell regression suites, architecture checks, separate Windows Release builds, artifact verification and native dependency inventory on `push`. The standalone Java oracle is not invoked by the current workflow; its pinned differential run is a separate local acceptance result. The exact final `dev` SHA, hosted run URL/ID and job outcomes will be attributed in the implementation handoff after publication; no result is claimed here before that run exists.

## Deferred decisions and gates

- OQ-03: historical plaintext manifest name/metadata reading remains deferred.
- OQ-04: permissive historical manifest UUID reading remains deferred; the new constructors describe canonical creation/API text only.
- OQ-06: private fixture redistribution remains deferred; no such fixture is included.
- OQ-07 remains the approved AAD ownership principle; this phase implements only its authorized `vault-crypto → vault-format` edge.
- OQ-08 and the no-new-external-dependency policy remain in force.
- Direct Rust 1.95 compilation, complete VF1 conformance, two-way Java/Rust application interoperability and release readiness remain UNVERIFIED.
- Phase 1B P2-F01 through P2-F04 and its compact-layout residual observation remain open. Windows N01–N09 remain NOT RUN / UNVERIFIED; B01–B04 remain BLOCKED.
- Windows distribution, accessibility certification, Narrator/UI Automation, High Contrast, packaging, reproducible build certification, licensing and release gates are unchanged.

The final `dev` diff must receive independent Claude Code review from baseline `9beb0b7ebb3cd84b88e16b454f41e526304c29f1` before any owner disposition for integration. This record and its implementation do not authorize `main` integration, Phase 2A-5 or Phase 2A-6.
