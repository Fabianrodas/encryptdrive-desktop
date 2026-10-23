# UX3 UI Copy Audit

**Purpose:** Keep preview boundaries and unavailable product capabilities explicit without presenting nonfunctional actions as usable controls.

| Context | Copy rule / example | Status |
|---|---|---|
| Ordinary shell | “Choose a vault and account to browse files.” | No vault/file operation is connected |
| Empty Trash | Explain that a vault is required to view deleted items | No disabled Delete/Restore action |
| Design preview | “Design preview · Sample data” | Persistent preview marker |
| Offline status | “Local · Offline” | Claims the shell does not need network services |
| Export | Explain that the sample Export does not decrypt, open, or write a real file; include a plaintext warning | Preview only |
| Import | Explain that selection is simulated and no filesystem picker is opened | Preview only |
| Cancel/failure | State that sample operation stopped/failed and no data was changed | In-memory fixture only |
| Recovery | Describe a simulated recovery notification; do not imply recovery code exists | Preview only |
| Form submission | Use “Preview” wording and retain feedback in state | Does not authenticate/create a vault |

UX3 removes the misleading selected-items-may-be-hidden notice because changing the effective query clears selection. Sorting preserves selection by stable ID. The normal no-vault shell no longer displays permanently disabled “Create vault” and “Open vault” buttons; empty-state copy explains that vault support is unavailable in this engineering build.

No reviewed copy invites real passwords, real vault paths, or real file exports. Windows Narrator announcement of notices and inline validation is NOT RUN.
