# UX3 Internal UX Critic — Source Review

**Review type:** Independent read-only source review; no GUI launch or screenshot inspection.  
**Result:** Visual readiness BLOCKED; no design approval.

## Findings and response

- **Unlock glyph mapped to an absent/wrong codepoint:** fixed to Phosphor Lock Open U+E306; a regression test passes.
- **Clicking secondary cells did not establish stable row focus:** row response now combines the stable Name-cell response with the row response and requests focus on click. This needs live pointer/keyboard verification.
- **F2 could target the first ID in a multi-selection:** Rename now requires exactly one selected ID; unit tests pass.
- **Dark small-foreground action blue had poor contrast:** nav/folder icons and notices use semantic icon/text/focus roles. Automated semantic contrast checks pass; screenshot sampling is absent.
- **Theme did not set egui default text roles:** configured Heading/Body/Button/Small using the semantic scale; test passes.
- **Icon size, control dimensions, labels, and focus states were inconsistent:** shared Geometry, TypeScale, text-field helper, and custom icon+label button metadata were added. These custom-painted controls remain an accessibility risk until checked in UIA.
- **Long names were character-count clipped or ran across cells:** Name cell clips and label truncates by available width, with the full value available as a tooltip. Rendered behavior remains unverified.
- **Search and input names were unclear:** Search gets a TextInput label; reusable text fields associate their visible label with widget ID. Actual UIA mapping remains unverified.
- **The no-vault shell advertised unavailable actions as disabled buttons:** those controls were removed and replaced with explanatory copy.
- **Old review records claimed placeholder-brand and Light default:** current UX3 records supersede them; old docs are marked historical.

## Required follow-up

Run the owner manual script in [UX3 owner review guide](UX3_OWNER_REVIEW_GUIDE.md), capture authentic screens, inspect accessibility roles/names/focus/selection, and obtain owner visual approval. High Contrast, DPI, real keyboard-only operation, and Narrator remain NOT RUN. No reviewer claimed otherwise.
