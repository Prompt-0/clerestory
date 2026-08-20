//! NUMA nodes and Hugepages memory inspection.

use crate::error::Result;
use crate::model::host::{HugepageTier, NumaNode};
use crate::probe::cpu::parse_cpu_list;
use crate::probe::sysfs::SysfsProvider;
use std::path::PathBuf;

pub struct MemoryProbe<P: SysfsProvider> {
    sysfs: P,
}

impl<P: SysfsProvider> MemoryProbe<P> {
    pub fn new(sysfs: P) -> Self {
        Self { sysfs }
    }

    /// Probes all NUMA memory nodes.
    pub fn probe_numa(&self) -> Result<Vec<NumaNode>> {
        let mut nodes = Vec::new();
        let node_dirs = self
            .sysfs
            .read_dir("sys/devices/system/node")
            .unwrap_or_default();

        for path in node_dirs {
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            if let Some(num_str) = filename.strip_prefix("node") {
                if let Ok(node_id) = num_str.parse::<u32>() {
                    let rel_prefix = format!("sys/devices/system/node/{}", filename);
                    let cpulist_str = self
                        .sysfs
                        .read_to_string(&format!("{}/cpulist", rel_prefix))
                        .unwrap_or_default();
                    let cpu_ids = parse_cpu_list(&cpulist_str);

                    let (total, free) = self.parse_node_meminfo(&format!("{}/meminfo", rel_prefix));

                    nodes.push(NumaNode {
                        node_id,
                        cpu_ids,
                        total_memory_bytes: total,
                        free_memory_bytes: free,
                    });
                }
            }
        }

        if nodes.is_empty() {
            // Fallback for single node systems without full sysfs node entries
            let (total, free) = self.parse_global_meminfo();
            nodes.push(NumaNode {
                node_id: 0,
                cpu_ids: Vec::new(),
                total_memory_bytes: total,
                free_memory_bytes: free,
            });
        }

        nodes.sort_by_key(|n| n.node_id);
        Ok(nodes)
    }

    /// Probes hugepages tiers (1GB and 2MB).
    pub fn probe_hugepages(&self) -> Result<Vec<HugepageTier>> {
        let mut tiers = Vec::new();
        let huge_dirs = self
            .sysfs
            .read_dir("sys/kernel/mm/hugepages")
            .unwrap_or_default();

        for path in huge_dirs {
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            if let Some(size_str) = filename.strip_prefix("hugepages-") {
                if let Some(num_str) = size_str.strip_suffix("kB") {
                    if let Ok(page_size_kb) = num_str.parse::<u64>() {
                        let rel_prefix = format!("sys/kernel/mm/hugepages/{}", filename);
                        let total_pages = self
                            .read_u64(&format!("{}/nr_hugepages", rel_prefix))
                            .unwrap_or(0);
                        let free_pages = self
                            .read_u64(&format!("{}/free_hugepages", rel_prefix))
                            .unwrap_or(0);

                        let mount_point = if self.sysfs.exists("dev/hugepages") {
                            Some(PathBuf::from("/dev/hugepages"))
                        } else {
                            None
                        };

                        tiers.push(HugepageTier {
                            page_size_kb,
                            total_pages,
                            free_pages,
                            mount_point,
                        });
                    }
                }
            }
        }

        tiers.sort_by_key(|t| std::cmp::Reverse(t.page_size_kb));
        Ok(tiers)
    }

    fn parse_node_meminfo(&self, path: &str) -> (u64, u64) {
        let content = self.sysfs.read_to_string(path).unwrap_or_default();
        let mut total = 0;
        let mut free = 0;

        for line in content.lines() {
            if line.contains("MemTotal:") {
                total = parse_kb_line(line);
            } else if line.contains("MemFree:") {
                free = parse_kb_line(line);
            }
        }

        (total, free)
    }

    fn parse_global_meminfo(&self) -> (u64, u64) {
        let content = self
            .sysfs
            .read_to_string("proc/meminfo")
            .unwrap_or_default();
        let mut total = 0;
        let mut free = 0;

        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                total = parse_kb_line(line);
            } else if (line.starts_with("MemAvailable:") || line.starts_with("MemFree:"))
                && free == 0
            {
                free = parse_kb_line(line);
            }
        }

        (total, free)
    }

    fn read_u64(&self, path: &str) -> Option<u64> {
        self.sysfs.read_to_string(path).ok()?.trim().parse().ok()
    }
}

fn parse_kb_line(line: &str) -> u64 {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        if let Ok(kb) = parts[parts.len() - 2].parse::<u64>() {
            return kb * 1024;
        }
    }
    0
}
