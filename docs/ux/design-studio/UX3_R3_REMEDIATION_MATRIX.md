# UX3 R3 Remediation Matrix

**Prepared:** 2026-10-20  
**Implementation status:** Source corrections and automated regressions recorded below.  
**Owner validation:** Pending; no final visual, accessibility, or product approval is asserted.

## Evidence boundary

The R3 request says Claude Code issued a **CONDITIONALLY READY** verdict and summarizes several remaining defects, but the complete `UX3_INDEPENDENT_EXTERNAL_AUDIT_R3.md` was not present in the supplied attachment or repository. The complete R1/R2 reports and R2 live-test annex were also not present. The available R1/R2 summaries are recorded in [`UX3_R2_REMEDIATION_MATRIX.md`](UX3_R2_REMEDIATION_MATRIX.md), which is not a substitute for those originals. No sanitized copies have been fabricated; see [`external-audits/README.md`](external-audits/README.md).

Consequences:

- “R1 summary 01–11” below are row ordinals in the existing Codex summary matrix, **not original auditor IDs**.
- `R2 P1-N*` identifiers are preserved only where they appeared in the owner’s pasted summaries. Other findings use an explicit summary key, not an invented auditor ID.
- “R3 request A–F” are headings from the remediation request, **not original R3 auditor IDs**.
- The original-ID mapping, the individual dispositions of all 18 R1 P2 findings, any omitted R2/R3 findings, and exact original acceptance wording are **BLOCKED** until the complete source reports are supplied.
- GUI events below are tested through egui’s headless event/render path. They are not physical Windows input or live visual verification.

## R1 findings available in the pasted summary

| Available key / severity | Finding as summarized | Correction and evidence | Remaining evidence | Status |
|---|---|---|---|---|
| R1 summary 01 / P1 | Ubuntu Light appeared instead of Segoe UI; weak weight hierarchy. | Windows system Segoe UI/Segoe UI Semibold loading and explicit typography roles remain in source. Typography tests pass. | Final Release glyph rendering and font fallback need owner screenshot review. | **FIXED, LIVE VALIDATION PENDING** |
| R1 summary 02 / P1 | `.strong()` changed color rather than weight, causing selected-color headings. | Explicit semibold font family and semantic text colors remain; affected UI does not use `.strong()`. | Release pixels not captured. | **FIXED, LIVE VALIDATION PENDING** |
| R1 summary 03 / P1 | Components, spacing, sidebar, and headers were inconsistent. | Shared component helpers and shared shell/preview header and navigation remain in use. R3 also separates hover and focus treatments using Design System V3 tokens. Geometry and state tests pass. | Owner must inspect shell/preview and component states in the Release window. | **FIXED, LIVE VALIDATION PENDING** |
| R1 summary 04 / P1 | Table headings and breadcrumbs looked like bordered buttons. | Breadcrumbs use quiet controls; sort headers are drawn without the icon-font sort glyph and place the geometric marker after the full label. | Release appearance and clipping not observed. | **FIXED, LIVE VALIDATION PENDING** |
| R1 summary 05 / P1 | Selection and focus were conflated; initial focus and weak Light selection contrast. | Stable-ID selection, row focus outline, and semantic Light/Dark contrast assertions remain separate. | UIA state and pixels need owner inspection. | **FIXED, LIVE VALIDATION PENDING** |
| R1 summary 06 / P1 | Primary action focus ring was difficult to see in Light. | Shared focus painter and token contrast contracts remain; focused Light text-field shape test passes. | Release rasterization and Narrator/UIA remain pending. | **FIXED, LIVE VALIDATION PENDING** |
| R1 summary 07 / P1 | Table keyboard behavior and dialog Enter/tab handling were incomplete. | Stable-ID table navigation, shortcut routing, text-edit exclusion, dialog opening-frame protection, and non-repeat Enter confirmation have event-driven regressions. See test report. | Native Windows keyboard, Tab order, Shift+F10, Alt+Left/Up, and focus handoff are not live-tested. | **FIXED, LIVE VALIDATION PENDING** |
| R1 summary 08 / P1 | Selection moved the table and risked misdirecting a double-click. | Selection actions now use reserved geometry. Rendered explorer tests compare row position through selection and feedback changes; compact layout also has a geometry test. | Logical headless geometry is not a screenshot or physical pointer test. | **FIXED, LIVE VALIDATION PENDING** |
| R1 summary 09 / P1 | Duplicate/out-of-view notifications and clipped progress. | One feedback slot with fixed 36 logical-pixel height renders before the table; operation text, count, progress, Cancel, and percentage have separate layout. Geometry tests verify selection, notice, error, and progress do not move the row. | Long-list visual visibility and clipping need Release review. | **FIXED, LIVE VALIDATION PENDING** |
| R1 summary 10 / P1 | Contradictory vault copy, inaccurate empty-folder copy, and redundant context menu heading. | Preview and shell status copy remain distinct; empty folder, empty Trash, and search-empty copy differ; redundant context-menu heading is absent. Source and copy tests cover these rules. | Exact original wording and Release layout need source report and owner review. | **FIXED, LIVE VALIDATION PENDING** |
| R1 summary 11 / P1 | Destructive action order differed; redundant Close button. | Dialogs use shared action placement with Cancel on the right; redundant Close is absent. | Native focus order and rendered button order need owner review. | **FIXED, LIVE VALIDATION PENDING** |
| Original R1 P2 IDs/descriptions unavailable / 18 findings | The pasted summary says 18 P2 issues remain but does not enumerate them. | No issue text, ID, acceptance condition, or disposition has been inferred. | Complete R1 report required to review and resolve each of the 18 findings. | **BLOCKED** |
| R1 original-ID mapping | The current summaries provide row ordinals, not original auditor IDs or line references. | No IDs were invented. | Complete R1 source required for ID-by-ID traceability. | **BLOCKED** |

