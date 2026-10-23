# EncryptDrive Desktop — Proposed Design System

**Status:** Proposed for review  
**Visual direction:** Native explorer familiarity with restrained premium polish  
**No framework is selected.** These rules are platform-independent and should be checked in both framework candidates.

## Identity

Use a compact wordmark and a simple, original line mark derived from a file drawer or drive bay: a square outline with a clear horizontal seam and one offset file tab. Keep it legible at 16–32 logical pixels. Do not use a generic glowing padlock, shield collage, mascot, gradient, or copied product mark. The Windows taskbar/app icon must remain distinguishable in monochrome and light/dark contexts. Final mark and wordmark require brand review.

## Visual principles

- Content and operations take visual priority over decoration.
- Use Windows title-bar and window conventions.
- Keep one clear primary action per dialog or form.
- Use neutral surfaces and one restrained blue action color.
- Distinguish selected, focused, hovered, disabled, and destructive states without depending on color alone.
- Prefer separators and spacing over nested cards and shadows.
- Use consistent outline icons sparingly; every icon-only control needs a visible tooltip and accessible name. Never use emoji as icons.
- No decorative gradient, glow, blurred glass, animated counters, or large empty hero region.

## Typography

- Use the platform system UI typeface: Segoe UI Variable on Windows where available, with the OS system sans-serif fallback on other targets.
- Do not fetch fonts remotely or require a network for typography.
- Default body text: 14 logical px; compact secondary metadata: 12 logical px minimum; section titles: 18–20; dialog/page title: 20–24.
- Use regular weight for file rows and medium weight for active labels/actions. Use bold only for hierarchy or irreversible-action titles.
- Allow system text scaling and reflow; never truncate an error, field label, button action, or confirmation consequence.

## Color tokens

The palette below is a proposal. Contrast ratios shown are calculated for the listed foreground/background pairs. Recheck all actual states, disabled controls, and framework-rendered colors before implementation. Normal text must meet at least 4.5:1; essential control boundaries and non-text state indicators must meet at least 3:1 where applicable.

| Role | Light | Dark | Use |
|---|---|---|---|
| Canvas | #F7F8FA | #1A1D21 | Window background |
| Surface | #FFFFFF | #242930 | Main content and dialog surfaces |
| Surface secondary | #EEF1F4 | #2C333B | Sidebar and secondary regions |
| Hover | #F1F4F7 | #2E353E | Hovered rows and controls |
| Selection | #E6F0FC | #254A73 | Selected row background |
| Selection text | #183C66 | #D6E8FF | Text on selected row |
| Primary text | #18212B | #F2F4F7 | Main labels and file names |
| Secondary text | #596573 | #B5C0CB | Supporting labels and metadata |
| Primary action | #155EEF | #155EEF | Primary buttons, active marker |
| On primary action | #FFFFFF | #FFFFFF | Primary button label |
| Light focus ring | #155EEF | #91BEFF | 2 logical px outer focus ring |
| Control outline | #7D8995 | #707D89 | Inputs and controls requiring boundaries |
| Soft separator | #E0E4E8 | #3B444E | Non-interactive row/section dividers |
| Error action | #B42318 | #B42318 | Destructive action |
| Error surface/text | #FCE8E7 / #8B1E16 | #3A1F20 / #FFB4AB | Error state with icon and text |
| Success surface/text | #E8F5EC / #166534 | #1D3427 / #91D7A7 | Completed state with icon and text |
| Warning surface/text | #FFF2D1 / #714000 | #3B2E17 / #FFD28A | Export/recovery warning with icon and text |

Calculated reference ratios for sample pairings: light primary text 15.30:1; light secondary text 5.59:1; white on primary action 5.41:1; white on error action 6.57:1; dark primary text 15.35:1; dark secondary text 7.92:1; dark selection text 7.31:1. Control-outline ratios against their surfaces are 3.57:1 light and 3.47:1 dark. These are token-pair checks, not a substitute for validating focus, disabled, selected, and high-contrast states in the actual framework.

Do not encode file/folder type, success, corruption, lock, or selection by color alone. Pair semantic color with a label, icon, position, or shape. The standard Windows high-contrast setting must remain usable.

## Spacing, geometry, and density

