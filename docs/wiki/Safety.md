# Safety

**No data loss is Release 1 P0.** Fun UI and natural language ship after apply + undo are safe.

## Invariants

- Last **verified** copy is never deleted. Suspect / unhashed / placeholder / unplugged disk does not count.
- Every copy is checksum-verified before it is a replica. Upload verify is the cloud’s native digest vs the bytes we sent — not “BLAKE3 equals MD5.”
- Apply requires typing the exact **plan id**. Empty, `y`, and `yes` are rejected.
- The LLM **never applies**. It compiles a sentence to policy + a dry-run plan.
- Undo is a **reverse plan** plus a second typed id. It refuses if that reverse would lose the last copy or breach the staging reserve. It never auto-redownloads.
- Local free space (internal **and** shuttle volume) never goes below `staging_reserve` (default 1 GiB) at any step, including during a transfer.
- OneDrive / Google Drive for Desktop placeholders are never opened and are not replicas. Cloud truth is the API.
- Uploads stay private. Inherited “anyone with the link” on a parent folder fails the op. If we created the object, we delete it.
- No silent overwrite. Same path + different content is a conflict.
- No telemetry. Tokens live in the OS credential store, never in YAML.

## Evicts

Only collections marked **cloud-only** or **unpinned**. Never evict `keep_local: required` or pinned files. `prefer` may drop local with a warning.

Cloud delete is **off** in R1 (`--allow-cloud-delete` is parsed and rejected).
