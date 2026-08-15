//! File rows: batched insert with proto-blob creation, and lookups.
//!
//! Inserting a `byte` file that is a real copy (any hydration except
//! `placeholder`) with a known size creates a **proto-blob** — a `blobs`
//! row whose `content_id` is NULL until PR 5 hashes it — plus a `suspect`
//! replica. Placeholders never get a blob or replica: a placeholder is not
//! a copy (design K6/K10). Directories and shortcuts carry no bytes.

use rusqlite::{OptionalExtension, Row};

use portage_core::ids::ContentId;

use crate::db::Catalog;
use crate::Result;

/// What a `files` row represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    /// A regular byte stream.
    Byte,
    /// A directory.
    Directory,
    /// A provider shortcut/link; `shortcut_target_ref` points at the target.
    Shortcut,
}

impl FileKind {
    /// The TEXT stored in `files.kind`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Byte => "byte",
            Self::Directory => "directory",
            Self::Shortcut => "shortcut",
        }
    }
}

/// How much of the file's body is really present at this location.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hydration {
    /// Full local bytes on disk.
    LocalFull,
    /// OneDrive/DriveFS placeholder — never opened, never a replica.
    Placeholder,
    /// Lives in a cloud provider; bytes are remote.
    CloudNative,
}

impl Hydration {
    /// The TEXT stored in `files.hydration`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LocalFull => "local_full",
            Self::Placeholder => "placeholder",
            Self::CloudNative => "cloud_native",
        }
    }
}

/// A file to insert during a scan. Paths are provider-relative (no `..`).
#[derive(Debug, Clone)]
pub struct NewFile {
    /// Provider-relative path.
    pub path: String,
    /// Base name.
    pub name: String,
    /// byte | directory | shortcut.
    pub kind: FileKind,
    /// Body size in bytes, when known.
    pub size: Option<u64>,
    /// Modification time, RFC 3339 UTC.
    pub mtime_utc: Option<String>,
    /// NTFS file reference number (local volumes only).
    pub ntfs_file_id: Option<String>,
    /// Volume serial (local volumes only).
    pub volume_serial: Option<String>,
    /// MIME type, when known.
    pub mime: Option<String>,
    /// Hydration state at this location.
    pub hydration: Hydration,
    /// Provider item id (cloud only).
    pub remote_ref: Option<String>,
    /// Target reference iff `kind == Shortcut`.
    pub shortcut_target_ref: Option<String>,
}

impl NewFile {
    /// A fully-local byte file — the common local-walk case.
    pub fn local_byte(path: impl Into<String>, name: impl Into<String>, size: u64) -> Self {
        Self {
            path: path.into(),
            name: name.into(),
            kind: FileKind::Byte,
            size: Some(size),
            mtime_utc: None,
            ntfs_file_id: None,
            volume_serial: None,
            mime: None,
            hydration: Hydration::LocalFull,
            remote_ref: None,
            shortcut_target_ref: None,
        }
    }
}

/// A stored `files` row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileRow {
    /// Catalog-assigned row id.
    pub id: i64,
    /// Owning location.
    pub location_id: String,
    /// Provider-relative path.
    pub path: String,
    /// Base name.
    pub name: String,
    /// byte | directory | shortcut.
    pub kind: String,
    /// Body size, when known.
    pub size: Option<u64>,
    /// local_full | placeholder | cloud_native.
    pub hydration: String,
    /// Provider item id (cloud only).
    pub remote_ref: Option<String>,
    /// Scan that last saw this row.
    pub last_scan_id: Option<i64>,
}

fn file_row(row: &Row<'_>) -> rusqlite::Result<FileRow> {
    Ok(FileRow {
        id: row.get(0)?,
        location_id: row.get(1)?,
        path: row.get(2)?,
        name: row.get(3)?,
        kind: row.get(4)?,
        size: row.get::<_, Option<i64>>(5)?.map(crate::db::from_db_u64),
        hydration: row.get(6)?,
        remote_ref: row.get(7)?,
        last_scan_id: row.get(8)?,
    })
}

const FILE_COLUMNS: &str =
    "id, location_id, path, name, kind, size, hydration, remote_ref, last_scan_id";

