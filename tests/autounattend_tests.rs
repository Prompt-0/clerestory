use clerestory::model::vm::UnattendedConfig;
use clerestory::slipstream::autounattend::AutounattendGenerator;

#[test]
fn test_autounattend_xml_generation() {
    let config = UnattendedConfig {
        enabled: true,
        admin_username: "ritesh".to_string(),
        admin_password: Some("SecureP@ss123".to_string()),
        bypass_msa: true,
        install_qga: true,
        driver_disk_path: None,
    };

    let xml =
        AutounattendGenerator::generate(&config).expect("Answer file generation must succeed");

    // Assert Schema Namespaces
    assert!(xml.contains("xmlns=\"urn:schemas-microsoft-com:unattend\""));
    assert!(xml.contains("pass=\"windowsPE\""));
    assert!(xml.contains("pass=\"specialize\""));
    assert!(xml.contains("pass=\"oobeSystem\""));

    // Assert Driver Injections
    assert!(xml.contains("<Path>D:\\drivers</Path>"));
    assert!(xml.contains("<Path>E:\\drivers</Path>"));

    // Assert Disk Partitioning
    assert!(xml.contains("<DiskID>0</DiskID>"));
    assert!(xml.contains("<WillWipeDisk>true</WillWipeDisk>"));

    // Assert User & MSA Bypass
    assert!(xml.contains("<Name>ritesh</Name>"));
    assert!(xml.contains("<Value>SecureP@ss123</Value>"));
    assert!(xml.contains("<HideOnlineAccountScreens>true</HideOnlineAccountScreens>"));
    assert!(xml.contains("<ProtectYourPC>3</ProtectYourPC>"));

    // Assert First Logon Commands (Guest Agent)
    assert!(xml.contains("qemu-ga-x86_64.msi"));
}
