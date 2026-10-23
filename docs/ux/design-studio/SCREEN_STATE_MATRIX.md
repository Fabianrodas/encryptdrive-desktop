# Screen and state matrix

> **Historical Phase 1A-UX inventory.** UX3 changed several selection, theme, control, and branding details. Use [UX3 iteration log](UX3_ITERATION_LOG.md), [component standardization](COMPONENT_STANDARDIZATION.md), and [UX3 final visual report](UX3_FINAL_VISUAL_REPORT.md) for current behavior and evidence.

All sample data and actions are in-memory only. Switch states using the Preview state control at the top of the opted-in preview.

| Preview state | Representation | Interactions / validation | Evidence status |
|---|---|---|---|
| Vault selection | Create/Open vault choices | Routes to representative forms; no filesystem path picker | Source implemented; GUI NOT RUN |
| Create vault | Name and masked password fields | Preview submit says no vault was created | Source implemented; GUI NOT RUN |
| Unlock vault | Sample vault choice and masked password field | No authentication is called | Source implemented; GUI NOT RUN |
| Account selection | Two sample account choices | Routes to sample login; no account is loaded | Source implemented; GUI NOT RUN |
| Account login | Sample identity and masked secret | Preview submit does not sign in or contact a service | Source implemented; GUI NOT RUN |
| Register account | Sample identity and masked secret | Preview submit does not register or send mail | Source implemented; GUI NOT RUN |
| Explorer | Deterministic root fixture | Navigation, search, selection, sort, create/rename/delete/export warning | Model tests PASS; pointer integration NOT RUN |
| Empty explorer | Empty folder fixture | Search and breadcrumb navigation | Model fixture test PASS; GUI NOT RUN |
| Folder navigation | Documents → Projects | Breadcrumb, Back, Up, Enter/double-click folder | Model navigation test PASS; GUI keyboard NOT RUN |
| Search results | `project` query | Filter across the sample tree; selected stable IDs persist | Model search/filter test PASS; GUI NOT RUN |
| Loading | Spinner and synthetic progress | Repaint-driven transition to Explorer; actions disabled while busy | Operation-state tests PASS; GUI NOT RUN |
| Multi-selection | First four visible entries selected | Ctrl/Shift click and Ctrl+A models; selection counter | Model selection tests PASS; GUI NOT RUN |
| Import progress | Simulated progress | Cancel; no file picker or read | Operation-state tests PASS; GUI NOT RUN |
| Export warning | Plaintext warning dialog | Explicit continue starts simulation; no decrypt/open/write | Source implemented; GUI NOT RUN |
| Trash and restore | Flattened deleted sample entries | Restore preserves IDs; deleted ancestors are restored with a child | Model restore tests PASS; GUI NOT RUN |
| Permanent deletion | Confirmation against selected Trash item | Permanently removes only in-memory fixture nodes | Model permanent-delete test PASS; GUI NOT RUN |
| Account settings | Sample account context | Illustrative controls only, no persistence | Source implemented; GUI NOT RUN |
| Vault settings | Sample vault context | Illustrative controls only, no persistence | Source implemented; GUI NOT RUN |
| Recovery notice | Synthetic metadata warning copy | Notice only; no recovery algorithm runs | Source implemented; GUI NOT RUN |
| Busy | Operation progress and Cancel | Mutating controls disabled; event loop requested repaint | Source implemented; GUI NOT RUN |
| Error | Failure notice and failed operation state | No data changed | Operation-state tests PASS; GUI NOT RUN |
| Disabled | Explicit preview-only explanation | Mutating controls are disabled | Source implemented; GUI NOT RUN |

The fixtures expose 100, 1,000, and 10,000 active root rows. Names, stable IDs, synthetic sizes, dates, nested/empty folders, Unicode/long names, and twelve deleted samples are deterministic. Controlled sibling collisions are tested by folder creation and rename validation.
