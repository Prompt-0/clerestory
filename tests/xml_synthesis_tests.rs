use clerestory::model::enlightenments::{ClockTimers, HyperVEnlightenments};
use clerestory::model::vm::{
    CpuPinningMap, GpuMode, OutputTarget, StorageDisk, UnattendedConfig, VmConfig,
};
use clerestory::synthesis::libvirt_xml::LibvirtXmlSynthesizer;
use std::path::PathBuf;

#[test]
fn test_libvirt_xml_synthesis_complete_hyperv_and_timers() {
    let config = VmConfig {
        name: "win11-test".to_string(),
        memory_mb: 16384,
        use_hugepages: false,
        hugepage_size_kb: None,
        sockets: 1,
        cores_per_socket: 8,
        threads_per_core: 2,
        pinning: Some(CpuPinningMap {
            vcpu_pins: vec![(0, 0), (1, 16), (2, 1), (3, 17)],
            emulator_pins: vec![8, 24],
            iothread_pins: vec![9, 25],
        }),
        hyperv: HyperVEnlightenments::default(),
        clocks: ClockTimers::default(),
        disk: StorageDisk {
            path: PathBuf::from("/var/lib/libvirt/images/win11-test.qcow2"),
            size_gb: 100,
            format: "qcow2".to_string(),
            io_engine: "io_uring".to_string(),
            cache: "none".to_string(),
            discard_unmap: true,
            queues: 4,
        },
        gpu: GpuMode::LookingGlass { shm_size_mb: 64 },
        shared_directory: Some(PathBuf::from("/home/ritesh/Shared")),
        unattended: UnattendedConfig {
            enabled: true,
            admin_username: "ritesh".to_string(),
            admin_password: None,
            bypass_msa: true,
            install_qga: true,
            driver_disk_path: Some(PathBuf::from("/tmp/oemdrv.img")),
        },
        target: OutputTarget::Libvirt,
        win11_iso_path: PathBuf::from("/iso/win11.iso"),
        swtpm_path: Some(PathBuf::from("/usr/bin/swtpm")),
        ovmf_code: Some(PathBuf::from("/usr/share/edk2/ovmf/OVMF_CODE.secboot.fd")),
        ovmf_vars: Some(PathBuf::from("/usr/share/edk2/ovmf/OVMF_VARS.secboot.fd")),
    };

    let xml = LibvirtXmlSynthesizer::synthesize(&config, true).expect("XML synthesis must succeed");

    // Assert Domain Name & Memory
    assert!(xml.contains("<name>win11-test</name>"));
    assert!(xml.contains("<memory unit='MiB'>16384</memory>"));

    // Assert CPU Pinning & Housekeeping Isolation
    assert!(xml.contains("<vcpupin vcpu='0' cpuset='0'/>"));
    assert!(xml.contains("<vcpupin vcpu='1' cpuset='16'/>"));
    assert!(xml.contains("<emulatorpin cpuset='8,24'/>"));
    assert!(xml.contains("<iothreadpin iothread='1' cpuset='9,25'/>"));

    // Assert All 12 Hyper-V Enlightenments
    assert!(xml.contains("<relaxed state='on'/>"));
    assert!(xml.contains("<vapic state='on'/>"));
    assert!(xml.contains("<spinlocks state='on' retries='8191'/>"));
    assert!(xml.contains("<vpindex state='on'/>"));
    assert!(xml.contains("<runtime state='on'/>"));
    assert!(xml.contains("<synic state='on'/>"));
    assert!(xml.contains("<stimer state='on'>"));
    assert!(xml.contains("<direct state='on'/>"));
    assert!(xml.contains("<reset state='on'/>"));
    assert!(xml.contains("<frequencies state='on'/>"));
    assert!(xml.contains("<reenlightenment state='on'/>"));
    assert!(xml.contains("<tlbflush state='on'/>"));
    assert!(xml.contains("<ipi state='on'/>"));

    // Assert Timer Invariants: HPET disabled & Invariant TSC
    assert!(xml.contains("<timer name='hpet' present='no'/>"));
    assert!(xml.contains("<timer name='tsc' present='yes' mode='native'/>"));
    assert!(xml.contains("<feature policy='require' name='invtsc'/>"));
    assert!(xml.contains("<feature policy='require' name='topoext'/>"));

    // Assert VirtIO-SCSI with io_uring and TRIM
    assert!(xml.contains("model='virtio-scsi'"));
    assert!(xml.contains("io='io_uring'"));
    assert!(xml.contains("cache='none'"));
    assert!(xml.contains("discard='unmap'"));

    // Assert TPM 2.0 with CRB interface
    assert!(xml.contains("<tpm model='tpm-crb'>"));

    // Assert Looking Glass IVSHMEM
    assert!(xml.contains("<shmem name='looking-glass'>"));
    assert!(xml.contains("<size unit='M'>64</size>"));

    // Assert VirtioFS with memfd shared memory
    assert!(xml.contains("<source type='memfd'/>"));
    assert!(xml.contains("<access mode='shared'/>"));
    assert!(xml.contains("<driver type='virtiofs'"));
}
