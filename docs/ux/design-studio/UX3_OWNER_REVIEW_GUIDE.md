# UX3 R3 Owner Review Guide

This guide prepares owner-operated validation for one exact Release candidate. No final UX, accessibility, or production approval is recorded.

## Run without rebuilding

Verify this SHA-256 first:

    Get-FileHash 'C:\Users\Asus Vivobook\Desktop\encryptdrive-desktop\target\release\encryptdrive-desktop.exe' -Algorithm SHA256

Expected: 0230F8EBFFF2717BB7F8FB64E2AFB8BFD359D32A00EBD6F703C72412468E62BD

Launch the synthetic preview:

    & 'C:\Users\Asus Vivobook\Desktop\encryptdrive-desktop\target\release\encryptdrive-desktop.exe' --design-preview

Launch the production shell from the same binary:

    & 'C:\Users\Asus Vivobook\Desktop\encryptdrive-desktop\target\release\encryptdrive-desktop.exe'

These are direct executable commands; neither rebuilds. The two modes share one output file, so another build replaces the candidate. The preview reads or writes no real files and performs no vault or cryptographic operation.

## Validation sequence

Use the 12-step [manual R3 checklist](UX3_MANUAL_R3_VALIDATION_MATRIX.md). In particular, verify the Export warning cannot be bypassed with Enter, a focused Cancel prevents destructive confirmation, table focus and shortcuts behave as specified, filename clicks reach the row, and selection/status feedback do not move the table.

For genuine evidence, note the executable hash, preview/shell mode, Windows build, window dimensions, display scale, theme, dataset size, and action sequence. Only capture the EncryptDrive window. Do not mark visual, native-input, offline-launch, DPI, High Contrast, UIA, or Narrator checks as passed based on source or headless tests.

## Current gates

- The locked default and preview checks, tests, strict Clippy, Release builds, and architecture boundary verification are recorded in the [R3 regression report](UX3_REGRESSION_TEST_REPORT.md) and [Release candidate report](UX3_RELEASE_CANDIDATE_REPORT.md).
- The complete R1/R2/R3 external audit originals and R2 live-test annex were not supplied. The [R3 remediation matrix](UX3_R3_REMEDIATION_MATRIX.md) records this as BLOCKED and does not infer missing IDs or dispositions.
- The supplied request reports Claude’s R3 verdict as CONDITIONALLY READY; the full R3 report was not included.
- Release GUI validation remains NOT RUN. Owner visual, DPI, High Contrast, UIA and Narrator review is still required.
- This candidate does not authorize Phase 1B, production vault behavior, or Vault Format 1.
