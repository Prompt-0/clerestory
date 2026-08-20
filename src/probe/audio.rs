//! Audio server, PipeWire, and latency quantum inspection.

use crate::error::Result;
use crate::model::host::AudioCapability;
use crate::probe::sysfs::SysfsProvider;

pub struct AudioProbe<P: SysfsProvider> {
    _sysfs: P,
}

impl<P: SysfsProvider> AudioProbe<P> {
    pub fn new(sysfs: P) -> Self {
        Self { _sysfs: sysfs }
    }

    /// Probes host audio environment.
    pub fn probe_audio(&self) -> Result<AudioCapability> {
        let is_pipewire = std::env::var("PIPEWIRE_RUNTIME_DIR").is_ok()
            || std::path::Path::new("/run/user/1000/pipewire-0").exists()
            || std::path::Path::new("/run/user/0/pipewire-0").exists();

        let is_pulseaudio = std::env::var("PULSE_SERVER").is_ok()
            || std::path::Path::new("/run/user/1000/pulse/native").exists();

        Ok(AudioCapability {
            is_pipewire,
            is_pulseaudio: is_pulseaudio && !is_pipewire,
            sample_rate: 48000,
            quantum_latency_ms: 10.66, // 512 samples @ 48kHz
        })
    }
}
