use clerestory::model::enlightenments::{ClockTimers, HyperVEnlightenments};
use clerestory::model::vm::{GpuMode, OutputTarget, StorageDisk, UnattendedConfig, VmConfig};
use clerestory::synthesis::qemu_script::QemuScriptSynthesizer;
use std::path::PathBuf;

#[test]
fn test_qemu_script_generation() {
    let config = VmConfig {
        name: "win11-standalone".to_string(),
        memory_mb: 8192,
        use_hugepages: false,
        hugepage_size_kb: None,
        sockets: 1,
        cores_per_socket: 4,
        threads_per_core: 2,
        pinning: None,
        hyperv: HyperVEnlightenments::default(),
        clocks: ClockTimers::default(),
        disk: StorageDisk {
            path: PathBuf::from("/vms/win11.qcow2"),
            size_gb: 64,
            format: "qcow2".to_string(),
            io_engine: "io_uring".to_string(),
            cache: "none".to_string(),
            discard_unmap: true,
            queues: 8,
        },
        gpu: GpuMode::SpiceVirtio,
        shared_directory: None,
        unattended: UnattendedConfig {
            enabled: true,
            admin_username: "Admin".to_string(),
            admin_password: None,
            bypass_msa: true,
            install_qga: true,
            driver_disk_path: Some(PathBuf::from("/vms/oemdrv.img")),
        },
        target: OutputTarget::StandaloneQemu,
        win11_iso_path: PathBuf::from("/vms/win11.iso"),
        swtpm_path: None,
        ovmf_code: None,
        ovmf_vars: None,
    };

    let script = QemuScriptSynthesizer::synthesize(&config).expect("Script synthesis must succeed");

    assert!(script.starts_with("#!/usr/bin/env bash"));
    assert!(script.contains("qemu-system-x86_64"));
    assert!(script.contains("-name 'win11-standalone'"));
    assert!(script.contains("-m 8192M"));
    assert!(script.contains("virtio-scsi-pci"));
    assert!(script.contains("hv_stimer_direct"));
    assert!(script.contains("hv_frequencies"));
}
