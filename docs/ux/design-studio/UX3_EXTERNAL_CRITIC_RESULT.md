# UX3 External Critic Result

**Reviewer:** Separate external UX critic agent; read-only source review.  
**Date:** 2026-10-19.  
**Verdict:** **Visual readiness BLOCKED. No approval.** No GUI was launched, no screenshot was inspected, and no Windows UIA/Narrator test was run.

## Findings

1. Unlock used a mismatched glyph. **Resolved in source:** map set to U+E306 and regression test passes.
2. Row focus was tied too narrowly to the Name response. **Addressed in source:** Name-cell response is the stable identity and is unioned with the full row response; row click requests focus. **Needs live verification:** focus after clicking each column, sorting, resizing, and scrolling.
3. Multi-selection Rename could target a deterministic but unintended first ID. **Resolved in source:** toolbar and F2 require exactly one selected row; unit tests pass.
4. Dark action blue did not work as a small foreground. **Addressed in source:** icon/text feedback now uses semantic readable colors; contrast tests include selected, hover, and focus pairs. **Needs rendered verification:** actual pixels, disabled states, feedback, and native title bar.
5. Default egui text style values did not follow the shared scale. **Resolved in source:** Heading, Body, Button, and Small roles configured and tested.
6. Custom controls and truncation needed consistent semantics. **Addressed in source:** icon-label buttons have explicit human names; search has a TextInput name; shared fields link labels; long names use cell clipping and a tooltip. **Needs UIA/Narrator verification.**
7. Existing records were stale about branding and default Light. **Resolved in documentation:** V2 and earlier visual-direction docs are marked superseded; UX3 records describe System default and the restored logo provenance.

## Gate to approval

The critic requires owner-run visual review with genuine screenshots, real keyboard-only checks, Windows DPI/High Contrast review, accessibility-tree inspection, and Narrator testing. The husky image is proprietary in the source repository; public redistribution rights still require product/legal approval. These are unresolved gates, not score deductions or inferred passes.
