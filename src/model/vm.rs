//! Virtual machine domain configuration and deployment specification.

use crate::model::enlightenments::{ClockTimers, HyperVEnlightenments};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// vCPU and helper thread pinning specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CpuPinningMap {
    /// 1:1 mapping of (vcpu_id -> host_cpu_id).
    pub vcpu_pins: Vec<(u32, u32)>,
    /// Host CPU IDs reserved for QEMU main loop / emulator threads.
    pub emulator_pins: Vec<u32>,
    /// Host CPU IDs reserved for VirtIO-SCSI IOThread worker.
    pub iothread_pins: Vec<u32>,
}

/// Graphics and display tier selection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuMode {
    /// VirtIO-GPU / Spice with QXL/Virgl support.
    SpiceVirtio,
    /// Looking Glass low-latency IVSHMEM frame buffer streaming.
    LookingGlass { shm_size_mb: u32 },
    /// Dedicated PCIe GPU passthrough with isolated IOMMU group.
    VfioPassthrough { pci_address: String },
}

/// Primary storage disk specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageDisk {
    pub path: PathBuf,
    pub size_gb: u64,
    pub format: String,    // "qcow2" | "raw"
    pub io_engine: String, // "io_uring" | "native" | "threads"
    pub cache: String,     // "none" | "writeback"
    pub discard_unmap: bool,
    pub queues: u32,
}

/// Unattended Windows 11 installation preferences.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnattendedConfig {
    pub enabled: bool,
    pub admin_username: String,
    pub admin_password: Option<String>,
    pub bypass_msa: bool,
    pub install_qga: bool,
    pub driver_disk_path: Option<PathBuf>,
}

/// Target output format for synthesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputTarget {
    Libvirt,
    StandaloneQemu,
    Both,
}

/// Optimization profile selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationProfile {
    Auto,
    Gaming,
    Compute,
}

/// Complete compiled Virtual Machine specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VmConfig {
    pub name: String,
    pub memory_mb: u64,
    pub use_hugepages: bool,
    pub hugepage_size_kb: Option<u64>,
    pub sockets: u32,
    pub cores_per_socket: u32,
    pub threads_per_core: u32,
    pub pinning: Option<CpuPinningMap>,
    pub hyperv: HyperVEnlightenments,
    pub clocks: ClockTimers,
    pub disk: StorageDisk,
    pub gpu: GpuMode,
    pub shared_directory: Option<PathBuf>,
    pub unattended: UnattendedConfig,
    pub target: OutputTarget,
    pub win11_iso_path: PathBuf,
    pub swtpm_path: Option<PathBuf>,
    pub ovmf_code: Option<PathBuf>,
    pub ovmf_vars: Option<PathBuf>,
}
