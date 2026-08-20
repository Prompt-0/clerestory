//! Sysfs filesystem abstraction trait supporting hermetic mock testing.

use std::fs;
use std::path::{Path, PathBuf};

/// Trait providing filesystem read access to /sys and /proc hierarchies.
pub trait SysfsProvider: Send + Sync + Clone {
    /// Root path of the sysfs hierarchy.
    fn root(&self) -> &Path;

    /// Reads a sysfs / procfs file to string.
    fn read_to_string(&self, relative_or_abs_path: &str) -> std::io::Result<String> {
        let clean = relative_or_abs_path.trim_start_matches('/');
        let full_path = self.root().join(clean);
        fs::read_to_string(full_path)
    }

    /// Checks if a path exists within the sysfs hierarchy.
    fn exists(&self, relative_or_abs_path: &str) -> bool {
        let clean = relative_or_abs_path.trim_start_matches('/');
        let full_path = self.root().join(clean);
        full_path.exists()
    }

    /// Reads directory entries within the sysfs hierarchy.
    fn read_dir(&self, relative_or_abs_path: &str) -> std::io::Result<Vec<PathBuf>> {
        let clean = relative_or_abs_path.trim_start_matches('/');
        let full_path = self.root().join(clean);
        let mut entries = Vec::new();
        for entry in fs::read_dir(full_path)? {
            entries.push(entry?.path());
        }
        Ok(entries)
    }
}

/// Real Linux host implementation targeting root `/`.
#[derive(Debug, Default, Clone, Copy)]
pub struct RealSysfs;

impl SysfsProvider for RealSysfs {
    fn root(&self) -> &Path {
        Path::new("/")
    }
}

/// Mock implementation for hermetic test fixtures.
#[derive(Debug, Clone)]
pub struct MockSysfs {
    pub fixture_root: PathBuf,
}

impl MockSysfs {
    pub fn new<P: Into<PathBuf>>(fixture_root: P) -> Self {
        Self {
            fixture_root: fixture_root.into(),
        }
    }
}

impl SysfsProvider for MockSysfs {
    fn root(&self) -> &Path {
        &self.fixture_root
    }
}
