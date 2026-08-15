//! Scan rows: one per index run, referenced by `files.last_scan_id`.
//!
//! `start_scan` inserts a `running` row; `finish_scan` closes it with
//! `ok` or `error` and the number of files seen. Delta cursors live in
//! `scan_cursors` (cloud providers, PR 7/8).

use rusqlite::{OptionalExtension, Row};

use crate::db::{now_rfc3339, Catalog};
use crate::Result;

/// A stored `scans` row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanRow {
    /// Catalog-assigned scan id.
    pub id: i64,
    /// The provider that was scanned.
    pub provider_id: String,
    /// RFC 3339 start time.
    pub started_at: String,
    /// RFC 3339 finish time, `None` while running.
    pub finished_at: Option<String>,
    /// Files seen by the scan.
    pub files_seen: u64,
    /// running | ok | error.
    pub status: String,
}

fn scan_row(row: &Row<'_>) -> rusqlite::Result<ScanRow> {
    Ok(ScanRow {
        id: row.get(0)?,
        provider_id: row.get(1)?,
        started_at: row.get(2)?,
        finished_at: row.get(3)?,
        files_seen: crate::db::from_db_u64(row.get(4)?),
        status: row.get(5)?,
    })
}

impl Catalog {
    /// Begin a scan for `provider_id`; returns the new scan id.
    pub fn start_scan(&self, provider_id: &str) -> Result<i64> {
        self.conn().execute(
            "INSERT INTO scans (provider_id, started_at, status) VALUES (?1, ?2, 'running')",
            (provider_id, now_rfc3339()),
        )?;
        Ok(self.conn().last_insert_rowid())
    }

    /// Close a scan with its final status and file count.
    pub fn finish_scan(&self, scan_id: i64, files_seen: u64, ok: bool) -> Result<()> {
        self.conn().execute(
            "UPDATE scans SET finished_at = ?1, files_seen = ?2,
                              status = CASE WHEN ?3 THEN 'ok' ELSE 'error' END
             WHERE id = ?4",
            (now_rfc3339(), crate::db::to_db_u64(files_seen), ok, scan_id),
        )?;
        Ok(())
    }

    /// Fetch one scan row.
    pub fn scan(&self, scan_id: i64) -> Result<Option<ScanRow>> {
        Ok(self
            .conn()
            .query_row(
                "SELECT id, provider_id, started_at, finished_at, files_seen, status
                 FROM scans WHERE id = ?1",
                [scan_id],
                scan_row,
            )
            .optional()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lock::LockMode;

    #[test]
    fn start_then_finish_scan_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let catalog = Catalog::open(dir.path(), LockMode::Exclusive).unwrap();
        catalog.upsert_provider("local-d", "local", None).unwrap();

        let id = catalog.start_scan("local-d").unwrap();
        let running = catalog.scan(id).unwrap().unwrap();
        assert_eq!(running.status, "running");
        assert_eq!(running.provider_id, "local-d");
        assert_eq!(running.finished_at, None);

        catalog.finish_scan(id, 42, true).unwrap();
        let done = catalog.scan(id).unwrap().unwrap();
        assert_eq!(done.status, "ok");
        assert_eq!(done.files_seen, 42);
        assert!(done.finished_at.is_some());

        catalog.finish_scan(id, 42, false).unwrap();
        assert_eq!(catalog.scan(id).unwrap().unwrap().status, "error");

        assert!(catalog.scan(id + 100).unwrap().is_none());
    }
}
