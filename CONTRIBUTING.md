# Contributing

PRs are welcome for bugs and small fixes. Larger changes should start as an issue.

## Before you start

1. Read [PROJECT.md](PROJECT.md) and [docs/design.md](docs/design.md) (PR Plan).
2. Read [AGENTS.md](AGENTS.md) and [`.github/copilot-instructions.md`](.github/copilot-instructions.md).
3. Keep [SECURITY.md](SECURITY.md) in mind — no secrets, no share-link APIs, no unexpected network.
4. Implement the next numbered PR only. Planner PRs need P-space and P-last-copy tests.

## Workflow

1. Branch from `main`.
2. Add or update tests with the behavior change.
3. Run the CI-equivalent commands locally.
4. Open a PR against `main`. Fill in the PR template.
5. CI (`Build & Test` on Windows and Ubuntu) should stay green.

## PowerShell

Use approved verbs and `Verb-Noun` file names. Scripts that are meant to be invoked
directly should have `[CmdletBinding()]`.
