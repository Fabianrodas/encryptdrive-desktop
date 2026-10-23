# UX3 External Review Packet

## Request

Review the Phase 1A-UX.3 native Rust design candidate against the requirements below. The implementation uses eframe/egui and remains isolated from production vault, crypto, and storage behavior. The opt-in `design-preview` uses deterministic in-memory fixtures only.

## Focus areas

1. Explorer density and hierarchy at 960×640 and 1180×792 logical points.
2. Files/Trash navigation, breadcrumbs, search, sortable table, selection, and action hierarchy.
3. Light/Dark/System colors, typography, icon clarity, long/Unicode filename treatment, and focused/hovered/selected states.
4. Plain language around simulated Import/Export, plaintext warning, vault-unavailable empty shell, and error/recovery feedback.
5. Keyboard-only discoverability and focus behavior; accessible name/role/state quality for custom controls and rows.
6. Product risks: custom painting, contrast states, High Contrast, screen reader behavior, font and logo attribution.

## Available evidence

- Source/model/build/test evidence summarized in [UX3 final visual report](UX3_FINAL_VISUAL_REPORT.md).
- Current token contract: [Design System V3](DESIGN_SYSTEM_V3.md).
- Independent source review result: [UX3 external critic result](UX3_EXTERNAL_CRITIC_RESULT.md).
- Manual reproduction and safety boundaries: [UX3 owner review guide](UX3_OWNER_REVIEW_GUIDE.md).

## Review boundary

No screenshot or live screen is attached because no UI launch or computer-use automation was performed. Do not infer rendered quality from source. If live visual review is authorized later, use only synthetic data and do not enter personal passwords, paths, vaults, or files. Record actual observations, build identity, screen size, scaling, theme, and any screen reader/tool used. This packet requests critique; it does not grant asset redistribution rights or approve Phase 1 implementation.
