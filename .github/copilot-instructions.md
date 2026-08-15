# Copilot instructions — file-portage

Windows-first local project. Read `PROJECT.md` first for current status, blockers,
and the session resume checklist; keep it updated when you stop work.

Approved design: `docs/design.md`. Feature checklist: `docs/FEATURES.md`.
Implement the next PR in that plan. Do not invent a different stack or skip safety gates.

Project skills are in `.agents/skills/`. Use them for commit, PR, review, debug,
TDD, planning, security review, and PowerShell.

## Architecture

After PR 1, business logic lives in `crates/portage-*`. The CLI is a thin clap
binary named `portage`. Until then, do not put app code in `src/`.

## Hard rules

- **No share-link APIs.** No `anyoneWithLink`, `createLink`, or anonymous ACLs.
- **No placeholder hydration.** Overlay roots are not local replicas.
- **No delete without `LastCopyGuard`.** Suspect ≠ verified.
- **No apply without typing the plan id.**
- **No telemetry.** Tokens never in YAML or git.
- **Never commit secrets or real file inventories.**
- **Never push or merge to `main`.** Open a PR. Greg (`@lundgren-greg`) is the only merger.

## Build, test, run

```powershell
# After PR 1:
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

CI must stay green. Planner PRs require P-space and P-last-copy tests.

## Conventions

- Rust edition 2021, clippy `-D warnings`.
- PowerShell: approved verbs, `Verb-Noun`, `[CmdletBinding()]`, 4-space indent.
- Follow `.editorconfig`.
- Tests create temp dirs/files and clean up afterwards.
