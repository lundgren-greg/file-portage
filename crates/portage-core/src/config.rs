//! Path resolution for the Portage data dir, config, catalog, and lock.
//!
//! Resolution order (design.md, API / Interface Changes):
//!
//! 1. `PORTAGE_CONFIG` if set (config file only)
//! 2. Windows: `%LOCALAPPDATA%\Portage\config.yaml`
//! 3. Linux/macOS: `~/.local/share/portage/config.yaml`
//!
//! Catalog (`catalog.sqlite`), lock (`portage.lock`), and logs live next to
//! the config under the same data dir. Nothing here touches the filesystem;
//! `portage init` creates the dir and `portage-catalog` opens the catalog.

use std::path::{Path, PathBuf};

/// File name of the SQLite catalog inside the data dir.
pub const CATALOG_FILE: &str = "catalog.sqlite";

/// File name of the single-writer lock inside the data dir.
pub const LOCK_FILE: &str = "portage.lock";

/// File name of the YAML config inside the data dir.
pub const CONFIG_FILE: &str = "config.yaml";

/// Default data dir: `%LOCALAPPDATA%\Portage` or `~/.local/share/portage`.
///
/// `None` when the platform base env var (`LOCALAPPDATA` / `HOME`) is unset.
pub fn default_data_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("LOCALAPPDATA").map(|base| PathBuf::from(base).join("Portage"))
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share/portage"))
    }
}

/// The config file path: `PORTAGE_CONFIG` if set, else the default data dir.
pub fn config_path() -> Option<PathBuf> {
    if let Some(explicit) = std::env::var_os("PORTAGE_CONFIG") {
        return Some(PathBuf::from(explicit));
    }
    default_data_dir().map(|dir| dir.join(CONFIG_FILE))
}

/// The SQLite catalog path inside a data dir.
pub fn catalog_path(data_dir: &Path) -> PathBuf {
    data_dir.join(CATALOG_FILE)
}

/// The single-writer lock path inside a data dir.
pub fn lock_path(data_dir: &Path) -> PathBuf {
    data_dir.join(LOCK_FILE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_and_lock_live_in_the_data_dir() {
        let dir = Path::new("some/data/dir");
        assert_eq!(catalog_path(dir), dir.join("catalog.sqlite"));
        assert_eq!(lock_path(dir), dir.join("portage.lock"));
    }

    #[test]
    fn default_data_dir_ends_with_platform_leaf() {
        // CI always has HOME (Unix) or LOCALAPPDATA (Windows) set.
        let dir = default_data_dir().expect("platform base env var");
        #[cfg(windows)]
        assert!(dir.ends_with("Portage"));
        #[cfg(not(windows))]
        assert!(dir.ends_with(".local/share/portage"));
    }

    #[test]
    fn config_path_defaults_next_to_the_catalog() {
        // PORTAGE_CONFIG is not set in the test environment; mutating the
        // process env from a test would race the other tests in this binary.
        if std::env::var_os("PORTAGE_CONFIG").is_none() {
            let config = config_path().expect("platform base env var");
            assert_eq!(config, default_data_dir().unwrap().join("config.yaml"));
        }
    }
}
