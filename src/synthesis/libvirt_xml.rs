//! Production Libvirt Domain XML synthesis for Windows 11 on KVM.

use crate::error::Result;
use crate::model::vm::{GpuMode, VmConfig};

pub struct LibvirtXmlSynthesizer;

impl LibvirtXmlSynthesizer {
    /// Compiles a fully tuned Libvirt Domain XML.
    pub fn synthesize(config: &VmConfig, is_amd: bool) -> Result<String> {
        let mut xml = String::with_capacity(4096);

        let total_vcpus = if let Some(pinning) = &config.pinning {
            pinning.vcpu_pins.len() as u32
        } else {
            config.sockets * config.cores_per_socket * config.threads_per_core
        };

        xml.push_str("<domain type='kvm'>\n");
        xml.push_str(&format!("  <name>{}</name>\n", config.name));
        xml.push_str(&format!(
            "  <memory unit='MiB'>{}</memory>\n",
            config.memory_mb
        ));
        xml.push_str(&format!(
            "  <currentMemory unit='MiB'>{}</currentMemory>\n",
            config.memory_mb
        ));
        xml.push_str(&format!(
            "  <vcpu placement='static'>{}</vcpu>\n",
            total_vcpus
        ));

        // cputune & iothreads
        xml.push_str("  <iothreads>1</iothreads>\n");
        if let Some(pinning) = &config.pinning {
            xml.push_str("  <cputune>\n");
            for (vcpu, host_cpu) in &pinning.vcpu_pins {
                xml.push_str(&format!(
                    "    <vcpupin vcpu='{}' cpuset='{}'/>\n",
                    vcpu, host_cpu
                ));
            }
            if !pinning.emulator_pins.is_empty() {
                let emu_str = pinning
                    .emulator_pins
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                xml.push_str(&format!("    <emulatorpin cpuset='{}'/>\n", emu_str));
            }
            if !pinning.iothread_pins.is_empty() {
                let io_str = pinning
                    .iothread_pins
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                xml.push_str(&format!(
                    "    <iothreadpin iothread='1' cpuset='{}'/>\n",
                    io_str
                ));
            }
            xml.push_str("  </cputune>\n");
        }

        // Shared memory backing for VirtioFS / Hugepages
        xml.push_str("  <memoryBacking>\n");
        if config.use_hugepages {
            if let Some(sz) = config.hugepage_size_kb {
                xml.push_str(&format!(
                    "    <hugepages><page size='{}' unit='KiB'/></hugepages>\n",
                    sz
                ));
            } else {
                xml.push_str("    <hugepages/>\n");
            }
        }
        xml.push_str("    <source type='memfd'/>\n");
        xml.push_str("    <access mode='shared'/>\n");
        xml.push_str("  </memoryBacking>\n");

        // OS firmware & SecureBoot
        xml.push_str("  <os firmware='efi'>\n");
        xml.push_str("    <type arch='x86_64' machine='q35'>hvm</type>\n");
        xml.push_str("    <firmware>\n");
        xml.push_str("      <feature enabled='yes' name='secure-boot'/>\n");
        xml.push_str("      <feature enabled='yes' name='enrolled-keys'/>\n");
        xml.push_str("    </firmware>\n");
        if let Some(code) = &config.ovmf_code {
            xml.push_str(&format!(
                "    <loader readonly='yes' type='pflash'>{}</loader>\n",
                code.display()
            ));
        }
        if let Some(vars) = &config.ovmf_vars {
            let vm_vars = format!("/var/lib/libvirt/qemu/nvram/{}_VARS.fd", config.name);
            xml.push_str(&format!(
                "    <nvram template='{}'>{}</nvram>\n",
                vars.display(),
                vm_vars
            ));
        }
        xml.push_str("    <boot dev='hd'/>\n");
        xml.push_str("    <boot dev='cdrom'/>\n");
        xml.push_str("  </os>\n");

        // Features & Hyper-V Enlightenments
        xml.push_str("  <features>\n");
        xml.push_str("    <acpi/>\n");
        xml.push_str("    <apic/>\n");
        xml.push_str("    <hyperv mode='custom'>\n");
        if config.hyperv.relaxed {
            xml.push_str("      <relaxed state='on'/>\n");
        }
        if config.hyperv.vapic {
            xml.push_str("      <vapic state='on'/>\n");
        }
        xml.push_str(&format!(
            "      <spinlocks state='on' retries='{}'/>\n",
            config.hyperv.spinlocks_retries
        ));
        if config.hyperv.vpindex {
            xml.push_str("      <vpindex state='on'/>\n");
        }
        if config.hyperv.runtime {
            xml.push_str("      <runtime state='on'/>\n");
        }
        if config.hyperv.synic {
            xml.push_str("      <synic state='on'/>\n");
        }
        if config.hyperv.stimer_direct {
            xml.push_str(
                "      <stimer state='on'>\n        <direct state='on'/>\n      </stimer>\n",
            );
        } else {
            xml.push_str("      <stimer state='on'/>\n");
        }
        if config.hyperv.reset {
            xml.push_str("      <reset state='on'/>\n");
        }
        if config.hyperv.frequencies {
            xml.push_str("      <frequencies state='on'/>\n");
        }
        if config.hyperv.reenlightenment {
            xml.push_str("      <reenlightenment state='on'/>\n");
        }
        if config.hyperv.tlbflush {
            xml.push_str("      <tlbflush state='on'/>\n");
        }
        if config.hyperv.ipi {
            xml.push_str("      <ipi state='on'/>\n");
        }
        if config.hyperv.evmcs {
            xml.push_str("      <evmcs state='on'/>\n");
        }
        if config.hyperv.avic {
            xml.push_str("      <avic state='on'/>\n");
        }
        xml.push_str("    </hyperv>\n");
        xml.push_str("    <kvm>\n      <hidden state='on'/>\n    </kvm>\n");
        xml.push_str("    <ioapic driver='kvm'/>\n");
        xml.push_str("  </features>\n");

        // CPU Model & Features
        xml.push_str("  <cpu mode='host-passthrough' check='none' migratable='off'>\n");
        xml.push_str(&format!(
            "    <topology sockets='{}' dies='1' cores='{}' threads='{}'/>\n",
            config.sockets, config.cores_per_socket, config.threads_per_core
        ));
        xml.push_str("    <feature policy='require' name='invtsc'/>\n");
        xml.push_str("    <feature policy='require' name='tsc-deadline'/>\n");
        if is_amd {
            xml.push_str("    <feature policy='require' name='topoext'/>\n");
        }
        xml.push_str("  </cpu>\n");

        // Clock & Invariant Timers
        xml.push_str("  <clock offset='localtime'>\n");
        if config.clocks.rtc_catchup {
            xml.push_str("    <timer name='rtc' tickpolicy='catchup'/>\n");
        }
        if config.clocks.pit_discard {
            xml.push_str("    <timer name='pit' tickpolicy='discard'/>\n");
        }
        if config.clocks.hpet_disabled {
            xml.push_str("    <timer name='hpet' present='no'/>\n");
        }
        if config.clocks.kvmclock_disabled {
            xml.push_str("    <timer name='kvmclock' present='no'/>\n");
        }
        if config.clocks.hypervclock_enabled {
            xml.push_str("    <timer name='hypervclock' present='yes'/>\n");
        }
        if config.clocks.tsc_native {
            xml.push_str("    <timer name='tsc' present='yes' mode='native'/>\n");
        }
        xml.push_str("  </clock>\n");

        // Devices
        xml.push_str("  <devices>\n");
        xml.push_str("    <emulator>/usr/bin/qemu-system-x86_64</emulator>\n");

        // VirtIO-SCSI Controller
        xml.push_str(&format!(
            "    <controller type='scsi' model='virtio-scsi' index='0'>\n      <driver iothread='1' queues='{}'/>\n    </controller>\n",
            config.disk.queues
        ));

        // Primary Storage Disk
        let discard_str = if config.disk.discard_unmap {
            "discard='unmap' detect_zeroes='unmap'"
        } else {
            ""
        };
        xml.push_str(&format!(
            "    <disk type='file' device='disk'>\n      <driver name='qemu' type='{}' cache='{}' io='{}' {}/>\n      <source file='{}'/>\n      <target dev='sda' bus='scsi'/>\n      <address type='drive' controller='0' bus='0' target='0' unit='0'/>\n    </disk>\n",
            config.disk.format, config.disk.cache, config.disk.io_engine, discard_str, config.disk.path.display()
        ));

        // Unattended Driver Disk / OEMDRV
        if let Some(drv_path) = &config.unattended.driver_disk_path {
            xml.push_str(&format!(
                "    <disk type='file' device='cdrom'>\n      <driver name='qemu' type='raw'/>\n      <source file='{}'/>\n      <target dev='sdb' bus='scsi'/>\n      <readonly/>\n      <address type='drive' controller='0' bus='0' target='0' unit='1'/>\n    </disk>\n",
                drv_path.display()
            ));
        }

        // Windows 11 ISO
        xml.push_str(&format!(
            "    <disk type='file' device='cdrom'>\n      <driver name='qemu' type='raw'/>\n      <source file='{}'/>\n      <target dev='sdc' bus='scsi'/>\n      <readonly/>\n      <address type='drive' controller='0' bus='0' target='0' unit='2'/>\n    </disk>\n",
            config.win11_iso_path.display()
        ));

        // TPM 2.0 CRB
        xml.push_str("    <tpm model='tpm-crb'>\n      <backend type='emulator' version='2.0'/>\n    </tpm>\n");

        // Network (VirtIO)
        xml.push_str("    <interface type='network'>\n      <source network='default'/>\n      <model type='virtio'/>\n    </interface>\n");

        // Sound (PipeWire / Intel HDA ich9)
        xml.push_str("    <sound model='ich9'>\n      <audio id='1'/>\n    </sound>\n");
        xml.push_str("    <audio id='1' type='pipewire'/>\n");

        // VirtioFS Shared Directory
        if let Some(shared_dir) = &config.shared_directory {
            xml.push_str(&format!(
                "    <filesystem type='mount' accessmode='passthrough'>\n      <driver type='virtiofs' queue='1024'/>\n      <binary path='/usr/libexec/virtiofsd' xattr='on'/>\n      <source dir='{}'/>\n      <target dir='clerestory_share'/>\n    </filesystem>\n",
                shared_dir.display()
            ));
        }

        // Graphics / Looking Glass
        match &config.gpu {
            GpuMode::SpiceVirtio => {
                xml.push_str("    <graphics type='spice' autoport='yes'>\n      <listen type='address'/>\n      <image compression='off'/>\n      <gl enable='no'/>\n    </graphics>\n");
                xml.push_str("    <video>\n      <model type='qxl' ram='65536' vram='65536' vgamem='16384' heads='1' primary='yes'/>\n    </video>\n");
            }
            GpuMode::LookingGlass { shm_size_mb } => {
                xml.push_str(&format!(
                    "    <shmem name='looking-glass'>\n      <model type='ivshmem-plain'/>\n      <size unit='M'>{}</size>\n    </shmem>\n",
                    shm_size_mb
                ));
                xml.push_str("    <graphics type='spice' autoport='yes'>\n      <listen type='address'/>\n    </graphics>\n");
                xml.push_str("    <video>\n      <model type='qxl' ram='65536' vram='65536' heads='1' primary='yes'/>\n    </video>\n");
            }
            GpuMode::VfioPassthrough { pci_address } => {
                let parts: Vec<&str> = pci_address.split(':').collect();
                if parts.len() >= 2 {
                    let domain = parts[0];
                    let bus = parts[1];
                    let slot_func: Vec<&str> = parts.get(2).unwrap_or(&"00.0").split('.').collect();
                    let slot = slot_func[0];
                    let function = slot_func.get(1).unwrap_or(&"0");

                    xml.push_str(&format!(
                        "    <hostdev mode='subsystem' type='pci' managed='yes'>\n      <source>\n        <address domain='0x{}' bus='0x{}' slot='0x{}' function='0x{}'/>\n      </source>\n    </hostdev>\n",
                        domain, bus, slot, function
                    ));
                }
            }
        }

        // QEMU Guest Agent channel
        xml.push_str("    <channel type='unix'>\n      <target type='virtio' name='org.qemu.guest_agent.0'/>\n    </channel>\n");

        xml.push_str("  </devices>\n");
        xml.push_str("</domain>\n");

        Ok(xml)
    }
}
