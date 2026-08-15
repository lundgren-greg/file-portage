# Portage brainstorm (session context)

For the next agent: read this, then `PROJECT.md` and `docs/design.md`. Do not invent a new architecture. Greg wants to **keep discussing** from here.

Date: 2026-08-14 / 2026-08-15  
Local clone: `C:\Repos\portage-app`  
GitHub: https://github.com/lundgren-greg/portage-app (public)  
Older clone path you may also see: `C:\Repos\file-portage` (same remote; prefer `portage-app`)

---

## What this product is

**Portage** inventories files across **multiple local disks** (internal + USB) **and** clouds (OneDrive, Google Drive, later others), then **migrates / consolidates / frees space** under a plan the user confirms.

The pain is not “make two clouds talk.” It is:

> I have several places files can live. None of them is big enough or clear enough. I do not want to lose a file or spend a weekend dragging folders.

Verbs: **migrate, consolidate, make room.** “Optimize” is the result, not the brand (do not sound like CCleaner).

The fun product: the user states **desire** (what goes where) and **priority** (what to free or keep first). An **agent — local or online** — asks until that is clear, then the deterministic planner prints a dry-run. **The agent never applies.** The user types the plan id.

---

## What we already shipped (docs only — no Rust app yet)

- Repo kit, MIT, Greg owns `main` (PR-only, CODEOWNERS `* @lundgren-greg`)
- Approved design: `docs/design.md` (14–16 PRs)
- Feature checklist: `docs/FEATURES.md`
- README with trail hero + use-case table (gaming clips **and** docs/archives/tight disk/dups)
- Wiki pages: `docs/wiki/` — GitHub Wiki tab still needs one “Create first page” click, then `scripts/Publish-Wiki.ps1`
- Hero image: `docs/images/portage-trail.jpg`

**Next implementation:** design PR 1 only (workspace + `portage init`). Branch + pull request. Do not push `main`.

---

## Locked decisions

| Topic | Decision |
| --- | --- |
| Name | Product **Portage**. Repo **portage-app**. Binary `portage`. Not Gentoo; not “file-*”. |
| First user | Greg’s machine only |
| Machines | One PC in R1 |
| Accounts | One Google, one personal Microsoft |
| Organize | Agent conversation clarifies. Engine default is **placement**, not rename/rebuild |
| External USB | **Shuttle hop and/or final dest.** Volume serial, not drive letter. Unplugged = fail closed. Not a last copy. |
| Evict | Only cloud-only or unpinned. Never pinned / `required` |
| Apply | Type the **plan id**. `y`/`yes` rejected |
| Undo | Reverse plan + second typed id. Refuse last-copy or reserve breach |
| Agent | Desire + priority → clarify (max 3 questions) → Intent → planner. **Never applies** |
| Agent where | **Online (Grok)** and **local** (Ollama / LM Studio / any OpenAI-compatible localhost). Same trait |
| Online privacy | Redacted catalog digest unless user opts into sending paths |
| Stack | Rust, SQLite WAL, BLAKE3. Not rclone, not Python engine |
| Cloud-to-cloud | Always a local shuttle (often via USB if C:/D: is tight) |
| P0 | No data loss. TUI (PR 15) and ask (PR 16) after apply+undo |
| README audience | Gaming clips **alongside** everyday cases. Not clips-only, not fully generic |

---

## Market (what we agreed)

- Viable as a **power-user / prosumer** tool, not the next Dropbox.
- People already pay MultCloud / RcloneView ~$5–10/mo for dumb copy. They do not pay for another sync folder.
- The **gap** is: many **local volumes + clouds**, a **plan**, last-copy, USB as hop or home, no public-link SaaS.
- Pitch: *You have more than one place. We figure out what should live where, and we will not lose a file doing it.*
- Simple as an **outcome** (one conversation, one confirm). Not simple if the first screen is YAML.
- Build for Greg first. Charge later only if strangers run `apply` twice.

---

## Agent loop (factor this in — latest brainstorm)

```
desire + priority → agent may ask 1–3 questions → Intent JSON
     → compile.rs (deterministic) → planner dry-run → user types plan id
```

- Desire = where classes of files should live (D: + roomier cloud; archives on USB; etc.).
- Priority = what to do first when space is tight (free C:, evict unpinned archives, hop via USB).
- Priorities cannot override last-copy, pin, or `required`.
- Local model: names/paths stay on the PC.
- Online (Grok default): better clarifier; do not send full path lists unless opted in.

This is written into the **working tree of `C:\Repos\file-portage`** (uncommitted design edits) but may not be on `portage-app` `main` yet. Treat this file as source of truth for the conversation. Merge/port those design edits in a PR when you implement or continue spec work.

---

## Example things a user would say

- “Free 50 GB on C: without losing anything.”
- “Keep recent gaming clips on D: and whichever cloud has more space.”
- “Put the archive folder on the USB and Google Drive.”
- “Free C: first, keep clips on D:, use the USB as the hop.”

Agent asks if dest, volume, or priority is missing. It does not guess a delete.

---

## Do not

- Auto-apply. Agent never holds `Executor`.
- Open OneDrive / DriveFS placeholders.
- Treat an unplugged USB as a last copy.
- Sound like a junk cleaner or a photo/DAM app.
- Skip ahead of design PR 1 unless Greg asks to keep brainstorming (discussion is fine; implementing apply is not).

---

## Open for more discussion (not locked)

- Exact local-model UX (which runtime to detect first: Ollama vs LM Studio).
- How much catalog digest the online agent sees by default.
- When (if ever) to publish a shared OAuth client id.
- Whether `C:\Repos\file-portage` should be deleted now that `C:\Repos\portage-app` is the clone.
- GitHub Wiki tab still uninitialized (one UI click).
