# eframe prototype

Native Rust window using eframe 0.37.0, egui_extras 0.37.0, and the shared in-memory model. The selected renderer is WGPU. Native AccessKit and `system_fonts` support are enabled. The table uses `egui_extras::TableBuilder` with a fixed header and visible-row rendering. The test suite includes a Windows system-font shaping check for the Japanese fixture name; it does not validate the live GUI or Narrator.

The independently locked dependencies are recorded in `Cargo.lock`. Rust 1.95 or later, the `x86_64-pc-windows-msvc` target and Windows MSVC build tools are required to build. From this directory run `cargo test --locked` and `cargo build --locked --release`. Supported `--entries` values are 100, 1000, and 10000. The app does no file, vault, or network I/O.

The Cargo versions are exact direct pins. eframe/egui 0.37 requires Rust 1.95 or later. Current Phase 0.7E source checks and Release build pass. The current executable is built at `../target/phase-0-7e/release/encryptdrive-eframe-spike.exe` from this crate directory. It has not been visually relaunched; see `../results/OWNER_GUI_REVIEW.md` and the parent results for evidence limits.