impl Catalog {
    /// Insert (or refresh) a batch of files seen by `scan_id` at
    /// `location_id`, in one transaction. Returns the row ids in input
    /// order. Each new real `byte` copy gets a proto-blob and a `suspect`
    /// replica; re-scans keep the existing blob linkage.
    pub fn insert_files(
        &mut self,
        location_id: &str,
        scan_id: i64,
        files: &[NewFile],
    ) -> Result<Vec<i64>> {
        let tx = self.conn_mut().transaction()?;
        let mut ids = Vec::with_capacity(files.len());
        {
            let mut upsert = tx.prepare(
                "INSERT INTO files (location_id, path, name, kind, shortcut_target_ref,
                                    is_dir, size, mtime_utc, ntfs_file_id, volume_serial,
                                    mime, hydration, remote_ref, last_scan_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
                 ON CONFLICT(location_id, path) DO UPDATE SET
                   name = excluded.name, kind = excluded.kind,
                   shortcut_target_ref = excluded.shortcut_target_ref,
                   is_dir = excluded.is_dir, size = excluded.size,
                   mtime_utc = excluded.mtime_utc, ntfs_file_id = excluded.ntfs_file_id,
                   volume_serial = excluded.volume_serial, mime = excluded.mime,
                   hydration = excluded.hydration, remote_ref = excluded.remote_ref,
                   last_scan_id = excluded.last_scan_id
                 RETURNING id",
            )?;
            let mut has_replica = tx.prepare("SELECT count(*) FROM replicas WHERE file_id = ?1")?;
            let mut insert_blob = tx.prepare("INSERT INTO blobs (size) VALUES (?1)")?;
            let mut insert_replica = tx.prepare(
                "INSERT INTO replicas (blob_id, file_id, state) VALUES (?1, ?2, 'suspect')",
            )?;

            for file in files {
                let file_id: i64 = upsert.query_row(
                    (
                        location_id,
                        &file.path,
                        &file.name,
                        file.kind.as_str(),
                        &file.shortcut_target_ref,
                        (file.kind == FileKind::Directory) as i64,
                        file.size.map(crate::db::to_db_u64),
                        &file.mtime_utc,
                        &file.ntfs_file_id,
                        &file.volume_serial,
                        &file.mime,
                        file.hydration.as_str(),
                        &file.remote_ref,
                        scan_id,
                    ),
                    |row| row.get(0),
                )?;

                // Proto-blob: only real byte copies of known size (blobs.size
                // is NOT NULL and 0 would masquerade as an empty file).
                // Placeholders are not copies; directories and shortcuts
                // have no bytes. A later scan that learns the size will
                // create the blob then.
                let is_copy = file.kind == FileKind::Byte
                    && file.hydration != Hydration::Placeholder
                    && file.size.is_some();
                if is_copy {
                    let existing: i64 = has_replica.query_row([file_id], |row| row.get(0))?;
                    if existing == 0 {
                        insert_blob
                            .execute([crate::db::to_db_u64(file.size.unwrap_or_default())])?;
                        let blob_id = tx.last_insert_rowid();
                        insert_replica.execute((blob_id, file_id))?;
                    }
                }
                ids.push(file_id);
            }
        }
        tx.commit()?;
        Ok(ids)
    }

    /// Look up one file by location and provider-relative path.
    pub fn file_by_path(&self, location_id: &str, path: &str) -> Result<Option<FileRow>> {
        Ok(self
            .conn()
            .query_row(
                &format!("SELECT {FILE_COLUMNS} FROM files WHERE location_id = ?1 AND path = ?2"),
                (location_id, path),
                file_row,
            )
            .optional()?)
    }

