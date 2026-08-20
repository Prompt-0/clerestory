//! VirtIO driver archive verification and download helper.

use crate::error::{ClerestoryError, Result};
use std::path::{Path, PathBuf};

pub const VIRTIO_WIN_STABLE_URL: &str =
    "https://fedorapeople.org/groups/virt/virtio-win/direct-downloads/stable-virtio/virtio-win.iso";

pub struct VirtioDriverHelper;

impl VirtioDriverHelper {
    /// Locates or suggests the path for virtio-win.iso.
    pub fn find_local_iso() -> Option<PathBuf> {
        let candidates = [
            "/usr/share/virtio-win/virtio-win.iso",
            "/var/lib/libvirt/images/virtio-win.iso",
            "/tmp/virtio-win.iso",
        ];

        for &p in &candidates {
            let path = Path::new(p);
            if path.exists() {
                return Some(path.to_path_buf());
            }
        }
        None
    }

    /// Verifies if a given ISO path is readable.
    pub fn verify_iso<P: AsRef<Path>>(iso_path: P) -> Result<()> {
        let p = iso_path.as_ref();
        if !p.exists() {
            return Err(ClerestoryError::MissingPrerequisite {
                name: format!("VirtIO Driver ISO at {}", p.display()),
                install_hint: format!("Download from: {}", VIRTIO_WIN_STABLE_URL),
            });
        }
        Ok(())
    }
}
