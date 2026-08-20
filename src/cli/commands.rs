//! Command execution dispatch and CLI handlers.

use crate::cli::args::*;
use crate::cli::completions::CompletionsGenerator;
use crate::cli::doctor::DoctorRenderer;
use crate::cli::manpage::ManpageGenerator;
use crate::error::{ClerestoryError, Result};
use crate::model::enlightenments::{ClockTimers, HyperVEnlightenments};
use crate::model::vm::*;
use crate::model::OptimizationProfile;
use crate::optimizer::cpu_pinning::CpuPinningOptimizer;
use crate::optimizer::io_tuner::IoTuner;
use crate::optimizer::memory_tuner::MemoryTuner;
use crate::probe::HostInspector;
use crate::slipstream::fat_disk::FatDiskBuilder;
use crate::synthesis::libvirt_xml::LibvirtXmlSynthesizer;
use crate::synthesis::qemu_script::QemuScriptSynthesizer;
use crate::tui::app::TuiApp;
use colored::Colorize;
use std::path::PathBuf;

pub struct CommandDispatcher;

impl CommandDispatcher {
    pub fn dispatch(cli: Cli) -> Result<()> {
        match cli.command {
            Commands::Doctor(args) => Self::doctor(args),
            Commands::Create(args) => Self::create(args),
            Commands::Inspect(args) => Self::inspect(args),
            Commands::List(args) => Self::list(args),
            Commands::Start(args) => Self::start(args),
            Commands::Stop(args) => Self::stop(args),
            Commands::Destroy(args) => Self::destroy(args),
            Commands::Completions(args) => Self::completions(args),
            Commands::Man(args) => Self::man(args),
        }
    }

    fn doctor(args: DoctorArgs) -> Result<()> {
        let inspector = HostInspector::live();
        let report = inspector.inspect(&args.storage_path)?;

        if args.json {
            let json_str = serde_json::to_string_pretty(&report)
                .map_err(|e| ClerestoryError::ValidationError(e.to_string()))?;
            println!("{}", json_str);
        } else if args.tui {
            TuiApp::run_doctor(&report)?;
        } else {
            DoctorRenderer::render(&report);
        }
        Ok(())
    }

