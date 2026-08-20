//! CPU hardware topology, cache domains (L3/CCD), SMT thread pairing, and hybrid core parser.

use crate::error::{ClerestoryError, Result};
use crate::model::host::{CacheDomain, CorePair, CoreType, CpuTopology};
use crate::probe::sysfs::SysfsProvider;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

/// Hardware CPU topology and cache inspector.
pub struct CpuProbe<P: SysfsProvider> {
    sysfs: P,
}

impl<P: SysfsProvider> CpuProbe<P> {
    pub fn new(sysfs: P) -> Self {
        Self { sysfs }
    }

    /// Parses the complete host CPU topology.
    pub fn probe(&self) -> Result<CpuTopology> {
        let online_str = self
            .sysfs
            .read_to_string("sys/devices/system/cpu/online")
            .map_err(|e| ClerestoryError::ProbeError {
                path: PathBuf::from("sys/devices/system/cpu/online"),
                message: format!("Failed to read online CPUs: {}", e),
            })?;

        let online_cpus = parse_cpu_list(&online_str);
        if online_cpus.is_empty() {
            return Err(ClerestoryError::ProbeError {
                path: PathBuf::from("sys/devices/system/cpu/online"),
                message: "No online CPUs found".to_string(),
            });
        }

        let (vendor, model_name, flags) = self.parse_cpuinfo()?;

        let mut raw_cores: BTreeMap<(u32, u32), CoreBuilder> = BTreeMap::new(); // (socket_id, core_id) -> CoreBuilder
        let mut l3_domains: BTreeMap<u32, L3Builder> = BTreeMap::new(); // l3_id -> L3Builder
        let mut sockets = BTreeSet::new();

        for &cpu_id in &online_cpus {
            let top_prefix = format!("sys/devices/system/cpu/cpu{}/topology", cpu_id);
            let core_id: u32 = self
                .read_u32(&format!("{}/core_id", top_prefix))
                .unwrap_or(cpu_id);
            let socket_id: u32 = self
                .read_u32(&format!("{}/physical_package_id", top_prefix))
                .unwrap_or(0);
            sockets.insert(socket_id);

            let siblings_str = self
                .sysfs
                .read_to_string(&format!("{}/thread_siblings_list", top_prefix))
                .unwrap_or_else(|_| cpu_id.to_string());
            let siblings = parse_cpu_list(&siblings_str);

            let core_type_val = self.read_u32(&format!("{}/core_type", top_prefix));
            let max_freq_khz = self
                .read_u64(&format!(
                    "sys/devices/system/cpu/cpu{}/cpufreq/cpuinfo_max_freq",
                    cpu_id
                ))
                .unwrap_or(0);

            let core_type = match core_type_val {
                Some(0) => CoreType::Performance,
                Some(1) => CoreType::Efficient,
                _ => CoreType::Standard,
            };

            let core_key = (socket_id, core_id);
            let builder = raw_cores.entry(core_key).or_insert_with(|| CoreBuilder {
                thread_ids: BTreeSet::new(),
                core_type,
                max_freq_khz,
            });
            builder.thread_ids.extend(siblings);
            if max_freq_khz > builder.max_freq_khz {
                builder.max_freq_khz = max_freq_khz;
            }

            // L3 Cache Index 3
            let cache_prefix = format!("sys/devices/system/cpu/cpu{}/cache/index3", cpu_id);
            let l3_id = self.read_u32(&format!("{}/id", cache_prefix)).unwrap_or(0);
            let l3_size_bytes = self
                .read_size_bytes(&format!("{}/size", cache_prefix))
                .unwrap_or(33554432); // default 32MB

            let l3_builder = l3_domains.entry(l3_id).or_insert_with(|| L3Builder {
                l3_cache_id: l3_id,
                socket_id,
                cpu_ids: BTreeSet::new(),
                size_bytes: l3_size_bytes,
            });
            l3_builder.cpu_ids.insert(cpu_id);
            if l3_size_bytes > l3_builder.size_bytes {
                l3_builder.size_bytes = l3_size_bytes;
            }
        }

        let total_physical_cores = raw_cores.len() as u32;
        let total_threads = online_cpus.len() as u32;
        let has_smt = total_threads > total_physical_cores;

        let has_p = raw_cores
            .values()
            .any(|c| c.core_type == CoreType::Performance);
        let has_e = raw_cores
            .values()
            .any(|c| c.core_type == CoreType::Efficient);
        let is_hybrid = has_p && has_e;

        // Build CacheDomains
        let mut cache_domains = Vec::new();
        for (_, l3_b) in l3_domains {
            let mut domain_core_pairs = Vec::new();
            let mut visited_cores = BTreeSet::new();

            for &cpu_id in &l3_b.cpu_ids {
                for ((socket_id, core_id), builder) in &raw_cores {
                    if *socket_id == l3_b.socket_id
                        && builder.thread_ids.contains(&cpu_id)
                        && visited_cores.insert((*socket_id, *core_id))
                    {
                        domain_core_pairs.push(CorePair {
                            physical_core_id: *core_id,
                            socket_id: *socket_id,
                            thread_ids: builder.thread_ids.iter().copied().collect(),
                            core_type: builder.core_type,
                            max_freq_khz: builder.max_freq_khz,
                        });
                    }
                }
            }

            // 3D V-Cache check: typically >= 64MB or 96MB on Ryzen consumer dies
            let has_3d_vcache = l3_b.size_bytes >= (64 * 1024 * 1024);

            cache_domains.push(CacheDomain {
                l3_cache_id: l3_b.l3_cache_id,
                socket_id: l3_b.socket_id,
                cpu_ids: l3_b.cpu_ids.into_iter().collect(),
                core_pairs: domain_core_pairs,
                size_bytes: l3_b.size_bytes,
                has_3d_vcache,
            });
        }

        Ok(CpuTopology {
            model_name,
            vendor,
            sockets: sockets.len().max(1) as u32,
            total_physical_cores,
            total_threads,
            cache_domains,
            has_smt,
            is_hybrid,
            has_invtsc: flags.contains(&"invtsc".to_string())
                || flags.contains(&"constant_tsc".to_string()),
            has_svm_or_vmx: flags.contains(&"svm".to_string())
                || flags.contains(&"vmx".to_string()),
            has_topoext: flags.contains(&"topoext".to_string()),
        })
    }

