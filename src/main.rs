//! Clerestory CLI application binary entrypoint.

use clap::Parser;
use clerestory::cli::{Cli, CommandDispatcher};
use clerestory::exit_codes::EX_OK;
use colored::Colorize;
use std::process::exit;

fn main() {
    let cli = Cli::parse();

    if cli.no_color || std::env::var("NO_COLOR").is_ok() {
        colored::control::set_override(false);
    }

    if cli.verbose > 0 {
        let filter = match cli.verbose {
            1 => "info",
            2 => "debug",
            _ => "trace",
        };
        tracing_subscriber::fmt().with_env_filter(filter).init();
    }

    if let Err(err) = CommandDispatcher::dispatch(cli) {
        eprintln!("{} {}", "Error:".bold().red(), err);
        let code = err.exit_code();
        exit(code);
    }

    exit(EX_OK);
}
