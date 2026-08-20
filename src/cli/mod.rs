pub mod args;
pub mod commands;
pub mod completions;
pub mod doctor;
pub mod manpage;

pub use args::{Cli, Commands};
pub use commands::CommandDispatcher;
pub use doctor::DoctorRenderer;
