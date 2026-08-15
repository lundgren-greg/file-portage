//! Open, configure, and migrate the SQLite catalog.
//!
//! Every open sets `journal_mode=WAL`, `foreign_keys=ON`, and
//! `busy_timeout=5000` (design.md, Data Model Changes), then applies any
//! pending numbered migrations inside immediate transactions. The catalog
//! version lives in `PRAGMA user_version`. Never edit an applied migration.

use std::path::Path;

use rusqlite::{Connection, OptionalExtension, TransactionBehavior};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::lock::{CatalogLock, LockMode};
use crate::{Error, Result};

/// Numbered migrations, embedded at compile time. Append only.
const MIGRATIONS: &[(i64, &str, &str)] = &[
    (
        1,
        "0001_init",
        include_str!("../../../migrations/0001_init.sql"),
    ),
    (
        2,
        "0002_plans_journal",
        include_str!("../../../migrations/0002_plans_journal.sql"),
    ),
];

/// An open, migrated catalog plus the held `portage.lock`.
#[derive(Debug)]
pub struct Catalog {
    conn: Connection,
    _lock: CatalogLock,
}

impl Catalog {
    /// Open (creating if absent) `catalog.sqlite` in `data_dir`, holding
    /// `portage.lock` in `mode` for the life of the returned value.
    pub fn open(data_dir: &Path, mode: LockMode) -> Result<Self> {
        let lock = CatalogLock::acquire(&portage_core::config::lock_path(data_dir), mode)?;
        let mut conn = Connection::open(portage_core::config::catalog_path(data_dir))?;
        configure(&conn)?;
        migrate(&mut conn)?;
        Ok(Self { conn, _lock: lock })
    }

    /// The applied catalog schema version (`PRAGMA user_version`).
    pub fn user_version(&self) -> Result<i64> {
        Ok(self
            .conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))?)
    }

    /// Run `PRAGMA integrity_check`; returns the problem rows (empty = ok).
    pub fn integrity_check(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare("PRAGMA integrity_check")?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows.into_iter().filter(|line| line != "ok").collect())
    }

    /// Run `PRAGMA foreign_key_check`; returns the number of violations.
    pub fn foreign_key_check(&self) -> Result<usize> {
        let mut stmt = self.conn.prepare("PRAGMA foreign_key_check")?;
        let rows = stmt.query_map([], |_| Ok(()))?;
        Ok(rows.count())
    }

    /// Insert or update a provider row.
    pub fn upsert_provider(&self, id: &str, kind: &str, account: Option<&str>) -> Result<()> {
        self.conn.execute(
            "INSERT INTO providers (id, kind, account, created_at) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET kind = excluded.kind, account = excluded.account",
            (id, kind, account, now_rfc3339()),
        )?;
        Ok(())
    }

    /// Insert or update a location row (volume or cloud root).
    pub fn upsert_location(
        &self,
        id: &str,
        provider_id: &str,
        kind: &str,
        label: Option<&str>,
        root: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO locations (id, provider_id, kind, label, root)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
               provider_id = excluded.provider_id, kind = excluded.kind,
               label = excluded.label, root = excluded.root",
            (id, provider_id, kind, label, root),
        )?;
        Ok(())
    }

    /// Look up a location id by provider and root (used by tests and PR 4).
    pub fn location_id(&self, provider_id: &str, root: &str) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row(
                "SELECT id FROM locations WHERE provider_id = ?1 AND root = ?2",
                (provider_id, root),
                |row| row.get(0),
            )
            .optional()?)
    }

    pub(crate) fn conn(&self) -> &Connection {
        &self.conn
    }

    pub(crate) fn conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }
}

/// Current UTC time as RFC 3339 text (the catalog's timestamp format).
pub(crate) fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| String::from("1970-01-01T00:00:00Z"))
}

/// SQLite stores INTEGER as i64; clamp byte counts on the way in.
pub(crate) fn to_db_u64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

/// …and saturate negatives (impossible for our writers) on the way out.
pub(crate) fn from_db_u64(value: i64) -> u64 {
    u64::try_from(value).unwrap_or(0)
}

fn configure(conn: &Connection) -> Result<()> {
    // WAL is persistent but cannot be set inside a transaction, so it lives
    // here rather than in migration 0001.
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "busy_timeout", 5000)?;
    Ok(())
}

fn migrate(conn: &mut Connection) -> Result<()> {
    for &(version, name, sql) in MIGRATIONS {
        // Check-and-apply inside one immediate transaction so concurrent
        // shared openers cannot double-apply a migration. Dropping the
        // transaction without commit rolls it back (RAII).
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let applied: i64 = tx.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if applied >= version {
            continue;
        }
        tx.execute_batch(sql)
            .and_then(|()| tx.pragma_update(None, "user_version", version))
            .map_err(|source| Error::Migration { name, source })?;
        tx.commit()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_temp() -> (tempfile::TempDir, Catalog) {
        let dir = tempfile::tempdir().unwrap();
        let catalog = Catalog::open(dir.path(), LockMode::Exclusive).unwrap();
        (dir, catalog)
    }

    #[test]
    fn open_applies_all_migrations_and_pragmas() {
        let (_dir, catalog) = open_temp();
        assert_eq!(catalog.user_version().unwrap(), 2);

        let journal: String = catalog
            .conn()
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        assert_eq!(journal.to_lowercase(), "wal");

        let fk: i64 = catalog
            .conn()
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert_eq!(fk, 1);

        // Both migrations' tables exist.
        for table in ["files", "blobs", "replicas", "plans", "journal_ops"] {
            let count: i64 = catalog
                .conn()
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "missing table {table}");
        }
    }

    #[test]
    fn reopen_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        {
            let catalog = Catalog::open(dir.path(), LockMode::Exclusive).unwrap();
            assert_eq!(catalog.user_version().unwrap(), 2);
        }
        let again = Catalog::open(dir.path(), LockMode::Shared).unwrap();
        assert_eq!(again.user_version().unwrap(), 2);
        assert!(again.integrity_check().unwrap().is_empty());
        assert_eq!(again.foreign_key_check().unwrap(), 0);
    }

    #[test]
    fn second_writer_is_rejected_with_pid() {
        let dir = tempfile::tempdir().unwrap();
        let _held = Catalog::open(dir.path(), LockMode::Exclusive).unwrap();
        let err = Catalog::open(dir.path(), LockMode::Exclusive).unwrap_err();
        assert_eq!(
            err.to_string(),
            format!("catalog locked by pid {}", std::process::id())
        );
    }

    #[test]
    fn provider_and_location_upserts_round_trip() {
        let (_dir, catalog) = open_temp();
        catalog.upsert_provider("local-d", "local", None).unwrap();
        catalog
            .upsert_provider("local-d", "local", Some("me"))
            .unwrap();
        catalog
            .upsert_location("vol-1234", "local-d", "volume", Some("D:"), Some("D:\\"))
            .unwrap();
        catalog
            .upsert_location("vol-1234", "local-d", "volume", Some("Data"), Some("D:\\"))
            .unwrap();
        assert_eq!(
            catalog.location_id("local-d", "D:\\").unwrap().as_deref(),
            Some("vol-1234")
        );
        assert_eq!(catalog.location_id("local-d", "E:\\").unwrap(), None);
    }

    #[test]
    fn timestamps_are_rfc3339() {
        let stamp = now_rfc3339();
        assert!(stamp.contains('T') && stamp.ends_with('Z'), "{stamp}");
    }
}
