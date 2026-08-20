//! Strongly-typed domain errors for Clerestory.

use crate::exit_codes::*;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClerestoryError {
    #[error("Hardware prerequisite missing: {name} (Hint: {install_hint})")]
    MissingPrerequisite { name: String, install_hint: String },

    #[error("CPU topology error: requested {requested} vCPUs, but single cache domain only provides {available}")]
    TopologyCapacityError { requested: u32, available: u32 },

    #[error("Hardware probe failure at {path}: {message}")]
    ProbeError { path: PathBuf, message: String },

    #[error("I/O error at {path}: {source}")]
    IoError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("XML synthesis error: {0}")]
    XmlSynthesisError(String),

    #[error("Driver slipstream error: {0}")]
    SlipstreamError(String),

    #[error("Libvirt command execution error: {0}")]
    LibvirtError(String),

    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),
}

impl ClerestoryError {
    /// Maps a domain error to its standard POSIX exit code.
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::MissingPrerequisite { .. } => EX_UNAVAILABLE,
            Self::TopologyCapacityError { .. } => EX_DATAERR,
            Self::ProbeError { .. } => EX_SOFTWARE,
            Self::IoError { .. } => EX_IOERR,
            Self::ValidationError(_) => EX_USAGE,
            Self::XmlSynthesisError(_) => EX_SOFTWARE,
            Self::SlipstreamError(_) => EX_DATAERR,
            Self::LibvirtError(_) => EX_SOFTWARE,
            Self::UnsupportedPlatform(_) => EX_UNAVAILABLE,
        }
    }
}

pub type Result<T> = std::result::Result<T, ClerestoryError>;
