//! Hardware virtualization (/dev/kvm), swtpm TPM 2.0, and OVMF SecureBoot firmware discovery.

use crate::error::Result;
use crate::model::host::SecurityPrerequisites;
use crate::probe::sysfs::SysfsProvider;
use std::path::{Path, PathBuf};

pub struct SecurityProbe<P: SysfsProvider> {
    sysfs: P,
}

impl<P: SysfsProvider> SecurityProbe<P> {
    pub fn new(sysfs: P) -> Self {
        Self { sysfs }
    }

    /// Probes all security and firmware prerequisites.
    pub fn probe_security(&self) -> Result<SecurityPrerequisites> {
        let has_kvm_device = self.sysfs.exists("dev/kvm") || Path::new("/dev/kvm").exists();
        let swtpm_bin = self.find_swtpm();
        let (ovmf_code_fd, ovmf_vars_fd) = self.find_ovmf_firmware();
        let supports_nested = self.check_nested_virt();

        Ok(SecurityPrerequisites {
            has_kvm_device,
            swtpm_bin,
            ovmf_code_fd,
            ovmf_vars_fd,
            supports_nested,
        })
    }

    fn find_swtpm(&self) -> Option<PathBuf> {
        let candidates = ["/usr/bin/swtpm", "/usr/local/bin/swtpm", "/bin/swtpm"];

        for &path in &candidates {
            if self.sysfs.exists(path.trim_start_matches('/')) || Path::new(path).exists() {
                return Some(PathBuf::from(path));
            }
        }
        None
    }

    fn find_ovmf_firmware(&self) -> (Option<PathBuf>, Option<PathBuf>) {
        // Pairs of (CODE_FD, VARS_FD)
        let pairs = [
            // Fedora / RHEL / CentOS
            (
                "/usr/share/edk2/ovmf/OVMF_CODE.secboot.fd",
                "/usr/share/edk2/ovmf/OVMF_VARS.secboot.fd",
            ),
            (
                "/usr/share/edk2/ovmf/OVMF_CODE.fd",
                "/usr/share/edk2/ovmf/OVMF_VARS.fd",
            ),
            // Debian / Ubuntu (4M)
            (
                "/usr/share/OVMF/OVMF_CODE_4M.ms.fd",
                "/usr/share/OVMF/OVMF_VARS_4M.ms.fd",
            ),
            (
                "/usr/share/OVMF/OVMF_CODE_4M.secboot.fd",
                "/usr/share/OVMF/OVMF_VARS_4M.secboot.fd",
            ),
            (
                "/usr/share/OVMF/OVMF_CODE.fd",
                "/usr/share/OVMF/OVMF_VARS.fd",
            ),
            // Arch Linux
            (
                "/usr/share/edk2/x64/OVMF_CODE.secboot.4m.fd",
                "/usr/share/edk2/x64/OVMF_VARS.4m.fd",
            ),
            (
                "/usr/share/edk2/x64/OVMF_CODE.4m.fd",
                "/usr/share/edk2/x64/OVMF_VARS.4m.fd",
            ),
        ];

        for (code, vars) in pairs {
            let code_exists =
                self.sysfs.exists(code.trim_start_matches('/')) || Path::new(code).exists();
            let vars_exists =
                self.sysfs.exists(vars.trim_start_matches('/')) || Path::new(vars).exists();
            if code_exists && vars_exists {
                return (Some(PathBuf::from(code)), Some(PathBuf::from(vars)));
            }
        }

        // Return standard Fedora fallback default
        (
            Some(PathBuf::from("/usr/share/edk2/ovmf/OVMF_CODE.secboot.fd")),
            Some(PathBuf::from("/usr/share/edk2/ovmf/OVMF_VARS.secboot.fd")),
        )
    }

    fn check_nested_virt(&self) -> bool {
        if let Ok(val) = self
            .sysfs
            .read_to_string("sys/module/kvm_amd/parameters/nested")
        {
            return val.trim() == "1" || val.trim().eq_ignore_ascii_case("Y");
        }
        if let Ok(val) = self
            .sysfs
            .read_to_string("sys/module/kvm_intel/parameters/nested")
        {
            return val.trim() == "1" || val.trim().eq_ignore_ascii_case("Y");
        }
        false
    }
}