    /// All files holding a replica of the blob with this `ContentId`.
    pub fn files_by_content_id(&self, content_id: &ContentId) -> Result<Vec<FileRow>> {
        let mut stmt = self.conn().prepare(&format!(
            "SELECT {} FROM files f
             JOIN replicas r ON r.file_id = f.id
             JOIN blobs b ON b.id = r.blob_id
             WHERE b.content_id = ?1
             ORDER BY f.id",
            FILE_COLUMNS
                .split(", ")
                .map(|c| format!("f.{c}"))
                .collect::<Vec<_>>()
                .join(", ")
        ))?;
        let rows = stmt
            .query_map([content_id.to_string()], file_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lock::LockMode;

    fn seeded() -> (tempfile::TempDir, Catalog) {
        let dir = tempfile::tempdir().unwrap();
        let catalog = Catalog::open(dir.path(), LockMode::Exclusive).unwrap();
        catalog.upsert_provider("local-d", "local", None).unwrap();
        catalog
            .upsert_location("vol-1", "local-d", "volume", Some("D:"), Some("D:\\"))
            .unwrap();
        (dir, catalog)
    }

    fn scan(catalog: &Catalog) -> i64 {
        catalog.start_scan("local-d").unwrap()
    }

    #[test]
    fn batch_insert_creates_proto_blobs_for_real_copies_only() {
        let (_dir, mut catalog) = seeded();
        let scan_id = scan(&catalog);

        let mut placeholder = NewFile::local_byte("Clips/pinned.mp4", "pinned.mp4", 10);
        placeholder.hydration = Hydration::Placeholder;
        let mut dir_row = NewFile::local_byte("Clips", "Clips", 0);
        dir_row.kind = FileKind::Directory;
        dir_row.size = None;

        let ids = catalog
            .insert_files(
                "vol-1",
                scan_id,
                &[
                    NewFile::local_byte("Clips/boss.mp4", "boss.mp4", 2_200),
                    placeholder,
                    dir_row,
                ],
            )
            .unwrap();
        assert_eq!(ids.len(), 3);

        let blobs: i64 = catalog
            .conn()
            .query_row("SELECT count(*) FROM blobs", [], |r| r.get(0))
            .unwrap();
        let replicas: i64 = catalog
            .conn()
            .query_row("SELECT count(*) FROM replicas", [], |r| r.get(0))
            .unwrap();
        assert_eq!((blobs, replicas), (1, 1), "only the real byte copy");

        // Proto-blob: content_id is NULL until PR 5 hashes it.
        let content: Option<String> = catalog
            .conn()
            .query_row("SELECT content_id FROM blobs", [], |r| r.get(0))
            .unwrap();
        assert_eq!(content, None);
    }

    #[test]
    fn rescan_updates_in_place_and_keeps_the_blob() {
        let (_dir, mut catalog) = seeded();
        let first = scan(&catalog);
        let ids1 = catalog
            .insert_files(
                "vol-1",
                first,
                &[NewFile::local_byte("a.mp4", "a.mp4", 100)],
            )
            .unwrap();

        let second = scan(&catalog);
        let mut grown = NewFile::local_byte("a.mp4", "a.mp4", 150);
        grown.mtime_utc = Some("2026-08-15T00:00:00Z".into());
        let ids2 = catalog.insert_files("vol-1", second, &[grown]).unwrap();
        assert_eq!(ids1, ids2, "same row across scans");

        let row = catalog.file_by_path("vol-1", "a.mp4").unwrap().unwrap();
        assert_eq!(row.size, Some(150));
        assert_eq!(row.last_scan_id, Some(second));

        let blobs: i64 = catalog
            .conn()
            .query_row("SELECT count(*) FROM blobs", [], |r| r.get(0))
            .unwrap();
        assert_eq!(blobs, 1, "no duplicate proto-blob on re-scan");
    }

    #[test]
    fn unknown_size_defers_the_proto_blob_until_a_scan_learns_it() {
        let (_dir, mut catalog) = seeded();
        let first = scan(&catalog);
        let mut unknown = NewFile::local_byte("odd.bin", "odd.bin", 0);
        unknown.size = None;
        let ids = catalog.insert_files("vol-1", first, &[unknown]).unwrap();

        assert!(
            catalog.blob_for_file(ids[0]).unwrap().is_none(),
            "no proto-blob for an unknown size — 0 would look like an empty file"
        );

        let second = scan(&catalog);
        catalog
            .insert_files(
                "vol-1",
                second,
                &[NewFile::local_byte("odd.bin", "odd.bin", 77)],
            )
            .unwrap();
        let blob = catalog.blob_for_file(ids[0]).unwrap().unwrap();
        assert_eq!(blob.size, 77);
    }

    #[test]
    fn lookup_by_path_and_by_content_id() {
        let (_dir, mut catalog) = seeded();
        let scan_id = scan(&catalog);
        let ids = catalog
            .insert_files(
                "vol-1",
                scan_id,
                &[
                    NewFile::local_byte("Videos/clip.mp4", "clip.mp4", 42),
                    NewFile::local_byte("Backup/clip.mp4", "clip.mp4", 42),
                ],
            )
            .unwrap();

        assert!(catalog.file_by_path("vol-1", "nope.mp4").unwrap().is_none());
        let found = catalog
            .file_by_path("vol-1", "Videos/clip.mp4")
            .unwrap()
            .unwrap();
        assert_eq!(found.id, ids[0]);
        assert_eq!(found.kind, "byte");
        assert_eq!(found.hydration, "local_full");

        // Hash lands (PR 5 does this for real): bind both blobs' files.
        let content: ContentId = ContentId::from_bytes(*blake3::hash(b"clip").as_bytes());
        let blob = catalog.blob_for_file(ids[0]).unwrap().unwrap();
        catalog.set_blob_content_id(blob.id, &content).unwrap();

        let files = catalog.files_by_content_id(&content).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "Videos/clip.mp4");
    }
}