## R2 findings available from the owner’s summaries

| Available identifier / severity | Finding | Correction and regression evidence | Remaining evidence / disposition | Status |
|---|---|---|---|---|
| R2 summary P0 / P0; original ID unavailable | Enter on a selected file opened and confirmed Export in one frame, skipping the plaintext warning. | A rendered file Enter event leaves Export warning open and operation idle. The button path separates pointer activation from keyboard activation; only a fresh non-repeat Enter on the focused control can confirm. | Physical key repeat and Release UI require owner validation. | **FIXED, LIVE VALIDATION PENDING** |
| P1-N1 / P1 | Enter confirmed destructive action while Cancel showed focus. | Enter with initial Cancel focus cancels both Move to Trash and Permanent Delete. Tests also cover held Enter, repeated Enter, and deliberate focus change before a fresh confirmation. | Physical keyboard/focus-ring confirmation pending. | **FIXED, LIVE VALIDATION PENDING** |
| P1-N2 / P1 | Light text-field focus appeared borderless. | Shared focus ring uses the semantic Light focus token; egui output-shape test asserts the 2 px focus stroke. | The earlier live observation contradicts the source test expectation. Release screenshot is required; do not treat the shape test as visual proof. | **FIXED, LIVE VALIDATION PENDING** |
| R2 arrow-focus summary / P1; original ID unavailable | ArrowUp moved focus out of the table after one row. | Table event filter keeps vertical and horizontal arrows and Escape in the table; Tab/Shift+Tab may leave. Sequence tests cover navigation and focus retention. | Native keyboard and window manager behavior remain unobserved. | **FIXED, LIVE VALIDATION PENDING** |
| P1-08 / P1 | Selecting a row moved the table vertically. | Headless rendering compares the same row’s Y position before/after selection; selection and feedback geometry tests pass. | Owner must verify the Release pixels and click target stability. | **FIXED, LIVE VALIDATION PENDING** |
| P1-03 / P1 | Shell/preview sidebar differed and hover looked like focus. | Shared navigation helper remains; hover fill, keyboard-focus outline, selected fill/indicator, pressed surface, and disabled styling use separate semantic treatments. | Side-by-side window screenshots and UIA remain pending. | **FIXED, LIVE VALIDATION PENDING** |
| P1-04 / P1 | Sorted label weight/order and “Modified · sample” were reported wrong. | Header uses the full label and a geometric sort marker after the text. The source label is `Modified`; sort mapping test passes. | Release weight, marker position, and clipping need visual confirmation. | **FIXED, LIVE VALIDATION PENDING** |
| P1-05 / P1 | Selection blue was considered too saturated. | Existing approved Design System V3 selection tokens were preserved; no arbitrary replacement color was introduced. | This is still an owner product-direction decision. Inspect the current candidate before changing the approved token. | **BLOCKED** |
| P1-10 / P1 | Production shell reportedly showed five stacked phrases including “Choose a vault…” without a matching control. | Current source contains the shared `No vault open` header and one explanatory shell body sentence; no unsupported vault action was added. | The exact five-phrase stack is not represented by current source, but without the complete report and live shell capture the visual claim cannot be closed. | **NOT REPRODUCIBLE** |
| P1-N4 / P1 | Sorted Name header displayed “^ N”. | Sort mark no longer comes from the icon font; header test verifies the full label remains separate from the geometric marker. | Release rendering still needs confirmation. | **FIXED, LIVE VALIDATION PENDING** |
| P1-N5 / P1 | Type/Size/Modified cells sat about 11 px above Name. | Metadata center test asserts text center within 0.5 logical px of the row center. | Perceived Windows glyph baseline needs owner screenshot review. | **FIXED, LIVE VALIDATION PENDING** |
| P1-N6 / P1 | Table commands were blocked because `egui_wants_keyboard_input()` is true for any focused widget. | Table shortcut routing is gated by text-edit focus. Event tests cover Ctrl+A/Ctrl+F, row keys, and normal text-edit exclusion. | Native Windows keyboard remains pending. | **FIXED, LIVE VALIDATION PENDING** |
| P1-N7 / P1 | Clicking filename text did not select; folder double-click did not navigate. | Headless pointer events on rendered filename text select stable ID; double-clicking rendered folder navigates; file double-click leaves Export warning open. | Physical pointer behavior in Release remains pending. | **FIXED, LIVE VALIDATION PENDING** |
| P1-N8 / P1 | 6–8 px black strip below table header. | No source cause or verified correction has been established. | Requires a current Release capture and reproducible rendering case before a safe correction can be chosen. | **OPEN** |
| R2 “five new P1s” count / missing P1-N3 | Summary annex names N1, N2, N4–N8 but does not provide P1-N3 or a complete original P1 list. | No missing finding was inferred. | Complete R2 report and live annex required for count/ID reconciliation. | **BLOCKED** |
| R2 first-capture list offset / later withdrawn | List was reported to start several rows down, then the reviewer said it did not reproduce. | No deterministic source defect is claimed. | Revisit only if owner reproduces this in the R3 candidate. | **NOT REPRODUCIBLE** |
| R2 Rename/Move-to-Trash differing fills / P1 summary | Light-mode buttons appeared to have different fills; pointer/focus states were not known. | Both use the shared button helper; hover, pressed, focus, and disabled treatments have separate semantic rules. | Original capture state is unknown; the current button pair needs owner inspection. | **FIXED, LIVE VALIDATION PENDING** |