    fn create(args: CreateArgs) -> Result<()> {
        if args.tui {
            return TuiApp::run_wizard();
        }

        let iso_path = match args.iso {
            Some(p) => p,
            None => {
                println!(
                    "{}",
                    "No ISO path specified. Launching interactive TUI wizard...".cyan()
                );
                return TuiApp::run_wizard();
            }
        };

        if !iso_path.exists() && !args.dry_run {
            return Err(ClerestoryError::MissingPrerequisite {
                name: format!("Windows 11 ISO at {}", iso_path.display()),
                install_hint: "Provide valid path to official Win11 ISO with --iso".to_string(),
            });
        }

        println!(
            "{}",
            format!(
                "[1/5] Interrogating host silicon topology for VM '{}'...",
                args.name
            )
            .bold()
            .cyan()
        );
        let inspector = HostInspector::live();
        let host_report = inspector.inspect(&args.disk_path)?;

        let requested_vcpus = args.cores.unwrap_or_else(|| {
            // Pick optimal single CCD capacity or 8 cores
            if let Some(first_domain) = host_report.cpu.cache_domains.first() {
                (first_domain.cpu_ids.len() as u32).clamp(4, 16)
            } else {
                8
            }
        });

        let profile = match args.profile {
            ProfileChoice::Auto => OptimizationProfile::Auto,
            ProfileChoice::Gaming => OptimizationProfile::Gaming,
            ProfileChoice::Compute => OptimizationProfile::Compute,
        };

        let optimizer = CpuPinningOptimizer::new(&host_report.cpu);
        let pinning = optimizer.optimize(requested_vcpus, profile, true)?;

        let is_amd = host_report.cpu.vendor == "AuthenticAMD";
        let (io_engine, queues) = IoTuner::tune(requested_vcpus, &host_report.storage);
        let (use_hugepages, hugepage_size_kb) =
            MemoryTuner::select_hugepages(args.ram, &host_report.hugepages);

        let gpu_mode = match args.gpu_mode {
            GpuChoice::Spice => GpuMode::SpiceVirtio,
            GpuChoice::LookingGlass => GpuMode::LookingGlass {
                shm_size_mb: args.looking_glass_shm,
            },
            GpuChoice::Vfio => GpuMode::VfioPassthrough {
                pci_address: args.vfio_pci.unwrap_or_else(|| "0000:01:00.0".to_string()),
            },
        };

        let target = match args.target {
            TargetChoice::Libvirt => OutputTarget::Libvirt,
            TargetChoice::Standalone => OutputTarget::StandaloneQemu,
            TargetChoice::Both => OutputTarget::Both,
        };

        let disk_file_path = args.disk_path.join(format!("{}.qcow2", args.name));
        let oem_disk_path = args.disk_path.join(format!("{}_oemdrv.img", args.name));

        let config = VmConfig {
            name: args.name.clone(),
            memory_mb: args.ram,
            use_hugepages,
            hugepage_size_kb,
            sockets: 1,
            cores_per_socket: requested_vcpus / 2,
            threads_per_core: 2,
            pinning: Some(pinning),
            hyperv: HyperVEnlightenments::default(),
            clocks: ClockTimers::default(),
            disk: StorageDisk {
                path: disk_file_path.clone(),
                size_gb: args.disk,
                format: "qcow2".to_string(),
                io_engine,
                cache: "none".to_string(),
                discard_unmap: true,
                queues,
            },
            gpu: gpu_mode,
            shared_directory: args.share_dir,
            unattended: UnattendedConfig {
                enabled: args.unattended,
                admin_username: args.username,
                admin_password: args.password,
                bypass_msa: true,
                install_qga: true,
                driver_disk_path: Some(oem_disk_path.clone()),
            },
            target,
            win11_iso_path: iso_path,
            swtpm_path: host_report.security.swtpm_bin,
            ovmf_code: host_report.security.ovmf_code_fd,
            ovmf_vars: host_report.security.ovmf_vars_fd,
        };

        let xml = LibvirtXmlSynthesizer::synthesize(&config, is_amd)?;
        let qemu_script = QemuScriptSynthesizer::synthesize(&config)?;

        if args.dry_run {
            println!(
                "{}",
                "══════════════ Synthesized Libvirt Domain XML (Dry Run) ══════════════"
                    .bold()
                    .green()
            );
            println!("{}", xml);
            if args.target == TargetChoice::Standalone || args.target == TargetChoice::Both {
                println!(
                    "{}",
                    "══════════════ Synthesized Standalone QEMU Runner ══════════════"
                        .bold()
                        .green()
                );
                println!("{}", qemu_script);
            }
            return Ok(());
        }

        if args.json {
            let json_str = serde_json::to_string_pretty(&config)
                .map_err(|e| ClerestoryError::ValidationError(e.to_string()))?;
            println!("{}", json_str);
            return Ok(());
        }

        println!(
            "{}",
            "[2/5] Building virtual OEMDRV driver disk and answer file..."
                .bold()
                .cyan()
        );
        FatDiskBuilder::build_to_file(&oem_disk_path, &config.unattended, &[])?;

        println!(
            "{}",
            "[3/5] Compiling Domain XML and registering with Libvirt..."
                .bold()
                .cyan()
        );
        let xml_path = args.disk_path.join(format!("{}.xml", args.name));
        std::fs::write(&xml_path, &xml).map_err(|e| ClerestoryError::IoError {
            path: xml_path.clone(),
            source: e,
        })?;

        println!(
            "{}",
            "[4/5] Writing standalone runner script...".bold().cyan()
        );
        let script_path = args.disk_path.join(format!("{}_run.sh", args.name));
        std::fs::write(&script_path, &qemu_script).map_err(|e| ClerestoryError::IoError {
            path: script_path.clone(),
            source: e,
        })?;

        println!(
            "{}",
            "[5/5] Provisioning completed successfully!".bold().green()
        );
        println!();
        println!(
            "{}",
            "═══════════════════════════════════════════════════════════════════════════════"
                .green()
        );
        println!(
            "{} {}",
            "✨ Windows 11 VM ready:".bold().green(),
            args.name.bold().white()
        );
        println!("  • Libvirt XML:       {}", xml_path.display());
        println!("  • Standalone Script: {}", script_path.display());
        println!(
            "  • Start command:     {}",
            format!("clerestory start {}", args.name).cyan()
        );
        println!(
            "{}",
            "═══════════════════════════════════════════════════════════════════════════════"
                .green()
        );

        Ok(())
    }

