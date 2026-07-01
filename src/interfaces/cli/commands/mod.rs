use crate::error::AppError;

use super::{AboutCommand, Cli, Command};

mod about;
mod serve;

/// Executes the selected CLI command.
pub async fn dispatch(cli: Cli) -> Result<(), AppError> {
  match cli.command {
    Some(Command::About { command }) => {
      about::run(command, cli.quiet);
      Ok(())
    }
    None => serve::run(cli).await,
  }
}
