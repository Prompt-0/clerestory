//! CPU Topology-Aware Core Allocation and SMT Pinning Engine.

use crate::error::{ClerestoryError, Result};
use crate::model::host::{CoreType, CpuTopology};
use crate::model::vm::{CpuPinningMap, OptimizationProfile};
use std::collections::BTreeSet;

pub struct CpuPinningOptimizer<'a> {
    topology: &'a CpuTopology,
}

impl<'a> CpuPinningOptimizer<'a> {
    pub fn new(topology: &'a CpuTopology) -> Self {
        Self { topology }
    }

    /// Calculates optimal 1:1 vCPU pinning, emulatorpin, and iothreadpin.
    pub fn optimize(
        &self,
        requested_vcpus: u32,
        profile: OptimizationProfile,
        isolate_host: bool,
    ) -> Result<CpuPinningMap> {
        if requested_vcpus == 0 {
            return Err(ClerestoryError::ValidationError(
                "Requested vCPUs must be at least 1".to_string(),
            ));
        }

        // If hybrid (Intel 12th/13th/14th/15th Gen)
        if self.topology.is_hybrid {
            return self.optimize_hybrid(requested_vcpus, isolate_host);
        }

        // Standard or AMD multi-CCD architecture
        self.optimize_ccd(requested_vcpus, profile, isolate_host)
    }

    fn optimize_hybrid(&self, requested_vcpus: u32, isolate_host: bool) -> Result<CpuPinningMap> {
        // Collect all P-core pairs
        let mut p_cores = Vec::new();
        let mut e_cores = Vec::new();

        for domain in &self.topology.cache_domains {
            for pair in &domain.core_pairs {
                if pair.core_type == CoreType::Performance {
                    p_cores.push(pair.clone());
                } else {
                    e_cores.push(pair.clone());
                }
            }
        }

        let total_p_threads: usize = p_cores.iter().map(|p| p.thread_ids.len()).sum();
        if (requested_vcpus as usize) > total_p_threads {
            return Err(ClerestoryError::TopologyCapacityError {
                requested: requested_vcpus,
                available: total_p_threads as u32,
            });
        }

        let mut vcpu_pins = Vec::new();
        let mut allocated_host_cpus = BTreeSet::new();
        let mut current_vcpu = 0;

        for pair in &p_cores {
            for &tid in &pair.thread_ids {
                if current_vcpu < requested_vcpus {
                    vcpu_pins.push((current_vcpu, tid));
                    allocated_host_cpus.insert(tid);
                    current_vcpu += 1;
                }
            }
        }

        // Assign emulator & iothreads to E-cores if available, else remaining P-cores
        let mut helper_cpus = Vec::new();
        for e_pair in &e_cores {
            for &tid in &e_pair.thread_ids {
                helper_cpus.push(tid);
            }
        }

        if helper_cpus.is_empty() && isolate_host {
            // Find unallocated threads
            for i in 0..self.topology.total_threads {
                if !allocated_host_cpus.contains(&i) {
                    helper_cpus.push(i);
                }
            }
        }

        let emulator_pins = if helper_cpus.is_empty() {
            vec![0]
        } else {
            vec![helper_cpus[0]]
        };

        let iothread_pins = if helper_cpus.len() > 1 {
            vec![helper_cpus[1]]
        } else {
            emulator_pins.clone()
        };

        Ok(CpuPinningMap {
            vcpu_pins,
            emulator_pins,
            iothread_pins,
        })
    }

    fn optimize_ccd(
        &self,
        requested_vcpus: u32,
        profile: OptimizationProfile,
        isolate_host: bool,
    ) -> Result<CpuPinningMap> {
        let mut candidate_domains = self.topology.cache_domains.clone();

        // Sort domains based on optimization profile
        match profile {
            OptimizationProfile::Gaming => {
                candidate_domains.sort_by_key(|d| !d.has_3d_vcache); // 3D V-Cache first
            }
            OptimizationProfile::Compute => {
                candidate_domains.sort_by_key(|d| d.has_3d_vcache); // standard high-freq first
            }
            OptimizationProfile::Auto => {
                // Pick domain with greatest size
                candidate_domains.sort_by_key(|d| std::cmp::Reverse(d.size_bytes));
            }
        }

        // Find primary domain that can fit requested vCPUs
        let primary_domain = candidate_domains
            .iter()
            .find(|d| {
                let domain_threads: usize = d.core_pairs.iter().map(|p| p.thread_ids.len()).sum();
                (requested_vcpus as usize) <= domain_threads
            })
            .or_else(|| candidate_domains.first())
            .ok_or(ClerestoryError::TopologyCapacityError {
                requested: requested_vcpus,
                available: 0,
            })?;

        let mut vcpu_pins = Vec::new();
        let mut allocated_host_cpus = BTreeSet::new();
        let mut current_vcpu = 0;

        for pair in &primary_domain.core_pairs {
            for &tid in &pair.thread_ids {
                if current_vcpu < requested_vcpus {
                    vcpu_pins.push((current_vcpu, tid));
                    allocated_host_cpus.insert(tid);
                    current_vcpu += 1;
                }
            }
        }

        // If requested vCPUs spilled beyond single domain
        if current_vcpu < requested_vcpus {
            for other_domain in &candidate_domains {
                if other_domain.l3_cache_id == primary_domain.l3_cache_id {
                    continue;
                }
                for pair in &other_domain.core_pairs {
                    for &tid in &pair.thread_ids {
                        if current_vcpu < requested_vcpus && !allocated_host_cpus.contains(&tid) {
                            vcpu_pins.push((current_vcpu, tid));
                            allocated_host_cpus.insert(tid);
                            current_vcpu += 1;
                        }
                    }
                }
            }
        }

        // Select host isolation cores from remaining unallocated CPUs
        let mut unallocated_cpus = Vec::new();
        for domain in &candidate_domains {
            if domain.l3_cache_id != primary_domain.l3_cache_id {
                for &cpu in &domain.cpu_ids {
                    if !allocated_host_cpus.contains(&cpu) {
                        unallocated_cpus.push(cpu);
                    }
                }
            }
        }

        if unallocated_cpus.is_empty() && isolate_host {
            for i in 0..self.topology.total_threads {
                if !allocated_host_cpus.contains(&i) {
                    unallocated_cpus.push(i);
                }
            }
        }

        let emulator_pins = if unallocated_cpus.is_empty() {
            vec![0]
        } else {
            vec![unallocated_cpus[0]]
        };

        let iothread_pins = if unallocated_cpus.len() > 1 {
            vec![unallocated_cpus[1]]
        } else {
            emulator_pins.clone()
        };

        Ok(CpuPinningMap {
            vcpu_pins,
            emulator_pins,
            iothread_pins,
        })
    }
}
