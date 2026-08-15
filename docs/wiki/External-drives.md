# External drives

A USB / external disk is a first-class **local** location, not a later provider type. It can be a hop, a home, or both. Default role is **both**. Each plan says how that run uses the disk.

| Role | Meaning |
| --- | --- |
| **Shuttle** | Intermediate transfer location. Cloud A → this disk → Cloud B. Staging is deleted after verify. Passing through does not make it a replica. |
| **Final** | Storage destination. Files may live here as a verified copy, alone or next to a cloud replica. |

## Rules

- Identity is **volume serial**, not `E:`. A letter change does not create a second location.
- Unplugged → fail closed (`VolumeOffline`). An absent disk never authorizes deleting the last remaining copy.
- Bytes in `.portage-staging` are journal `Partial`, not `verified`.
- Same overlay rules as any local root: a DriveFS virtual letter is not “an external drive.”
- Capacity and the 1 GiB reserve apply on the shuttle volume too.

```text
portage provider add local --root E:\ --id ext-media --role both
```

When the internal disk has only a few gigabytes free, the planner should hop via a connected shuttle disk instead of failing or writing the machine to zero.
