use clerestory::cli::doctor::DoctorRenderer;
use clerestory::model::host::*;
use std::path::PathBuf;

fn create_mock_report(has_kvm: bool, has_swtpm: bool, has_virt: bool) -> HostReport {
    HostReport {
        cpu: CpuTopology {
            model_name: "Test Processor".to_string(),
            vendor: "GenuineIntel".to_string(),
            sockets: 1,
            total_physical_cores: 4,
            total_threads: 8,
            cache_domains: vec![CacheDomain {
                l3_cache_id: 0,
                socket_id: 0,
                cpu_ids: vec![0, 1, 2, 3, 4, 5, 6, 7],
                core_pairs: vec![],
                size_bytes: 8 * 1024 * 1024,
                has_3d_vcache: false,
            }],
            has_smt: true,
            is_hybrid: false,
            has_invtsc: true,
            has_svm_or_vmx: has_virt,
            has_topoext: false,
        },
        numa_nodes: vec![NumaNode {
            node_id: 0,
            cpu_ids: vec![0, 1, 2, 3, 4, 5, 6, 7],
            total_memory_bytes: 16 * 1024 * 1024 * 1024,
            free_memory_bytes: 8 * 1024 * 1024 * 1024,
        }],
        hugepages: vec![],
        storage: StorageCapability {
            supports_io_uring: true,
            supports_trim: true,
            is_rotational: false,
            target_filesystem: "ext4".to_string(),
        },
        security: SecurityPrerequisites {
            has_kvm_device: has_kvm,
            swtpm_bin: if has_swtpm {
                Some(PathBuf::from("/usr/bin/swtpm"))
            } else {
                None
            },
            ovmf_code_fd: Some(PathBuf::from("/usr/share/edk2/ovmf/OVMF_CODE.secboot.fd")),
            ovmf_vars_fd: Some(PathBuf::from("/usr/share/edk2/ovmf/OVMF_VARS.secboot.fd")),
            supports_nested: true,
        },
        audio: AudioCapability {
            is_pipewire: true,
            is_pulseaudio: false,
            sample_rate: 48000,
            quantum_latency_ms: 10.6,
        },
    }
}

#[test]
fn test_doctor_render_all_healthy_executes_cleanly() {
    let report = create_mock_report(true, true, true);
    // Doctor renderer writes to stdout
    DoctorRenderer::render(&report);
}

#[test]
fn test_doctor_render_missing_kvm_executes_cleanly() {
    let report = create_mock_report(false, true, true);
    DoctorRenderer::render(&report);
}

#[test]
fn test_doctor_render_missing_swtpm_executes_cleanly() {
    let report = create_mock_report(true, false, true);
    DoctorRenderer::render(&report);
}
