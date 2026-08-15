//! Single-writer `portage.lock` enforcement (design.md, Process model).
//!
//! `portage init` (PR 1) creates the empty lock file; this module owns the
//! locking. Commands that mutate the catalog (`index`, `plan`, `apply`,
//! `resume`) take the exclusive lock and record their pid in the file.
//! Read-only commands (`capacity` cached, `plan show`, `search`, `list`)
//! take a shared lock. A conflicting acquire fails immediately with
//! `catalog locked by pid N` — no waiting, no daemon.

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use fs2::FileExt;
use portage_core::Error as CoreError;

use crate::Result;

/// How the catalog is being opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockMode {
    /// Single writer. Records this process id in the lock file.
    Exclusive,
    /// Concurrent readers. Blocks writers while held.
    Shared,
}

/// A held lock on `portage.lock`. Released on drop.
#[derive(Debug)]
pub struct CatalogLock {
    file: File,
    path: PathBuf,
    mode: LockMode,
}

impl CatalogLock {
    /// Acquire the lock at `path`, creating the file if `portage init` has
    /// not run yet. Fails fast with `catalog locked by pid N` on conflict;
    /// the pid is the last exclusive holder's (best effort).
    pub fn acquire(path: &Path, mode: LockMode) -> Result<Self> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .map_err(|source| CoreError::Io {
                path: path.to_path_buf(),
                source,
            })?;

        let locked = match mode {
            LockMode::Exclusive => FileExt::try_lock_exclusive(&file),
            LockMode::Shared => FileExt::try_lock_shared(&file),
        };
        if locked.is_err() {
            return Err(CoreError::CatalogLocked {
                pid: read_holder_pid(path),
            }
            .into());
        }

        if mode == LockMode::Exclusive {
            // Best-effort holder record for the error message above.
            let _ = write_pid(&mut file, std::process::id());
        }

        Ok(Self {
            file,
            path: path.to_path_buf(),
            mode,
        })
    }

    /// The lock file path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The mode this lock was acquired in.
    pub fn mode(&self) -> LockMode {
        self.mode
    }
}

impl Drop for CatalogLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.file);
    }
}

/// Read the pid recorded by the current/last exclusive holder. `0` if the
/// file is empty or unreadable (e.g. only shared locks were ever taken).
fn read_holder_pid(path: &Path) -> u32 {
    let mut text = String::new();
    if let Ok(mut file) = File::open(path) {
        let _ = file.read_to_string(&mut text);
    }
    text.trim().parse().unwrap_or(0)
}

fn write_pid(file: &mut File, pid: u32) -> std::io::Result<()> {
    file.set_len(0)?;
    file.seek(SeekFrom::Start(0))?;
    write!(file, "{pid}")?;
    file.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lock_path(dir: &tempfile::TempDir) -> PathBuf {
        dir.path().join("portage.lock")
    }

    #[test]
    fn exclusive_blocks_second_exclusive_and_reports_pid() {
        let dir = tempfile::tempdir().unwrap();
        let path = lock_path(&dir);
        let held = CatalogLock::acquire(&path, LockMode::Exclusive).unwrap();
        assert_eq!(held.mode(), LockMode::Exclusive);
        assert_eq!(held.path(), path.as_path());

        let err = CatalogLock::acquire(&path, LockMode::Exclusive).unwrap_err();
        let msg = err.to_string();
        assert_eq!(msg, format!("catalog locked by pid {}", std::process::id()));
    }

    #[test]
    fn shared_allows_concurrent_readers_but_blocks_writers() {
        let dir = tempfile::tempdir().unwrap();
        let path = lock_path(&dir);
        let _r1 = CatalogLock::acquire(&path, LockMode::Shared).unwrap();
        let _r2 = CatalogLock::acquire(&path, LockMode::Shared).unwrap();
        assert!(CatalogLock::acquire(&path, LockMode::Exclusive).is_err());
    }

    #[test]
    fn exclusive_blocks_shared_and_drop_releases() {
        let dir = tempfile::tempdir().unwrap();
        let path = lock_path(&dir);
        {
            let _w = CatalogLock::acquire(&path, LockMode::Exclusive).unwrap();
            assert!(CatalogLock::acquire(&path, LockMode::Shared).is_err());
        }
        // Dropped: both modes acquire again.
        let again = CatalogLock::acquire(&path, LockMode::Exclusive).unwrap();
        drop(again);
        CatalogLock::acquire(&path, LockMode::Shared).unwrap();
    }

    #[test]
    fn missing_lock_dir_maps_to_io_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing").join("portage.lock");
        let err = CatalogLock::acquire(&path, LockMode::Exclusive).unwrap_err();
        assert!(err.to_string().starts_with("io error at"));
    }
}
