//! Standalone QEMU shell script synthesizer.

use crate::error::Result;
use crate::model::vm::VmConfig;

pub struct QemuScriptSynthesizer;

impl QemuScriptSynthesizer {
    /// Generates a self-contained portable `run.sh` script.
    pub fn synthesize(config: &VmConfig) -> Result<String> {
        let mut script = String::with_capacity(2048);
        script.push_str("#!/usr/bin/env bash\n");
        script.push_str("# Auto-generated standalone Windows 11 launcher by Clerestory\n");
        script.push_str("set -euo pipefail\n\n");

        script.push_str("qemu-system-x86_64 \\\n");
        script.push_str(&format!("  -name '{}' \\\n", config.name));
        script.push_str("  -enable-kvm \\\n");
        script.push_str("  -machine q35,smm=on,accel=kvm \\\n");
        script.push_str(&format!("  -m {}M \\\n", config.memory_mb));
        script.push_str(&format!(
            "  -smp {},sockets={},cores={},threads={} \\\n",
            config.sockets * config.cores_per_socket * config.threads_per_core,
            config.sockets,
            config.cores_per_socket,
            config.threads_per_core
        ));

        // CPU & Hyper-V
        script.push_str("  -cpu host,migratable=off,+invtsc,+tsc-deadline,hv_relaxed,hv_vapic,hv_spinlocks=0x1fff,hv_vpindex,hv_runtime,hv_synic,hv_stimer,hv_stimer_direct,hv_frequencies,hv_tlbflush,hv_ipi,hv_reenlightenment \\\n");

        // Disks
        script.push_str(&format!(
            "  -device virtio-scsi-pci,id=scsi0,iothread=iothread0,num_queues={} \\\n",
            config.disk.queues
        ));
        script.push_str("  -object iothread,id=iothread0 \\\n");
        script.push_str(&format!(
            "  -drive file='{}',if=none,id=drive0,format={},cache={},aio={} \\\n",
            config.disk.path.display(),
            config.disk.format,
            config.disk.cache,
            config.disk.io_engine
        ));
        script.push_str(
            "  -device scsi-hd,drive=drive0,bus=scsi0.0,id=hd0,discard_granularity=512 \\\n",
        );

        if let Some(drv_path) = &config.unattended.driver_disk_path {
            script.push_str(&format!(
                "  -drive file='{}',if=none,id=oemdrv,format=raw,media=cdrom,readonly=on \\\n",
                drv_path.display()
            ));
            script.push_str("  -device scsi-cd,drive=oemdrv,bus=scsi0.0,id=cd0 \\\n");
        }

        script.push_str(&format!(
            "  -drive file='{}',if=none,id=winiso,format=raw,media=cdrom,readonly=on \\\n",
            config.win11_iso_path.display()
        ));
        script.push_str("  -device scsi-cd,drive=winiso,bus=scsi0.0,id=cd1 \\\n");

        // Network
        script.push_str("  -netdev user,id=net0 -device virtio-net-pci,netdev=net0 \\\n");

        // Display
        script.push_str("  -vga qxl -spice port=5900,disable-ticketing=on \\\n");
        script.push_str("  -device virtio-serial-pci -device virtserialport,chardev=qga0,name=org.qemu.guest_agent.0 \\\n");
        script.push_str("  -chardev socket,path=/tmp/qga.sock,server=on,wait=off,id=qga0\n");

        Ok(script)
    }
}
