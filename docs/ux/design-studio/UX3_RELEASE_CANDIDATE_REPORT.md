# UX3 R3 Release Candidate Identity and Build Evidence

**Candidate:** UX3 R3 source-remediation candidate for owner-assisted validation  
**Built:** 2026-10-20 (UTC timestamp recorded below)  
**State:** Release artifact built and hashed. It was not launched; GUI/manual validation remains pending.

## Exact final preview candidate

| Field | Value |
|---|---|
| Repository | C:\Users\Asus Vivobook\Desktop\encryptdrive-desktop |
| Branch | main |
| HEAD | Unborn branch; no commit exists; remote tracking ref is gone. |
| Starting/final Git visibility | Repository paths are untracked due to the unborn branch. No files were staged or committed. Changes remain visible in the working tree. |
| Final build command | $env:RUSTUP_TOOLCHAIN='stable'; cargo +stable build --locked --package encryptdrive-desktop --release --features design-preview |
| Additional Release check | cargo +stable build --locked --workspace --release passed immediately before the final preview build. The final preview command overwrote the shared executable path. |
| Profile / feature | Optimized Release / design-preview |
| Target | x86_64-pc-windows-msvc |
| Rust | rustc 1.99.1 (58cf8f9c3 2026-10-13); LLVM 23.1.1 |
| Cargo | cargo 1.99.1 (5f94df478 2026-08-27) |
| Executable | C:\Users\Asus Vivobook\Desktop\encryptdrive-desktop\target\release\encryptdrive-desktop.exe |
| Executable size | 15,580,672 bytes |
| SHA-256 | 0230F8EBFFF2717BB7F8FB64E2AFB8BFD359D32A00EBD6F703C72412468E62BD |
| Last write (UTC) | 2026-10-20T23:59:37.9183611Z |
| Cargo.lock SHA-256 | 3CB09A9815F937F30137C3B9169F33834B1412CEAD428406613A65C99A64D754 |
| Lock state | Present and unchanged by the --locked commands; untracked because the repository has no commit. |

The full hash and byte size were measured from the final preview Release artifact after both Release builds completed. No EncryptDrive process was running at the measurement check. The earlier R2 SHA 1E0F5497055273128313FA99F7ED67EBF0320A590DEE989510C595F58DCA50F3 is historical and does not identify this binary.

## Locked dependencies and runtime

The lockfile resolves the principal GUI/runtime crates to eframe/egui/egui_extras 0.37.0, winit 0.30.13, wgpu 30.0.1, and accesskit 0.25.1. The build is native Windows Rust with no WebView. No PE dependency-inspection tool was run in this task, so the imported system DLL inventory and runtime dependency closure are NOT RUN. AccessKit’s presence in Cargo.lock does not establish Windows UIA or Narrator behavior.

## Verification

The final source passed formatting, default and preview workspace checks, strict Clippy in both configurations, default tests (27 passed), preview tests (84 passed), both requested Release builds, and architecture boundary verification. Exact command outputs and coverage are in the regression test report.

Cargo emitted incremental-cache hard-link warnings in debug validation because the workspace filesystem does not support those links; Cargo copied the affected cache files. Release build commands completed without errors.

## Launch directly without rebuilding

First confirm the candidate SHA:

    Get-FileHash 'C:\Users\Asus Vivobook\Desktop\encryptdrive-desktop\target\release\encryptdrive-desktop.exe' -Algorithm SHA256

Then launch the exact preview candidate directly:

    & 'C:\Users\Asus Vivobook\Desktop\encryptdrive-desktop\target\release\encryptdrive-desktop.exe' --design-preview

To inspect the production shell from the same feature-enabled binary, launch without the preview argument:

    & 'C:\Users\Asus Vivobook\Desktop\encryptdrive-desktop\target\release\encryptdrive-desktop.exe'

These commands do not rebuild. Do not run another build before manual review unless you intentionally want to replace this candidate; both modes share the executable path.

The preview uses deterministic in-memory synthetic data. No real vault, password, key, encrypted data, local import/export, network operation, or filesystem mutation is performed. This candidate was not launched by Codex. No screenshot is associated with this SHA.

See the manual R3 validation matrix. This report records a review candidate only; it grants no product or design approval.
