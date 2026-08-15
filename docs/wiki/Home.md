# Portage

**Portage** inventories files where they already live — internal disks, external drives, OneDrive, Google Drive, and later other providers — then moves them under a plan you confirm.

The product is **Portage**. The binary is `portage`. The GitHub repo is [lundgren-greg/portage-app](https://github.com/lundgren-greg/portage-app) (not Gentoo Portage).

![Two people carrying a canoe over a rainy granite ridge](https://raw.githubusercontent.com/lundgren-greg/portage-app/main/docs/images/portage-trail.jpg)

The metaphor is the trail between two lakes: you carry the cargo overland because you cannot sail the gap. Here the land is often a nearly full internal disk or a USB drive, and the water is each cloud.

**This wiki** is the human map: what we are building, how we work, and the decisions we already made. The implementable spec stays in the repo:

- [README](https://github.com/lundgren-greg/portage-app/blob/main/README.md)
- [Design + PR plan](https://github.com/lundgren-greg/portage-app/blob/main/docs/design.md)
- [Feature checklist](https://github.com/lundgren-greg/portage-app/blob/main/docs/FEATURES.md)
- [PROJECT.md](https://github.com/lundgren-greg/portage-app/blob/main/PROJECT.md) (resume here)

## In one breath

1. Index local volumes (including USB) and connected clouds. Do not open OneDrive / Drive desktop placeholders.
2. Apply placement rules (YAML or a sentence to Grok).
3. Print a dry-run plan with residual free space after every step.
4. You type the **plan id** to apply. `y` / `yes` is rejected. The LLM never applies.
5. Never delete the last verified copy. Never make a file public. Never fill the disk below the staging reserve.

## Where to go next

| Page | What it is |
| --- | --- |
| [Use cases](Use-cases) | Gaming clips and everyday split libraries |
| [Safety](Safety) | Non-negotiable invariants |
| [External drives](External-drives) | USB as hop and/or home |
| [Decisions](Decisions) | Answers we already locked |
| [Roadmap](Roadmap) | PR 1 → 16 |
| [How we work](How-we-work) | Public repo, PR-only `main`, Greg merges |
| [For agents](For-agents) | What to do in the next session |