    fn parse_cpuinfo(&self) -> Result<(String, String, Vec<String>)> {
        let content = self
            .sysfs
            .read_to_string("proc/cpuinfo")
            .unwrap_or_default();

        let mut vendor = "Unknown".to_string();
        let mut model_name = "Unknown Processor".to_string();
        let mut flags = Vec::new();

        for line in content.lines() {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                let key = parts[0].trim();
                let val = parts[1].trim();
                if key == "vendor_id" && vendor == "Unknown" {
                    vendor = val.to_string();
                } else if key == "model name" && model_name == "Unknown Processor" {
                    model_name = val.to_string();
                } else if (key == "flags" || key == "Features") && flags.is_empty() {
                    flags = val.split_whitespace().map(|s| s.to_string()).collect();
                }
            }
        }

        Ok((vendor, model_name, flags))
    }

    fn read_u32(&self, path: &str) -> Option<u32> {
        self.sysfs.read_to_string(path).ok()?.trim().parse().ok()
    }

    fn read_u64(&self, path: &str) -> Option<u64> {
        self.sysfs.read_to_string(path).ok()?.trim().parse().ok()
    }

    fn read_size_bytes(&self, path: &str) -> Option<u64> {
        let s = self.sysfs.read_to_string(path).ok()?;
        let trimmed = s.trim();
        if let Some(num) = trimmed.strip_suffix('K') {
            num.parse::<u64>().ok().map(|n| n * 1024)
        } else if let Some(num) = trimmed.strip_suffix('M') {
            num.parse::<u64>().ok().map(|n| n * 1024 * 1024)
        } else {
            trimmed.parse::<u64>().ok()
        }
    }
}

struct CoreBuilder {
    thread_ids: BTreeSet<u32>,
    core_type: CoreType,
    max_freq_khz: u64,
}

struct L3Builder {
    l3_cache_id: u32,
    socket_id: u32,
    cpu_ids: BTreeSet<u32>,
    size_bytes: u64,
}

/// Parses CPU range lists like "0-7,16-23" or "0,2,4,6" into Vec<u32>.
pub fn parse_cpu_list(list_str: &str) -> Vec<u32> {
    let mut cpus = BTreeSet::new();
    for part in list_str.trim().split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if part.contains('-') {
            let bounds: Vec<&str> = part.split('-').collect();
            if bounds.len() == 2 {
                if let (Ok(start), Ok(end)) = (bounds[0].parse::<u32>(), bounds[1].parse::<u32>()) {
                    for cpu in start..=end {
                        cpus.insert(cpu);
                    }
                }
            }
        } else if let Ok(cpu) = part.parse::<u32>() {
            cpus.insert(cpu);
        }
    }
    cpus.into_iter().collect()
}
