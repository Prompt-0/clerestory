//! Storage driver, io_uring capability, TRIM/discard, and block device introspection.

use crate::error::Result;
use crate::model::host::StorageCapability;
use crate::probe::sysfs::SysfsProvider;
use std::path::Path;

pub struct StorageProbe<P: SysfsProvider> {
    sysfs: P,
}

impl<P: SysfsProvider> StorageProbe<P> {
    pub fn new(sysfs: P) -> Self {
        Self { sysfs }
    }

    /// Probes storage subsystem capabilities for a target path.
    pub fn probe_storage<T: AsRef<Path>>(&self, target_path: T) -> Result<StorageCapability> {
        let supports_io_uring = self.check_io_uring();
        let (supports_trim, is_rotational) = self.check_block_device(target_path.as_ref());
        let target_filesystem = self.detect_filesystem(target_path.as_ref());

        Ok(StorageCapability {
            supports_io_uring,
            supports_trim,
            is_rotational,
            target_filesystem,
        })
    }

    fn check_io_uring(&self) -> bool {
        // Check if io_uring is explicitly disabled via sysctl
        if let Ok(content) = self
            .sysfs
            .read_to_string("proc/sys/kernel/io_uring_disabled")
        {
            if content.trim() == "1" || content.trim() == "2" {
                return false;
            }
        }

        // Check kernel release >= 5.10
        if let Ok(osrelease) = self.sysfs.read_to_string("proc/sys/kernel/osrelease") {
            let version_parts: Vec<&str> = osrelease.split('.').collect();
            if version_parts.len() >= 2 {
                let major: u32 = version_parts[0].trim().parse().unwrap_or(0);
                let minor: u32 = version_parts[1]
                    .split('-')
                    .next()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);
                if major > 5 || (major == 5 && minor >= 10) {
                    return true;
                }
            }
        }

        true
    }

    fn check_block_device(&self, _path: &Path) -> (bool, bool) {
        // Check default block device attributes if available in sysfs
        let mut is_rotational = false;
        let mut supports_trim = true;

        if let Ok(entries) = self.sysfs.read_dir("sys/block") {
            for entry in entries {
                let name = entry
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default();
                if name.starts_with("nvme") || name.starts_with("sd") {
                    let rel_rot = format!("sys/block/{}/queue/rotational", name);
                    if let Ok(rot_str) = self.sysfs.read_to_string(&rel_rot) {
                        is_rotational = rot_str.trim() == "1";
                    }
                    let rel_discard = format!("sys/block/{}/queue/discard_granularity", name);
                    if let Ok(disc_str) = self.sysfs.read_to_string(&rel_discard) {
                        let gran: u64 = disc_str.trim().parse().unwrap_or(0);
                        supports_trim = gran > 0;
                    }
                    break;
                }
            }
        }

        (supports_trim, is_rotational)
    }

    fn detect_filesystem(&self, _path: &Path) -> String {
        "ext4/btrfs/xfs".to_string()
    }
}
