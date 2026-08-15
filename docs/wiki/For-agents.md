# For agents

You are continuing Portage. Greg may be away. Do not invent a new architecture.

## Start

1. Read [PROJECT.md](https://github.com/lundgren-greg/portage-app/blob/main/PROJECT.md).
2. Read the **PR Plan** at the bottom of [docs/design.md](https://github.com/lundgren-greg/portage-app/blob/main/docs/design.md).
3. `git -C C:\Repos\portage-app status` and `git log -1 --oneline`.
4. `gh auth status`. Remote is `https://github.com/lundgren-greg/portage-app.git`.

## Do now

Implement **PR 1 only**: Cargo workspace, eight crate stubs, `portage --help`, `portage init` (measure C:, recommend `data_dir` if C: < 8 GiB), rust-toolchain, example config copy.

Branch + pull request. **Do not push `main`.**

## Do not

- Skip ahead to providers, planner, or apply.
- Add share-link / `anyoneWithLink` helpers.
- Open OneDrive or DriveFS placeholders.
- Auto-apply plans. The LLM never applies.
- Commit secrets, tokens, or a real file inventory.
- Force-push `main`.

## Hard rules (even in PR 1, keep the door open)

Last-copy, typed plan id, staging reserve, private-only uploads, volume serial for USB, fail closed if a needed disk is unplugged. Details: [Safety](Safety) and [External drives](External-drives).
