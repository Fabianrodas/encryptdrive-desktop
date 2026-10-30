# Application Command, Event, and Session Contracts

**Status:** PROPOSED — PENDING OWNER APPROVAL; independent review is READY FOR OWNER DISPOSITION
**Baseline:** `b7d134bf5e3a6778f3c89b03e63e659fa7b2e50d`
**Scope:** Future contract between the native Desktop UI and `encryptdrive-core`; no production API is implemented or frozen here.

This document defines a small contract for expressing user intent, checking it in the core, reporting operation outcomes, and fencing work against a changing session. Names in examples are conceptual. They are not Rust types, executable commands, or promises that vault operations exist.

## 1. Verified implementation baseline

| Area | Verified at the baseline commit |
|---|---|
| Workspace | Five packages: `encryptdrive-desktop`, `encryptdrive-core`, `vault-format`, `vault-crypto`, and `vault-storage`. |
| Dependency direction | The Desktop declares a direct dependency on `encryptdrive-core`; the core and three lower reserved crates have no direct dependencies. `verify-architecture.ps1` checks the declared package boundary, not API call sites. |
| Core API and use | `encryptdrive-core` exposes product identity, an `EngineeringPreview` readiness state, and nine capabilities; `readiness()` reports no available capabilities. The Desktop currently calls no core API, and its shell does not consume `readiness()`. There are no command, event, session, or operation types. |
| Lower crates | `vault-format`, `vault-crypto`, and `vault-storage` are reserved boundaries without functional format, cryptography, or storage implementations. There is no VF1 parser or placeholder operation API. |
| Production shell | The default Desktop shell begins with `NoVaultOpen`. It has no vault, authentication, file, storage, network, or telemetry operation. |
| Design preview | `design-preview` gates the preview UI. It uses `PreviewModel` synthetic in-memory fixtures and simulated operations; it has no production-storage dependency. Preview APIs and results are not a contract for real vault operations. |
| Visual baseline | UX6.3 remains the owner-approved visual baseline; this proposal does not change it. |

