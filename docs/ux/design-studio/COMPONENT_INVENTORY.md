# Component inventory

> This inventory is a pre-UX3 record; current branding, controls, and semantics are summarized in [component standardization](COMPONENT_STANDARDIZATION.md) and [UX3 final visual report](UX3_FINAL_VISUAL_REPORT.md). The rows below are amended where a prior claim became stale.

| Component | Implementation | Semantic/state behavior | Covered states |
|---|---|---|---|
| App header | `ui/app.rs` | Product name, one compact workspace status, settings, preview label | Default, preview, Light/Dark |
| Husky logo | `ui/components.rs`, `shell.rs` | Original product PNG; AccessKit Image label; source provenance documented | Header and native window |
| Sidebar navigation | `ui/components.rs`, `ui/navigation.rs` | Files/Trash label, selected surface/leading marker, explicit selected Button metadata and focus outline | Default and preview |
| Preview selector | `ui/navigation.rs` | Scenario and fixture-size selectors | 22 preview states; 100/1k/10k |
| Breadcrumbs | `ui/explorer.rs` | Clickable root-to-leaf path plus Back/Up | Root and nested folders |
| Search | `ui/explorer.rs` | Local sample-name filter, Ctrl+F focus, transient state | Search and empty results |
| Command actions | `ui/explorer.rs`, `ui/navigation.rs` | New Folder/Import and selection actions live in preview; the default shell explains unavailable vault features without permanently disabled Create/Open controls | Default, preview, busy, disabled |
| Virtualized table | `ui/explorer.rs` | Four named columns; `TableBuilder::rows`; row accessible name/role/selection; stable IDs | Populated, empty, loading, Trash |
| Context menu | `ui/explorer.rs` | Files: applicable Export/Open/Rename/Move actions; Trash: Restore/Permanent Delete; selected-row group actions preserve selection; Shift+F10 route | File/folder and Trash rows |
| Dialog frame | `ui/dialogs.rs` | Centered modal, action-specific title, Cancel/confirm order, inline errors | Create, rename, Trash, permanent delete, import, export |
| Progress and notices | `ui/explorer.rs` | `ProgressIndicator`, text progress, Cancel, success/failure | Import, busy, loading, error |
| Settings | `ui/app.rs`, `shell.rs` | In-memory System/Light/Dark choice; System is default | All three preference values |
| Auth/settings examples | `ui/screens.rs` | Clearly sample-only identity fields; masked secrets; no backend | Vault/account selection, create/unlock/login/register/settings |

Status labels describe the source implementation. Model behavior and Delete-key routing have automated coverage; actual rendered appearance, focus, dialogs, and accessibility remain NOT RUN.
