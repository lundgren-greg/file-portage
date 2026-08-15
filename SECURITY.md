# Security Policy

## Scope

**Portage** (`lundgren-greg/portage-app`, binary: `portage`) is a local-first inventory and shuttle tool. It will
eventually talk to Google Drive and Microsoft Graph **only** after the user runs
`provider add` and later `apply`. There is no telemetry and no background daemon in MVP.

Treat paths, filenames, the SQLite catalog, and OAuth tokens as sensitive.

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.x     | Yes (active development) |

## Reporting a Vulnerability

If you discover a security issue, please **do not** open a public GitHub issue.

Use [GitHub private vulnerability reporting](https://github.com/lundgren-greg/portage-app/security/advisories/new)
when available, or contact the maintainer directly.

We will acknowledge receipt within 72 hours.

## Non-negotiable controls

Documented in full in [docs/design.md](docs/design.md). Summary:

- **Private only.** Provider code must not implement share-link / `anyoneWithLink` /
  anonymous ACL helpers, even as dead code. After every upload, assert the item and
  its ancestors are not public. If we created the item and the ACL check fails, delete it.
- **Tokens.** Refresh tokens live in the OS credential store (Windows: Credential
  Manager / DPAPI). Fallback file is `%data_dir%/tokens.dpapi` with a user-only ACL.
  Never YAML, never logs, never git.
- **Last copy.** `Provider::delete` requires a `LastCopyGuard` permit. Placeholders
  and unhashed (suspect) cloud objects do not count as replicas.
- **No hydration.** Do not open OneDrive Files On-Demand or Drive for Desktop
  streamed files. Overlay roots are excluded from the local walker.
- **No silent overwrite.** Same dest path + different content is a conflict, not a write.
- **Confirmation.** `apply` and `undo` require typing the plan id. `y` / `yes` is rejected.
- **Path safety.** Reject `..`, alternate data streams, and reparse points that escape
  the configured root.

## OAuth

v1 is bring-your-own desktop client ids via `PORTAGE_GOOGLE_CLIENT_ID` /
`PORTAGE_MS_CLIENT_ID` (and optional secrets). Loopback PKCE only
(`127.0.0.1` for Google, `localhost` for Microsoft). Google scope is full `drive`
so existing clips can be inventoried; we still never call permissions-create-anyone.

## Security Considerations

- **No network access until the user connects a provider.** Index of local disks is offline.
- **Confidential inputs.** The catalog is an inventory of the user's files. Do not
  upload it. Store it on a volume with space (`data_dir`).
- **Dependencies.** Thin REST over Drive/Graph. Do not embed rclone (AGPL + share APIs).
- **No secrets in repo.** Synthetic samples only under `samples/`. Local confidential
  fixtures belong in `samples/private/` (gitignored). Ignore `secrets.env`,
  `tokens.dpapi`, and `*.sqlite`.