The stated main CI run, [40995708450](https://github.com/Fabianrodas/encryptdrive-desktop/actions/runs/40995708450), completed successfully for this baseline SHA. The earlier Phase 1B-2 report preserves its at-the-time local/CI status; this current baseline records the later integration and run without rewriting that historical evidence.

The established package boundary is recorded in [Phase 1A crate boundaries](PHASE_1A_CRATE_BOUNDARIES.md) and [ADR-012](ADR-012-EGUI-EFRAME-DESKTOP.md). The UX baseline is recorded in the [Phase 1A-UX closeout](../ux/design-studio/PHASE_1A_UX_CLOSEOUT.md).

## 2. Proposed command boundary

```text
future egui input
  → Desktop-owned user intent and presentation validation
  → typed, framework-independent request to encryptdrive-core
  → core capability, session, authorization, domain, and scope checks
  → future lower-layer work, only when the capability is implemented
  → correlated outcome/update returned to the Desktop
```

This is a proposed future flow, not the current call graph: the Desktop presently declares the core dependency but does not call its API. When real product operations are separately implemented, the Desktop should express intent through GUI-independent core contracts and use core capability readiness; the core owns security enforcement. The Desktop gathers user choices, displays required confirmations, and may catch obvious input mistakes for immediate feedback. None of those UI actions authorizes an operation. The core must repeat all security-relevant validation against current state before any side effect.

The future request boundary should preserve these rules:

- Every distinct user intent has its own fresh request identity, including separate activations such as a double-click. A redelivery or retry of the same logical request reuses its original identity. Identity does not carry secrets, grant permission, or select arbitrary account/key material.
- Session context is opaque and issued/validated by the core. A UI-provided vault, account, entry, or parent identifier is a selector to validate, never proof of ownership or permission.
- Requests that can mutate data must revalidate capability, live session, permissions, target identity, name/conflict rules, and the exact operation scope immediately before the first side effect. Destructive confirmations must describe that same scope. A stale plan is rejected and must be shown again for confirmation.
- A repeated delivery of an existing request identity refers to the original logical request. Return its known acceptance, operation identity, current status, or terminal outcome; do not create a new request result that contradicts it. If the original outcome cannot be established, report it as `Indeterminate`, not as a rejection claiming no effect. Never automatically repeat an irreversible mutation solely because its acknowledgement was lost.
- If an identity is no longer recognized because deduplication history expired, or its validity/history cannot otherwise be established, do not infer that it names a new user intent or authorize a destructive mutation from it. Return `Indeterminate` or another safely non-executing status. Target-state revalidation alone does not prove replay is harmless. A new intent may be admitted only with a fresh identity and current authorization/precondition checks.
- `Rejected` applies to a new request refused before it starts and means only that this request caused no side effect. It makes no claim about earlier accepted requests. It is not the response for a duplicate whose original operation might have run.
- A fresh independent request, including a separate user activation, has a distinct identity. Conflicting mutations from independent requests require current-state/revision revalidation and core-side per-session or per-target arbitration before effects.
- Request-identity deduplication is for redelivery/retry of the same logical request. It does not promise permanent exactly-once execution or unlimited history; the concrete retention window is deferred to implementation.
- For future application operations, the Desktop should use `encryptdrive-core`; it must not reach into storage or crypto crates, construct authenticated-data contexts, or interpret persistent-format bytes.
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

Prefer an immediate request reply for acceptance or rejection, plus correlated updates only for accepted operations that outlive the call. A repeated delivery refers to that original reply/status; it is not a new acceptance or rejection. `Indeterminate` records that the original outcome cannot currently be established and does not assert whether an effect occurred. This keeps the contract understandable without committing to a framework or event-bus implementation.

| Reply, event, or status | Meaning | Correlation |
|---|---|---|
| Accepted (nonterminal) | A new request was accepted for processing; acceptance does not mean commit or success. A repeat returns this known state and its original operation identity, if assigned. | Request identity; accepted long-running work also receives an operation identity. |
| Rejected (new request only) | A new request was refused before it started; no side effect is attributable to that request. This says nothing about prior accepted requests and is never a substitute for unknown duplicate status. | Request identity and stable error category. |
| Indeterminate | The original request's outcome cannot be established, including when its identifier/history has expired or is no longer verifiable. It may have been accepted or may have produced effects; do not claim rejection, treat it as a fresh destructive request, or automatically replay an irreversible mutation. | Original request identity and any known operation identity; status itself must be safely scoped. |
| Progress (nonterminal) | Truthful completed/total units when known; never invent a percentage. | Request and operation identities, plus the session/attempt context when applicable. |
| Completed (terminal) | The operation reached its defined success/commit result. | Request and operation identities and applicable session context. |
| Cancelled (terminal) | The core confirms cancellation at a safe boundary. Cancellation does not imply rollback; disclose any partial effects safely. If an irreversible commit already occurred, report the actual completed/partial result, not cancellation with no effects. | Request and operation identities and applicable session context. |
| Failed (terminal) | The operation ended in failure, with a typed category and any explicit partial result. A recoverable error permits a new user request; it does not mean this operation continues. | Request and operation identities and applicable session context. |
| Session state changed | A selected/locked/unlocked/closed/invalidated transition completed under the lifecycle rules below. | Non-reused selection, unlock-attempt, or session identity/generation as applicable; never key material. |
| Capability availability changed | Core readiness changed after an implementation or runtime condition. This is a proposed future notification; the current Desktop does not consume `readiness()`. | Capability/readiness revision; may be application-scoped rather than session-scoped. |

Rejection is normally a reply rather than a later event: the UI must distinguish “new request not accepted” from “accepted, then failed” and from “original outcome indeterminate.” A terminal outcome ends its operation. Once recorded, later progress, duplicate terminal delivery, or out-of-order events must not change that outcome or create a contradictory one. Every event must be correlated to the correct request, operation, and applicable session or unlock attempt. A recoverable failure permits the user to make a new request; it does not continue the failed operation. An implementation may choose another transport after review, but must preserve these semantics. Progress and terminal payloads must not contain file contents, passwords, keys, plaintext paths, or other secret material.

### Correlation and stale results

- Each distinct user intent has a fresh request identity; redelivery of the same logical request reuses it. Accepted asynchronous work receives an operation identity stable through its terminal outcome. IDs exposed in logs or support messages are opaque, non-sensitive, unique within their defined scope, and generated independently of paths, account identities, usernames, secrets, and file contents. Use randomness or an equivalent non-derived source when appropriate; an ID is never an authorization credential. No specific ID format or library is selected.
- Session references and generation identities are core-issued and must not be reused within a running process. A new vault selection or session generation cannot accidentally match an invalidated one. Concrete representation is deferred; no distributed identity service or persistence requirement is imposed here.
- The core rejects commands against an invalid generation. The UI must not apply a result from an old request, operation, session, or unlock attempt to active vault/session state; it may preserve the original operation's safe status separately under the policy below. UI filtering supplements but never replaces core enforcement.
- Stale events cannot modify the active vault/session state. A late completion cannot restore an unlocked view, repopulate cleared private UI state, or be applied to another session. A completed mutating operation must not be silently presented as if it never occurred: preserve or surface a suitably scoped, non-sensitive outcome when needed for user understanding or recovery. A status shown after close or switch must not expose information from the previous vault in an unrelated active context; a safe outcome category and count may be sufficient, without paths or file names. Presentation and retention policy remain deferred.
- Request identity tracks the logical request independently of session validity: redelivery does not become a new operation merely because its originating session is invalid. An invalid session cannot authorize a fresh mutation. Historical-status handling must apply appropriate scope and privacy protections; no universal lookup-before-authorization order is prescribed. The core may return a properly authorized, sanitized known result or `Indeterminate`, without disclosing sensitive prior-vault information. A stale request cannot reactivate its old session or expose another vault's details.
- A terminal outcome is definitive for that operation; later or reordered delivery cannot replace it. Unknown status remains explicitly indeterminate until safely resolved and is not a license to replay an irreversible action.
- Ordering/delivery guarantees beyond these invariants, persistence of request identities across process restart, operation-record retention, and the concrete request-deduplication retention window remain open implementation decisions. No permanent or unlimited deduplication guarantee is made.

## 4. Conceptual session lifecycle

These states are a proposed future domain lifecycle, not current product functionality or a key-management design. The table is exhaustive for this proposal: transitions not listed are illegal and must be rejected without changing state.

| Current state | Request/condition | Next state | Required rule |
|---|---|---|---|
| No vault selected | Select a supported target | Vault selected, locked | Issue a fresh selection identity. Selection exposes no decrypted content and creates no authenticated session. |
| Vault selected, locked | Begin unlock | Unlocking | Assign a fresh unlock-attempt identity and bind it to the current selection identity/generation before authentication can complete. Do not create an authorized session yet. Keep credentials out of ordinary commands, events, logs, and diagnostics after secure handoff. |
| Vault selected, locked | Select a different target | Vault selected, locked (new selection) | Invalidate the old selection identity and issue a non-reused identity for the new selection. |
| Unlocking | Authentication and integrity checks succeed, and attempt plus selection are still current | Unlocked and authorized | Only the matching, active attempt may create a session. Issue fresh, non-reused session reference/generation after checks succeed. |
| Unlocking | User cancels or authentication/integrity checks fail | Vault selected, locked | Invalidate the attempt and its authorization potential. Failure uses safe generic wording and leaves no partially authorized session. |
| Unlocking | Close or select a different target | No vault selected, or Vault selected, locked (new selection) | Invalidate the old attempt before changing context. A late completion cannot authenticate the closed or superseded selection. |
| Unlocking | Expiry or forced invalidation | Vault selected, locked, or No vault selected | Invalidate the attempt and retain only the still-valid selection, if any. Late results are ignored for authorization. |
| Unlocked and authorized | Lock, close, or forced invalidation begins | Locking | Stop admitting commands for the old generation and resolve in-flight work at safe boundaries. Invalidate the session reference/generation before any later command can use it. |
| Unlocked and authorized | Unexpected session or integrity failure | Locking | Fail closed; do not retain apparent authorization after core validation fails. |
| Locking | Current locking transition reaches its safe boundary; session material and authorization are invalidated | Vault selected, locked | Clear session-derived UI state before presenting the locked state. Any new session must use a fresh identity/generation. |
| Vault selected, locked | Close, including a close request deferred during Locking | No vault selected | Invalidate the selection identity and preserve any necessary operation outcome separately from active-vault state. |

A close requested while `Locking` is deferred until that locking transition reaches its safe boundary. It then follows the listed close transition from `Vault selected, locked` to `No vault selected`. The proposal does not cancel or restart locking, and does not add a concurrent close transition.

Every unlock attempt is fenced before a successful authenticated session exists. A completion whose attempt/selection preconditions no longer hold cannot create or reactivate a session. Sensitive derived material from an invalidated attempt must no longer be usable to authorize operations or become available through a subsequently created session. This is a future security contract, not a claim about current behavior; it does not select key-storage or memory-management primitives.

Content operations (listing, search, import/export, rename, Trash, restore, and deletion) require an unlocked, authorized, current session and operation-specific permission. Selecting/opening a target, unlocking, locking, closing, and cancellation have separate preconditions. The final capability list and account model must be reviewed when those operations are actually scoped.

The core owns session validity and any future sensitive state. The UI may retain only the minimum transient display state needed for interaction; it must clear session-derived names, selections, search results, and pending confirmations when the generation is invalidated. This proposal does not select key storage, zeroization primitives, KDF settings, or memory-erasure guarantees.

## 5. Cancellation and concurrency

| Boundary | Proposed behavior |
|---|---|
| Before acceptance | Reject a new request before it starts; no side effect is attributable to that request. This does not make claims about other requests. |
| Accepted but not started | Core may confirm cancellation; no side effect may begin afterward. |
| Read-only work in progress | Cancel cooperatively at a documented safe point. If cancellation cannot be honored, report the actual terminal result rather than claiming cancellation. |
| Mutation before its irreversible/commit point | Cancellation is allowed only if the implementation can leave a valid state and clean up safely. The storage milestone must define and test this point. |
| After the commit point | Cancellation is too late. Report the committed outcome or truthful partial/failure result; never describe it as cancelled with no side effects and never claim rollback. |
| Partial effects before cancellation/failure | Terminal outcome must explicitly disclose that partial effects occurred at a safe level of detail, such as a category and count. Do not imply rollback. |
| Lock/close/session invalidation | Stop admitting work for the old generation and bring accepted operations to safe boundaries before invalidation completes. Preserve a completed mutating outcome separately when needed; invalidating session/UI state does not erase knowledge of the operation. |

Every operation must define whether it is read-only, cancellable, partially completing, and where its commit point lies before implementation. Independent conflicting mutations require core-side per-session or per-target arbitration; the UI must not assume parallel safety. Whether safe reads may overlap a mutation, and whether writes are serialized, rejected as busy, or queued, is operation- and storage-dependent and remains open. No concurrency framework or storage recovery behavior is selected here.

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
| Cancellation / too late | Report cancelled only when confirmed; otherwise report that the operation already completed or could not be cancelled. | Record terminal category and opaque correlation IDs only. |
| Partial completion | State safely what category/count completed and what remains, without implying rollback. | Per-target diagnostic IDs only if the owner-approved privacy policy permits them. |
| Unexpected internal failure | Generic message with a non-sensitive support/correlation identifier. | Sanitized error category and redacted source chain; no automatic raw-chain dump. |

Ordinary logs, UI notifications, and crash reports must never contain passwords, derived keys, authentication secrets, raw file contents, or decrypted metadata. Full filesystem paths, vault IDs, account/user IDs, and filenames are sensitive by default: show only when the active product workflow requires it, and redact them from diagnostics unless a reviewed policy explicitly permits a narrowly scoped value. Error chains must be sanitized before logging; a lower-layer source error is not automatically safe. This is a proposed contract, not a claim about current logging, which has no production vault operations to exercise.

## 7. Contract invariants

| Invariant | Status at Phase 1B-3 |
|---|---|
| The Desktop declares a direct Cargo dependency on `encryptdrive-core`; the validator checks package membership and dependency direction, not API call sites. | Existing verified boundary. |
| The Desktop currently calls no `encryptdrive-core` API and does not consume `readiness()`; no production command, event, or session behavior exists. | Verified current implementation. |
| Future Desktop application behavior uses framework-independent contracts through `encryptdrive-core`; lower crates remain independent of egui widgets and presentation state. | Proposed future boundary; implementation must be validated. |
| UX action or confirmation is not authorization to mutate a vault. | Proposed contract. |
| Future UI capability presentation reflects core readiness after real operations are separately implemented. | Proposed; current shell does not consume readiness. |
| A command cannot execute against a missing, locked, expired, or stale session. | Proposed contract; enforce in core before side effects. |
| A repeated delivery of the same request identity refers to its original status; unknown outcome stays indeterminate and is never treated as a new no-effect rejection. | Proposed contract; dedup retention is deferred. |
| Distinct user intents receive distinct identities; conflicting mutations revalidate revisions and are arbitrated per session or target. | Proposed contract; operation details are deferred. |
| Unlock attempts and session generations are fenced and never reused within one process; stale completions cannot reactivate a closed or superseded context. | Proposed future contract; identity representation and memory handling are deferred. |
| Terminal outcomes end operations; late, duplicate, or reordered events cannot contradict them. | Proposed contract; transport/delivery mechanism is deferred. |
| A stale completion cannot alter active session state, and a completed mutation is not silently presented as if it never occurred. | Proposed contract; safe status presentation and retention are deferred. |
| Destructive scope is revalidated and matches the confirmation before any side effect. | Proposed contract; persistence/transaction mechanism deferred. |
| Synthetic preview state and commands cannot reach production storage or real vault operations. | Existing feature/source boundary; preserve and test. |
| Cancellation is not rollback; partial effects are disclosed safely, and a committed mutation is not described as cancelled without effect. | Proposed contract; commit points/recovery are deferred. |
| Identifiers exposed in logs/support are opaque, non-sensitive, unique within scope, independently generated, and never authorization credentials. | Proposed contract; no format/library is selected. |
| Errors are never silently converted into success or optimistic completion. | Proposed contract. |
| User-visible diagnostics and ordinary logs contain no secret material or raw file data. | Proposed contract; logging implementation and review remain future work. |
| Persistent format and conformance rules are independent of UI framework and design. | Existing architecture direction; no format API is implemented here. |

## 8. Open decisions for review

1. What exact core-owned session reference and generation model will later APIs use, and how are generations invalidated across close, lock, account switch, and expiry?
2. What bounded in-process request-deduplication window is appropriate, which operation classes need request/operation identity retention across restart, and how should an indeterminate outcome be safely resolved after interruption?
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
4. Core rejects only a new request before that request has side effects, or accepts it with a correlated operation identity. A repeated delivery refers to the original status; if that status is unknowable, return `Indeterminate` and do not automatically repeat an irreversible mutation.
5. For accepted work, core emits truthful progress and one terminal completion/cancellation/failure result. The UI applies only results still matching the current request and session generation.

The specific payloads, validation rules, commit points, and event transport belong to future operation designs. A later CLI or other client may reuse a GUI-independent core boundary only after separate scope, compatibility, privacy, and release approval; this document does not authorize another client.

See the [Phase 1B-3 milestone record](../roadmap/PHASE_1B_APPLICATION_CONTRACTS.md) for review gates and handoff.
