# Decisions

Treat these as final until a later decision changes them.

| # | Topic | Decision |
|---|---|---|
| 1 | OAuth client ids | Bring-your-own in R1 (`PORTAGE_GOOGLE_CLIENT_ID`, `PORTAGE_MS_CLIENT_ID`). Browser → pick account → approve → return. |
| 2 | OneDrive | Personal `/me/drive` in Release 1. Microsoft 365 / SharePoint is Release 2. |
| 3 | Talking to it | Clarify-then-plan agent. You state **desire** (what goes where) and **priority** (what to free or keep first). Agent asks until `Intent` is clear, then the planner prints a dry-run. **Never applies.** **Local or online** (Grok default; Ollama/LM Studio for on-box). |
| 4 | Undo | Reverse plan + second typed id. Refuse last-copy or reserve breach. Never auto-redownload. |
| 5 | Catalog location | `init` measures C:. If C: < 8 GiB, recommend the largest safe non-overlay volume. NL may confirm. Engine rejects unsafe dirs. No silent move. |
| 6 | Google scope | Full `drive` so existing files can be inventoried. Consent copy explains why `drive.file` cannot. |
| 7 | TUI | `portage-tui` (color, hotkeys) after the safety MVP (PR 15). Apply still types the plan id. |
| 8 | First apply user | One personal machine. No published OAuth client in R1. |
| 9 | Machines in R1 | One PC. Multi-machine is later. |
| 10 | “Organize” | LLM conversation clarifies. Engine default is placement, not rename/rebuild. |
| 11 | External drive | Both hop and home. Each plan chooses. |
| 12 | Cloud accounts | One Google, one personal Microsoft. |
| 13 | Evict | Only cloud-only or unpinned. |
| 14 | Name | Product **Portage**. Repo **portage-app**. Binary **portage**. |
| 15 | Audience in the README | Gaming clips **and** everyday split-library cases. Not clips-only, not fully generic. |
| 16 | Stack | Rust, SQLite WAL, BLAKE3. Not Python, not C#, not rclone. |
| 17 | Cloud-to-cloud | Always a local shuttle. No third-party transfer SaaS. No public-link tricks. |

Rationale and the 4 GiB worked example live in [docs/design.md](https://github.com/lundgren-greg/portage-app/blob/main/docs/design.md).
