# Feature set

Implement from [design.md](design.md). This file is the product checklist, not a substitute for the design.

## MVP (PRs 1–14) — usable on a 4 GiB-free Windows box

### Inventory

- [ ] `portage init` — data dir, default config, lock file. Warn if `C:` has less than 8 GiB free and offer `data_dir` on another volume.
- [ ] Register local roots (`provider add local`). Refuse OneDrive / DriveFS overlay roots and Cloud Filter volumes.
- [ ] Walk local NTFS without following junctions out of root. Never open Files On-Demand / DriveFS streamed files.
- [ ] Incremental BLAKE3 of `LocalFull` files only (`ntfs_file_id + size + mtime` skip).
- [ ] Google Drive list/delta + quota (`'me' in owners`, ignore Docs/Sheets/shortcuts-as-content).
- [ ] OneDrive Graph delta + quota (personal `/me/drive` only).
- [ ] Unified catalog: files, proto-blobs, verified/suspect replicas, provider checksum bindings.
- [ ] `search`, `list --collection`, `dups` (confirmed `ContentId` groups vs name+size suspects).
- [ ] `capacity` — used / free / quota / staging reserve.

### Policy

- [ ] YAML collections, first match wins. Archives before Gaming Clips in the example.
- [ ] `keep_local`: `required` | `prefer` | `cloud_only`.
- [ ] `cloud: most_free | specific | any`.
- [ ] `min_replicas`, `pin_local`, `prefer_local`, `dest_subdir` + basename.
- [ ] `prefer` that cannot keep a file local is Satisfiable with a `replica_shortfall` warning.

### Planner (user-confirmed, never auto-destructive)

- [ ] Dry-run `portage plan` / `plan show` with per-op `residual_during` and `residual_after`.
- [ ] Priority: upload-then-evict → evict → upload-keep → **shuttle → download**.
- [ ] Never emit a plan whose trough on any local volume is below `staging_reserve` (default 1 GiB).
- [ ] Last-copy: never schedule delete of the only **verified** replica. Suspect ≠ replica.
- [ ] Cloud-to-cloud is a local shuttle (download to staging, upload, delete staging).
- [ ] `--allow-cloud-delete` parsed and rejected in MVP.
- [ ] Unsatisfiable plans print suggestions; they do not apply anything.
- [ ] Property tests: P-space, P-last-copy, P-prefix-safe. YAML-loaded 4 GiB fixture.

### Apply

- [ ] `portage apply PLAN` requires typing the exact plan id (`y` / `yes` / empty rejected).
- [ ] Serial executor. Dual SpaceDrift preflight before each op.
- [ ] tmp + same-volume rename. Dest is not final until hash verify.
- [ ] Upload verify = native digest (MD5 / SHA1 / …) computed on the bytes we sent vs API checksum. Not BLAKE3==MD5.
- [ ] `assert_parent_private` before write; `assert_private` walks ancestors. `we_created` compensation delete.
- [ ] Crash journal + `resume`. Last-copy `Permit` required for any delete.
- [ ] `status`, `flags.max_apply_bytes` soak limiter.

### Safety / ops

- [ ] Tokens in OS keyring; Windows fallback `%data_dir%/tokens.dpapi`. Never YAML.
- [ ] `undo` builds a reverse plan and requires a second typed id.
- [ ] `doctor` — catalog integrity, overlay detection, tokens, `NeedsAttention`, last-upload ACL recheck.
- [ ] CI `rg` forbids share-link / `anyoneWithLink` endpoints in provider code.
- [ ] No telemetry. No public-link feature.

## v1 (after MVP)

- [ ] Dropbox provider.
- [ ] S3-compatible (Backblaze B2, Wasabi, AWS) with block-public-ACLs.
- [ ] SMB / NAS provider.
- [ ] `--allow-cloud-delete` actually implemented, default off, still last-copy gated.
- [ ] USN Journal incremental local index.
- [ ] Optional `ffprobe` for duration/resolution when `PORTAGE_FFPROBE` is set.
- [ ] FTS5 search.
- [ ] Microsoft 365 / SharePoint site drives (if product decision says so).

## Later / non-goals for now

- Real-time sync daemon, Cloud Filter provider, FUSE.
- TUI (`ratatui`) or GUI — another binary on the same crates.
- Chunk-level dedup, transcoding, backup versioning / ransomware timelines.
- Shipping a verified public OAuth client id (v1 is bring-your-own).
- iOS / Android.
- Encryption-at-rest of user file contents (not a zero-knowledge vault).
- Any "anyone with the link" or third-party cloud-to-cloud SaaS.

## Gaming-clips policy (example)

Checked in at [configs/examples/gaming-clips.yaml](../configs/examples/gaming-clips.yaml).

| Collection | Match | Placement |
| --- | --- | --- |
| Archives (first) | path contains Archive / Archives / OldClips, or zip/7z/rar | cloud-only Google Drive |
| Gaming Clips | video mime/ext and **not** those archive paths | prefer local D: + most-free of Drive/OneDrive, min 2 replicas |
| Documents | office/pdf/md | prefer local + most-free cloud |
| default | everything else | prefer local + most-free cloud |

Worked 4 GiB case (see design): evict a 3.50 GiB archive clip to Google, then download 2.20 + 2.00 + 1.80 of keep-local clips. Minimum residual on D: is **1.50 GiB** (reserve 1.00).
