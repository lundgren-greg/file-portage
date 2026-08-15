//! SQLite catalog for Portage (PR 3).
//!
//! Owns the on-disk inventory: providers, locations, scans, files, blobs,
//! replicas, checksums, and capacity snapshots — plus the single-writer
//! `portage.lock` (created by `portage init`, **enforced** here).
//!
//! One process, one catalog. Writers take the exclusive lock; read-only
//! commands take a shared lock. A second writer exits with
//! `catalog locked by pid N` (design.md, Process model).

pub mod blobs;
pub mod capacity;
pub mod db;
pub mod files;
pub mod lock;
pub mod replicas;
pub mod scans;

pub use blobs::BlobRow;
pub use capacity::CapacitySnapshot;
pub use db::Catalog;
pub use files::{FileKind, FileRow, Hydration, NewFile};
pub use lock::{CatalogLock, LockMode};
pub use replicas::{ReplicaRow, ReplicaState};
pub use scans::ScanRow;

/// Errors from the catalog layer.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A shared Portage error (`catalog locked by pid N`, I/O, …).
    #[error(transparent)]
    Core(#[from] portage_core::Error),

    /// An underlying SQLite error.
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// A migration failed to apply; the transaction was rolled back.
    #[error("catalog migration {name} failed: {source}")]
    Migration {
        /// The migration file stem, e.g. `0001_init`.
        name: &'static str,
        /// The SQLite error that aborted it.
        #[source]
        source: rusqlite::Error,
    },
}

/// Catalog result alias.
pub type Result<T> = std::result::Result<T, Error>;
