use clerestory::model::host::CoreType;
use clerestory::probe::cpu::CpuProbe;
use clerestory::probe::sysfs::MockSysfs;
use std::path::PathBuf;

#[test]
fn test_amd_7950x3d_topology_probe() {
    let fixture_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sysfs_amd_7950x3d");
    let mock = MockSysfs::new(fixture_path);
    let probe = CpuProbe::new(mock);

    let topology = probe.probe().expect("AMD probe must succeed");

    assert_eq!(topology.vendor, "AuthenticAMD");
    assert_eq!(topology.model_name, "AMD Ryzen 9 7950X3D 16-Core Processor");
    assert_eq!(topology.sockets, 1);
    assert_eq!(topology.total_physical_cores, 16);
    assert_eq!(topology.total_threads, 32);
    assert!(topology.has_smt);
    assert!(topology.has_invtsc);
    assert!(topology.has_svm_or_vmx);
    assert!(topology.has_topoext);

    // Verify 2 CCDs
    assert_eq!(topology.cache_domains.len(), 2);

    // CCD0 should have 3D V-Cache (96MB)
    let ccd0 = &topology.cache_domains[0];
    assert_eq!(ccd0.l3_cache_id, 0);
    assert!(ccd0.has_3d_vcache);
    assert_eq!(ccd0.core_pairs.len(), 8);
    assert_eq!(ccd0.cpu_ids.len(), 16);

    // CCD1 should have standard 32MB cache
    let ccd1 = &topology.cache_domains[1];
    assert_eq!(ccd1.l3_cache_id, 1);
    assert!(!ccd1.has_3d_vcache);
    assert_eq!(ccd1.core_pairs.len(), 8);
    assert_eq!(ccd1.cpu_ids.len(), 16);
}

#[test]
fn test_intel_14900k_hybrid_topology_probe() {
    let fixture_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sysfs_intel_14900k");
    let mock = MockSysfs::new(fixture_path);
    let probe = CpuProbe::new(mock);

    let topology = probe.probe().expect("Intel probe must succeed");

    assert_eq!(topology.vendor, "GenuineIntel");
    assert_eq!(topology.model_name, "Intel(R) Core(TM) i9-14900K");
    assert_eq!(topology.total_physical_cores, 24); // 8P + 16E
    assert_eq!(topology.total_threads, 32); // 16T + 16T
    assert!(topology.is_hybrid);

    let p_cores: Vec<_> = topology
        .cache_domains
        .iter()
        .flat_map(|d| &d.core_pairs)
        .filter(|p| p.core_type == CoreType::Performance)
        .collect();

    let e_cores: Vec<_> = topology
        .cache_domains
        .iter()
        .flat_map(|d| &d.core_pairs)
        .filter(|p| p.core_type == CoreType::Efficient)
        .collect();

    assert_eq!(p_cores.len(), 8);
    assert_eq!(e_cores.len(), 16);
}
