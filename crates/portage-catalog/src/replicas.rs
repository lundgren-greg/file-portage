//! Replica rows: where a blob's bytes verifiably live.
//!
//! Replica states (design K10): `verified` counts for last-copy protection;
//! `suspect` and `partial` never do. Placeholders are not replicas at all —
//! `files.rs` refuses to create one. Nothing in this crate deletes replicas;
//! only the executor may, behind a `LastCopyGuard` (PR 12).

use rusqlite::Row;

use crate::db::Catalog;
use crate::{Error, Result};

/// Trust state of a replica.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplicaState {
    /// Bytes were hash-verified at this location. Counts for last-copy.
    Verified,
    /// Believed present but not verified. Never counts for last-copy.
    Suspect,
    /// A partial transfer exists (journal in flight).
    Partial,
}

impl ReplicaState {
    /// The TEXT stored in `replicas.state`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::Suspect => "suspect",
            Self::Partial => "partial",
        }
    }

    /// Parse the TEXT stored in `replicas.state`.
    pub fn parse(text: &str) -> Result<Self> {
        match text {
            "verified" => Ok(Self::Verified),
            "suspect" => Ok(Self::Suspect),
            "partial" => Ok(Self::Partial),
            other => Err(Error::Core(portage_core::Error::InvalidId {
                what: "ReplicaState",
                input: other.chars().take(80).collect(),
            })),
        }
    }
}

/// A stored `replicas` row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplicaRow {
    /// Catalog-assigned row id.
    pub id: i64,
    /// The blob whose bytes this replica holds.
    pub blob_id: i64,
    /// The file row where those bytes live.
    pub file_id: i64,
    /// verified | suspect | partial.
    pub state: String,
}

fn replica_row(row: &Row<'_>) -> rusqlite::Result<ReplicaRow> {
    Ok(ReplicaRow {
        id: row.get(0)?,
        blob_id: row.get(1)?,
        file_id: row.get(2)?,
        state: row.get(3)?,
    })
}

impl Catalog {
    /// All replicas of a blob, in row order.
    pub fn replicas_for_blob(&self, blob_id: i64) -> Result<Vec<ReplicaRow>> {
        let mut stmt = self.conn().prepare(
            "SELECT id, blob_id, file_id, state FROM replicas WHERE blob_id = ?1 ORDER BY id",
        )?;
        let rows = stmt
            .query_map([blob_id], replica_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    /// How many **verified** replicas the blob has. Suspect ≠ verified:
    /// this is the only count last-copy logic may use.
    pub fn verified_replica_count(&self, blob_id: i64) -> Result<u64> {
        let count: i64 = self.conn().query_row(
            "SELECT count(*) FROM replicas WHERE blob_id = ?1 AND state = 'verified'",
            [blob_id],
            |row| row.get(0),
        )?;
        Ok(crate::db::from_db_u64(count))
    }

    /// Update a replica's trust state (e.g. after a hash verify in PR 5).
    pub fn set_replica_state(&self, replica_id: i64, state: ReplicaState) -> Result<()> {
        self.conn().execute(
            "UPDATE replicas SET state = ?1 WHERE id = ?2",
            (state.as_str(), replica_id),
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::NewFile;
    use crate::lock::LockMode;

    #[test]
    fn suspect_until_verified_and_counts_reflect_it() {
        let dir = tempfile::tempdir().unwrap();
        let mut catalog = Catalog::open(dir.path(), LockMode::Exclusive).unwrap();
        catalog.upsert_provider("local-d", "local", None).unwrap();
        catalog
            .upsert_location("vol-1", "local-d", "volume", None, Some("D:\\"))
            .unwrap();
        let scan = catalog.start_scan("local-d").unwrap();
        let ids = catalog
            .insert_files("vol-1", scan, &[NewFile::local_byte("a.bin", "a.bin", 5)])
            .unwrap();

        let blob = catalog.blob_for_file(ids[0]).unwrap().unwrap();
        let replicas = catalog.replicas_for_blob(blob.id).unwrap();
        assert_eq!(replicas.len(), 1);
        assert_eq!(replicas[0].state, "suspect");
        assert_eq!(replicas[0].file_id, ids[0]);
        assert_eq!(catalog.verified_replica_count(blob.id).unwrap(), 0);

        catalog
            .set_replica_state(replicas[0].id, ReplicaState::Verified)
            .unwrap();
        assert_eq!(catalog.verified_replica_count(blob.id).unwrap(), 1);

        catalog
            .set_replica_state(replicas[0].id, ReplicaState::Partial)
            .unwrap();
        assert_eq!(catalog.verified_replica_count(blob.id).unwrap(), 0);
    }

    #[test]
    fn state_round_trips_and_rejects_junk() {
        for state in [
            ReplicaState::Verified,
            ReplicaState::Suspect,
            ReplicaState::Partial,
        ] {
            assert_eq!(ReplicaState::parse(state.as_str()).unwrap(), state);
        }
        assert!(ReplicaState::parse("hydrated").is_err());
    }
}
