//! Command-line inbound adapter.
//!
//! The CLI owns command parsing and dispatch. Command handlers translate process
//! input into application wiring or use-case calls; they must not embed domain
//! rules or persistence details directly.

mod args;
mod commands;

pub use args::{AboutCommand, Cli, Command};

use crate::error::AppError;

/// Parses process arguments and executes the selected command.
pub async fn run() -> Result<(), AppError> {
  commands::dispatch(Cli::parse()).await
}
