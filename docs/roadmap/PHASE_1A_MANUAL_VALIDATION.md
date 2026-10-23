# Phase 1A manual validation checklist

**Status:** NOT RUN in the Phase 1A implementation turn. No screenshot or interactive observation is claimed. The earlier explicit stop on desktop UI automation remains in force for this run.

Run on a Windows x64 machine with the target display scale and runtime recorded. Start with the Release build from the repository root:

```powershell
.\target\release\encryptdrive-desktop.exe
```

Record Windows version/build, GPU/driver, monitor scale, window dimensions, runtime installation state, and test date. Do not select or inspect user files or vaults.

## Shell behavior

- [ ] Launch and confirm the window title is EncryptDrive — Engineering preview.
- [ ] Confirm Files is initially selected and the visible state says no vault is open.
- [ ] Select Trash and return to Files; confirm the status remains accurate and no file entries are invented.
- [ ] Confirm New folder, Import, Export, Rename, Delete, and Search are disabled and explain why on pointer hover.
- [ ] Open Settings; switch Light → Dark → Light and confirm the interface uses the corresponding semantic palette.
- [ ] Confirm the appearance preference is described as in-memory only and is not written to disk.
- [ ] Confirm the window resizes down to 760×520 and that the navigation, breadcrumbs, toolbar, table, and status remain readable.

## Keyboard and accessibility

- [ ] Use Tab and Shift+Tab through the header, Files/Trash navigation, disabled actions, and Settings; record focus visibility and order.
- [ ] Switch the appearance radio options using only the keyboard.
- [ ] Inspect the UI Automation tree with Accessibility Insights for Windows or equivalent; record roles and accessible names for the application status, destinations, actions, table headings, and Settings.
- [ ] Test with Windows Narrator; record announcement order and whether disabled controls expose their unavailable state and reason.
- [ ] Repeat with Windows text scaling and display scale at 100%, 125%, 150%, and 200%; inspect clipping and focus-ring visibility.
- [ ] Test High Contrast mode and record any contrast or visibility issues.

The production shell does not yet implement file-table gestures, file-operation dialogs, context menus, custom file-manager shortcuts (Enter, Ctrl+A, Ctrl+F, F2, Delete, arrows, Shift+F10, Alt+Left, or Alt+Up), or operation progress. Those behaviors were exercised only in the isolated Phase 0.7 synthetic prototypes and must not be treated as present in this application.

## Runtime and offline behavior

- [ ] Launch on a clean supported Windows image and record required Microsoft runtime components.
- [ ] Disconnect network access before a cold launch; confirm no runtime download is attempted and the shell remains usable.
- [ ] Inspect process file/network activity to confirm no user files are accessed and no network requests are made.
- [ ] Save screenshots at 1180×760 logical pixels for Light, Dark, the Files view, and the Trash view; attach them to the review record without modifying private assets.

