//! Path containment: `ensure_inside(root, candidate)`.
//!
//! Defense against traversal (`..`), NTFS alternate data streams
//! (`file.mp4:zone.identifier`), and symlink/junction escapes. Any path the
//! engine writes or deletes must pass this gate (design threat model).

use std::path::{Component, Path, PathBuf};

use crate::error::Error;

/// Validate that `candidate` stays inside `root` and return the joined path.
///
/// `candidate` may be relative (joined onto `root`) or absolute (must start
/// with `root` lexically). Checks, in order:
///
/// 1. **Lexical:** no `..` components, no NTFS ADS colons in any component
///    (a Windows drive prefix like `C:` is fine), no root/prefix components
///    in the middle of a relative candidate.
/// 2. **Filesystem:** the deepest existing ancestor of the result must
///    canonicalize to somewhere inside the canonicalized root, so a symlink
///    or junction inside the tree cannot escape it.
pub fn ensure_inside(root: &Path, candidate: &Path) -> Result<PathBuf, Error> {
    let escape = |reason: &str| Error::PathEscape {
        root: root.to_path_buf(),
        candidate: candidate.to_path_buf(),
        reason: reason.to_string(),
    };

    // Lexical screening of the candidate's own components.
    let relative = if candidate.is_absolute() {
        candidate
            .strip_prefix(root)
            .map_err(|_| escape("absolute path is outside the root"))?
    } else {
        candidate
    };
    for component in relative.components() {
        match component {
            Component::ParentDir => return Err(escape("contains `..`")),
            Component::RootDir | Component::Prefix(_) => {
                return Err(escape("contains a root or drive prefix"))
            }
            Component::CurDir => {}
            Component::Normal(part) => {
                let part = part.to_string_lossy();
                if part.contains(':') {
                    return Err(escape("contains `:` (NTFS alternate data stream)"));
                }
            }
        }
    }

    let joined = root.join(relative);

    // Filesystem check: the deepest existing ancestor must resolve inside root.
    let canonical_root = root.canonicalize().map_err(|source| Error::Io {
        path: root.to_path_buf(),
        source,
    })?;
    let anchor = deepest_existing(&joined);
    let canonical_anchor = anchor.canonicalize().map_err(|source| Error::Io {
        path: anchor.to_path_buf(),
        source,
    })?;
    if !canonical_anchor.starts_with(&canonical_root) {
        return Err(escape("resolves outside the root (symlink or junction)"));
    }

    Ok(joined)
}

/// The deepest ancestor of `path` that exists on disk (at worst the root of
/// the volume). Used to canonicalize paths that are about to be created.
fn deepest_existing(path: &Path) -> &Path {
    let mut current = path;
    while !current.exists() {
        match current.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => current = parent,
            _ => break,
        }
    }
    current
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        std::fs::create_dir_all(root.join("sub")).unwrap();
        (dir, root)
    }

    #[test]
    fn accepts_normal_relative_paths() {
        let (_guard, root) = root();
        let ok = ensure_inside(&root, Path::new("sub/clip.mp4")).unwrap();
        assert!(ok.starts_with(&root));
        // Paths that do not exist yet are fine — the engine creates them.
        assert!(ensure_inside(&root, Path::new("new-dir/new-file.bin")).is_ok());
    }

    #[test]
    fn accepts_absolute_paths_inside_root() {
        let (_guard, root) = root();
        let inside = root.join("sub").join("clip.mp4");
        assert!(ensure_inside(&root, &inside).is_ok());
    }

    #[test]
    fn rejects_parent_traversal() {
        let (_guard, root) = root();
        for bad in ["../outside.txt", "sub/../../outside.txt", ".."] {
            let err = ensure_inside(&root, Path::new(bad)).unwrap_err();
            assert!(matches!(err, Error::PathEscape { .. }), "accepted: {bad}");
        }
    }

    #[test]
    fn rejects_ads_streams() {
        let (_guard, root) = root();
        let err = ensure_inside(&root, Path::new("clip.mp4:zone.identifier")).unwrap_err();
        assert!(matches!(err, Error::PathEscape { .. }));
        let err = ensure_inside(&root, Path::new("sub/clip.mp4:$DATA")).unwrap_err();
        assert!(matches!(err, Error::PathEscape { .. }));
    }

    #[test]
    fn rejects_absolute_paths_outside_root() {
        let (_guard, root) = root();
        let other = tempfile::tempdir().unwrap();
        let err = ensure_inside(&root, &other.path().join("x.txt")).unwrap_err();
        assert!(matches!(err, Error::PathEscape { .. }));
    }

    #[test]
    fn rejects_rooted_candidate_components() {
        let (_guard, root) = root();
        #[cfg(windows)]
        let bad = Path::new("D:evil.txt");
        #[cfg(not(windows))]
        let bad = Path::new("/etc/passwd");
        let err = ensure_inside(&root, bad).unwrap_err();
        assert!(matches!(err, Error::PathEscape { .. }));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_escape() {
        let (_guard, root) = root();
        let outside = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(outside.path(), root.join("link")).unwrap();
        let err = ensure_inside(&root, Path::new("link/steal.txt")).unwrap_err();
        assert!(
            matches!(err, Error::PathEscape { .. }),
            "symlink escape accepted"
        );
    }

    #[cfg(windows)]
    #[test]
    fn rejects_symlink_escape_when_creatable() {
        // Symlink creation on Windows needs Developer Mode or admin; skip
        // gracefully when unavailable so CI and dev boxes both pass honestly.
        let (_guard, root) = root();
        let outside = tempfile::tempdir().unwrap();
        match std::os::windows::fs::symlink_dir(outside.path(), root.join("link")) {
            Ok(()) => {
                let err = ensure_inside(&root, Path::new("link\\steal.txt")).unwrap_err();
                assert!(matches!(err, Error::PathEscape { .. }));
            }
            Err(e) => eprintln!("skipping symlink test (no privilege): {e}"),
        }
    }
}
