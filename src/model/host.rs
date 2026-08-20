//! Host hardware topology, CPU cache domains, SMT sibling pairs, NUMA and device models.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Core classification for modern hybrid microarchitectures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoreType {
    /// High-performance compute core (e.g. Intel Golden Cove / Raptor Cove).
    Performance,
    /// High-efficiency core without SMT (e.g. Intel Gracemont / Skymont).
    Efficient,
    /// Standard homogeneous core (e.g. AMD Zen 3 / Zen 4 / Zen 5).
    Standard,
}

/// SMT thread sibling pair belonging to a single physical core.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorePair {
    pub physical_core_id: u32,
    pub socket_id: u32,
    pub thread_ids: Vec<u32>,
    pub core_type: CoreType,
    pub max_freq_khz: u64,
}

/// L3 Cache / Core Complex Die (CCD) domain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheDomain {
    pub l3_cache_id: u32,
    pub socket_id: u32,
    pub cpu_ids: Vec<u32>,
    pub core_pairs: Vec<CorePair>,
    pub size_bytes: u64,
    pub has_3d_vcache: bool,
}

/// Overall host CPU topology.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CpuTopology {
    pub model_name: String,
    pub vendor: String, // "AuthenticAMD" | "GenuineIntel"
    pub sockets: u32,
    pub total_physical_cores: u32,
    pub total_threads: u32,
    pub cache_domains: Vec<CacheDomain>,
    pub has_smt: bool,
    pub is_hybrid: bool,
    pub has_invtsc: bool,
    pub has_svm_or_vmx: bool,
    pub has_topoext: bool,
}

/// Hugepages availability tier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HugepageTier {
    pub page_size_kb: u64,
    pub total_pages: u64,
    pub free_pages: u64,
    pub mount_point: Option<PathBuf>,
}

/// NUMA node architecture description.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NumaNode {
    pub node_id: u32,
    pub cpu_ids: Vec<u32>,
    pub total_memory_bytes: u64,
    pub free_memory_bytes: u64,
}

/// Storage driver and kernel I/O capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageCapability {
    pub supports_io_uring: bool,
    pub supports_trim: bool,
    pub is_rotational: bool,
    pub target_filesystem: String,
}

/// Security firmware and helper binary status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityPrerequisites {
    pub has_kvm_device: bool,
    pub swtpm_bin: Option<PathBuf>,
    pub ovmf_code_fd: Option<PathBuf>,
    pub ovmf_vars_fd: Option<PathBuf>,
    pub supports_nested: bool,
}

/// Audio subsystem capability.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioCapability {
    pub is_pipewire: bool,
    pub is_pulseaudio: bool,
    pub sample_rate: u32,
    pub quantum_latency_ms: f64,
}

/// Consolidated host diagnostic inspection report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostReport {
    pub cpu: CpuTopology,
    pub numa_nodes: Vec<NumaNode>,
    pub hugepages: Vec<HugepageTier>,
    pub storage: StorageCapability,
    pub security: SecurityPrerequisites,
    pub audio: AudioCapability,
}
