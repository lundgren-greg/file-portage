# portage-app — project tracker

> **Resume here** when starting a new session. Keep this file current when you stop work.
> Optional Grok-Context thread: `C:\Repos\Grok-Context\threads\file-portage\`

| Field | Value |
|-------|--------|
| **Local path** | `C:\Repos\file-portage` |
| **GitHub** | `lundgren-greg/portage-app` |
| **Branch** | `main` |
| **Last commit** | Run `git log -1 --oneline` |
| **Remote** | `origin` → https://github.com/lundgren-greg/portage-app.git |
| **Status** | Public. `main` is PR-only; Greg (`@lundgren-greg`) reviews every merge. Design approved. Next: PR 1. |
| **Updated** | 2026-08-14 |

---

## Goal

Ship a Windows-first tool (`portage` CLI, then `portage-tui`, plus `portage ask`) that inventories local disks, Google Drive, and personal OneDrive, then produces a user-confirmed, space-safe shuttle plan so gaming clips and other libraries can live where the user asked — locally and/or on the cloud with more free space — **without losing files**, filling a 4 GiB-free disk, deleting the last copy, corrupting a multi-GB file, or making anything public. NL (Grok first) proposes; it never applies.

---

## Stopped at

1. Created `lundgren-greg/file-portage` from `repo-template` (Rust CI, MIT).
2. Wrote and reviewed the design (`docs/design.md`). Reviewer approved after 3 rounds.
3. Checked in feature set (`docs/FEATURES.md`) and `configs/examples/gaming-clips.yaml`.
4. **Next session / implementation agent: execute PR 1** in `docs/design.md` (workspace + CLI stub). Branch + pull request; do not push `main`. Do not skip ahead to providers or apply.

---

## Next steps (ordered)

1. PR 1 — Cargo workspace, eight crate stubs, `portage --help`, `portage init` (measure C:, recommend `data_dir` if C: < 8 GiB), copy example config, rust-toolchain. CI `cargo` steps start running.
2. Follow PRs 2–13 exactly as written. **No-data-loss P0** (apply + undo refuse) before TUI/NL. Merge gates for planner PRs: P-space and P-last-copy tests.
3. PR 14 polish → PR 15 `portage-tui` → PR 16 `portage ask` / Grok.
4. Keep this file's **Stopped at** current.

---

## Blockers

| Blocker | Detail | Unblock |
|---------|--------|---------|
| OAuth client ids | v1 is bring-your-own (`PORTAGE_GOOGLE_CLIENT_ID`, `PORTAGE_MS_CLIENT_ID`) | Needed only at PR 6+ live tests. Mocks first. |

---

## Resolved questions (user, 2026-08-14 — final)

| # | Question | Answer |
|---|----------|--------|
| 1 | OAuth client ids | **BYO in R1:** `PORTAGE_GOOGLE_CLIENT_ID`, `PORTAGE_MS_CLIENT_ID` |
| 2 | OneDrive scope | **Personal `/me/drive` in Release 1.** M365/SharePoint = **Release 2** |
| 3 | Interaction / keep_local | **LLM front door (Grok first).** Compiles to policy + plan. **Never applies.** Gaming Clips default `prefer` + warning; “must stay on C:” → `required`. Browser PKCE UX. |
| 4 | Undo / data loss | **No data loss = R1 P0.** Reverse-plan + second typed id. Refuse last-copy or reserve breach. Never auto-redownload. |
| 5 | Catalog location | `init` measures C:. If < 8 GiB, recommend largest non-overlay volume. NL may confirm `data_dir`. Engine rejects unsafe dirs. No silent move. |
| 6 | Google scope | **Full `drive`.** Consent copy explains why `drive.file` cannot inventory existing clips. |
| 7 | TUI | **`portage-tui` in R1 after safety MVP (PR 15).** Color/hotkeys. Does not block PRs 1–13. |
| 8 | First user of apply | **Greg's machine only.** No published OAuth client, no friend-install push in R1. |
| 9 | Machines in R1 | **One PC.** Catalog on that host. Multi-machine is later. |
| 10 | What “organize” means | **LLM conversation clarifies with the user** each time (`portage ask`). Engine default remains placement (where copies live). No autonomous rename/rebuild. |
| 11 | External drive job | **Both hop and home.** Default `roles: [shuttle, final]`. Each plan chooses how that run uses the disk. |
| 12 | Cloud accounts in R1 | **One Google, one Microsoft** (personal). A second account is another provider later. |
| 13 | Evict aggressiveness | **Only cloud-only or unpinned collections.** Never evict `required` / pinned. `prefer` may drop local with a warning. |
| 14 | Name | **Product Portage. Repo `portage-app`.** Binary `portage`. |

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
| PR 11–13 confirmed apply + undo | First real transfers; P0 no-data-loss gate |
| PR 14 polish | Safety MVP tag |
| PR 15 `portage-tui` | Color plan review; typed-id apply |
| PR 16 `portage ask` / Grok | NL → policy + plan; never applies |
| Release 2+ | M365/SharePoint, Dropbox/S3/SMB, extra LLMs — see design Future releases |

---

## Decisions log

| Date | Decision |
|------|----------|
| 2026-08-14 | Standard repo kit from `lundgren-greg/repo-template`. License MIT. Default branch `main`. |
| 2026-08-14 | Product name **Portage**; repo **portage-app** (renamed from file-portage); binary **portage**. |
| 2026-08-14 | Stack: Rust + SQLite WAL + BLAKE3. Not Python, not C#, not rclone. |
| 2026-08-14 | Cloud-to-cloud is always a local shuttle. Placeholders are not replicas. |
| 2026-08-14 | Apply requires typing the plan id. Last-copy + private ACL + staging reserve are non-negotiable. |
| 2026-08-14 | Design approved after 3 review rounds. Source of truth: `docs/design.md`. |
| 2026-08-14 | Repo is **public**. `main` is PR-only. CODEOWNERS `* @lundgren-greg`. Greg reviews and merges everything. |
| 2026-08-15 | External / USB volumes are first-class (`shuttle` hop and/or `final` dest; identity = volume serial). README leads with capabilities and names concrete situations (gaming clips **and** docs, archives, full disk, dups) so the audience is not one niche. |
| 2026-08-14 | User resolved open questions: BYO OAuth; personal OneDrive in R1 / M365 in R2; Grok-first NL never applies; no-data-loss P0; undo = reverse-plan + second id; init+NL catalog recommendation with engine reject; full Google `drive`; TUI PR 15 after safety MVP. |

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
- Auto-apply plans. No daemon in R1. **LLM proposes, never applies.**
- Start TUI (PR 15) or NL compile-to-plan (PR 16) before PR 13 undo is merged.
- Skip the planner property tests (P-space, P-last-copy).
- Push or merge to `main`. Open a pull request; Greg (`@lundgren-greg`) merges.
- Force-push or rewrite history on `main`.
- Implement PRs out of order unless the design marks them independent (PR 6 can proceed after PR 4).
