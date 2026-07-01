use std::net::SocketAddr;

use axum::Router;
use tokio::net::TcpListener;
use tracing::{info, info_span};

use crate::{
  banner,
  bootstrap::AppContainer,
  error::AppError,
  interfaces::http::{build_router, state::AppState},
  logger,
};

use super::Cli;

/// Runs the default HTTP server command.
pub async fn run(cli: Cli) -> Result<(), AppError> {
  #[cfg(debug_assertions)]
  let _ = dotenvy::dotenv();

  let _bye_guard = banner::show_banner(cli.quiet);
  logger::setup_logging(cli.verbose, cli.quiet);

  let _startup = info_span!("startup").entered();

  let addr: SocketAddr = format!("{}:{}", cli.host, cli.port).parse()?;
  let listener = TcpListener::bind(addr).await.map_err(AppError::BindFailed)?;
  let router = build_http_router().await?;

  info!(target: "BANNER", "Listening on http://{}", addr);

  axum::serve(listener, router)
    .with_graceful_shutdown(shutdown_signal())
    .await
    .map_err(AppError::ServeFailed)?;

  Ok(())
}

async fn build_http_router() -> Result<Router, AppError> {
  let container = AppContainer::bootstrap().await?;
  let state = AppState::from(container);

  Ok(build_router(state))
}

async fn shutdown_signal() {
  tokio::signal::ctrl_c().await.expect("failed to install CTRL+C handler");
  println!();
}
