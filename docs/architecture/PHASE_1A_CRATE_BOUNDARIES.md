# Phase 1A crate boundaries

The production workspace contains five packages and keeps the Phase 0.7 spike outside its membership.

| Package | Phase 1A contents | Direct dependencies |
|---|---|---|
| `encryptdrive-desktop` | Native eframe shell, view state, semantic tokens | `encryptdrive-core`, eframe, egui_extras |
| `encryptdrive-core` | Product identity and explicit engineering-preview readiness | None |
| `vault-format` | Documented boundary only | None |
| `vault-crypto` | Documented boundary only | None |
| `vault-storage` | Documented boundary only | None |

The empty lower crates are deliberate. Phase 1A requested the architecture foundations but did not authorize format parsing, cryptographic primitives, vault persistence, or domain operations. Giving these crates dependencies or stub functions before any contract exists would imply implementation that is not present. The Phase 1A dependency graph therefore validates only the Desktop-to-core edge. Add later edges when real APIs are designed and tested.

`scripts/verify-architecture.ps1` checks exact workspace membership and direct dependency boundaries using Cargo metadata. The nested Phase 0.7 `common`, eframe, and Slint manifests remain excluded and keep their existing lockfiles.
