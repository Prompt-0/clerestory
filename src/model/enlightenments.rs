//! Hyper-V enlightenments and sub-microsecond timing definitions.

use serde::{Deserialize, Serialize};

/// Hyper-V performance enlightenments matrix for Windows NT kernel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HyperVEnlightenments {
    pub relaxed: bool,
    pub vapic: bool,
    pub spinlocks_retries: u32,
    pub vpindex: bool,
    pub runtime: bool,
    pub synic: bool,
    pub stimer_direct: bool,
    pub reset: bool,
    pub frequencies: bool,
    pub reenlightenment: bool,
    pub tlbflush: bool,
    pub ipi: bool,
    pub evmcs: bool,
    pub avic: bool,
}

impl Default for HyperVEnlightenments {
    fn default() -> Self {
        Self {
            relaxed: true,
            vapic: true,
            spinlocks_retries: 8191,
            vpindex: true,
            runtime: true,
            synic: true,
            stimer_direct: true,
            reset: true,
            frequencies: true,
            reenlightenment: true,
            tlbflush: true,
            ipi: true,
            evmcs: false,
            avic: false,
        }
    }
}

/// Real-time clock and timer invariants configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClockTimers {
    pub rtc_catchup: bool,
    pub pit_discard: bool,
    pub hpet_disabled: bool,
    pub hypervclock_enabled: bool,
    pub tsc_native: bool,
    pub kvmclock_disabled: bool,
}

impl Default for ClockTimers {
    fn default() -> Self {
        Self {
            rtc_catchup: true,
            pit_discard: true,
            hpet_disabled: true,
            hypervclock_enabled: true,
            tsc_native: true,
            kvmclock_disabled: true,
        }
    }
}
