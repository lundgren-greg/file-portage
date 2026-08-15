# Use cases

The product is the same for any large library that has wandered. **Gaming clips are a real one. They are not the only one.**

Portage does not become a game tool, a photo app, or a document suite. It places **bytes**.

| Situation | What you tell Portage |
| --- | --- |
| Game captures half on OneDrive, half on Google Drive, SSD almost full | Keep recent clips on the fast disk **and** on whichever cloud has more space. Older ones can leave the SSD once a verified cloud copy exists. |
| Work docs and PDFs in both clouds, copies you cannot tell apart | One inventory, confirmed duplicates by content, keep two verified copies where you asked — not three mystery ones. |
| A folder that should live on the USB drive *and* in the cloud | External disk is the **home**; the cloud is the replica. Or the reverse. |
| Internal disk has 4 GiB free and you need to move something large between clouds | Plug in the USB drive as a **hop**. Download → verify → upload → delete staging. The machine stays above its reserve. |
| Old zip/backup folders still sitting on C: or D: | Mark them archive / cloud-only. Evict local only after the remote copy verifies. |
| Same file under three names in three places | `dups` shows confirmed content matches. The plan does not delete the last verified copy. |

## What “organize” means in Release 1

The LLM (`portage ask`) talks through what you meant. The engine default is **placement**: where copies live, how many, on which volume or cloud.

It does not autonomously rename files or rebuild folder trees. If you want a dest folder pattern, you say so and it becomes policy. Apply still requires the plan id.

## Who this is for (R1)

Greg’s machine. One PC. One Google account, one personal Microsoft account. Other people can read the public repo; they are not the first `apply` user.
