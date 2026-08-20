//! End-to-End Real-World Hypervisor & Schema Validation Test Suite.

use fatfs::FileSystem;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

use clerestory::model::enlightenments::{ClockTimers, HyperVEnlightenments};
use clerestory::model::vm::*;
use clerestory::slipstream::fat_disk::FatDiskBuilder;
use clerestory::synthesis::libvirt_xml::LibvirtXmlSynthesizer;

#[test]
fn test_e2e_virt_xml_validate_amd_and_intel_profiles() {
    let temp_dir = tempdir().expect("tempdir");

    // 1. AMD 7950X3D Profile
    let amd_config = VmConfig {
        name: "win11-amd-7950x3d".to_string(),
        memory_mb: 32768,
        use_hugepages: true,
        hugepage_size_kb: Some(2048),
        sockets: 1,
        cores_per_socket: 8,
        threads_per_core: 2,
        pinning: Some(CpuPinningMap {
            vcpu_pins: (0..16).map(|v| (v, v)).collect(),
            emulator_pins: vec![16, 17],
            iothread_pins: vec![18, 19],
        }),
        hyperv: HyperVEnlightenments::default(),
        clocks: ClockTimers::default(),
        disk: StorageDisk {
            path: PathBuf::from("/var/lib/libvirt/images/win11-amd.qcow2"),
            size_gb: 200,
            format: "qcow2".to_string(),
            io_engine: "io_uring".to_string(),
            cache: "none".to_string(),
            discard_unmap: true,
            queues: 8,
        },
        gpu: GpuMode::LookingGlass { shm_size_mb: 128 },
        shared_directory: Some(PathBuf::from("/home/user/workspace")),
        unattended: UnattendedConfig {
            enabled: true,
            admin_username: "gamer".to_string(),
            admin_password: None,
            bypass_msa: true,
            install_qga: true,
            driver_disk_path: Some(PathBuf::from(
                "/var/lib/libvirt/images/win11-amd_oemdrv.img",
            )),
        },
        target: OutputTarget::Libvirt,
        win11_iso_path: PathBuf::from("/iso/win11.iso"),
        swtpm_path: Some(PathBuf::from("/usr/bin/swtpm")),
        ovmf_code: Some(PathBuf::from("/usr/share/edk2/ovmf/OVMF_CODE.secboot.fd")),
        ovmf_vars: Some(PathBuf::from("/usr/share/edk2/ovmf/OVMF_VARS.secboot.fd")),
    };

    let amd_xml = LibvirtXmlSynthesizer::synthesize(&amd_config, true).expect("AMD XML synth");
    let amd_xml_path = temp_dir.path().join("win11_amd.xml");
    std::fs::write(&amd_xml_path, &amd_xml).expect("write xml");

    // Execute virt-xml-validate on generated XML
    let val_output = Command::new("virt-xml-validate")
        .arg(&amd_xml_path)
        .output()
        .expect("virt-xml-validate execution failed");

    assert!(
        val_output.status.success(),
        "AMD XML failed virt-xml-validate:\nSTDOUT: {}\nSTDERR: {}",
        String::from_utf8_lossy(&val_output.stdout),
        String::from_utf8_lossy(&val_output.stderr)
    );

    // 2. Intel 14900K VFIO Passthrough Profile
    let intel_config = VmConfig {
        name: "win11-intel-14900k-vfio".to_string(),
        memory_mb: 65536,
        use_hugepages: false,
        hugepage_size_kb: None,
        sockets: 1,
        cores_per_socket: 8,
        threads_per_core: 2,
        pinning: Some(CpuPinningMap {
            vcpu_pins: (0..16).map(|v| (v, v)).collect(),
            emulator_pins: vec![16, 17, 18, 19],
            iothread_pins: vec![20, 21],
        }),
        hyperv: HyperVEnlightenments::default(),
        clocks: ClockTimers::default(),
        disk: StorageDisk {
            path: PathBuf::from("/var/lib/libvirt/images/win11-intel.qcow2"),
            size_gb: 500,
            format: "qcow2".to_string(),
            io_engine: "io_uring".to_string(),
            cache: "none".to_string(),
            discard_unmap: true,
            queues: 16,
        },
        gpu: GpuMode::VfioPassthrough {
            pci_address: "0000:01:00.0".to_string(),
        },
        shared_directory: None,
        unattended: UnattendedConfig {
            enabled: true,
            admin_username: "pro_user".to_string(),
            admin_password: Some("EnterprisePass99!".to_string()),
            bypass_msa: true,
            install_qga: true,
            driver_disk_path: Some(PathBuf::from("/var/lib/libvirt/images/oemdrv.img")),
        },
        target: OutputTarget::Both,
        win11_iso_path: PathBuf::from("/iso/win11.iso"),
        swtpm_path: Some(PathBuf::from("/usr/bin/swtpm")),
        ovmf_code: Some(PathBuf::from("/usr/share/edk2/ovmf/OVMF_CODE.secboot.fd")),
        ovmf_vars: Some(PathBuf::from("/usr/share/edk2/ovmf/OVMF_VARS.secboot.fd")),
    };

    let intel_xml =
        LibvirtXmlSynthesizer::synthesize(&intel_config, false).expect("Intel XML synth");
    let intel_xml_path = temp_dir.path().join("win11_intel.xml");
    std::fs::write(&intel_xml_path, &intel_xml).expect("write xml");

    let val_intel = Command::new("virt-xml-validate")
        .arg(&intel_xml_path)
        .output()
        .expect("virt-xml-validate execution failed");

    assert!(
        val_intel.status.success(),
        "Intel XML failed virt-xml-validate:\nSTDOUT: {}\nSTDERR: {}",
        String::from_utf8_lossy(&val_intel.stdout),
        String::from_utf8_lossy(&val_intel.stderr)
    );
}

