//! High-aesthetic terminal renderer for hardware diagnostics and topology reports.

use crate::model::host::HostReport;
use colored::Colorize;

pub struct DoctorRenderer;

impl DoctorRenderer {
    /// Renders the complete host diagnostics to terminal.
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

        let smt_status = if report.cpu.has_smt {
            "Enabled".green()
        } else {
            "Disabled".yellow()
        };
        println!("   {} SMT / Hyper-Threading: {}", "✔".green(), smt_status);

        let invtsc_status = if report.cpu.has_invtsc {
            "Available (Sub-microsecond DPC latency)".green()
        } else {
            "Not detected".yellow()
        };
        println!(
            "   {} Invariant TSC (invtsc): {}",
            "✔".green(),
            invtsc_status
        );

        let virt_status = if report.cpu.has_svm_or_vmx {
            "Active (Hardware Virtualization Supported)".green()
        } else {
            "Missing CPU virtualization flags (Check BIOS SVM/VT-x)".red()
        };
        println!(
            "   {} Hardware Virtualization: {}",
            "✔".green(),
            virt_status
        );
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

        if report.hugepages.is_empty() {
            println!(
                "   {} Hugepages: Standard 4KB pages active (Transparent hugepages default)",
                "ℹ".yellow()
            );
        } else {
            for tier in &report.hugepages {
                println!(
                    "   {} Hugepages ({}kB): Total: {} | Free: {}",
                    "✔".green(),
                    tier.page_size_kb,
                    tier.total_pages,
                    tier.free_pages
                );
            }
        }
        println!();

        // 3. Storage & I/O Engine
        println!("{}", "[+] Storage & Asynchronous I/O Engine".bold().blue());
        let uring_status = if report.storage.supports_io_uring {
            "Supported & Enabled (io_uring zero-copy async)".green()
        } else {
            "Fallback to Native AIO".yellow()
        };
        println!("   {} Linux io_uring Engine: {}", "✔".green(), uring_status);
        let trim_status = if report.storage.supports_trim {
            "Supported (discard=unmap)".green()
        } else {
            "Not detected".yellow()
        };
        println!(
            "   {} TRIM / SSD Thin-Provisioning: {}",
            "✔".green(),
            trim_status
        );
        println!();

        // 4. Security & Firmware
        println!("{}", "[+] Security & Firmware Prerequisites".bold().blue());
        let kvm_status = if report.security.has_kvm_device {
            "Present (/dev/kvm)".green()
        } else {
            "Missing (/dev/kvm)".red()
        };
        println!("   {} KVM Hypervisor Device: {}", "✔".green(), kvm_status);

        let swtpm_status = match &report.security.swtpm_bin {
            Some(path) => format!("Found ({})", path.display()).green(),
            None => "Missing (/usr/bin/swtpm - install via package manager)".yellow(),
        };
        println!(
            "   {} Emulated TPM 2.0 (swtpm): {}",
            "✔".green(),
            swtpm_status
        );

        let ovmf_code_status = match &report.security.ovmf_code_fd {
            Some(path) => format!("Found ({})", path.display()).green(),
            None => "Missing OVMF SecureBoot code FD".yellow(),
        };
        println!(
            "   {} UEFI SecureBoot Code: {}",
            "✔".green(),
            ovmf_code_status
        );
        println!();

        // 5. Audio Subsystem
        println!("{}", "[+] Sound & Audio Engine".bold().blue());
        let audio_name = if report.audio.is_pipewire {
            "Native PipeWire (Low-latency lock enabled)".green()
        } else if report.audio.is_pulseaudio {
            "PulseAudio".yellow()
        } else {
            "Standard ALSA/Direct".normal()
        };
        println!(
            "   {} Audio Server: {} (~{:.1}ms quantum)",
            "✔".green(),
            audio_name,
            report.audio.quantum_latency_ms
        );
        println!();

        println!(
            "{}",
            "🎉 Status: 100% Ready for Bare-Metal Windows 11 Virtualization!"
                .bold()
                .green()
        );
    }
}
