# Contributing

This is Greg's project ([@lundgren-greg](https://github.com/lundgren-greg)).
External PRs are welcome for bugs and small fixes; larger changes should start as an issue.

**Nothing is pushed to `main`.** Every change lands through a pull request that Greg
reviews and merges. `CODEOWNERS` requires `@lundgren-greg` on every path. Force-push
and deleting `main` are blocked.

## Before you start

1. Read [PROJECT.md](PROJECT.md) and [docs/design.md](docs/design.md) (PR Plan).
2. Read [AGENTS.md](AGENTS.md) and [`.github/copilot-instructions.md`](.github/copilot-instructions.md).
3. Keep [SECURITY.md](SECURITY.md) in mind — no secrets, no share-link APIs, no unexpected network.
4. Implement the next numbered PR only. Planner PRs need P-space and P-last-copy tests.

## Workflow

1. Branch from `main`. Do not commit on `main`.
2. Add or update tests with the behavior change.
3. Run the CI-equivalent commands locally.
4. Open a PR against `main`. Fill in the PR template.
5. CI (`Build & Test` on Windows and Ubuntu) must be green.
6. Wait for Greg (`@lundgren-greg`) to review and merge. Do not merge your own PR unless you are Greg.

## PowerShell

Use approved verbs and `Verb-Noun` file names. Scripts that are meant to be invoked
directly should have `[CmdletBinding()]`.
