//! In-memory and file-backed virtual FAT32 OEMDRV disk image builder.

use crate::error::{ClerestoryError, Result};
use crate::model::vm::UnattendedConfig;
use crate::slipstream::autounattend::AutounattendGenerator;
use fatfs::{FileSystem, FormatVolumeOptions};
use std::fs::File;
use std::io::{Cursor, Write};
use std::path::Path;

pub struct FatDiskBuilder;

impl FatDiskBuilder {
    /// Builds a FAT32 OEMDRV image containing autounattend.xml and driver payloads.
    pub fn build_to_file<P: AsRef<Path>>(
        output_path: P,
        unattended: &UnattendedConfig,
        driver_files: &[(String, Vec<u8>)],
    ) -> Result<()> {
        let size_bytes: usize = 32 * 1024 * 1024; // 32MB virtual volume
        let mut buffer = vec![0u8; size_bytes];

        {
            let mut cursor = Cursor::new(&mut buffer[..]);
            let format_options = FormatVolumeOptions::new().volume_label(*b"OEMDRV     ");

            fatfs::format_volume(&mut cursor, format_options).map_err(|e| {
                ClerestoryError::SlipstreamError(format!("FAT format error: {}", e))
            })?;
        }

        {
            let mut cursor = Cursor::new(&mut buffer[..]);
            let fs = FileSystem::new(&mut cursor, fatfs::FsOptions::new())
                .map_err(|e| ClerestoryError::SlipstreamError(format!("FAT open error: {}", e)))?;

            let root_dir = fs.root_dir();

            // 1. Write autounattend.xml
            let autounattend_xml = AutounattendGenerator::generate(unattended)?;
            let mut xml_file = root_dir.create_file("autounattend.xml").map_err(|e| {
                ClerestoryError::SlipstreamError(format!(
                    "Failed to create autounattend.xml: {}",
                    e
                ))
            })?;
            xml_file
                .write_all(autounattend_xml.as_bytes())
                .map_err(|e| {
                    ClerestoryError::SlipstreamError(format!(
                        "Failed to write autounattend.xml: {}",
                        e
                    ))
                })?;

            // 2. Write extra driver/script files
            for (rel_path, data) in driver_files {
                let clean_name = rel_path.trim_start_matches('/');
                let mut f = root_dir.create_file(clean_name).map_err(|e| {
                    ClerestoryError::SlipstreamError(format!(
                        "Failed to create {}: {}",
                        clean_name, e
                    ))
                })?;
                f.write_all(data).map_err(|e| {
                    ClerestoryError::SlipstreamError(format!(
                        "Failed to write {}: {}",
                        clean_name, e
                    ))
                })?;
            }
        }

        let mut out = File::create(output_path.as_ref()).map_err(|e| ClerestoryError::IoError {
            path: output_path.as_ref().to_path_buf(),
            source: e,
        })?;

        out.write_all(&buffer)
            .map_err(|e| ClerestoryError::IoError {
                path: output_path.as_ref().to_path_buf(),
                source: e,
            })?;

        Ok(())
    }
}
