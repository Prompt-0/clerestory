use clerestory::model::vm::OptimizationProfile;
use clerestory::optimizer::cpu_pinning::CpuPinningOptimizer;
use clerestory::probe::cpu::CpuProbe;
use clerestory::probe::sysfs::MockSysfs;
use std::path::PathBuf;

#[test]
fn test_amd_7950x3d_single_ccd_gaming_pinning() {
    let fixture_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sysfs_amd_7950x3d");
    let mock = MockSysfs::new(fixture_path);
    let probe = CpuProbe::new(mock);
    let topology = probe.probe().expect("AMD probe must succeed");

    let optimizer = CpuPinningOptimizer::new(&topology);
    let pinning = optimizer
        .optimize(8, OptimizationProfile::Gaming, true)
        .expect("Pinning 8 vCPUs must succeed");

    assert_eq!(pinning.vcpu_pins.len(), 8);

    // All 8 vCPUs must reside strictly inside CCD0 (CPUs 0-7 or 16-23)
    let ccd0_cpus: Vec<u32> = (0..8).chain(16..24).collect();
    for (vcpu, host_cpu) in &pinning.vcpu_pins {
        assert!(
            ccd0_cpus.contains(host_cpu),
            "vCPU {} mapped to host CPU {} outside CCD0!",
            vcpu,
            host_cpu
        );
    }

    // Emulator and IOThread pins must be outside CCD0 (in CCD1) for host isolation
    assert!(!pinning.emulator_pins.is_empty());
    assert!(!pinning.iothread_pins.is_empty());
    for &cpu in &pinning.emulator_pins {
        assert!((8..32).contains(&cpu) && !(16..24).contains(&cpu));
    }
}

#[test]
fn test_intel_14900k_p_core_pinning() {
    let fixture_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sysfs_intel_14900k");
    let mock = MockSysfs::new(fixture_path);
    let probe = CpuProbe::new(mock);
    let topology = probe.probe().expect("Intel probe must succeed");

    let optimizer = CpuPinningOptimizer::new(&topology);
    let pinning = optimizer
        .optimize(8, OptimizationProfile::Auto, true)
        .expect("Pinning 8 vCPUs must succeed");

    assert_eq!(pinning.vcpu_pins.len(), 8);

    // P-Cores are CPUs 0..16
    for (_vcpu, host_cpu) in &pinning.vcpu_pins {
        assert!(*host_cpu < 16, "vCPU assigned to E-Core {}!", host_cpu);
    }

    // Emulator / IOThread pins should be offloaded to E-cores (CPUs 16..32)
    for &cpu in &pinning.emulator_pins {
        assert!(cpu >= 16, "Emulator pin {} should be on E-core!", cpu);
    }
}
