//! Integration test: full catalog life cycle on a temp DB (PR 3).
//!
//! Boundary covered: SQLite on a real filesystem — open/migrate, lock
//! enforcement, scan → batched file insert → proto-blob → hash binding →
//! lookups → capacity snapshot → integrity. Temp dirs clean up on drop.

use portage_core::ids::ContentId;

use portage_catalog::{Catalog, Hydration, LockMode, NewFile, ReplicaState};

#[test]
fn end_to_end_inventory_round_trip() {
    let dir = tempfile::tempdir().expect("temp dir");
    let mut catalog = Catalog::open(dir.path(), LockMode::Exclusive).expect("open");
    assert!(
        dir.path().join("catalog.sqlite").exists(),
        "catalog file created next to the lock"
    );
    assert_eq!(
        catalog.user_version().unwrap(),
        2,
        "both migrations applied"
    );

    // A second writer must be refused while we hold the exclusive lock.
    let refused = Catalog::open(dir.path(), LockMode::Exclusive);
    assert!(
        refused
            .err()
            .map(|e| e.to_string())
            .unwrap_or_default()
            .starts_with("catalog locked by pid "),
        "second writer must see `catalog locked by pid N`"
    );

    // Provider + location, then a scan inserting a small batch.
    catalog.upsert_provider("local-d", "local", None).unwrap();
    catalog
        .upsert_location("vol-abcd", "local-d", "volume", Some("D:"), Some("D:\\"))
        .unwrap();

    let scan_id = catalog.start_scan("local-d").unwrap();
    let mut placeholder = NewFile::local_byte("OneDrive/pinned.mp4", "pinned.mp4", 500);
    placeholder.hydration = Hydration::Placeholder;
    let ids = catalog
        .insert_files(
            "vol-abcd",
            scan_id,
            &[
                NewFile::local_byte("Videos/Captures/boss.mp4", "boss.mp4", 2_200_000),
                NewFile::local_byte("Videos/Captures/raid.mp4", "raid.mp4", 1_800_000),
                placeholder,
            ],
        )
        .unwrap();
    catalog
        .finish_scan(scan_id, ids.len() as u64, true)
        .unwrap();
    assert_eq!(catalog.scan(scan_id).unwrap().unwrap().status, "ok");

    // Lookup by path; the placeholder must not have produced a replica.
    let boss = catalog
        .file_by_path("vol-abcd", "Videos/Captures/boss.mp4")
        .unwrap()
        .expect("boss.mp4 indexed");
    assert_eq!(boss.size, Some(2_200_000));
    let blob = catalog.blob_for_file(boss.id).unwrap().expect("proto-blob");
    assert_eq!(blob.content_id, None, "unhashed proto-blob");
    let pinned = catalog
        .file_by_path("vol-abcd", "OneDrive/pinned.mp4")
        .unwrap()
        .unwrap();
    assert!(
        catalog.blob_for_file(pinned.id).unwrap().is_none(),
        "a placeholder is not a copy and gets no blob/replica"
    );

    // Hash lands (as PR 5 will do): bind content id, verify the replica.
    let content = ContentId::from_bytes(*blake3::hash(b"boss fight bytes").as_bytes());
    catalog.set_blob_content_id(blob.id, &content).unwrap();
    let by_content = catalog.files_by_content_id(&content).unwrap();
    assert_eq!(by_content.len(), 1);
    assert_eq!(by_content[0].id, boss.id);

    assert_eq!(catalog.verified_replica_count(blob.id).unwrap(), 0);
    let replica = &catalog.replicas_for_blob(blob.id).unwrap()[0];
    catalog
        .set_replica_state(replica.id, ReplicaState::Verified)
        .unwrap();
    assert_eq!(
        catalog.verified_replica_count(blob.id).unwrap(),
        1,
        "suspect became verified only after the explicit state change"
    );

    // Capacity snapshot insert + latest.
    catalog
        .insert_capacity_snapshot(
            "vol-abcd",
            Some(500_000_000_000),
            496_000_000_000,
            4_000_000_000,
            None,
        )
        .unwrap();
    let latest = catalog.latest_capacity("vol-abcd").unwrap().unwrap();
    assert_eq!(
        latest.free_bytes, 4_000_000_000,
        "the 4 GB worked-example volume"
    );

    // Health checks pass on a clean catalog.
    assert!(catalog.integrity_check().unwrap().is_empty());
    assert_eq!(catalog.foreign_key_check().unwrap(), 0);

    // Reopen read-only: data survived, shared lock is enough for reads.
    drop(catalog);
    let readonly = Catalog::open(dir.path(), LockMode::Shared).expect("reopen shared");
    let boss_again = readonly
        .file_by_path("vol-abcd", "Videos/Captures/boss.mp4")
        .unwrap()
        .unwrap();
    assert_eq!(boss_again.id, boss.id);
}
