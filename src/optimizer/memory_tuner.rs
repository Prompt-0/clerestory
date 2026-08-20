//! Memory allocation and hugepage alignment optimization.

use crate::model::host::HugepageTier;

pub struct MemoryTuner;

impl MemoryTuner {
    /// Determines whether hugepages should be activated and the optimal page size.
    pub fn select_hugepages(requested_mb: u64, tiers: &[HugepageTier]) -> (bool, Option<u64>) {
        for tier in tiers {
            let available_mb = (tier.free_pages * tier.page_size_kb) / 1024;
            if available_mb >= requested_mb && tier.free_pages > 0 {
                return (true, Some(tier.page_size_kb));
            }
        }
        (false, None)
    }
}
