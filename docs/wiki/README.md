# Portage wiki

Human-facing map of what we are building and how we work. The implementable spec is still [`docs/design.md`](../design.md).

GitHub’s Wiki tab is [lundgren-greg/portage-app/wiki](https://github.com/lundgren-greg/portage-app/wiki). The `.wiki.git` remote does not exist until someone saves the first page in the UI. After that, publish these files with:

```powershell
cd C:\Repos\portage-app
.\scripts\Publish-Wiki.ps1
```

| Page | What it is |
| --- | --- |
| [Home](Home.md) | What Portage is |
| [Use cases](Use-cases.md) | Gaming clips and everyday split libraries |
| [Safety](Safety.md) | Non-negotiable invariants |
| [External drives](External-drives.md) | USB as hop and/or home |
| [Decisions](Decisions.md) | Answers already locked |
| [Roadmap](Roadmap.md) | PR 1 → 16 |
| [How we work](How-we-work.md) | Public repo, usual path is a PR |
| [For agents](For-agents.md) | Next session |
