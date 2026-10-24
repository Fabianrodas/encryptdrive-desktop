# Git workflow

This policy keeps the Rust repository reviewable while EncryptDrive moves from product and UX foundations into implementation. It applies to branch creation, commits, integration, and publication.

## Branches

- `main` is the primary branch and the baseline for reviewed, coherent work. The current design and architecture phase can land directly on `main` in small, reviewable changes.
- Create the long-lived `dev` branch only when authorized functional product implementation begins. Create it from the then-current `main`; do not create it in advance just to reserve the name.
- During functional implementation, integrate ordinary work into `dev`. Merge `dev` into `main` after the increment has passed review and the required validation.
- Use `feature/<short-name>` for substantial functionality that benefits from independent development or review. Create it from the active integration branch, normally `dev`, and merge it back there after review. Do not create branches for routine UX refinements, small documentation edits, or tightly related fixes.
- Name short-lived repair branches `fix/<short-name>` only when isolation materially helps review or coordination. Avoid parallel branches that edit the same files without a clear integration owner.
- Do not rewrite shared history. Do not force-push. Do not delete or rename a branch that contains work without first establishing where that work is preserved.

## Commit cadence and messages

- Commit one coherent, reviewable change at a time, after checking the affected behavior. Avoid arbitrary micro-commits and oversized commits that combine unrelated changes.
- Keep incomplete work uncommitted unless a meaningful, independently validated increment can be described accurately. Never create placeholder or misleading commits to make a history look complete.
- Use Conventional Commit subjects in the form `type(scope): imperative summary`, for example `docs(git): document branch workflow` or `fix(explorer): preserve selection after sorting`.
- Use the established types where applicable: `feat`, `fix`, `refactor`, `perf`, `test`, `docs`, `build`, `ci`, and `chore`. A scope is optional when it adds no clarity. Keep the subject concise and describe the actual change.
- Before committing, inspect the status and diff, stage only the intended paths, then inspect the staged diff and run `git diff --cached --check`. Never use broad staging as a substitute for reviewing what will enter the commit.

## Validation and review

- Run the repository's relevant checks before a commit. For production workspace changes, follow [the development guide](../development.md): formatting, locked workspace check, Clippy with warnings denied, tests, architecture verification, and the applicable build.
- For isolated spikes, use that spike's own manifest and lockfile; do not fold its dependencies or targets into the production workspace without an approved architecture change.
- Match validation to the change. UI work should include the relevant interaction or visual checks; security-sensitive changes require focused tests at the affected boundary. State any checks that could not run in the change or its review evidence.
- Review the final staged diff for scope, accidental generated files, private material, and secrets. Keep credentials, keys, real vault data, private user data, and material from restricted repositories out of this repository.

## Integration and publication

- Review a branch and run its required checks before merging it. Prefer a merge strategy that preserves the repository's reviewable change history; follow an explicit project-level release or hosting policy if one is approved later.
- UX and product-foundation work may land directly on `main` while `dev` has not been created. Once functional development starts, use `dev` as the integration branch and return reviewed increments to `main` at deliberate milestones.
- Prepare releases only from validated `main`, with the required owner approval and release checks. Do not create tags, publish packages, change repository visibility, or push to a remote unless the owner explicitly authorizes that action.
- Check the actual remote branches and upstream configuration before syncing. A local branch name does not imply that a matching remote branch exists. If the intended upstream is absent or ambiguous, report that state and leave publication to an authorized follow-up.

## Repository hygiene

- Preserve existing user work. Do not clean or reset unrelated files to make a commit easier.
- Do not commit build output, caches, local environment files, screenshots containing private data, or generated artifacts unless they are deliberate, reviewable deliverables.
- Keep experimental prototypes isolated under `spikes/`; they do not establish production APIs or architecture by themselves.