## R3 findings summarized in the supplied remediation request

| Trace key (request heading, not original ID) | Finding | Correction and evidence | Remaining evidence | Status |
|---|---|---|---|---|
| R3 request A / original ID unavailable | Enter-repeat/held key could open and immediately dismiss a dialog or bypass the plaintext Export warning. | Fresh non-repeat event detection is applied to dialog confirmation; initial dialog frame suppresses activation; form focus is restored only when Enter—not Tab—caused focus loss. Event tests cover ordinary, repeated, held, Escape, initial Cancel focus, and deliberate focus changes, including destructive confirmation. | Headless egui routing is not physical Windows key input. | **FIXED, LIVE VALIDATION PENDING** |
| R3 request B / original ID unavailable | Left, Right, and Escape could transfer table focus unexpectedly. | Explicit policy: arrows and Escape stay within table; Escape clears selection while preserving the focused stable row; Tab/Shift+Tab traverse out normally; text editing retains its shortcuts. `app.rs` and event-filter sequence tests cover the policy. | Native Tab order and physical key behavior remain pending. | **FIXED, LIVE VALIDATION PENDING** |
| R3 request C / original ID unavailable | Hover and focus shared indistinguishable styling for some controls. | Existing tokens now separate hover fill/outline, focus ring, selection fill/indicator, pressed surface/border, and disabled surface/muted text/border. Light/Dark semantic tests and strict Clippy pass. | Only owner screenshots/UIA can verify actual rendered distinction. | **FIXED, LIVE VALIDATION PENDING** |
| R3 request D / original ID unavailable | About 75 px unused vertical space between commands and table. | Selection toolbar content moved into the command area where it fits and is reserved at a fixed height in compact layout. The normal test window’s first row is at 173 logical px; the prior headless measurement before this refactor was 197 px (24 px reduction). Compact and selection geometry tests pass. | These are headless logical coordinates, not live pixels; owner should assess density and clipping. | **FIXED, LIVE VALIDATION PENDING** |
| R3 request E / original ID unavailable | Status notifications pushed the table down. | Fixed 36 logical-pixel status region is always allocated before the table; notice, error, and running-operation states retain that region. Rendered-row tests compare identical coordinates across no-feedback, selection+notice, error, and progress. | Long-list scrolling, visual clipping, and perceived spacing require owner review. | **FIXED, LIVE VALIDATION PENDING** |
| R3 request F / original ID unavailable | Reconcile all historical R1/R2 issues, including 18 P2 findings and prior P0/P1 fixes. | Every finding present in the owner’s pasted summaries is included above; no P0/P1 was dismissed. | Full R1/R2/R3 originals and R2 annex were not supplied, so completeness and original IDs cannot be established. | **BLOCKED** |