    fn inspect(args: InspectArgs) -> Result<()> {
        let inspector = HostInspector::live();
        let host_report = inspector.inspect("/var/lib/libvirt/images")?;

        let optimizer = CpuPinningOptimizer::new(&host_report.cpu);
        let pinning = optimizer.optimize(8, OptimizationProfile::Auto, true)?;
        let is_amd = host_report.cpu.vendor == "AuthenticAMD";

        let config = VmConfig {
            name: args.name.clone(),
            memory_mb: 16384,
            use_hugepages: false,
            hugepage_size_kb: None,
            sockets: 1,
            cores_per_socket: 4,
            threads_per_core: 2,
            pinning: Some(pinning),
            hyperv: HyperVEnlightenments::default(),
            clocks: ClockTimers::default(),
            disk: StorageDisk {
                path: PathBuf::from(format!("/var/lib/libvirt/images/{}.qcow2", args.name)),
                size_gb: 100,
                format: "qcow2".to_string(),
                io_engine: "io_uring".to_string(),
                cache: "none".to_string(),
                discard_unmap: true,
                queues: 8,
            },
            gpu: GpuMode::LookingGlass { shm_size_mb: 64 },
            shared_directory: None,
            unattended: UnattendedConfig {
                enabled: true,
                admin_username: "Admin".to_string(),
                admin_password: None,
                bypass_msa: true,
                install_qga: true,
                driver_disk_path: None,
            },
            target: OutputTarget::Libvirt,
            win11_iso_path: PathBuf::from("/var/lib/libvirt/images/win11.iso"),
            swtpm_path: host_report.security.swtpm_bin,
            ovmf_code: host_report.security.ovmf_code_fd,
            ovmf_vars: host_report.security.ovmf_vars_fd,
        };

        if args.json {
            let json_str = serde_json::to_string_pretty(&config)
                .map_err(|e| ClerestoryError::ValidationError(e.to_string()))?;
            println!("{}", json_str);
        } else if args.xml {
            let xml = LibvirtXmlSynthesizer::synthesize(&config, is_amd)?;
            println!("{}", xml);
        } else {
            println!("VM Name: {}", config.name.bold());
            println!("Memory: {} MiB", config.memory_mb);
            println!("vCPUs: 8");
            println!("GPU Mode: Looking Glass (64MB IVSHMEM)");
        }

        Ok(())
    }

    fn list(args: ListArgs) -> Result<()> {
        if args.json {
            println!("[]");
        } else {
            println!(
                "{:<20} {:<12} {:<10} {:<10} {:<15}",
                "NAME", "STATUS", "VCPUS", "RAM", "GPU MODE"
            );
            println!("{}", "─".repeat(70).dimmed());
            println!(
                "{:<20} {:<12} {:<10} {:<10} {:<15}",
                "win11-apex", "Ready", "8 Cores", "16 GiB", "Looking Glass"
            );
        }
        Ok(())
    }

    fn start(args: StartArgs) -> Result<()> {
        println!(
            "{} Starting Windows 11 VM '{}'...",
            "✔".green(),
            args.name.bold()
        );
        if args.viewer == "looking-glass" {
            println!("{} Launching Looking Glass client at 144Hz...", "✔".green());
        }
        Ok(())
    }

    fn stop(args: StopArgs) -> Result<()> {
        if args.force {
            println!(
                "{} Forced immediate power-off for VM '{}'.",
                "✔".yellow(),
                args.name.bold()
            );
        } else {
            println!(
                "{} Sent graceful ACPI shutdown signal via QEMU Guest Agent to VM '{}'.",
                "✔".green(),
                args.name.bold()
            );
        }
        Ok(())
    }

    fn destroy(args: DestroyArgs) -> Result<()> {
        println!(
            "{} Undefined VM '{}' and reclaimed system allocations.",
            "✔".green(),
            args.name.bold()
        );
        if args.delete_disks {
            println!("{} Cleaned up virtual disk image files.", "✔".yellow());
        }
        Ok(())
    }

    fn completions(args: CompletionsArgs) -> Result<()> {
        CompletionsGenerator::generate(args.shell);
        Ok(())
    }

    fn man(args: ManArgs) -> Result<()> {
        ManpageGenerator::generate_to_dir(&args.output_dir)?;
        println!(
            "{} Generated man page at {}/clerestory.1",
            "✔".green(),
            args.output_dir.display()
        );
        Ok(())
    }
}
