# UX3 Final Visual Report — Source Candidate, Visual Gate Pending

> **Historical snapshot.** This 2026-10-19 report predates the R2 defects and the current Release candidate; its prior pass counts and build hashes are not current. The R3 source remediation, fresh locked checks, candidate identity, and owner validation gates are recorded in the [`R3 remediation matrix`](UX3_R3_REMEDIATION_MATRIX.md), [`regression report`](UX3_REGRESSION_TEST_REPORT.md), [`Release identity`](UX3_RELEASE_CANDIDATE_REPORT.md), and [`manual checklist`](UX3_MANUAL_R3_VALIDATION_MATRIX.md).

> **Current audit status:** This report records the earlier UX3 pass. External report reconciliation remains blocked because the complete R1/R2/R3 originals were not supplied; current findings and limitations are listed in [`UX3_R3_REMEDIATION_MATRIX.md`](UX3_R3_REMEDIATION_MATRIX.md). Live visual acceptance remains pending; this report has no screenshots for the R3 candidate.

**Date:** 2026-10-19  
**Implementation state:** Source changes and tests are complete for this turn.  
**Visual acceptance:** **NOT RUN / BLOCKED.** No app launch, genuine screenshot, screen recording, visual measurement, UI Automation inspection, Windows Narrator session, or owner visual decision is recorded.

## Delivered source changes

- System/Light/Dark appearance preference with Dark fallback if host theme cannot be read.
- Exact legacy husky PNG restored in the Rust application and recorded with matching source/destination SHA-256. Public asset redistribution approval remains outstanding.
- Shared typography and geometry, semantic color roles, icon mapping, Phosphor license notice, labels, and custom-control accessibility metadata.
- Safer filtered-selection behavior, keyboard focus/selection tests, single-item Rename rule, row focus identity, search metadata, and removed permanently disabled vault buttons.
- Earlier Phase 1A design-system/visual-direction notes are explicitly marked historical or superseded.

## Evidence recorded

- Toolchain: Rust `1.99.1 (58cf8f9c3 2026-10-13)`, Cargo `1.99.1 (5f94df478 2026-08-27)`, target `x86_64-pc-windows-msvc`, Windows 11 Professional build `10.0.26200`.
- `cargo fmt --all -- --check`, workspace/preview `cargo check`, workspace/preview Clippy with `-D warnings`, workspace/preview tests, workspace release build, preview release build, and `scripts/verify-architecture.ps1` all passed. The workspace suite passed 20 tests; the preview-feature suite passed 40 tests (including 16 model integration tests).
- Release files: default shell `target/release/encryptdrive-desktop.exe`, 14,982,144 bytes, SHA-256 `9AD26D340E90EBEB5B618C36E6FB43979C92B224AF8158C3811D851085017B59`; preview build 15,526,912 bytes, SHA-256 `2BD4DADBE72007FBE1A6C06EA5657484B4AF0CED9F2BEED771C8264EDE83B2AB`. Cargo writes both configurations to the same path; rebuild the desired feature set before launch.
- A single release-mode model benchmark reported: 100 entries—fixture 0.111 ms, cold projection 0.070 ms, cached 1,000 projections 0.017 ms; 1,000—0.869 / 0.435 / 0.021 ms; 10,000—7.189 / 3.692 / 0.017 ms. This is model-only timing, not frame rate, memory use, or interactive performance.
- Contrast and token consistency tests passed, but they evaluate configured values rather than captured pixels.
- Internal and external critic agents independently completed source-only reviews and withheld visual approval; [external critic result](UX3_EXTERNAL_CRITIC_RESULT.md).
- No screenshots are available. No visual score is assigned.

Cargo reported that hard linking incremental compilation cache files was unavailable and copied them instead. The build/test commands still returned exit code 0. No PE dependency inspection utility (`dumpbin`, `llvm-readobj`, `llvm-objdump`, or `objdump`) was available, so dynamic runtime dependencies have not been independently enumerated.

## Blocking owner checks

Follow the exact manual sequence in [UX3 owner review guide](UX3_OWNER_REVIEW_GUIDE.md), including real launch, target window sizes, Light/Dark/System, 100/150/200% scaling, keyboard-only workflows, accessibility tree, and Narrator. Keep screenshots synthetic and record the executable SHA-256. Resolve logo redistribution rights with the owner/legal reviewer before any public product distribution.

**Decision:** Keep the eframe design as an implementation candidate. It is not approved for production UI architecture on UX3 evidence alone. No Phase 1 production feature, vault operation, or cryptographic implementation is authorized by this report.
