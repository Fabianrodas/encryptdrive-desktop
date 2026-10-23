# Phase 1A — Production workspace and architecture foundation

**Status:** Implemented locally; review required  
**Date:** 2026-10-19

## Delivered

- Production Cargo workspace with the Desktop shell and four explicit crate boundaries.
- Rust toolchain channel declaration, root lockfile, Windows CI, and dependency-boundary verification.
- Native eframe/egui shell with Files/Trash navigation, Name/Type/Size table, disabled actions, explicit no-vault state, and in-memory Light/Dark settings.
- Core identity/readiness API with no available product capabilities.
- Public architecture record in [ADR-012](../architecture/ADR-012-EGUI-EFRAME-DESKTOP.md) and package boundaries in [the crate map](../architecture/PHASE_1A_CRATE_BOUNDARIES.md).

## Validation status

See the [Phase 1A implementation report](PHASE_1A_IMPLEMENTATION_REPORT.md) for exact compiler versions, repository status, build/test results, release binary measurements, and limitations. The [manual checklist](PHASE_1A_MANUAL_VALIDATION.md) records the outstanding GUI scenarios. The GUI was not opened in this run; visual inspection, Narrator/UI Automation, keyboard interaction, DPI, high-contrast, and clean-host offline launch remain unverified.

## Explicitly out of scope

No Vault Format 1 parser, cryptography, password derivation, vault file I/O, account authentication, file import/export, Trash/recovery operations, telemetry, network service, installer, portable package, or Phase 1B work is included. The Rust product license and final GUI release acceptance remain open.