#[test]
fn test_e2e_fat32_oemdrv_volume_unpack_and_unattended_validation() {
    let temp_dir = tempdir().expect("tempdir");
    let img_path = temp_dir.path().join("oemdrv_test.img");

    let unattended = UnattendedConfig {
        enabled: true,
        admin_username: "automated_admin".to_string(),
        admin_password: Some("StrongKey2026!".to_string()),
        bypass_msa: true,
        install_qga: true,
        driver_disk_path: None,
    };

    let dummy_driver = ("netkvm.sys".to_string(), vec![0x90; 1024]);
    FatDiskBuilder::build_to_file(&img_path, &unattended, &[dummy_driver])
        .expect("Building FAT32 image failed");

    assert!(img_path.exists());
    let metadata = std::fs::metadata(&img_path).expect("image metadata");
    assert_eq!(
        metadata.len(),
        32 * 1024 * 1024,
        "Image must be exactly 32MB"
    );

    // Open and parse the FAT32 volume
    let mut file = File::open(&img_path).expect("open FAT image");
    let fs = FileSystem::new(&mut file, fatfs::FsOptions::new()).expect("open FAT filesystem");
    let root = fs.root_dir();

    // Verify autounattend.xml inside FAT32 volume
    let mut xml_file = root
        .open_file("autounattend.xml")
        .expect("autounattend.xml must exist in FAT32 root");
    let mut xml_content = String::new();
    xml_file
        .read_to_string(&mut xml_content)
        .expect("read XML from FAT32");

    assert!(xml_content.contains("<Name>automated_admin</Name>"));
    assert!(xml_content.contains("<Value>StrongKey2026!</Value>"));
    assert!(xml_content.contains("BypassTPMCheck"));
    assert!(xml_content.contains("BypassSecureBootCheck"));
    assert!(xml_content.contains("qemu-ga-x86_64.msi"));

    // Verify extra driver file inside FAT32 volume
    let mut drv_file = root
        .open_file("netkvm.sys")
        .expect("netkvm.sys must exist in FAT32");
    let mut drv_data = Vec::new();
    drv_file
        .read_to_end(&mut drv_data)
        .expect("read driver from FAT32");
    assert_eq!(drv_data.len(), 1024);
    assert_eq!(drv_data[0], 0x90);
}
