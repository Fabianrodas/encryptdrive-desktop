# Phase 1A-UX — Formal closeout

**Decision date:** 2026-10-26<br>
**Decision authority:** Project owner<br>
**Decision:** UX6.3 **APPROVED AS VISUAL BASELINE**<br>
**Milestone:** Phase 1A-UX visual design **CLOSED**<br>
**Reference commit:** `25efccc6471443db03b97ae9c79c74685834e951`<br>
**Independent technical review:** **READY FOR OWNER REVIEW**; the owner reports no confirmed P0/P1 findings.<br>
**Owner acceptance:** Confirmed.

The independent review outcome above is recorded from the project owner's supplied approval instruction. Its full audit artifact is not present in this repository. This closeout records the owner's decision; it does not claim a new technical audit or rerun prior checks.

## Approved visual baseline

- Native Rust Desktop using egui/eframe 0.37.
- Option C: balanced visual density with a Windows-native, Fluent-inspired desktop interface.
- The existing EncryptDrive husky identity, with provenance documented in [BRAND_ASSET_PROVENANCE.md](BRAND_ASSET_PROVENANCE.md).
- System appearance by default, with Light and Dark overrides.
- The current responsive explorer, navigation, and command-bar behavior.
- UX6 typography, component geometry, and iconography as refined and recorded through UX6.3.
- UX6.3's DPI-aware initial-window sizing strategy as the current technical baseline.
- Design-preview data is synthetic. This approval introduces no real vault operations.

UX6.3 is the reference for future visual comparisons. Future UI changes should be targeted, justified, validated, and incrementally committed. Significant changes to this visual direction require renewed owner approval. Ordinary bug fixes, accessibility improvements, and small usability refinements remain in scope.

## Evidence record

The following are existing records, not tests rerun for this documentation closeout:

- [UX6 report](UX6_REPORT.md) — visual decisions, implementation evidence, and baseline limitations.
- [UX6.1 report](UX6_1_REPORT.md) — responsive command bar, accessibility semantics, build identity, and 16 captures under `captures/ux61/`.
- [UX6.2 report](UX6_2_REPORT.md) — destructive-operation scope, confirmation behavior, tests, and captured delete dialogs.
- [UX6.3 report](UX6_3_REPORT.md) — DPI-safe startup sizing, simulated coverage, native 100% measurements, and unrun configurations.
- Commits `260d83a`, `59aa866`, `06f78a8`, `3692f76`, `8339d47`, `fd4148e`, `2a67d9b`, `ef43e08`, `c53c42e`, `f29d69e`, `58aac6a`, and `25efccc` record the reviewed UX6 through UX6.3 increments. The reference commit is the final commit in this list.

The UX6.1 captures are visual evidence for the interface structure and responsive states. UX6.2 and UX6.3 record subsequent behavior and window-sizing work; their reports identify what was and was not verified. Historical test and artifact claims remain attributable to those reports and their stated commits.

## Scope exclusions

Owner visual approval does **not** mean:

- Vault Format 1 is implemented or conformant.
- Real encryption or storage is implemented or ready.
- Accessibility is certified or complete.
- The Windows DPI compatibility matrix or multi-monitor behavior is complete.
- Installer or portable distribution is certified.
- A production release is approved.

No Phase 1B implementation, Vault Format 1, cryptography, storage, or real-vault work is authorized by this visual-baseline closeout.

## Open gates and risks

These remain future verification or release gates; they do not undo the visual-baseline decision:

| Open item | Status and next gate |
|---|---|
| Narrator and UI Automation | Not completed. Run against the Windows UIA tree and screen-reader experience before accessibility sign-off; UX6.1 records the current semantics and limitations. |
| High Contrast | Not completed. Verify the application in Windows High Contrast before accessibility sign-off. |
| Native DPI at 125%, 150%, and 200% | Not run for UX6.3. Measure the release candidate at each scale before declaring the supported Windows display matrix. |
| Secondary/mixed-DPI monitors, negative coordinates, and alternate taskbar placement | Not run natively. Verify on the corresponding hardware or a representative Windows test environment before declaring multi-monitor support. UX6.3 includes simulated cases, which do not replace this gate. |
| More-menu keyboard and interaction checks | Pending manual input checks recorded by UX6.1. Verify open, traversal, disabled items, Escape/focus return, and command invocation before declaring full keyboard-interaction acceptance. |
| Nonblocking DPI P2 observations | Remain open for the DPI/display-compatibility gate; this closeout does not waive or relabel the independent review's P2 observations. UX6.3's unrun native configurations remain explicit evidence gaps. |
| Reproducible builds and artifact separation | Follow up in build engineering before relying on reproducible release artifacts. UX6.1 documents link-generated metadata, registry paths, and the shared output-path risk. |
| Runtime redistributables and native dependencies | Verify from a clean Windows host as part of packaging readiness. |
| Installer/portable packaging and clean-host execution | Validate against the owner-approved Windows and distribution targets before release approval. |
| Security and Vault Format 1 conformance | Separate engineering and security gates before any real vault functionality or release. This UX decision supplies no cryptographic or storage approval. |

No completion dates are assigned here. Each item should close against its stated evidence gate, not by inference from visual approval.

## Engineering transition

- `main` remains the primary integrated branch, with UX6.3 as the approved visual reference.
- A `dev` branch may be created when functional product development actually begins; it is not created by this closeout.
- Functional feature branches may be created from `dev` when a change warrants separate development or review.
- Architecture, cryptography, and storage work require their own engineering gates.
- Claude Code remains the principal UX implementer/reviewer. Codex is the preferred functional engineering implementation agent, with independent review by Claude Code or another reviewer.

**Next roadmap transition:** prepare and separately authorize Phase 1B functional engineering. This closeout stops before that work begins.
