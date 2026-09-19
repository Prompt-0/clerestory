# Agent Guidelines for Clerestory

Clerestory is a hardware-adaptive Windows 11 KVM optimizer and provisioning engine written in Rust.

---

## 🛠️ Verification & Test Commands
All changes must pass compile checks and the automated test suite:
- **Run Unit & Topology Tests**: `cargo test --all-targets`
- **Compiler & Clippy Check**: `cargo clippy --all-targets -- -D warnings`
- **Release Build**: `cargo build --release`

---

## 🏛️ Core Architecture
- `src/probe/`: Silicon topology discovery (AMD CCD/L3 cache pools, 3D V-Cache asymmetry, Intel Hybrid P/E-core detection).
- `src/optimizer/`: Core pinning logic, Hyper-V enlightenment matrices, VirtIO-SCSI queue sizing.
- `src/xml/`: Libvirt domain XML generator.
- `src/slipstream/`: In-memory FAT32 OEMDRV volume synthesis and `autounattend.xml` answer-file generator.
- `src/cli/`: Clap v4 CLI commands, sysexits status codes, and JSON serialization.

---

## 📋 Engineering Standards & Guardrails
1. **Safety & Robustness**: Do not use `unwrap()` or `expect()` in library code (`src/lib.rs` modules); propagate errors cleanly using `anyhow` or `thiserror`.
2. **Deterministic Output**: XML and unattend answer-file synthesis must be reproducible and adhere to strict Libvirt schemas.
3. **Hardware Fallbacks**: Never assume a specific CPU brand or virtualization feature exists; always handle fallback topologies gracefully.
