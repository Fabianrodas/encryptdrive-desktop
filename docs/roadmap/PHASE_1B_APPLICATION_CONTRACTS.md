# Phase 1B-3 — Application Command, Event, and Session Contracts

**Status:** PROPOSED — READY FOR OWNER DISPOSITION; PENDING OWNER APPROVAL
**Baseline commit:** `b7d134bf5e3a6778f3c89b03e63e659fa7b2e50d`
**Baseline CI:** [Run 40995708450](https://github.com/Fabianrodas/encryptdrive-desktop/actions/runs/40995708450), completed successfully for the baseline SHA
**Scope:** Contract design only; no production interfaces or vault functionality are implemented.

Phase 1B-3 defines how future Desktop intent will cross into the GUI-independent core, how outcomes and session changes will be correlated, and which security/privacy rules later operations must preserve. It does not authorize Phase 1B-4, VF1, cryptography, storage, or real vault operations.

## 1. Baseline and current implementation

The workspace has five packages. The Desktop declares a direct dependency on `encryptdrive-core`, and `verify-architecture.ps1` checks package membership and dependency direction. That check does not establish API usage: the Desktop currently calls no core API, and the default shell does not consume `readiness()`. The core exposes identity and readiness with no available capabilities; no production command, event, or session behavior exists. `vault-format`, `vault-crypto`, and `vault-storage` are reserved boundaries without functional format, cryptography, or storage implementations. The production shell is a no-vault engineering shell. The feature-gated `design-preview` uses synthetic in-memory fixtures and simulated operations only.

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

## 4. Decision and disposition table

All new contract principles below remain proposals pending owner disposition; none is recorded as already approved. The candidate principles are recommended for owner acceptance, while implementation mechanisms remain deferred. The table separates stable principles from future mechanism choices.

| Disposition | Principle or verified fact | Phase 1B-3 status |
|---|---|---|
| Verified current | eframe/egui 0.37 and UX6.3 remain the approved UI baseline; the Desktop declares a direct dependency on `encryptdrive-core`, and the architecture validator checks the declared package boundary. | Existing decision and checked package graph; validator does not check API calls. |
| Verified current | The Desktop calls no core API and the shell does not consume `readiness()`; there is no implemented production command, event, or session behavior. | Confirmed current-source status; no proposed behavior should be described as implemented. |
| Verified current | The design preview remains synthetic and separate from production storage; persistent-format and conformance rules remain language-independent. | Existing boundaries to preserve. |
| Candidate principle for owner acceptance now | A duplicate delivery with the same request identity refers to its original known status. An expired/unrecognized identifier is not presumed to be a fresh destructive request; when prior execution cannot safely be ruled out, remain `Indeterminate` or otherwise non-executing. Admit a new intent only with a fresh identity and current authorization/precondition checks; never automatically replay an irreversible mutation or treat target revalidation alone as proof of safety. | Corrected P1 and N1 proposal; bounded retention, identity validity and recovery remain deferred; no permanent exactly-once or unlimited deduplication promise. |
| Candidate principle for owner acceptance now | Each independent user intent has a distinct request identity; a new request rejection makes a no-effect claim only about that request. IDs exposed for support are opaque, non-sensitive, scoped-unique, independently generated, and never authorization credentials. | Proposed identity and outcome semantics; representation remains open. |
| Candidate principle for owner acceptance now | A definitive terminal outcome ends its operation; late, duplicate, or reordered delivery cannot contradict it. Cancellation is not rollback, and partial effects are disclosed safely. | Proposed operation truthfulness and privacy principles. |
| Candidate principle for owner acceptance now | Redelivery remains the same logical request even if its originating session is invalid; invalid session context cannot authorize a fresh mutation. Historical status may be returned only in a properly scoped, sanitized form or as `Indeterminate`, without exposing unrelated or prior-vault details. | N2 proposal; status lookup ordering and mechanism remain open, with no universal lookup-before-authorization rule. |
| Conditional future principle requiring verification | Future Desktop operations use framework-independent core contracts and core capability readiness; the core validates live session, revision, authority, and exact mutation scope. | Requires implementation-specific security and behavior review when real capabilities are scoped. |
| Conditional future principle requiring verification | Unlock attempts are fenced to a current selection; session references/generations are not reused in-process; stale completion cannot reactivate a closed or superseded context. A close requested during `Locking` is deferred until the lock reaches its safe boundary, then closes the locked selection. | Future session/authentication design must validate lifecycle and sensitive-state handling; no cancellation/reentrancy behavior is added. |
| Conditional future principle requiring verification | Stale events cannot affect an active vault, while a completed mutation remains explainable through appropriately scoped, non-sensitive status when necessary. | Future UI and operation-record design must prevent cross-vault disclosure. |
| Mechanism deferred to implementation milestones | Request-dedup retention window, ID representation, event transport/order and persistence, operation-record retention, exact commit points and recovery, concurrency implementation, key storage/zeroization, and outcome presentation/retention. | Deferred; no library, async framework, storage behavior, or security primitive is selected here. |

These proposals do not create a public API or implementation claim. Changes to the approved visual baseline, dependency direction, or persistent-format boundary are outside this milestone and require the relevant owner decision.

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

1. The documents distinguish the declared-and-checked Cargo dependency from current API usage: the Desktop calls no core API and the shell does not consume readiness; no command/session behavior is implemented.
2. Duplicate delivery refers to the original request, indeterminate outcomes are not treated as rejection, irreversible mutations are not automatically repeated, and a new rejection claims no effect only for that new request.
3. Session/attempt fencing, non-reused in-process references/generations, illegal unlisted transitions, deferred close during `Locking`, terminal-event rules, truthful cancellation/partial effects, and safe handling of completed mutations after lock/close are explicit.
4. All contract principles are visibly proposed; the decision table separates owner-acceptance candidates, conditional future principles, and deferred mechanisms without claiming owner approval.
5. Synthetic preview isolation and the package boundary remain unchanged and verified; requested local checks pass or their exact blocker is recorded, with no failed check represented as a pass.
6. Claude independently reviews the corrected documents. The owner decides which proposed principles to accept before any implementation is scoped.

Review questions include whether expired/unrecognized request identities remain non-executing when history is unavailable, whether status handling protects invalid-session/prior-vault information without imposing lookup ordering, whether unlock attempts and session fencing prevent stale authentication, whether close deferral during `Locking` is unambiguous, whether terminal and partial outcomes remain truthful, whether post-close mutation status is scoped without cross-vault disclosure, whether support identifiers reveal no sensitive source data, and whether implementation mechanisms remain properly deferred.

## 7. Open decisions and later handoff

The architecture document lists open choices that depend on authentication, VF1, storage commit/recovery semantics, operation concurrency, and the product privacy policy. Keep those decisions open until the implementation that needs them is separately scoped and reviewed. In particular, the request-dedup retention window, ID representation, exact event transport and delivery guarantees, persistent operation-record policy, concrete session/key storage, commit points/recovery, cryptographic primitives, and a generic production API remain deferred here.

After independent review and owner disposition, prepare a separately authorized implementation scope for the specific approved contracts. This document does not name, start, or authorize the next phase. No Phase 1B-4 or Phase 2 work is included.
