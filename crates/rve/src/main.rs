use std::{net::SocketAddr, process::ExitCode};

use rve::bootstrap::AppContainer;
use tokio::net::TcpListener;
use tracing::{error, info, info_span};

use rve::http::build_router;
use rve::http::state::AppState;
use rve::{about, cli::AboutCommand, cli::App, cli::Command, error::AppError};
use rve::{banner, logger};

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
  let _ = dotenvy::dotenv();

  let app = App::new();

  if let Some(Command::About { command }) = app.command {
    match command {
      Some(AboutCommand::Licenses) => about::show_licenses(app.quiet),
      None => about::show_project_about(app.quiet),
    }

    return Ok(());
  }

  let _bye = banner::show_banner(app.quiet);
  logger::setup_logging(app.verbose, app.quiet);

  let _startup = info_span!("startup").entered();

  let addr: SocketAddr = format!("{}:{}", app.host, app.port).parse()?;
  let listener = TcpListener::bind(addr).await.map_err(AppError::BindFailed)?;

  let container = AppContainer::bootstrap().await?;
  let state = AppState::from(container);
  let router = build_router(state);

  info!(target: "BANNER", "Listening on http://{}", addr);

  axum::serve(listener, router)
    .with_graceful_shutdown(shutdown_signal())
    .await
    .map_err(AppError::ServeFailed)?;

  Ok(())
}

async fn shutdown_signal() {
  tokio::signal::ctrl_c().await.expect("failed to install CTRL+C handler");
  println!();
}
