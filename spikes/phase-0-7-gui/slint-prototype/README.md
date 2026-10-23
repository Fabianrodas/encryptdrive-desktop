# Slint prototype

Native Slint 1.18.1 Windows GUI using the shared GUI-independent state model. The file rows are custom `ListView` delegates because the documented `StandardTableView` current-row API does not itself establish the required multi-selection behavior. The delegates report stable entry IDs; modifier-click and range selection are held in the common state model.

Build from this directory with Rust 1.95 or later, target `x86_64-pc-windows-msvc`, and the Windows MSVC toolchain. Dependencies are locked in `Cargo.lock`:

```powershell
cargo test --locked
cargo build --locked --release
```

List size can also be changed in Settings / the footer control to 100, 1,000 or 10,000. Current Phase 0.7E source checks and Release build pass. The current executable is built at `../target/phase-0-7e/release/encryptdrive-slint-prototype.exe` from this crate directory. It has not been visually relaunched; see `../results/OWNER_GUI_REVIEW.md` and the parent results for evidence limits.

The implementation is an isolated spike. It must not be treated as product code or as a Slint license selection. See `../results/DEPENDENCY_AND_LICENSE_REVIEW.md`.
