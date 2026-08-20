//! VirtIO-SCSI queue calculation and asynchronous I/O engine selection.

use crate::model::host::StorageCapability;

pub struct IoTuner;

impl IoTuner {
    /// Selects optimal I/O engine ('io_uring' vs 'native') and queue count.
    pub fn tune(vcpus: u32, storage: &StorageCapability) -> (String, u32) {
        let engine = if storage.supports_io_uring {
            "io_uring".to_string()
        } else {
            "native".to_string()
        };

        let queues = vcpus.max(1);
        (engine, queues)
    }
}
