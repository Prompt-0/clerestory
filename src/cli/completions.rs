//! Shell auto-completions generation using clap_complete.

use crate::cli::args::Cli;
use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

pub struct CompletionsGenerator;

impl CompletionsGenerator {
    /// Generates shell completion script to stdout.
    pub fn generate(shell: Shell) {
        let mut cmd = Cli::command();
        generate(shell, &mut cmd, "clerestory", &mut io::stdout());
    }
}
