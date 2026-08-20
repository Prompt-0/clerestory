//! Declarative command-line argument parser definitions using Clap v4.

use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "clerestory",
    author = "Ritesh <developer@clerestory.dev>",
    version,
    about = "Hardware-Adaptive Windows 11 KVM Optimizer & Provisioning Engine",
    long_about = "Clerestory inspects Linux host silicon topology (L3 cache/CCD boundaries, SMT thread pairs, NUMA, hugepages) and synthesizes near-bare-metal Windows 11 KVM virtual machines with automated driver slipstreaming."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Suppress non-essential terminal output and progress spinners
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Increase logging verbosity (-v for INFO, -vv for DEBUG, -vvv for TRACE)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Disable ANSI color output
    #[arg(long, global = true)]
    pub no_color: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Interrogate host silicon hardware, kernel flags, and virtualization prerequisites
    Doctor(DoctorArgs),

    /// Provision a new hardware-optimized Windows 11 VM
    Create(CreateArgs),

    /// Inspect a provisioned VM configuration and topology mapping
    Inspect(InspectArgs),

    /// List all managed Windows 11 VMs and their runtime state
    List(ListArgs),

    /// Start a provisioned Windows 11 VM
    Start(StartArgs),

    /// Gracefully stop a running Windows 11 VM
    Stop(StopArgs),

    /// Undefine and clean up a Windows 11 VM
    Destroy(DestroyArgs),

    /// Generate shell auto-completion scripts
    Completions(CompletionsArgs),

    /// Generate ROFF man pages for distribution
    Man(ManArgs),
}

#[derive(Args, Debug)]
pub struct DoctorArgs {
    /// Output diagnostics in machine-readable JSON format
    #[arg(long)]
    pub json: bool,

    /// Storage path to inspect for io_uring and TRIM capabilities
    #[arg(short, long, default_value = "/var/lib/libvirt/images")]
    pub storage_path: PathBuf,

    /// Launch interactive TUI doctor visualizer
    #[arg(long)]
    pub tui: bool,
}

#[derive(Args, Debug)]
pub struct CreateArgs {
    /// Path to official Windows 11 installation ISO
    #[arg(short, long)]
    pub iso: Option<PathBuf>,

    /// Virtual machine domain identifier
    #[arg(short, long, default_value = "win11-apex")]
    pub name: String,

    /// Number of vCPUs to allocate (must be even for SMT; defaults to optimal single CCD/P-core count)
    #[arg(short, long)]
    pub cores: Option<u32>,

    /// Memory allocation in Megabytes (e.g. 16384 for 16GB)
    #[arg(short, long, default_value_t = 16384)]
    pub ram: u64,

    /// Primary virtual disk size in Gigabytes
    #[arg(short, long, default_value_t = 100)]
    pub disk: u64,

    /// Storage directory for virtual disk image
    #[arg(short = 'p', long, default_value = "/var/lib/libvirt/images")]
    pub disk_path: PathBuf,

    /// Optimization profile
    #[arg(long, value_enum, default_value_t = ProfileChoice::Auto)]
    pub profile: ProfileChoice,

    /// Graphics and display tier
    #[arg(short, long, value_enum, default_value_t = GpuChoice::Spice)]
    pub gpu_mode: GpuChoice,

    /// PCI address for dedicated GPU passthrough (e.g. 0000:01:00.0)
    #[arg(long)]
    pub vfio_pci: Option<String>,

    /// Looking Glass IVSHMEM buffer size in Megabytes
    #[arg(long, default_value_t = 64)]
    pub looking_glass_shm: u32,

    /// Host directory to share via VirtioFS with DAX cache
    #[arg(long)]
    pub share_dir: Option<PathBuf>,

    /// Enable zero-touch unattended setup
    #[arg(short, long, default_value_t = true)]
    pub unattended: bool,

    /// Offline Windows administrator username
    #[arg(long, default_value = "Admin")]
    pub username: String,

    /// Offline Windows administrator password (optional)
    #[arg(long)]
    pub password: Option<String>,

    /// Output target format
    #[arg(long, value_enum, default_value_t = TargetChoice::Libvirt)]
    pub target: TargetChoice,

    /// Print synthesized XML or QEMU script without creating disks or registering domain
    #[arg(long)]
    pub dry_run: bool,

    /// Output configuration in machine-readable JSON
    #[arg(long)]
    pub json: bool,

    /// Launch interactive Ratatui setup wizard
    #[arg(long)]
    pub tui: bool,
}

#[derive(Args, Debug)]
pub struct InspectArgs {
    /// Name of the VM to inspect
    pub name: String,

    /// Print synthesized Libvirt Domain XML
    #[arg(long)]
    pub xml: bool,

    /// Output in machine-readable JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Output in JSON format
    #[arg(long)]
    pub json: bool,
}

#[derive(Args, Debug)]
pub struct StartArgs {
    /// Name of the VM to start
    pub name: String,

    /// Launch viewer client automatically (spice, looking-glass, none)
    #[arg(long, default_value = "none")]
    pub viewer: String,
}

#[derive(Args, Debug)]
pub struct StopArgs {
    /// Name of the VM to stop
    pub name: String,

    /// Force immediate power-off instead of graceful ACPI shutdown
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Args, Debug)]
pub struct DestroyArgs {
    /// Name of the VM to destroy
    pub name: String,

    /// Also delete the underlying virtual disk images
    #[arg(long)]
    pub delete_disks: bool,
}

#[derive(Args, Debug)]
pub struct CompletionsArgs {
    /// Target shell for completion script
    #[arg(value_enum)]
    pub shell: clap_complete::Shell,
}

#[derive(Args, Debug)]
pub struct ManArgs {
    /// Directory to output generated ROFF man pages
    #[arg(short, long, default_value = "./docs/man")]
    pub output_dir: PathBuf,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfileChoice {
    Auto,
    Gaming,
    Compute,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuChoice {
    Spice,
    LookingGlass,
    Vfio,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetChoice {
    Libvirt,
    Standalone,
    Both,
}
