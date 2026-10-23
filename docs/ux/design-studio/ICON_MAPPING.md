# UX3 Semantic Icon Mapping

All codepoints map to the embedded Phosphor Regular font via `ui/icons.rs`. The mapping test verifies that all 23 values are valid private-use scalars and unique. This does not verify that a glyph is visually correct or available in the embedded font at runtime.

| Semantic icon | Code point | Use |
|---|---:|---|
| Files | U+E710 | Files destination |
| File | U+E230 | File row |
| Trash | U+E4A6 | Trash destination |
| Folder | U+E24A | Folder row |
| NewFolder | U+E258 | Create Folder action |
| Search | U+E30C | Search field |
| Rename | U+E3B4 | Rename action |
| Import | U+E4C0 | Simulated Import |
| Export | U+E20C | Simulated Export |
| Restore | U+E024 | Restore action |
| Back | U+E058 | Back navigation |
| Up | U+E08E | Parent navigation |
| Settings | U+E270 | Settings entry |
| Account | U+E4C2 | Account sample screen |
| Lock | U+E2FA | Create-vault sample screen |
| Unlock | U+E306 | Unlock-vault sample screen |
| Warning | U+E4E0 | Caution feedback |
| Info | U+E2CE | Informational feedback |
| Error | U+E4F8 | Failure feedback |
| Success | U+E184 | Success feedback |
| More | U+E208 | Context menu heading |
| SortAscending | U+E13C | Active ascending sort |
| SortDescending | U+E136 | Active descending sort |

Accessible text is supplied separately from PUA glyphs in custom controls and table metadata.
