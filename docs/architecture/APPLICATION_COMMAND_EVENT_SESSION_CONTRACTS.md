# Application Command, Event, and Session Contracts

**Status:** Phase 1B-3 proposal; independent review and owner approval pending
**Baseline:** `b7d134bf5e3a6778f3c89b03e63e659fa7b2e50d`
**Scope:** Future contract between the native Desktop UI and `encryptdrive-core`; no production API is implemented or frozen here.

This document defines a small contract for expressing user intent, checking it in the core, reporting operation outcomes, and fencing work against a changing session. Names in examples are conceptual. They are not Rust types, executable commands, or promises that vault operations exist.

## 1. Verified implementation baseline

| Area | Verified at the baseline commit |
|---|---|
| Workspace | Five packages: `encryptdrive-desktop`, `encryptdrive-core`, `vault-format`, `vault-crypto`, and `vault-storage`. |
| Dependency direction | The Desktop directly depends on `encryptdrive-core`; the core and three lower reserved crates have no direct dependencies. The architecture validator checks this boundary. |
| Core API | `encryptdrive-core` exposes product identity, a single `EngineeringPreview` readiness state, and a list of nine capabilities. `readiness()` reports no available capabilities. There are no command, event, session, or operation types. |
| Lower crates | `vault-format`, `vault-crypto`, and `vault-storage` are reserved boundaries without functional format, cryptography, or storage implementations. There is no VF1 parser or placeholder operation API. |
| Production shell | The default Desktop shell begins with `NoVaultOpen`. It has no vault, authentication, file, storage, network, or telemetry operation. |
| Design preview | `design-preview` gates the preview UI. It uses `PreviewModel` synthetic in-memory fixtures and simulated operations; it has no production-storage dependency. Preview APIs and results are not a contract for real vault operations. |
| Visual baseline | UX6.3 remains the owner-approved visual baseline; this proposal does not change it. |