## Additional implementation regression found during R3 validation

| Finding | Correction and evidence | Status |
|---|---|---|
| Default (no preview feature) compilation included preview-only sort/header helpers, which referenced preview-only type-scale constants; this caused `cargo check --locked --workspace --all-targets` to fail. | Preview-only helpers are gated by `design-preview`; preview/test-only focus and geometry/text tokens use matching cfg gates. Default and preview checks pass. | **FIXED AND TESTED** |
| Strict Clippy found duplicate hover/disabled border branches and a constant-only assertion in compact-layout test. | Hover retains the separator border while disabled falls back to the outline token; the geometry test asserts rendered row stability directly without a tautological constant assertion. Both strict Clippy variants pass. | **FIXED AND TESTED** |

## Verification evidence

- Final preview Release SHA-256: `0230F8EBFFF2717BB7F8FB64E2AFB8BFD359D32A00EBD6F703C72412468E62BD`; size 15,580,672 bytes. See [Release candidate report](UX3_RELEASE_CANDIDATE_REPORT.md).
- `cargo +stable fmt --all -- --check`: PASS.
- `cargo +stable check --locked --workspace --all-targets` and the same command with `--features encryptdrive-desktop/design-preview`: PASS.
- Strict Clippy with `-- -D warnings`, default and preview-feature variants: PASS.
- Locked workspace tests: default 27 passed; preview 84 passed; 0 failed in both. Zero-test targets are not counted.
- Locked default workspace Release build and final preview Release build: PASS.
- `scripts/verify-architecture.ps1`: PASS — workspace membership and direct dependency boundaries verified.
- Release was not launched; no GUI input injection, screenshot, DPI, High Contrast, UIA, or Narrator session was performed. See [owner checklist](UX3_MANUAL_R3_VALIDATION_MATRIX.md).

## Remaining gates

The external-source reconciliation is **BLOCKED** until the complete sanitized reports are provided. Current Release visual and native-interaction validation is **pending owner operation**. `OPEN` findings remain open; `BLOCKED` findings are neither dismissed nor treated as resolved. This register does not grant final UX, accessibility, or production approval and does not authorize Phase 1B or vault-format work.
