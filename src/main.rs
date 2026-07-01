use std::process::ExitCode;

use tracing::error;

use rve::{error::AppError, interfaces::cli};

#[tokio::main]
async fn main() -> ExitCode {
  match run().await {
    Ok(()) => ExitCode::SUCCESS,
    Err(e) => {
      error!(code = e.code(), error = %e, "Fatal");
      ExitCode::from(e.code())
    }
  }
}

async fn run() -> Result<(), AppError> {
  cli::run().await
}
