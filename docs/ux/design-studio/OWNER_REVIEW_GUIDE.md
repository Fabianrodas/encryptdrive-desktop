# Owner review guide

> **Superseded by [UX3 owner review guide](UX3_OWNER_REVIEW_GUIDE.md).** The earlier commands, binary sizes, keyboard notes, logo questions, and UI assertions below predate UX3 and are retained only as history. Use the current guide for owner validation.

## Safe launch

The default executable contains no synthetic file explorer and no vault operations. It displays the no-vault engineering shell. The design preview requires both a Cargo feature and an explicit argument; it uses only generated in-memory sample entries.

From the repository root in PowerShell:

```powershell
$env:RUSTUP_TOOLCHAIN = 'stable'
cargo build -p encryptdrive-desktop --release --locked
.\target\release\encryptdrive-desktop.exe
```

For the synthetic preview:

```powershell
$env:RUSTUP_TOOLCHAIN = 'stable'
cargo build -p encryptdrive-desktop --features design-preview --release --locked
.\target\release\encryptdrive-desktop.exe --design-preview
```

The latest verified builds were produced on 2026-10-19 with Rust `1.99.1 (58cf8f9c3 2026-10-13)`, Cargo `1.99.1 (5f94df478 2026-08-27)`, target `x86_64-pc-windows-msvc`:

| Build | Size | SHA-256 |
|---|---:|---|
| Default no-vault shell | 14,145,536 bytes | `B23F2D449346AE54F0E7B5DCD8548B02B0618DF09EF7B5716EF361AF43D17D05` |
| `design-preview` | 14,691,840 bytes | `35BAF8B5877934C8A540D7C1C6E0F70F006C9C21118284B7A3CCBB6FC5C90424` |

The commands above reproduce the respective build. Cargo writes both variants to the same executable path, so run the matching build command immediately before launch.

The feature build alone does not enable the preview. The `--design-preview` argument is also required. No real vault, account, password store, file picker, file access, network request, cryptography, or persistent storage is involved. Do not enter a real password in any sample form.

## Manual visual and interaction acceptance

Record the executable build/version used and save only synthetic screenshots under an owner-approved review directory.

1. Start with the preview. Confirm that the header says `DESIGN PREVIEW`, the picker says `sample-only data`, and no real vault/file action is possible.
2. Capture the initial 100-item list at 1180×792 in Light. Repeat at 960×640, 1600×900, and 1920×1080. Resize both wider and narrower to the 960×640 minimum. Check table header alignment, clipping, action wrapping, navigation width, focus visibility, and whether synthetic notices crowd the list.
3. Repeat 1,000 and 10,000 root entries. Scroll near the beginning, middle, and end. Compare responsiveness and selection behavior after changing list size.
4. Search, sort Name/Type/Size/Modified · sample, select one row, Ctrl-click, Shift-click, Ctrl+A, then filter/sort while selection remains. Confirm that action counts and the visible selected-outside-search explanation are correct.
5. Navigate folders with pointer and keyboard, use breadcrumb/Back/Up, and double-click a file. Confirm it opens only the explicit plaintext Export warning. Confirm no external file or app is opened.
6. Exercise Files/Trash, New Folder, Rename, Move to Trash, Restore, Permanent Delete confirmation, and name validation with blank, invalid, and colliding names. Confirm synthetic-only copy throughout.
7. Exercise Import, Export, progress, cancel, completion, failure, and busy/disabled actions. Verify focus remains predictable and feedback is announced.
8. Use keyboard only: Tab/Shift+Tab, Enter, Escape, Ctrl+A, Ctrl+F, F2, Delete, Up/Down, Shift+F10, Alt+Left, and Alt+Up. Record deviations and whether focus and selection remain distinct.
9. Review all 22 named scenarios from the Preview state picker, including empty, loading, error, recovery, settings, and account/vault representative forms. Never use a real password or private filename.
10. Repeat Light and Dark at 100%, 150%, and 200% Windows scaling. Enable Windows High Contrast and repeat core flows. Capture only synthetic screens.
11. Inspect Windows UI Automation roles, names, selected/focused states, dialogs, inputs, buttons, and progress. Run Windows Narrator through navigation, selection, dialog validation, and operation status. Record actual speech and focus order; do not infer from source annotations.
12. Build the release executable, disconnect networking, and confirm offline launch and normal rendering. Record any runtime prompts or missing native dependencies.

## Owner decisions

- Is the temporary husky placeholder acceptable until brand ownership and redistribution rights are confirmed?
- Does the visual hierarchy, table density, typography, and Light/Dark contrast meet the product direction at all required sizes and scaling levels?
- Are all authentication/account/vault examples clearly nonfunctional and product-vocabulary accurate?
- Is the current disabled no-vault shell the right Phase 1A default, with Create/Open visible but unavailable?
- What High Contrast behavior is required before accessibility acceptance?
- After reviewing evidence, should this design candidate proceed to implementation refinement, or should any visual direction change?

The owner makes the final visual approval. This guide does not authorize Phase 2, vault operations, cryptography, or changes to the other repositories.
