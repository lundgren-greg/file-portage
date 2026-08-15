//! Capacity snapshots: measured free space per location.
//!
//! `free_bytes` is authoritative for planning (design, Capacity model).
//! The planner reads the latest snapshot; the executor re-measures live
//! before every op (PR 11) — a snapshot is never a substitute for that.

use rusqlite::{OptionalExtension, Row};

use crate::db::{now_rfc3339, Catalog};
use crate::Result;

/// A stored `capacity_snapshots` row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapacitySnapshot {
    /// Catalog-assigned row id.
    pub id: i64,
    /// The measured location.
    pub location_id: String,
    /// Total bytes, `None` if the backend cannot report it.
    pub total_bytes: Option<u64>,
    /// Bytes in use.
    pub used_bytes: u64,
    /// Free bytes — authoritative for planning.
    pub free_bytes: u64,
    /// Quota, for cloud providers that have one.
    pub quota_bytes: Option<u64>,
    /// RFC 3339 measurement time.
    pub measured_at: String,
}

fn snapshot_row(row: &Row<'_>) -> rusqlite::Result<CapacitySnapshot> {
    Ok(CapacitySnapshot {
        id: row.get(0)?,
        location_id: row.get(1)?,
        total_bytes: row.get::<_, Option<i64>>(2)?.map(crate::db::from_db_u64),
        used_bytes: crate::db::from_db_u64(row.get(3)?),
        free_bytes: crate::db::from_db_u64(row.get(4)?),
        quota_bytes: row.get::<_, Option<i64>>(5)?.map(crate::db::from_db_u64),
        measured_at: row.get(6)?,
    })
}

impl Catalog {
    /// Record a capacity measurement for a location; returns the row id.
    pub fn insert_capacity_snapshot(
        &self,
        location_id: &str,
        total_bytes: Option<u64>,
        used_bytes: u64,
        free_bytes: u64,
        quota_bytes: Option<u64>,
    ) -> Result<i64> {
        self.conn().execute(
            "INSERT INTO capacity_snapshots
               (location_id, total_bytes, used_bytes, free_bytes, quota_bytes, measured_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                location_id,
                total_bytes.map(crate::db::to_db_u64),
                crate::db::to_db_u64(used_bytes),
                crate::db::to_db_u64(free_bytes),
                quota_bytes.map(crate::db::to_db_u64),
                now_rfc3339(),
            ),
        )?;
        Ok(self.conn().last_insert_rowid())
    }

    /// The most recent snapshot for a location, if any.
    pub fn latest_capacity(&self, location_id: &str) -> Result<Option<CapacitySnapshot>> {
        Ok(self
            .conn()
            .query_row(
                "SELECT id, location_id, total_bytes, used_bytes, free_bytes,
                        quota_bytes, measured_at
                 FROM capacity_snapshots WHERE location_id = ?1
                 ORDER BY id DESC LIMIT 1",
                [location_id],
                snapshot_row,
            )
            .optional()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lock::LockMode;

    #[test]
    fn insert_and_latest_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let catalog = Catalog::open(dir.path(), LockMode::Exclusive).unwrap();
        catalog.upsert_provider("local-d", "local", None).unwrap();
        catalog
            .upsert_location("vol-1", "local-d", "volume", None, Some("D:\\"))
            .unwrap();

        assert!(catalog.latest_capacity("vol-1").unwrap().is_none());

        catalog
            .insert_capacity_snapshot("vol-1", Some(1_000), 900, 100, None)
            .unwrap();
        catalog
            .insert_capacity_snapshot("vol-1", Some(1_000), 600, 400, Some(2_000))
            .unwrap();

        let latest = catalog.latest_capacity("vol-1").unwrap().unwrap();
        assert_eq!(latest.free_bytes, 400);
        assert_eq!(latest.used_bytes, 600);
        assert_eq!(latest.total_bytes, Some(1_000));
        assert_eq!(latest.quota_bytes, Some(2_000));
        assert!(latest.measured_at.contains('T'));
    }
}
