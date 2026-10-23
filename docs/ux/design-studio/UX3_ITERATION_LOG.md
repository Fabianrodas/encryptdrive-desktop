# UX3 Iteration Log

**Date:** 2026-10-19. **Scope:** design-system consistency, brand restoration with explicit provenance, source defects, and owner handoff. No changes were made to the Java or private documentation repositories.

| Finding | Change | Verification |
|---|---|---|
| System theme was not the initial preference | Added System/Light/Dark preference and Dark fallback when the system appearance is unknown | Workspace tests pass; OS theme transition not run |
| Placeholder animal mark did not match product branding | Restored exact PNG from Java source and used it in header/window; recorded matching source/destination hash | SHA-256 match; visual inspection not run; redistribution rights remain owner/legal gate |
| Typography was scattered | Added semantic TypeScale and applied roles to egui default style | Token/style unit tests pass; rendered comparison not run |
| Unbalanced measurements/radii | Added shared Geometry tokens; aligned control/table/navigation dimensions and main padding | Geometry test passes; responsive rendering not run |
| Mixed Unicode and unnamed PUA symbols | Added Phosphor semantic icon map, embedded font, explicit license/attribution file, human names on custom controls | uniqueness and Unlock codepoint tests pass; glyph appearance/UIA not run |
| Dark action blue used for icons/feedback | Use semantic icon/text/focus roles for small foregrounds; action blue remains action fill | 46 contrast-pair assertions pass; rendered sampling not run |
| Filtering retained hidden selected IDs | Effective query change clears selection; unchanged query/sort preserves IDs | Preview model tests pass |
| Arrow focus/selection behavior was ambiguous | Plain arrow selects focused row; Shift extends from anchor; Ctrl moves focus only | Model and app tests pass |
| Multi-selection could rename first sorted ID | Rename is offered only for one selected item and target helper rejects other counts | Unit tests pass |
| Row focus marker could cross cell boundaries | Paint a narrow focus indicator in Name cell; combine stable Name response with row response; request focus on row click | Source review only; live focus/hover rendering not run |
| Search and labels had weak source semantics | Added Search TextInput name and label association for shared fields | Compiles; UIA/Narrator not run |
| Empty shell showed permanently disabled vault buttons | Removed unimplemented controls and retained honest explanatory copy | Source review and build checks pass; visual review not run |

Internal and external critics completed separate source-only reviews. Both required actual GUI evidence before approval. See [internal critic](UX3_INTERNAL_CRITIC.md), [external result](UX3_EXTERNAL_CRITIC_RESULT.md), and [final visual report](UX3_FINAL_VISUAL_REPORT.md).
