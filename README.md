# file-portage

[![CI](https://github.com/lundgren-greg/file-portage/actions/workflows/ci.yml/badge.svg)](https://github.com/lundgren-greg/file-portage/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Portage** inventories local disks and cloud accounts, then shuttles files between them under a user-confirmed plan. The binary is `portage`. The GitHub repo is `file-portage` so it does not collide with Gentoo Portage.

**Author / maintainer:** [Greg](https://github.com/lundgren-greg) (`@lundgren-greg`). Nothing lands on `main` except through a pull request he reviews.

Gaming clips (and anything else) that are split across a nearly full SSD, OneDrive, and Google Drive become one catalog. You can write a placement policy — or **say it** (“keep my clips on D: and whichever cloud has more free space”) and Grok compiles that into policy plus a dry-run plan. Portage never drives local free space below a reserved staging budget, never deletes the last verified copy, and never creates a public share. **The LLM never applies.** You type the plan id.

## Why this project

Explorer, rclone, and the official sync clients can copy bytes. They will also hydrate a OneDrive placeholder onto a disk with 4 GiB free, overwrite a different file that happens to share a name, or leave a truncated video looking complete after a dropped connection. Portage is a **control plane** (catalog, policy, planner, journal) plus a **private data plane** (resumable upload/download with checksum verify). Cloud-to-cloud is always a local shuttle. Nothing is applied until you type the plan id.

## Status

**Design approved. Implementation not started.** An agent should execute [docs/design.md](docs/design.md) beginning at **PR 1**. See [PROJECT.md](PROJECT.md) and [docs/FEATURES.md](docs/FEATURES.md).

## Key features (MVP)

- Index local NTFS volumes **and** Google Drive / OneDrive via their APIs (not desktop placeholders).
- Content-addressed identity (BLAKE3). Provider checksums are bindings only.
- Capacity view per volume and per cloud, including a 1 GiB staging reserve.
- YAML collections and placement policies (`keep_local`, `most_free` cloud, replica count).
- Space-safe planner with residual free space after every step. User types the plan id to apply.
- Serial executor, crash journal, last-copy permit, private-only ACL assert, no silent overwrite.
- **No data loss is Release 1 P0.** Undo is a reverse plan you confirm with a second plan id.
- Natural language (`portage ask`) via Grok first (`XAI_API_KEY`). Compiles intent → plan. Never applies.
- `portage-tui` (ratatui) after the safety MVP: color, hotkeys, plan review. Apply still types the plan id.

## Architecture

```text
file-portage/
  crates/
    portage-core/       # ids, hashing, paths, config
    portage-catalog/    # SQLite WAL
    portage-auth/       # OAuth PKCE + DPAPI/keyring
    portage-providers/  # local + Drive + Graph (no share APIs)
    portage-media/      # cheap MP4 probe
    portage-engine/     # index, policy, planner, executor
    portage-cli/        # `portage` binary
    portage-sim/        # SimulatedWorld + property tests
    portage-tui/        # PR 15 — ratatui, after safety MVP
    portage-nl/         # PR 16 — Grok-first ask; never applies
  configs/examples/     # gaming-clips.yaml
  docs/design.md        # approved design + PR plan
  docs/FEATURES.md      # R1 P0 / safety MVP / TUI+NL / future releases
```

The workspace does not exist yet. PR 1 creates it. Until then CI skips Cargo steps.

## Requirements

- Windows 10/11 (primary). Linux/macOS must compile local+cloud paths.
- PowerShell 7+ (`pwsh`) for helper scripts.
- Rust stable (after PR 1).
- Bring-your-own OAuth client ids: `PORTAGE_GOOGLE_CLIENT_ID`, `PORTAGE_MS_CLIENT_ID`.
- Optional NL: `XAI_API_KEY` (Grok). Not required for CLI/TUI.

## Build and test

```powershell
# After PR 1 lands:
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

## Intended usage (after MVP)

```powershell
portage init
portage provider add local --root D:\ --id local-d
portage provider add google-drive
portage provider add onedrive
portage index
portage capacity
portage dups
portage plan --collection "Gaming Clips"
portage plan show
portage apply file-plan-7f3c   # type the plan id; y/yes is rejected
portage status
portage ask "keep my gaming videos on D: and the cloud with more free space"
portage-tui                    # after PR 15
```

Empty, `y`, or `yes` is **rejected** at apply. Confirmation is the exact plan id. `portage ask` only prints a plan.

## Safety invariants

- Last verified copy is never deleted.
- Every copy is checksum-verified before it counts as a replica.
- Local free space never goes below `staging_reserve` (default 1 GiB) at any step, including during a shuttle.
- OneDrive / Google Drive for Desktop placeholders are not replicas and are never opened.
- Uploads are private. Inherited "anyone with the link" on a parent folder fails the op; a file we created is deleted.
- No telemetry. Tokens live in the OS credential store, never in YAML.

## Security and privacy

See [SECURITY.md](SECURITY.md). Cloud transfers are opt-in after `provider add`. There is no background daemon and no public-link feature.

## Contribution and development notes

- Read [PROJECT.md](PROJECT.md) first.
- Implement in the order in [docs/design.md](docs/design.md) **PR Plan**.
- Keep CI green.
- **`main` is PR-only.** Branch, open a pull request, wait for CI. [Greg](https://github.com/lundgren-greg) (`@lundgren-greg`) is the code owner and the only person who merges.

## Roadmap (high level)

| Item | Status |
|------|--------|
| Standard repo kit | Done |
| Approved design + feature set | Done — `docs/design.md` |
| PR 1 workspace + CLI stub | Next |
| Local index + dups (PR 2–5) | Not started |
| Drive + OneDrive inventory (PR 6–8) | Not started |
| Planner dry-run (PR 9–10) | Not started |
| Confirmed apply + undo (PR 11–13) | Not started — P0 no-data-loss |
| Polish (PR 14) | Not started |
| TUI `portage-tui` (PR 15) | After safety MVP |
| NL `portage ask` / Grok (PR 16) | After planner; never applies |

## License

MIT. See [LICENSE](LICENSE).
