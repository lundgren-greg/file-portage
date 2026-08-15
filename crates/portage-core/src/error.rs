//! Portage error type. Exit-code mapping lives in the CLI.

use std::path::PathBuf;

/// Errors shared across Portage crates.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The catalog lock is held by another process.
    #[error("catalog locked by pid {pid}")]
    CatalogLocked {
        /// Process id of the lock holder, if known.
        pid: u32,
    },

    /// A chosen `data_dir` is unsafe (overlay, Cloud Filter, or low free space).
    #[error("unsafe data_dir {}: {reason}", dir.display())]
    UnsafeDataDir {
        /// The rejected directory.
        dir: PathBuf,
        /// Why it was rejected.
        reason: String,
    },

    /// Wrapper for I/O errors with the path that failed.
    #[error("io error at {}: {source}", path.display())]
    Io {
        /// The path being accessed.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },
}
