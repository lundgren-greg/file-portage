# file-portage — project tracker

> **Resume here** when starting a new session. Keep this file current when you stop work.
> Optional Grok-Context thread: `C:\Repos\Grok-Context\threads\file-portage\`

| Field | Value |
|-------|--------|
| **Local path** | `C:\Repos\file-portage` |
| **GitHub** | `lundgren-greg/file-portage` |
| **Branch** | `main` |
| **Last commit** | Run `git log -1 --oneline` |
| **Remote** | `origin` → https://github.com/lundgren-greg/file-portage.git |
| **Status** | Design approved. Implementation not started. Next: PR 1. |
| **Updated** | 2026-08-14 |

---

## Goal

Ship a Windows-first CLI (`portage`) that inventories local disks, Google Drive, and OneDrive, then produces a user-confirmed, space-safe shuttle plan so gaming clips and other libraries can live where the user asked — locally and/or on the cloud with more free space — without filling a 4 GiB-free disk, deleting the last copy, corrupting a multi-GB file, or making anything public.

---

## Stopped at

1. Created `lundgren-greg/file-portage` from `repo-template` (Rust CI, MIT).
2. Wrote and reviewed the design (`docs/design.md`). Reviewer approved after 3 rounds.
3. Checked in feature set (`docs/FEATURES.md`) and `configs/examples/gaming-clips.yaml`.
4. **Next session / implementation agent: execute PR 1** in `docs/design.md` (workspace + CLI stub). Do not skip ahead to providers or apply.

---

## Next steps (ordered)

1. PR 1 — Cargo workspace, eight crate stubs, `portage --help`, `portage init`, copy example config, rust-toolchain. CI `cargo` steps start running.
2. Follow PRs 2–14 exactly as written. Merge gates for planner PRs: P-space and P-last-copy tests.
3. Keep this file's **Stopped at** current.

---

## Blockers

| Blocker | Detail | Unblock |
|---------|--------|---------|
| OAuth client ids | v1 is bring-your-own (`PORTAGE_GOOGLE_CLIENT_ID`, `PORTAGE_MS_CLIENT_ID`) | Needed only at PR 6+ live tests. Mocks first. |

---

## Open questions (for user)

Recommendations from the design are the current defaults. Change them here if the user answers otherwise.

| # | Question | Why it matters | Answer |
|---|----------|----------------|--------|
| 1 | Ship a public OAuth client id or BYO? | Live Drive/Graph login | **BYO in v1** (recommended) |
| 2 | OneDrive personal only, or M365/SharePoint too? | Graph tenant + site drives | **Personal `/me/drive` in v1** |
| 3 | Gaming Clips `keep_local: prefer` or `required`? | Tight disk vs failed plans | **prefer + warning** |
| 4 | `undo` auto-redownload or reverse-plan confirm? | Space + safety | **reverse plan + typed id** |
| 5 | Auto-relocate catalog off C: or prompt? | Catalog on a full C: | **`init` measures and prompts** |
| 6 | Google scope `drive` vs `drive.file`? | Cannot inventory existing clips with `drive.file` | **full `drive`** |
| 7 | TUI in MVP? | Scope | **No. Post-MVP.** |

---

## What's implemented

### Layout

```
file-portage/
  PROJECT.md
  README.md, LICENSE, SECURITY.md, CODEOWNERS, AGENTS.md
  docs/design.md
  docs/FEATURES.md
  configs/examples/gaming-clips.yaml
  .github/workflows/ci.yml   # cargo steps skipped until Cargo.toml exists
  src/                       # unused; code will live in crates/ after PR 1
  tests/
  scripts/
  samples/
```

### Commands

```powershell
cd C:\Repos\file-portage
git status
git log -1 --oneline
```

No `portage` binary yet.

---

## Roadmap (not done)

| Item | Notes |
|------|--------|
| PR 1 workspace + CLI stub | First implementation slice |
| PR 2–5 local inventory | Useful on D: of clips before any cloud |
| PR 6–8 Drive + OneDrive read | Unified inventory, still no mutations |
| PR 9–10 planner dry-run | 4 GiB fixture, no writes |
| PR 11–12 confirmed apply | First real transfers |
| PR 13–14 undo / doctor / polish | v0.1.0 |

---

## Decisions log

| Date | Decision |
|------|----------|
| 2026-08-14 | Standard repo kit from `lundgren-greg/repo-template`. License MIT. Default branch `main`. |
| 2026-08-14 | Product name **Portage**; repo **file-portage**; binary **portage**. |
| 2026-08-14 | Stack: Rust + SQLite WAL + BLAKE3. Not Python, not C#, not rclone. |
| 2026-08-14 | Cloud-to-cloud is always a local shuttle. Placeholders are not replicas. |
| 2026-08-14 | Apply requires typing the plan id. Last-copy + private ACL + staging reserve are non-negotiable. |
| 2026-08-14 | Design approved after 3 review rounds. Source of truth: `docs/design.md`. |

---

## Session resume checklist

When starting a new agent/chat session:

1. Read **this file** (`PROJECT.md`).
2. Read `docs/design.md` **PR Plan** and implement the next unchecked PR only.
3. `git -C C:\Repos\file-portage status` and `git log -1 --oneline`.
4. `gh auth status`.
5. Update **Stopped at** / **Next steps** before ending the session.

---

## Do not

- Commit secrets, tokens, or real file inventories (use `samples/private/` locally; gitignored).
- Add share-link / `anyoneWithLink` / public ACL helpers. Ever.
- Hydrate OneDrive or DriveFS placeholders.
- Auto-apply plans. No daemon in MVP.
- Skip the planner property tests (P-space, P-last-copy).
- Force-push or rewrite history on `main` after the remote exists without asking.
- Implement PRs out of order unless the design marks them independent (PR 6 can proceed after PR 4).
