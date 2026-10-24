# UX6 — Balanced density and design refinement

**Start:** `main` at `ca05dfa` (UX5 visual baseline `a3d5d17`), clean working tree.
**Direction:** the owner chose *balanced density*. UX5 is refined, not replaced.
**Status:** candidate ready for independent Codex review. This is not owner approval.

## References

- [Microsoft: content layout and spacing](https://learn.microsoft.com/en-us/windows/apps/design/basics/content-basics). Gaps of 8 px between controls, 12 px between areas, 16 px to a surface edge. Body text 14 px; Standard sizing balances density and comfort; compact sizing is for dense, pointer-only regions.
- Linear's write-ups of its [2024 redesign](https://linear.app/blog/how-we-redesigned-the-linear-ui) and [later refresh](https://linear.app/blog/behind-the-latest-design-refresh). Keep information density while removing noise, give secondary chrome less weight than the content, and avoid overusing icons.
- I did not verify specific values for Raycast or 1Password and did not use them.

## Defects found in UX5 renders, and changes

| Area | Defect (seen in the 125% render) | Change |
|---|---|---|
| Typography | Table metadata and column headers at 12 px looked small and faint next to 14 px names; headers were lighter than the data. | One defined role, `METADATA` = **13 px**, for cells and headers. Caption (12) stays for the status bar and helper text. The rest of the ramp is unchanged (14 / 18 / 20). |
| Status bar | 32 px dropdowns in a 40 px bar looked heavy for secondary chrome. | Shared compact density (`use_compact_density`): **28 px** controls, bar of **36 px**. Button height now comes from `interact_size`, so one painter serves both densities. |
| Header | The "Sample vault" chip (≈27 px) inherited the 32 px interact height and looked like a button. | Chip fixed at **24 px** with a pill radius. |
| Inputs | Text fields and dropdowns used a heavy full outline (#868686) that clashed with the subtle buttons. | One Fluent input frame for text fields, search and dropdowns: subtle outline, **1 px bottom edge at 3:1** at rest, **2 px accent** on focus. The outer focus ring is kept on dropdowns. |
| Command bar | "Delete permanently" was the only outlined command in Trash. | New `SubtleDanger` kind: transparent at rest, danger-coloured text, red-tinted hover. Dialogs keep the outlined destructive button. |
| Responsive | At 960×640 the bar collapsed to icons only even with free space; search was fixed at 260 px; the sidebar took 220 px. | Sidebar **184 px** below 1100 px of window width. Progressive collapse: all labels from 840 px of content width, then only "New folder" (and both Trash commands) keep labels, then icons with tooltips. Search sized **160–320 px** from the spare width. |
| Dark warning | The plaintext export warning was tinted muddy olive in Dark. | Fluent caution colours (`#FFF4CE` / `#433519`) with matching strokes; contrast tests added. Text, icon and behaviour unchanged. |

**Kept from UX5, after review:**
- 36 px rows and 32 px standard controls (Fluent Standard sizing).
- 4/8 px radii and the accent and selection-pill colours.
- 16 px Phosphor icons. At 125%, their optical alignment with 14 px text in the command bar, sidebar, search and table was checked visually; no correction was needed.

## Verification

**Light and Dark.** Captured on Windows 11 at 125% scaling (genuine Release window only, synthetic data):
- Files and multi-selection.
- Trash.
- Export and delete dialogs.
- Error in the status bar.
- Unlock and create vault.
- Settings.
- Production shell.

**States:**

| State | Evidence |
|---|---|
| Selected (fill + pill) | Seen in captures. |
| Focus | Seen in captures (Cancel in dialogs, outer ring). |
| Disabled | Seen in captures. |
| Destructive | Seen in captures (dialog button and Trash command). |
| Hover | Only incidental (wherever the real cursor was); no input was injected. |

**Responsive** (`--window-size` flag; at 125% the work area is 1536×816 logical points):

| Requested size | Result |
|---|---|
| 880×600 (minimum) | Compact layout, no clipping. |
| 960×640 | Narrow sidebar, primary label kept, search 320 px. |
| 1180×792 | Height fitted to the work area (744 px inner); full labels. |
| 1600×900 and 1920×1080 | Fitted to the work area, centred; nothing behind the taskbar. |

**Scaling:**
- 125% verified natively.
- **100%, 150% and 200%: not verified.** Changing Windows scaling is a system setting, so I didn't touch it.
- The design uses only logical tokens and never applies its own scale on top of the native DPI.

**Behaviour and safety:** no input-routing logic was changed. These remain intact and tested:
- Activation gating on the frame a dialog opens.
- Initial focus on Cancel.
- Enter never confirms Trash/Delete/Import/Export.
- Key-repeat filtering.
- No egui context reentrancy (the click test performs a real click).
- Keyboard navigation and stable selection.

**Performance:** headless frame with 10,000 entries: **median 2.28 ms, max 104.7 ms** (the first frame builds the font atlas). All 18 native launches reported Responding=True.

## UI scale preference

I recommend **not** adding a 100/110/125% interface scale for now. With metadata at 13 px, Body at 14 px and compact density limited to secondary chrome, text reads comfortably at 125%, and Windows scaling already covers users who need larger text.

Even so, every size comes from `TypeScale`/`Geometry`, so a future preference could apply a single factor (for example through egui's `zoom_factor`, which is separate from the native pixels-per-point) without double scaling. Reconsider after reviewing 100% and 150% on real displays.

## Tests and builds

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace --locked` (default and preview) | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` (default and preview) | 0 warnings |
| `cargo test --workspace --locked` | 30 passed |
| Tests with `--features encryptdrive-desktop/design-preview` | 96 passed (3 core, 10 library, 65 UI, 18 model) |
| `scripts/verify-architecture.ps1` | PASS; `encryptdrive-core` keeps no GUI dependencies |
| Release shell | `target/release/encryptdrive-desktop.exe`, 14,866,432 bytes, SHA-256 `F7DC5A295E9CF6EE48DFCC02F9B53D5AF7123C5A6CAB250D002FD732F14AB6C9` |
| **Release preview (final candidate)** | same path, 15,464,960 bytes, SHA-256 **`84CB861D19DA59CB3900EB791CFAF375BCECAFA460C35A3CE24BDA4A640B1BBB`** |

Both builds write to the same path; the final file is the preview.

New tests cover:
- `--window-size` parsing.
- The sidebar breakpoint.
- Command-bar density and the search-width bounds.
- Caution-colour contrast.

## UX6 commits

| Commit | Subject |
|---|---|
| `98cb453` | style(ui): balance table typography and compact secondary chrome |
| `2bd5c0b` | style(ui): unify input, combo box and command geometry |
| `e12b540` | fix(ui): adapt sidebar, command bar and search to window width |
| `512fa69` | fix(ui): use Fluent caution colours for the export warning bar |
| this commit | docs(ux): record UX6 decisions and evidence |

## Evidence

`captures/ux6/`, captured with `PrintWindow` on the app window only:
- **`before_*`:** UX5 visuals at 960 and 1180.
- **`after_*`:** final SHA in Light and Dark, at 880, 960, 1180 and the fitted 1920.

Reproduce, for example:

```powershell
.\target\release\encryptdrive-desktop.exe --design-preview --theme=dark --window-size=960x640 --scenario=14
```

## Accessibility still pending

- Narrator, the UI Automation tree and High Contrast were not run.
- Disabled text is about 3:1, as intended for disabled controls.
- The input frame keeps 3:1 only on the bottom edge, which is the Fluent pattern; the subtle outline on the other three sides is below 3:1.

## Owner decisions

1. Accept 13 px for table metadata (one step outside the Windows ramp) or go back to 12.
2. The input style: Fluent bottom edge, versus the stronger full outline from UX5.
3. Blue folder icons (Explorer uses yellow).
4. Whether the 184 px narrow sidebar is enough, or a collapsible icon-only rail is wanted below about 1000 px.
5. Review at 100% and 150% on physical displays before deciding on an extra UI scale.