- Base spacing scale: 4, 8, 12, 16, 20, 24, 32 logical px.
- Main sidebar width: 184 logical px in the standard layout; minimum target 160.
- Toolbar height: 44 logical px, with 32–36 px controls.
- Table header: 32 logical px; compact row: 36 px; comfortable row: 42 px.
- Row height grows for larger text scaling and touch usage. Hit targets must not collapse below the applicable accessibility requirement.
- Standard control radius: 4 px; dialog radius: 8 px; status indicators may be circular. Do not use pill-shaped buttons or rounded cards as the default.
- Use 1 px separators for passive structure. Use a stronger boundary and label for interactive inputs.
- The file list uses aligned columns and a strong Name column; numbers are right-aligned. Avoid decorative panel shadows.

## Components

### Buttons

- Primary: solid blue fill with white text; one per form/dialog.
- Secondary: neutral surface, clear outline or state fill.
- Destructive: red label or red button only at the point of confirmation; never use red for routine navigation.
- Disabled: retain readable label and explain unmet requirement next to the control.
- All actions have a visible text label unless the icon is universally obvious, has a tooltip, and has an accessible name.

### Inputs

- Persistent labels, not placeholder-only labels.
- Focused input has a 2 px visible focus ring and readable caret.
- Password inputs are masked by default; visibility toggle, if present, is explicit and accessibly named.
- Errors appear next to the field with a short correction path and an announced error state.

### File table

- Name is the dominant column; Type and Size are secondary. No Modified column unless a reliable field is part of the data contract.
- Name shows file/folder label and extension where available. Folder identity is also textual/semantic, not color-only.
- Use a row hover fill, a high-contrast selection fill/outline, keyboard focus indication, and stable column headings.
- Empty-state text describes the next action without a decorative illustration.
- Keep sorting behavior and selection cardinality aligned with the verified reference product.

### Dialogs, tooltips, and notifications

- Dialog title names the action. Consequence text precedes actions. Cancel is visible and receives initial focus for destructive decisions.
- Tooltip appears after a short hover delay and is also available to keyboard focus. Tooltip is never the sole source of essential information.
- Use in-window toasts only for short success/informational feedback. Errors and partial failures persist inline until understood or dismissed.
- Never use Windows toast notifications for vault, account, or file details.

### Progress

- Determinate progress only when the operation can report meaningful units. Otherwise use an indeterminate indicator with a plain status sentence.
- Show completed/total counts where accurate, plus skipped/failed counts when relevant. Do not invent a percentage or claim a file is committed before the storage operation is complete.
- Announce meaningful progress changes to assistive technology at a restrained interval.
- Close/cancel behavior reflects safe storage boundaries.

## Themes and motion

- Default appearance follows the Windows light/dark preference.
- Provide explicit Light, Dark, and System options. Persisting this non-sensitive preference in local app settings is proposed; confirm in the decision log.
- Theme changes apply without restarting and do not flash through an unreadable intermediate palette.
- Motion is limited to short focus/selection/dialog transitions, normally 100–160 ms. No ambient animation or animated encryption theater.
- Respect reduced-motion settings; when reduced motion is requested, use immediate state changes or a minimal fade.

## Window and scaling behavior

- Use native title bar, system menu, resize/move behavior, taskbar identity, and file/folder pickers.
- Proposed minimum: 960×640 logical px, pending the framework comparison. At smaller available work areas, hide the optional Details pane first and reduce secondary columns while preserving Name, Files/Trash navigation, and primary commands.
- Reflow labels, paths, and dialogs before truncating content.
- Test Windows display scaling at 100%, 150%, and 200%, plus moving the window between monitors with different scale factors.
- Test high contrast and keyboard-only operation. The UI must not assume hover is available.

## Accessibility release gates

- Contrast: 4.5:1 for normal text and 3:1 for large text and essential graphical controls.
- Full keyboard operation for every workflow; no keyboard traps outside modal dialogs.
- Visible focus indicator in both themes and high-contrast mode.
- Correct roles, names, selection state, sort state, and progress announcements in the platform accessibility tree.
- Dialog focus containment and restoration.
- No status conveyed by color alone; no clipped content at text scaling.
- Test real Windows Narrator and accessibility-tree inspection, not just widget metadata.

