# Phase 1B-3 — Application Command, Event, and Session Contracts

**Status:** Local documentation candidate; independent review and owner approval pending
**Baseline commit:** `b7d134bf5e3a6778f3c89b03e63e659fa7b2e50d`
**Baseline CI:** [Run 40995708450](https://github.com/Fabianrodas/encryptdrive-desktop/actions/runs/40995708450), completed successfully for the baseline SHA
**Scope:** Contract design only; no production interfaces or vault functionality are implemented.

Phase 1B-3 defines how future Desktop intent will cross into the GUI-independent core, how outcomes and session changes will be correlated, and which security/privacy rules later operations must preserve. It does not authorize Phase 1B-4, VF1, cryptography, storage, or real vault operations.

## 1. Baseline and current implementation

The workspace has five packages. The default eframe Desktop depends on `encryptdrive-core`; the core currently contains only application identity and a truthful readiness/capability surface, with no available capabilities. `vault-format`, `vault-crypto`, and `vault-storage` are reserved boundaries without functional format, cryptography, or storage implementations. The production shell is a no-vault engineering shell. The feature-gated `design-preview` uses synthetic in-memory fixtures and simulated operations only.

Phase 1B-1 and Phase 1B-2 are reported closed and integrated at this baseline; the main CI run above is the verified current run. The older [Phase 1B-2 inventory record](PHASE_1B_DEPENDENCY_INVENTORY.md) retains the local/not-yet-pushed and CI-not-run wording that was accurate when that report was written. This milestone records the later integrated state without rewriting that historical snapshot.

## 2. Scope and exclusions

### Included

- A minimal proposed boundary for intent, request identity, core validation/authorization, acceptance/rejection, execution outcome, and stale-request protection.
- Minimal event/outcome categories, correlation rules, and stale-session handling.
- A conceptual selected/locked/unlocking/unlocked/locking/closed lifecycle with legal and illegal transitions.
- Cancellation and concurrency principles, typed error categories, diagnostic privacy constraints, contract invariants, and unresolved decisions.
- Independent review criteria for a future implementation milestone.

### Excluded

- Rust command/event/session types, public APIs, adapters, event buses, executors, worker pools, or async dependencies.
- Vault creation/opening, authentication, key/session material, VF1 parsing/serialization, encryption/decryption, or storage access.
- Real file listing, import/export, rename, Trash, restore, deletion, recovery, networking, or background vault operations.
- New production dependencies, Cargo manifest/lockfile changes, modifications to the shell or UX6.3 baseline, and changes to Java or `encryptdrive-docs`.

## 3. Deliverables

| Document | Purpose |
|---|---|
| [Application command, event, and session contracts](../architecture/APPLICATION_COMMAND_EVENT_SESSION_CONTRACTS.md) | Verified current baseline, proposed UI/core boundary, request identity, outcomes, conceptual lifecycle, cancellation, errors/privacy, invariants, examples, and open questions. |
| This milestone record | Scope, baseline, accepted boundaries versus proposed contracts, validation/review gates, unresolved issues, and future handoff. |

No ADR is created because this milestone proposes contracts for review rather than recording a newly approved architectural decision.

## 4. Existing decisions and proposed contracts

| Existing, verified boundary | New proposal requiring review |
|---|---|
| eframe/egui 0.37 remains the owner-approved Desktop framework; UX6.3 remains the visual baseline. | The UI submits framework-independent intent to `encryptdrive-core`, which repeats all security-relevant checks. |
| The Desktop depends on `encryptdrive-core`; core and lower layers remain GUI-independent. | Request/operation identity and a core-owned session generation fence duplicates and stale work. |
| Core currently exposes no available capabilities; lower vault crates remain API-empty. | Accepted asynchronous work reports correlated progress and one truthful terminal outcome; rejection is distinguished from failure after acceptance. |
| Design preview remains synthetic and separate from production storage. | Lock/close/expiry invalidates the old generation; stale outcomes cannot repopulate or reactivate the UI. |
| Persistent format and conformance are language-independent and not UI-owned. | Cancellation is only reported when confirmed; after a commit point it is not rollback. |

The proposed column is not an approved public API or implementation claim. Changes to the approved visual baseline, dependency direction, or persistent-format boundary are outside this milestone and require the relevant owner decision.

## 5. Review and verification gates

### Documentation review

- Verify every “current” statement against source at the baseline commit; distinguish implemented facts from proposals.
- Confirm examples remain illustrative, do not imply supported capabilities, and contain no secrets or real paths.
- Confirm session transitions, preconditions, cancellation boundaries, partial outcomes, and error wording are internally consistent.
- Resolve all local Markdown links in both documents; run `git diff --check`.
- Confirm no unrelated file, Rust source, Cargo manifest/lockfile, workflow, Java repository, or private documentation file changes.

### Technical checks

The full existing validation set is run because the contract constrains the current architecture, although the changes are documentation-only. No Release build is required for this milestone.

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace --locked` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --locked` | PASS — 49 tests passed, 0 failed. Windows emitted non-fatal incremental-cache hard-link fallback warnings. |
| Design-preview check, strict Clippy, and tests with `--features design-preview --locked` | PASS — check and Clippy passed; 137 tests passed, 0 failed. Windows emitted non-fatal incremental-cache hard-link fallback warnings. |
| `scripts/verify-architecture.ps1` | PASS — workspace membership and direct dependency boundaries verified. |
| Phase 1B-1 build-manifest rejection tests | PASS with one BLOCKED subcase — all other cases passed; Windows denied creation of a junction needed to exercise reparse-point rejection. |
| Phase 1B-2 inventory tests | PASS — validation-only wording/limitation, schema, identity/license consistency, lock/configuration, checksum, classification, and path-redaction cases passed. |
| PowerShell wrapper regression tests | PASS — standalone and workflow-wrapper success/failure propagation cases passed. |
| Markdown link and consistency checks | PASS — all local Markdown links in both Phase 1B-3 documents resolve; only the two intended documents are changed. |
| Workflow and dependency changes | PASS — `Cargo.toml`, `Cargo.lock`, and `.github/workflows/ci.yml` are unchanged. |

## 6. Acceptance and independent review

Phase 1B-3 is ready for architectural review when:

1. The two documents distinguish verified boundaries from proposed semantics and make no implementation claim.
2. Commands cannot be mistaken for authorization; session fencing, destructive-scope validation, stale completions, cancellation after commit, partial outcomes, and error privacy are explicit.
3. Synthetic preview isolation and the GUI-to-core boundary remain unchanged and verified.
4. Requested local checks pass or their exact blocker is recorded; no failed check is represented as a pass.
5. Claude independently reviews the documents for security, lifecycle, cancellation, error, privacy, and compatibility gaps. The owner then decides which proposed contract points are accepted before any implementation is scoped.

Review questions are whether the proposed request/session identities are minimal and safe, whether lock/close can be honored at operation-safe boundaries, whether event correlation covers stale results without an unnecessary event framework, whether error/diagnostic wording avoids sensitive data, and whether multi-item partial completion is represented honestly.

## 7. Open decisions and later handoff

The architecture document lists open choices that depend on authentication, VF1, storage commit/recovery semantics, operation concurrency, and the product privacy policy. Keep those decisions open until the implementation that needs them is separately scoped and reviewed. In particular, do not choose durable idempotency retention, exact event transport, concrete session/key storage, cryptographic primitives, or a generic production API here.

After independent review and owner disposition, prepare a separately authorized implementation scope for the specific approved contracts. This document does not name, start, or authorize the next phase. No Phase 1B-4 or Phase 2 work is included.
