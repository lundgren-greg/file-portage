# Agent instructions — file-portage

Read `PROJECT.md` first for current status, blockers, and the session resume checklist.
Keep it updated when you stop work.

Starter skills are in `.agents/skills/` (commit, PR, review, debug, TDD, plan, security, PowerShell, simplify, verify). Same pack is installed machine-wide from `C:\Repos\Scripts\Install-AgentSkills.ps1`.

## What to implement

This repo is a **greenfield** app. The approved design is [`docs/design.md`](docs/design.md). The feature checklist is [`docs/FEATURES.md`](docs/FEATURES.md).

**Start at PR 1** in the design's PR Plan. Do not invent a different architecture. Do not implement apply/providers before the catalog and planner exist, except where the plan says PR 6 is independent of hashing.

## Product rules

- Windows-first. Linux/macOS must still compile.
- Local-first. No telemetry. Cloud I/O only after the user runs `provider add` / `apply`.
- Never commit secrets, tokens, catalogs of real files, or OAuth client secrets.
- No force-push or history rewrite on `main` without asking.
- **Never** add share-link, `anyoneWithLink`, or public ACL APIs. CI must `rg` for them.
- **Never** open OneDrive / DriveFS placeholders. Overlay roots are not local replicas.
- **Never** delete a last *verified* copy. Suspect replicas do not count.
- **Never** apply a plan without the user typing the exact plan id.
- **LLM proposes, never applies.** `portage-nl` / `portage ask` may emit policy + a dry-run plan. It must not call the executor, delete, evict, or upload.
- No data loss is Release 1 P0. Do not start TUI (PR 15) or NL compile-to-plan (PR 16) before PR 13 undo.
- Planner and executor must keep local free space ≥ `staging_reserve` **during** every op.
- Delete requires a `LastCopyGuard` permit. There is no public `delete(path)`.
- `undo` is reverse-plan + second typed id. Refuse if reverse would drop a blob to zero verified replicas or breach reserve.

## Layout (after PR 1)

```text
crates/portage-core/
crates/portage-catalog/
crates/portage-auth/
crates/portage-providers/
crates/portage-media/
crates/portage-engine/
crates/portage-cli/        # [[bin]] name = "portage"
crates/portage-sim/
crates/portage-tui/        # PR 15
crates/portage-nl/         # PR 16; no Executor dependency
configs/examples/
docs/
migrations/
```

Until PR 1, `src/` and `tests/` are unused template dirs.

## Conventions

- Rust edition 2021, stable toolchain, `clippy -D warnings`.
- PowerShell helpers: approved verbs, PascalCase, `[CmdletBinding()]`, 4-space indent.
- YAML: 2-space indent.
- Markdown: do not trim trailing whitespace (EditorConfig).
- Tests create temp dirs/files and clean up.

## Build, test, commit

After PR 1:

```powershell
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

Planner PRs are incomplete without P-space and P-last-copy tests.

Update `PROJECT.md` (`Stopped at`, `Next steps`, `Decisions log`) at the end of a session.
