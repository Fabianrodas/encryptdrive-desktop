# ADR-012 — eframe implementation candidate for EncryptDrive Desktop

**Status:** Owner-approved for the Phase 1A production shell; final production acceptance remains gated  
**Date:** 2026-10-19  
**Scope:** Native Rust Desktop GUI and production workspace boundaries

## Context

Phase 0.7 compared eframe/egui with Slint using synthetic data. Its final report did not select a shipping framework: current GUI builds were not visually retested, and Narrator/UI Automation, keyboard, DPI, clean-host/offline, runtime, and owner/legal gates remain open. The Phase 1A owner request authorizes eframe/egui for the first production workspace shell.

This is an implementation choice for the current shell, not a claim that the mandatory product release gates have passed. Phase 0.7 evidence is summarized in [the evaluation record](../ux/PHASE_0_7_FRAMEWORK_EVALUATION.md) and detailed under `spikes/phase-0-7-gui/results/`.

## Owner approval

- Use the maintained eframe/egui framework for the Phase 1A native shell.
- Keep the Desktop UI separate from reusable, GUI-independent core crates.
- Build the shell as Windows-first native Rust and preserve the offline product direction.
- This approval does not approve a Rust product license, distribution runtime, release, or untested accessibility behavior.

## Engineering decision

- Pin `eframe` and `egui_extras` to `0.37.0` and resolve the graph through the production root `Cargo.lock`.
- Enable eframe AccessKit integration, native system font support, and the WGPU renderer as the Phase 1A configuration.
- Put the app and four requested crate boundaries in one production Cargo workspace. The GUI directly depends on `encryptdrive-core`; the lower crates remain intentionally API-empty until their own contracts are authorized and validated.
- Keep the Phase 0.7 spike packages explicitly excluded and independently locked.
- Provide Files/Trash navigation, a compact Explorer table, disabled vault actions, and in-memory Light/Dark settings. Start with no open vault and make no data or network calls.

## Compatibility discrepancy

Phase 0.7's framework-comparison brief requested a synthetic Modified field and sort control. The approved Phase 0.5 UX documents explicitly avoid that field because the product data contract does not establish a trustworthy modification time; the Phase 0.7 report also records that Java's displayed timestamp is not maintained as a reliable modified time and that Java table sorting is disabled. The production shell therefore uses Name, Type, and Size and does not add a Modified value or sorting claim. Revisit only with an approved data contract and product decision.

## Evidence and mandatory gates

Engineering evidence for this phase is recorded in the [Phase 1A implementation record](../roadmap/PHASE_1A_WORKSPACE_AND_SHELL.md). The eframe shell and metadata-boundary check compile locally; that does not constitute visible UI, Narrator, DPI, high-contrast, offline clean-host, or installer validation.

Before production acceptance, complete the outstanding Phase 0.7 UI review (including Narrator/UI Automation and display scaling), verify keyboard and focus behavior on the actual shell, audit the exact production dependency graph and Microsoft runtime redistribution, and obtain the Rust product license decision. The Rust license remains undecided; no `LICENSE` file is included.

## Consequences

- The initial native shell can be developed without introducing a web runtime.
- eframe/egui remains subject to the recorded accessibility and platform gates; AccessKit being enabled is not proof of correct roles or announcements.
- Phase 1A exposes no vault, file, crypto, authentication, storage, network, or telemetry capability.
- The repository remains an implementation workspace, not a public release or licensing decision.
