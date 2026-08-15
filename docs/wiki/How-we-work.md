# How we work

**Owner:** [Greg](https://github.com/lundgren-greg) (`@lundgren-greg`).

The repo is **public**. `main` is **PR-only**. Every change lands through a pull request Greg reviews and merges. Direct pushes to `main` are rejected. Force-push and deleting `main` are blocked.

- CODEOWNERS: `* @lundgren-greg`
- CI must be green: `Build & Test` on Windows and Ubuntu
- Stale reviews are dismissed; review threads must be resolved
- Greg can merge PRs; he cannot push `main` directly
- GitHub will not let you approve your own PR. Owner merge of a self-authored PR uses admin merge when that is the only path

## What lives where

| Place | Job |
| --- | --- |
| This wiki | Orientation. Decisions. How we work. |
| `README.md` | Public pitch and use cases |
| `docs/design.md` | Spec an engineer can implement |
| `docs/FEATURES.md` | Checkbox feature set |
| `PROJECT.md` | Session resume, blockers, next PR |
| `configs/examples/` | Policy fixtures (not product verticals) |

Do not treat the wiki as a second design doc. If the spec changes, change `docs/design.md` in a PR, then update the wiki to match.

## Agents and contributors

1. Read `PROJECT.md`, then the design **PR Plan**.
2. Implement the **next numbered PR only** (unless the design marks it independent).
3. Branch from `main`. Open a pull request. Do not push `main`.
4. Planner PRs are incomplete without P-space and P-last-copy tests.
5. No share-link APIs. No placeholder hydration. No apply without a typed plan id.

Local clone today is still `C:\Repos\file-portage`. The GitHub name is `portage-app`.
