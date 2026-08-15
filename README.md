# file-portage

[![CI](https://github.com/lundgren-greg/file-portage/actions/workflows/ci.yml/badge.svg)](https://github.com/lundgren-greg/file-portage/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Portage** inventories local disks and cloud accounts, then shuttles files between them under a user-confirmed plan. The binary is `portage`. The GitHub repo is `file-portage` so it does not collide with Gentoo Portage.

Gaming clips (and anything else) that are split across a nearly full SSD, OneDrive, and Google Drive become one catalog. You write a placement policy — keep clips local **and** on whichever cloud has more free space — and Portage produces a dry-run plan that never drives local free space below a reserved staging budget, never deletes the last verified copy, and never creates a public share.

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
  configs/examples/     # gaming-clips.yaml
  docs/design.md        # approved design + 14-PR plan
  docs/FEATURES.md      # MVP / v1 / later
```

The workspace does not exist yet. PR 1 creates it. Until then CI skips Cargo steps.

## Requirements

- Windows 10/11 (primary). Linux/macOS must compile local+cloud paths.
- PowerShell 7+ (`pwsh`) for helper scripts.
- Rust stable (after PR 1).
- Bring-your-own OAuth client ids: `PORTAGE_GOOGLE_CLIENT_ID`, `PORTAGE_MS_CLIENT_ID`.

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
```

Empty, `y`, or `yes` is **rejected** at apply. Confirmation is the exact plan id.

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
- Keep CI green. No force-push on `main` without asking.

## Roadmap (high level)

| Item | Status |
|------|--------|
| Standard repo kit | Done |
| Approved design + feature set | Done — `docs/design.md` |
| PR 1 workspace + CLI stub | Next |
| Local index + dups (PR 2–5) | Not started |
| Drive + OneDrive inventory (PR 6–8) | Not started |
| Planner dry-run (PR 9–10) | Not started |
| Confirmed apply (PR 11–12) | Not started |
| Undo / doctor / polish (PR 13–14) | Not started |

## License

MIT. See [LICENSE](LICENSE).
