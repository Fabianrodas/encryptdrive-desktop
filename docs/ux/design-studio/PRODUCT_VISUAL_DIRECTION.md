# Phase 1A-UX — Product visual direction

> **Superseded for current implementation by UX3.** This earlier document describes the pre-UX3 candidate. Its placeholder-brand, palette, font-size, control, and selection notes are historical. See [UX3 final visual report](UX3_FINAL_VISUAL_REPORT.md), [Design System V3](DESIGN_SYSTEM_V3.md), and [brand provenance](BRAND_ASSET_PROVENANCE.md). Visual acceptance remains pending.

**Status:** Implementation candidate; live visual review and owner approval pending  
**Direction:** Native Explorer × Premium Desktop  
**Evidence boundary:** Source, model tests, build checks, and one benchmark run are available. The app was not launched because the earlier explicit stop on GUI input remains in force. No screenshots or live accessibility observations exist.

## Design intent

Keep Files and Trash as the primary destinations, make the file list the main work surface, and make sensitive or destructive consequences explicit. Treat this as an engineering preview: the default shell says that vault support is unavailable, and the opted-in design preview is conspicuously labeled as synthetic. No vault, filesystem, account, network, authentication, crypto, or persistence code is called by the preview.

The direction follows the owner-approved product request. This is not final visual approval.

## Implemented structure

- **Window:** eframe native WGPU window; initial client size 1180×792 logical points, minimum 960×640. Sidebar defaults to 214 points and is resizable from 168 to 284. Main content uses 22-point padding. Source values are implemented; all requested window sizes still need live inspection.
- **Header and sidebar:** app name, temporary original geometric husky mark, local/offline indicator, settings, Files/Trash navigation, and preview state/dataset controls. The active destination has a leading bar, dot, selected surface, and selected accessibility state. The preview status is repeated in more than one place and should be judged for density during live review.
- **Commands:** New Folder and Import are prominent in the preview. Export, Rename, Move to Trash, Restore, and Permanent Delete are available when their selection/state permits, with contextual row actions. Permanent deletion requires a distinct confirmation dialog.
- **Explorer:** virtualized `egui_extras::TableBuilder` rows with Name, Type, Size, and `Modified · sample`; Name is the flexible column and other columns are resizable. Only synthetic fixtures expose the sample timestamp and sort. Production shell data does not exist.
- **Feedback:** in-window status, error, and operation progress. Preview labels and text identify synthetic operations. No OS notifications are used.
- **Themes:** Light and Dark semantic palettes share token roles. The preview starts in Light; choice is session-only. Rendered contrast, High Contrast, and text scaling remain unverified.
- **Brand:** the husky is drawn with egui shapes and is explicitly a temporary original placeholder. No Java artwork or unlicensed external icon/font asset is included. Logo rights and the final brand asset remain owner decisions.

Exact current semantic values and implementation mapping are in [Design System V3](DESIGN_SYSTEM_V3.md). Research and alternatives are in [Design Research](DESIGN_RESEARCH.md) and [Competitive Analysis](COMPETITIVE_ANALYSIS.md).

## Synthetic preview

The preview is compiled only with the `design-preview` feature and starts only with `--design-preview`. Without both, the application launches the no-vault shell. The preview offers 22 named screens/states and deterministic 100-, 1,000-, and 10,000-entry fixtures. Its model includes nested and empty folders, Unicode and long names, synthetic types/sizes/dates, stable IDs, and deleted records.

Preview actions mutate only the in-memory model. Import reads no files. Export does not decrypt, open, or write a file and shows a plaintext warning. Forms use transient sample fields and do not create accounts/vaults or authenticate. Recovery is explanatory sample copy, not a recovery algorithm. See the [Screen and State Matrix](SCREEN_STATE_MATRIX.md).

## Interaction contract in source

- Files and Trash navigation, folder navigation, breadcrumbs, Back, and Up are represented in the preview model and UI.
- Selection state uses stable entry IDs. Model tests cover plain, Ctrl, Shift-range, visible-result Ctrl+A, and stable IDs under filtering/sorting.
- Enter and double-click navigate into folders; Files items present the explicit simulated Export dialog. F2 renames Files items; plain Delete opens Move to Trash confirmation in Files; Shift+Delete opens permanent-delete confirmation in Trash. Row commands require table focus. Ctrl+F focuses search; Ctrl+A selects current visible results; Escape closes a dialog, clears a focused search, or clears selection; arrow keys move row focus; Alt+Left and Alt+Up navigate.
- Tab order is delegated to egui standard interaction handling. Shift+F10 opens the focused row's context menu in source. Trash menus expose only Restore and Permanent Delete; Files menus expose actions applicable to that entry.
- Dialogs have action-specific titles, descriptions, labels, inline folder-name validation, Cancel and confirmation actions. Create Folder and Rename request initial focus on the name field; destructive dialogs request initial focus on Cancel. Dialogs record the invoking widget and request focus back to it after cancellation, with a last-focused-row/Files-navigation fallback. After confirmed destructive removal, focus returns to the stable Files or Trash tab. Context actions on selected rows preserve multi-selection for group actions; Rename and Open remain single-item actions. These behaviors are implemented in source but have not been exercised live.

See [Interaction and Accessibility Review](INTERACTION_AND_ACCESSIBILITY_REVIEW.md) for source-level evidence and the owner validation steps.

## Decisions and remaining gates

The decision register explains choices, alternatives, and tradeoffs. The implementation reviews and known limitations are in [Iteration Log](ITERATION_LOG.md), [Critic Review](CRITIC_REVIEW.md), and [Visual Acceptance Report](VISUAL_ACCEPTANCE_REPORT.md).

**Not approved yet:** actual Light/Dark quality, layout at 960×640 / 1180×792 / 1600×900 / 1920×1080, 100% / 150% / 200% scaling, Windows High Contrast, Narrator/UI Automation semantics, dialog focus behavior, and final husky artwork. The owner must run the [Owner Review Guide](OWNER_REVIEW_GUIDE.md) before deciding visual acceptance. Product authentication, password requirements, recovery semantics, real operations, and branding/license gates remain outside this phase.
