# Design research dossier

**Scope:** Phase 1A-UX product-surface research for a native Windows file manager. This is source research and code review; the running app was not inspected because the owner previously stopped desktop input with Escape.

## Project evidence

Phase 0.5 establishes the compact Explorer-inspired information model, Files and Trash destinations, keyboard access, Light/Dark appearance, and explicit destructive consequences. ADR-012 keeps the production data table to Name, Type, and Size because no trustworthy Modified-time contract exists. The engineering shell is a no-vault state; source inspection found no vault, account, file, network, or authentication operation in it. See [Phase 0.5 design system](../DESIGN_SYSTEM.md), [navigation and interactions](../NAVIGATION_AND_INTERACTIONS.md), [security/privacy UX](../SECURITY_PRIVACY_UX.md), and [ADR-012](../../architecture/ADR-012-EGUI-EFRAME-DESKTOP.md).

**Evidence limit:** This dossier does not assert current visual quality, Narrator output, UI Automation roles, High Contrast behavior, focus order, or DPI correctness. Those require the live Windows application and accessibility tools.

## Windows conventions

**Source facts:** Microsoft’s Windows guidance covers familiar navigation, command surfaces, keyboard operation, accessible names and state, visible focus, text scaling, and accessibility checks with tools such as Narrator. It recommends simple hierarchy and consistency across common Windows controls. [Design basics](https://learn.microsoft.com/en-us/windows/apps/design/basics/) · [Keyboard interactions](https://learn.microsoft.com/en-us/windows/apps/design/input/keyboard-interactions) · [Accessibility checklist](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/accessibility-checklist) · [Contrast themes](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/high-contrast-themes)

**Design judgment:** Use a compact header, persistent labeled Files/Trash destinations, breadcrumbs, a focused command area, and the file table as the primary surface. This makes the core task obvious at first glance and keeps product familiarity. It does not make egui controls native WinUI controls; custom egui interactions still need deliberate names, focus, and platform verification.

**Tradeoff:** A command palette can make keyboard actions efficient but hides ordinary file operations and adds a custom focus model. Keep a standard toolbar/context menu as the primary discoverability path and treat a command palette as optional future work.

## egui and accessibility

**Source facts:** The project pins eframe/egui 0.37.0 with AccessKit enabled. The versioned egui accessibility guide explains that ordinary widgets expose semantics while custom-painted controls need explicit `WidgetInfo`; it also describes `egui_kittest` for querying roles and names. AccessKit models an accessible element tree. [egui 0.37.0 accessibility guide](https://github.com/emilk/egui/blob/0.37.0/docs/accessibility.md) · [AccessKit architecture](https://github.com/AccessKit/accesskit)

**Design judgment:** Use standard egui controls where practical and add explicit semantics to custom navigation and rows. A compiled AccessKit feature or source-level role is not proof of a useful Windows UIA tree or Narrator announcement.

**Accessibility/security consequence:** File rows need a useful name and selected state; keyboard focus must remain separate from selection; modal labels and Cancel actions must be announced. Avoid putting vault names, file names, or paths into OS-level notifications. Manual UIA and Narrator checks remain a release gate.

## High contrast and security language

**Source facts:** Microsoft documents contrast themes as separate from ordinary light/dark appearance, recommends adequate contrast, and calls for text scaling and assistive-technology testing. [Contrast themes](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/high-contrast-themes) · [Text scaling](https://learn.microsoft.com/en-us/windows/apps/develop/input/text-scaling)

**Design judgment:** Keep the Phase 0.5 restrained palette, but verify actual rendered contrast in each state and evaluate system High Contrast before calling the interface accessible. The current code adapts the semantic Light/Dark palette only; no live High Contrast behavior is established.

Use explicit wording for account/vault boundaries, plaintext export consequences, irreversible deletion, and unavailable features. Do not imply secure erasure, password recovery, cloud sync, or completed cryptography.

## Rejected patterns

- Phase 0.7’s synthetic “Modified” data is permitted only in this visibly synthetic preview and is labeled `Modified · sample`; production data semantics remain unresolved.
- Reject dashboard metrics, large decorative cards, gradients, global file search, cloud-sync indicators, remembered identities, and repeated lock badges.
- Do not copy legacy artwork, private screenshots, or fonts until ownership and redistribution rights are confirmed.
- Prefer one-step Move to Trash only if the product later has a reliable undo/restore contract; keep confirmation for permanent removal.

The comparable product patterns and their fit/tradeoffs are in [Competitive analysis](COMPETITIVE_ANALYSIS.md). Decisions and evidence limits are recorded in [Decision register](DESIGN_DECISION_REGISTER.md).
