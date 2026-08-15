//! Blob rows: content identity and lookups.
//!
//! A blob is one logical byte stream. `content_id` stays NULL (a
//! *proto-blob*) until PR 5 hashes the bytes; provider hash bindings are
//! recorded in `provider_checksums`, never as identity (design K4).

use rusqlite::{OptionalExtension, Row};

use portage_core::ids::ContentId;

use crate::db::Catalog;
use crate::Result;

/// A stored `blobs` row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobRow {
    /// Catalog-assigned row id.
    pub id: i64,
    /// BLAKE3 identity, `None` for a proto-blob that is not hashed yet.
    pub content_id: Option<ContentId>,
    /// Byte length.
    pub size: u64,
    /// MIME type, when known.
    pub mime: Option<String>,
}

fn blob_row(row: &Row<'_>) -> rusqlite::Result<BlobRow> {
    let content: Option<String> = row.get(1)?;
    let content_id = match content {
        Some(text) => Some(text.parse().map_err(|_| {
            rusqlite::Error::FromSqlConversionFailure(
                1,
                rusqlite::types::Type::Text,
                format!("bad content_id: {text}").into(),
            )
        })?),
        None => None,
    };
    Ok(BlobRow {
        id: row.get(0)?,
        content_id,
        size: crate::db::from_db_u64(row.get(2)?),
        mime: row.get(3)?,
    })
}

impl Catalog {
    /// The blob referenced by this file's replica, if any.
    pub fn blob_for_file(&self, file_id: i64) -> Result<Option<BlobRow>> {
        Ok(self
            .conn()
            .query_row(
                "SELECT b.id, b.content_id, b.size, b.mime
                 FROM blobs b JOIN replicas r ON r.blob_id = b.id
                 WHERE r.file_id = ?1",
                [file_id],
                blob_row,
            )
            .optional()?)
    }

    /// Look up a blob by its BLAKE3 content id.
    pub fn blob_by_content_id(&self, content_id: &ContentId) -> Result<Option<BlobRow>> {
        Ok(self
            .conn()
            .query_row(
                "SELECT id, content_id, size, mime FROM blobs WHERE content_id = ?1",
                [content_id.to_string()],
                blob_row,
            )
            .optional()?)
    }

    /// Record the hash of a proto-blob. Fails on a UNIQUE conflict if the
    /// content id already belongs to another blob — merging duplicate blobs
    /// is PR 5's job, not a silent side effect here.
    pub fn set_blob_content_id(&self, blob_id: i64, content_id: &ContentId) -> Result<()> {
        self.conn().execute(
            "UPDATE blobs SET content_id = ?1 WHERE id = ?2",
            (content_id.to_string(), blob_id),
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::NewFile;
    use crate::lock::LockMode;

    fn content(seed: &[u8]) -> ContentId {
        ContentId::from_bytes(*blake3::hash(seed).as_bytes())
    }

    fn seeded_with_file() -> (tempfile::TempDir, Catalog, i64) {
        let dir = tempfile::tempdir().unwrap();
        let mut catalog = Catalog::open(dir.path(), LockMode::Exclusive).unwrap();
        catalog.upsert_provider("local-d", "local", None).unwrap();
        catalog
            .upsert_location("vol-1", "local-d", "volume", None, Some("D:\\"))
            .unwrap();
        let scan = catalog.start_scan("local-d").unwrap();
        let ids = catalog
            .insert_files("vol-1", scan, &[NewFile::local_byte("a.bin", "a.bin", 9)])
            .unwrap();
        (dir, catalog, ids[0])
    }

    #[test]
    fn proto_blob_then_hash_then_lookup() {
        let (_dir, catalog, file_id) = seeded_with_file();

        let blob = catalog.blob_for_file(file_id).unwrap().unwrap();
        assert_eq!(blob.content_id, None, "proto-blob starts unhashed");
        assert_eq!(blob.size, 9);

        let id = content(b"a");
        assert!(catalog.blob_by_content_id(&id).unwrap().is_none());

        catalog.set_blob_content_id(blob.id, &id).unwrap();
        let found = catalog.blob_by_content_id(&id).unwrap().unwrap();
        assert_eq!(found.id, blob.id);
        assert_eq!(found.content_id, Some(id));
    }

    #[test]
    fn duplicate_content_id_is_a_unique_conflict_not_a_merge() {
        let (_dir, mut catalog, first_file) = seeded_with_file();
        let scan = catalog.start_scan("local-d").unwrap();
        let ids = catalog
            .insert_files("vol-1", scan, &[NewFile::local_byte("b.bin", "b.bin", 9)])
            .unwrap();

        let id = content(b"same-bytes");
        let first = catalog.blob_for_file(first_file).unwrap().unwrap();
        let second = catalog.blob_for_file(ids[0]).unwrap().unwrap();
        catalog.set_blob_content_id(first.id, &id).unwrap();
        assert!(
            catalog.set_blob_content_id(second.id, &id).is_err(),
            "PR 5 merges duplicates explicitly; the catalog must not"
        );
    }

    #[test]
    fn missing_file_has_no_blob() {
        let (_dir, catalog, _file) = seeded_with_file();
        assert!(catalog.blob_for_file(9999).unwrap().is_none());
    }
}
