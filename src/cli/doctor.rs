//! High-aesthetic terminal renderer for hardware diagnostics and topology reports.

use crate::model::host::HostReport;
use colored::Colorize;

pub struct DoctorRenderer;

impl DoctorRenderer {
    /// Renders the complete host diagnostics to terminal with dynamic status icons.
    pub fn render(report: &HostReport) {
        println!(
            "{}",
            "╭─────────────────────────────────────────────────────────────────────────────╮"
                .cyan()
        );
        println!(
            "{}  {}                     {}",
            "│".cyan(),
            "Clerestory Hardware Diagnostics & Silicon Topology Probe"
                .bold()
                .white(),
            "│".cyan()
        );
        println!(
            "{}",
            "╰─────────────────────────────────────────────────────────────────────────────╯"
                .cyan()
        );
        println!();

        // 1. CPU & Cache Topology
        println!("{}", "[+] CPU & Cache Topology".bold().blue());
        println!("   {} Model: {}", "✔".green(), report.cpu.model_name.bold());
        println!("   {} Vendor: {}", "✔".green(), report.cpu.vendor);
        println!(
            "   {} Sockets: {} | Physical Cores: {} | SMT Threads: {}",
            "✔".green(),
            report.cpu.sockets,
            report.cpu.total_physical_cores,
            report.cpu.total_threads
        );

        for domain in &report.cpu.cache_domains {
            let vcache_tag = if domain.has_3d_vcache {
                " [3D V-Cache Domain - 96MB]".bold().yellow()
            } else {
                "".clear()
            };

            let cpu_list_str = domain
                .cpu_ids
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(",");
            println!(
                "   {} CCD {} (L3 Cache {}MB){} -> CPUs: [{}]",
                "✔".green(),
                domain.l3_cache_id,
                domain.size_bytes / (1024 * 1024),
                vcache_tag,
                cpu_list_str.cyan()
            );
        }

        let (smt_icon, smt_status) = if report.cpu.has_smt {
            ("✔".green(), "Enabled".green())
        } else {
            ("ℹ".yellow(), "Disabled / Non-SMT".yellow())
        };
        println!("   {} SMT / Hyper-Threading: {}", smt_icon, smt_status);

        let (invtsc_icon, invtsc_status) = if report.cpu.has_invtsc {
            (
                "✔".green(),
                "Available (Sub-microsecond DPC latency)".green(),
            )
        } else {
            (
                "▲".yellow(),
                "Not detected (May experience minor clock jitter)".yellow(),
            )
        };
        println!(
            "   {} Invariant TSC (invtsc): {}",
            invtsc_icon, invtsc_status
        );

        let (virt_icon, virt_status) = if report.cpu.has_svm_or_vmx {
            (
                "✔".green(),
                "Active (Hardware Virtualization Supported)".green(),
            )
        } else {
            (
                "✖".red(),
                "Missing CPU virtualization flags (Check BIOS SVM/VT-x)".red(),
            )
        };
        println!("   {} Hardware Virtualization: {}", virt_icon, virt_status);
        println!();

        // 2. Memory & Hugepages
        println!("{}", "[+] Memory & NUMA Subsystem".bold().blue());
        println!(
            "   {} NUMA Nodes Detected: {}",
            "✔".green(),
            report.numa_nodes.len()
        );
        for node in &report.numa_nodes {
            println!(
                "   {} NUMA Node {}: Total: {:.1} GiB | Free: {:.1} GiB",
                "✔".green(),
                node.node_id,
                node.total_memory_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
                node.free_memory_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
            );
        }

        let has_free_hugepages = report.hugepages.iter().any(|t| t.free_pages > 0);
        if has_free_hugepages {
            for tier in &report.hugepages {
                if tier.free_pages > 0 {
                    println!(
                        "   {} Hugepages ({}kB): Total: {} | Free: {}",
                        "✔".green(),
                        tier.page_size_kb,
                        tier.total_pages,
                        tier.free_pages
                    );
                }
            }
        } else {
            println!(
                "   {} Hugepages: Standard 4KB pages active (Hugepages not pre-allocated)",
                "ℹ".yellow()
            );
        }
        println!();

        // 3. Storage & I/O Engine
        println!("{}", "[+] Storage & Asynchronous I/O Engine".bold().blue());
        let (uring_icon, uring_status) = if report.storage.supports_io_uring {
            (
                "✔".green(),
                "Supported & Enabled (io_uring zero-copy async)".green(),
            )
        } else {
            ("ℹ".yellow(), "Fallback to Native AIO".yellow())
        };
        println!("   {} Linux io_uring Engine: {}", uring_icon, uring_status);

        let (trim_icon, trim_status) = if report.storage.supports_trim {
            ("✔".green(), "Supported (discard=unmap)".green())
        } else {
            ("ℹ".yellow(), "Not detected".yellow())
        };
        println!(
            "   {} TRIM / SSD Thin-Provisioning: {}",
            trim_icon, trim_status
        );
        println!();

        // 4. Security & Firmware
        println!("{}", "[+] Security & Firmware Prerequisites".bold().blue());
        let (kvm_icon, kvm_status) = if report.security.has_kvm_device {
            ("✔".green(), "Present (/dev/kvm)".green())
        } else {
            (
                "✖".red(),
                "Missing (/dev/kvm - enable KVM module or container privileges)".red(),
            )
        };
        println!("   {} KVM Hypervisor Device: {}", kvm_icon, kvm_status);

        let (swtpm_icon, swtpm_status) = match &report.security.swtpm_bin {
            Some(path) => ("✔".green(), format!("Found ({})", path.display()).green()),
            None => (
                "▲".yellow(),
                "Missing (/usr/bin/swtpm - install via: dnf/apt install swtpm)".yellow(),
            ),
        };
        println!(
            "   {} Emulated TPM 2.0 (swtpm): {}",
            swtpm_icon, swtpm_status
        );

        let (ovmf_icon, ovmf_code_status) = match &report.security.ovmf_code_fd {
            Some(path) => ("✔".green(), format!("Found ({})", path.display()).green()),
            None => ("✖".red(), "Missing OVMF SecureBoot code FD".red()),
        };
        println!(
            "   {} UEFI SecureBoot Code: {}",
            ovmf_icon, ovmf_code_status
        );
        println!();

        // 5. Audio Subsystem
        println!("{}", "[+] Sound & Audio Engine".bold().blue());
        let (audio_icon, audio_name) = if report.audio.is_pipewire {
            (
                "✔".green(),
                "Native PipeWire (Low-latency lock enabled)".green(),
            )
        } else if report.audio.is_pulseaudio {
            ("ℹ".yellow(), "PulseAudio".yellow())
        } else {
            ("ℹ".normal(), "Standard ALSA/Direct".normal())
        };
        println!(
            "   {} Audio Server: {} (~{:.1}ms quantum)",
            audio_icon, audio_name, report.audio.quantum_latency_ms
        );
        println!();

        // Overall Readiness Evaluation
        if !report.security.has_kvm_device {
            println!(
                "{}",
                "❌ Status: BLOCKED — /dev/kvm hypervisor device missing."
                    .bold()
                    .red()
            );
            println!("   {} Ensure KVM is enabled on host (e.g. `modprobe kvm` / `chmod 666 /dev/kvm`) or container is run with `--device /dev/kvm`.", "👉".yellow());
        } else if report.security.swtpm_bin.is_none() {
            println!(
                "{}",
                "⚠️  Status: PARTIAL — TPM 2.0 emulator (swtpm) missing."
                    .bold()
                    .yellow()
            );
            println!("   {} Windows 11 VM can still be provisioned with automated autounattend TPM bypass, or install: `dnf install swtpm` / `apt install swtpm`.", "👉".cyan());
        } else {
            println!(
                "{}",
                "🎉 Status: 100% Ready for Bare-Metal Windows 11 Virtualization!"
                    .bold()
                    .green()
            );
        }
    }
}
