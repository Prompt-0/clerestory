pub mod audio;
pub mod cpu;
pub mod memory;
pub mod security;
pub mod storage;
pub mod sysfs;

use crate::error::Result;
use crate::model::host::HostReport;
use crate::probe::sysfs::{RealSysfs, SysfsProvider};
use std::path::Path;

/// Consolidated Host Hardware Inspector.
pub struct HostInspector<P: SysfsProvider> {
    sysfs: P,
}

impl HostInspector<RealSysfs> {
    /// Creates a host inspector using the live host `/sys` filesystem.
    pub fn live() -> Self {
        Self { sysfs: RealSysfs }
    }
}

impl<P: SysfsProvider> HostInspector<P> {
    pub fn with_sysfs(sysfs: P) -> Self {
        Self { sysfs }
    }

    /// Conducts a complete hardware and kernel diagnostics inspection.
    pub fn inspect<T: AsRef<Path>>(&self, target_storage_path: T) -> Result<HostReport> {
        let cpu_probe = cpu::CpuProbe::new(self.sysfs.clone());
        let mem_probe = memory::MemoryProbe::new(self.sysfs.clone());
        let storage_probe = storage::StorageProbe::new(self.sysfs.clone());
        let security_probe = security::SecurityProbe::new(self.sysfs.clone());
        let audio_probe = audio::AudioProbe::new(self.sysfs.clone());

        let cpu = cpu_probe.probe()?;
        let numa_nodes = mem_probe.probe_numa()?;
        let hugepages = mem_probe.probe_hugepages()?;
        let storage = storage_probe.probe_storage(target_storage_path)?;
        let security = security_probe.probe_security()?;
        let audio = audio_probe.probe_audio()?;

        Ok(HostReport {
            cpu,
            numa_nodes,
            hugepages,
            storage,
            security,
            audio,
        })
    }
}
