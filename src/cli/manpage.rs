//! ROFF man page generation using clap_mangen.

use crate::cli::args::Cli;
use crate::error::{ClerestoryError, Result};
use clap::CommandFactory;
use clap_mangen::Man;
use std::fs::{create_dir_all, File};
use std::path::Path;

pub struct ManpageGenerator;

impl ManpageGenerator {
    /// Generates man pages for distribution in man1 directory.
    pub fn generate_to_dir<P: AsRef<Path>>(output_dir: P) -> Result<()> {
        create_dir_all(output_dir.as_ref()).map_err(|e| ClerestoryError::IoError {
            path: output_dir.as_ref().to_path_buf(),
            source: e,
        })?;

        let cmd = Cli::command();
        let man_path = output_dir.as_ref().join("clerestory.1");
        let mut file = File::create(&man_path).map_err(|e| ClerestoryError::IoError {
            path: man_path.clone(),
            source: e,
        })?;

        let man = Man::new(cmd);
        man.render(&mut file)
            .map_err(|e| ClerestoryError::IoError {
                path: man_path,
                source: e,
            })?;

        Ok(())
    }
}