The stated main CI run, [40995708450](https://github.com/Fabianrodas/encryptdrive-desktop/actions/runs/40995708450), completed successfully for this baseline SHA. The earlier Phase 1B-2 report preserves its at-the-time local/CI status; this current baseline records the later integration and run without rewriting that historical evidence.

The established package boundary is recorded in [Phase 1A crate boundaries](PHASE_1A_CRATE_BOUNDARIES.md) and [ADR-012](ADR-012-EGUI-EFRAME-DESKTOP.md). The UX baseline is recorded in the [Phase 1A-UX closeout](../ux/design-studio/PHASE_1A_UX_CLOSEOUT.md).

## 2. Proposed command boundary

```text
egui input
  → Desktop-owned user intent and presentation validation
  → typed, framework-independent request to encryptdrive-core
  → core capability, session, authorization, domain, and scope checks
  → future lower-layer work, only when the capability is implemented
  → correlated outcome/update returned to the Desktop
```

The Desktop owns the interaction: it gathers a user choice, displays the required confirmation, and may catch obvious input mistakes for immediate feedback. That does not authorize an operation. The core is the enforcement boundary and must repeat all security-relevant validation against current state before any side effect. UI validation is never sufficient authorization.

The future request boundary should preserve these rules:

- A request identifies the user intent and carries a fresh correlation identity. It does not carry secrets, grant permission, or select arbitrary account/key material.
- Session context is opaque and issued/validated by the core. A UI-provided vault, account, entry, or parent identifier is a selector to validate, never proof of ownership or permission.
- Requests that can mutate data must revalidate capability, live session, permissions, target identity, name/conflict rules, and the exact operation scope immediately before the first side effect. Destructive confirmations must describe that same scope. A stale plan is rejected and must be shown again for confirmation.
- Rejection happens before side effects. Acceptance means the core accepted the request for processing; it does not mean the operation committed or will succeed.
- A duplicate delivery with the same request identity must never start a second mutation. During a live process, the core should return or refer to the original operation state, or reject the duplicate. Restart-persistent deduplication is not specified here.
- The Desktop communicates through `encryptdrive-core`; it does not reach into storage or crypto crates, construct authenticated-data contexts, or interpret persistent-format bytes.
- No event bus, executor, worker pool, async runtime, or public command API is required by this contract document.

### Illustrative future workflows

These examples show responsibility and ordering only. They are not implemented APIs.

| User intent | Future core checks and result |
|---|---|
| Request vault opening | The Desktop submits a selected target. The core decides whether opening is supported, validates the target and any required lock, and returns a selected-but-locked state or a typed rejection. A selected target is not an authenticated session. Path/lock ownership details remain for storage design. |
| Request file listing | The core requires an unlocked session, validates the requested folder against that session, and returns a snapshot/result associated with the current session generation. The UI cannot list data by supplying an arbitrary account or manifest ID. |
| Request operation cancellation | The UI identifies one accepted operation. The core checks that it is still live and at a cancellable boundary; a cancel request is not itself proof that cancellation succeeded. |
| Request session locking | The core stops admitting work for that session, reaches operation-safe boundaries, invalidates the session generation, and reports the locked state only after invalidation. A lock action does not promise rollback of already committed work. |
| Request restore from Trash | The core rechecks each selected item, session, destination, and name conflict before changing state. A multi-item result must expose partial completion if some roots succeed and a later root fails; the UI must not report all-or-nothing success unless the implementation provides it. |

## 3. Proposed outcomes and events

Prefer an immediate request reply for acceptance or rejection, plus correlated updates only for accepted operations that outlive the call. This keeps the contract understandable without committing to a framework or event-bus implementation.

| Category | Meaning | Correlation |
|---|---|---|
| Accepted | Core validated the request for processing. | Request identity; accepted long-running work also receives an operation identity. |
| Rejected | Request is unsupported, invalid, unauthorized, stale, or otherwise cannot start. No side effect has occurred. | Request identity and stable error category. |
| Progress | Truthful completed/total units when known; never invent a percentage. | Request and operation identities, plus the session generation when session-scoped. |
| Completed | The operation reached its defined success/commit result. | Request and operation identities and session generation. |
| Cancelled | Core confirms cancellation at a safe boundary. It does not imply rollback of committed effects. | Request and operation identities and session generation. |
| Failed | Recoverable or terminal failure, with a typed category and any explicit partial result. | Request and operation identities and session generation. |
| Session state changed | Selected/locked/unlocked/closed/invalidated transition was completed by the core. | Session identity/generation; do not include key material. |
| Capability availability changed | Core readiness changed after an implementation or runtime condition. | Capability/readiness revision; may be application-scoped rather than session-scoped. |

Rejection is normally a reply rather than a later event: the UI must distinguish “not accepted” from “accepted, then failed.” An implementation may choose another transport after review, but it must preserve these semantics. Progress payloads must not contain file contents, passwords, keys, plaintext paths, or other secret material.

### Correlation and stale results

- Each fresh user intent receives a request identity. Accepted asynchronous work receives an operation identity that remains stable through its terminal outcome.
- Session-scoped requests and outcomes are bound to a core-issued session generation (an epoch/fencing value). Lock, close, account/vault switch, or expiry invalidates the old generation before a later session can use the same UI state.
- The core rejects commands against an invalid generation. The UI also ignores events/results whose request, operation, or session generation is no longer current; UI filtering supplements but never replaces core enforcement.
- A late completion must not restore an unlocked view, repopulate cleared private UI state, or be applied to a different session. Non-sensitive completion status may be retained separately if needed to explain a result after locking.
- Ordering, delivery guarantees, persistence of request identities across process restart, and retention of terminal operation records remain open decisions.

## 4. Conceptual session lifecycle

These states are a proposed domain lifecycle, not current product functionality or a key-management design.

| Current state | Request/condition | Next state | Required rule |
|---|---|---|---|
| No vault selected | A supported target is selected/opened | Vault selected, locked | Selection alone exposes no decrypted content and creates no authenticated session. |
| Vault selected, locked | Unlock requested | Unlocking | Keep the credential input out of commands, events, logs, and diagnostics after the secure handoff. Do not create an authorized session yet. |
| Unlocking | Authentication and integrity checks succeed | Unlocked and authorized | Core establishes the session and a new generation only after checks succeed. |
| Unlocking | User cancels or unlock fails | Vault selected, locked | Cancellation returns to the locked state. Failure uses safe generic user wording and leaves no partially authorized session. |
| Unlocked and authorized | Lock requested | Locking | Stop admitting new operations for the generation and resolve in-flight work at documented safe boundaries. |
| Locking | Sensitive session state is invalidated | Vault selected, locked | Advance/invalidate the generation; clear session-derived UI state before showing the locked state. |
| Any selected state | Close, switch, expiry, or forced invalidation | Closed/invalidated, then No vault selected or Vault selected, locked | Revoke the old generation. Events from it cannot reactivate or repopulate the session. |
| Unlocked and authorized | Unexpected session or integrity failure | Locked or closed/invalidated | Fail closed. The exact recovery path must not retain apparent authorization after core validation fails. |

Content operations (listing, search, import/export, rename, Trash, restore, and deletion) require an unlocked, authorized, current session and operation-specific permission. Selecting/opening a target, unlocking, locking, closing, and cancellation have separate preconditions. The final capability list and account model must be reviewed when those operations are actually scoped.

The core owns session validity and any future sensitive state. The UI may retain only the minimum transient display state needed for interaction; it must clear session-derived names, selections, search results, and pending confirmations when the generation is invalidated. This proposal does not select key storage, zeroization primitives, KDF settings, or memory-erasure guarantees.

## 5. Cancellation and concurrency

| Boundary | Proposed behavior |
|---|---|
| Before acceptance | Reject/abandon the request; no operation or side effect exists. |
| Accepted but not started | Core may confirm cancellation; no side effect may begin afterward. |
| Read-only work in progress | Cancel cooperatively at a documented safe point. If cancellation cannot be honored, report the actual terminal result rather than claiming cancellation. |
| Mutation before its irreversible/commit point | Cancellation is allowed only if the implementation can leave a valid state and clean up safely. The storage milestone must define and test this point. |
| After the commit point | Cancellation is too late. Report the committed outcome (or a truthful recovery/partial outcome); never claim rollback. |
| Lock/close/session invalidation | Stop admitting work for the old generation and bring accepted operations to safe boundaries before invalidation completes. A canceled operation may have already committed a separately reportable effect. |

Every operation must define whether it is read-only, cancellable, partially completing, and where its commit point lies before implementation. Conflicting mutations for one session require core-side arbitration; the UI must not assume parallel safety. Whether safe reads may overlap a mutation, and whether writes are serialized, rejected as busy, or queued, is operation- and storage-dependent and remains open. No concurrency framework is selected here.

## 6. Error taxonomy and privacy

The future core should return stable categories, not raw platform error strings as product copy.

| Category | Example user-facing treatment | Diagnostic boundary |
|---|---|---|
| Invalid input / conflict | Explain the field or conflict and how to correct it. | Include validation code and request correlation; avoid logging the entered sensitive value. |
| Unsupported capability | State that the feature is unavailable in this build. | Record capability code/build context only. |
| Missing, locked, or stale session | Ask the user to unlock or reopen, without claiming an operation ran. | Record state category and generation mismatch, not session secrets. |
| Authentication/authorization failure | Use generic wording that does not disclose whether a user/account exists or which credential was correct. | Do not log credentials, derived keys, or secret-dependent comparisons. |
| Missing resource / permission | Say the requested item is unavailable or access was denied. | Stable code and operation identity; no plaintext path. |
| Recoverable I/O failure | Give a safe retry/choose-another-location action where appropriate. | Whitelist OS error category; redact path, account, and vault identifiers by default. |
| Integrity/corruption failure | Stop the unsafe operation and say the data could not be verified; make no recovery or secure-erasure promise. | Keep raw bytes, decrypted records, secrets, and sensitive paths out of ordinary logs and UI. |
| Cancellation / too late | Report canceled only when confirmed; otherwise report that the operation already completed or could not be canceled. | Record terminal category and correlation IDs only. |
| Partial completion | State what category/count completed and what remains, without implying rollback. | Per-target diagnostic IDs only if the owner-approved privacy policy permits them. |
| Unexpected internal failure | Generic message with a non-sensitive support/correlation identifier. | Sanitized error category and redacted source chain; no automatic raw-chain dump. |

Ordinary logs, UI notifications, and crash reports must never contain passwords, derived keys, authentication secrets, raw file contents, or decrypted metadata. Full filesystem paths, vault IDs, account/user IDs, and filenames are sensitive by default: show only when the active product workflow requires it, and redact them from diagnostics unless a reviewed policy explicitly permits a narrowly scoped value. Error chains must be sanitized before logging; a lower-layer source error is not automatically safe. This is a proposed contract, not a claim about current logging, which has no production vault operations to exercise.

## 7. Contract invariants

| Invariant | Status at Phase 1B-3 |
|---|---|
| Desktop calls application behavior through `encryptdrive-core`; core/lower crates do not depend on egui widgets or presentation state. | Existing package boundary; currently checked by `verify-architecture.ps1`. |
| UX action or confirmation is not authorization to mutate a vault. | Proposed contract. |
| Unsupported capabilities stay unavailable until the core implementation and its gates exist. | Existing readiness surface; preserve for future work. |
| A command cannot execute against a missing, locked, expired, or stale session. | Proposed contract; enforce in core before side effects. |
| A stale completion cannot reactivate or repopulate an invalidated session. | Proposed contract; core fencing plus UI correlation checks. |
| Destructive scope is revalidated and matches the confirmation before any side effect. | Proposed contract; persistence/transaction mechanism deferred. |
| Synthetic preview state and commands cannot reach production storage or real vault operations. | Existing feature/source boundary; preserve and test. |
| Cancellation is not rollback; after commit, report the actual committed/partial outcome. | Proposed contract. |
| Errors are never silently converted into success or optimistic completion. | Proposed contract. |
| User-visible diagnostics and ordinary logs contain no secret material or raw file data. | Proposed contract; logging implementation and review remain future work. |
| Persistent format and conformance rules are independent of UI framework and design. | Existing architecture direction; no format API is implemented here. |

## 8. Open decisions for review

1. What exact core-owned session reference and generation model will later APIs use, and how are generations invalidated across close, lock, account switch, and expiry?
2. Which future operation classes need request IDs, operation IDs, deduplication, and retention across restart? What is the caller-visible result after a process interruption?
3. Will long-running updates use callbacks, a receiver, polling, or another simple mechanism? What ordering and terminal-delivery guarantees are required?
4. How are vault selection, OS path selection, vault locks, and session ownership divided between Desktop and storage?
5. What are the safe cancellation and commit points for each VF1/storage operation, including permanent deletion and recovery?
6. Which per-session operations may run concurrently, and how are conflicting mutations serialized or rejected?
7. How should multi-selection restore/delete/import/export report partial success and permit a safe retry without repeating completed side effects?
8. Which error details and diagnostic identifiers may be retained or shown under the product privacy policy?
9. Are capability availability changes static per build or dynamic per selected session, and what user-visible transition should accompany a change?
10. Which Java behaviors are compatibility requirements for each future command, and which require a separate approved product decision? The Java reference is evidence for behavior, not a Rust runtime/API design mandate.

Resolve these questions in the phase that implements the affected capability. Do not create empty interfaces, choose cryptographic/session primitives, or freeze a generalized public API just to close this contract milestone.

## 9. Future implementation examples

Once separately authorized, a concrete command can follow this shape without importing a GUI type:

1. Desktop gathers intent, collects only necessary input, and shows a confirmation for destructive scope.
2. Desktop submits a fresh request identity and core-issued session context; it does not submit authority or secrets in ordinary event data.
3. Core validates capability, session generation, authorization, resource identity, current revision, and exact effect scope.
4. Core replies rejected with no effect, or accepted with a correlated operation identity.
5. For accepted work, core emits truthful progress and one terminal completion/cancellation/failure result. The UI applies only results still matching the current request and session generation.

The specific payloads, validation rules, commit points, and event transport belong to future operation designs. A later CLI or other client may reuse a GUI-independent core boundary only after separate scope, compatibility, privacy, and release approval; this document does not authorize another client.

See the [Phase 1B-3 milestone record](../roadmap/PHASE_1B_APPLICATION_CONTRACTS.md) for review gates and handoff.
