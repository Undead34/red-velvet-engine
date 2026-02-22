use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use tracing::error;

use crate::http::state::AppState;

#[derive(Serialize)]
pub struct ReloadResponse {
  pub status: &'static str,
  pub message: &'static str,
}

#[utoipa::path(
  post,
  path = "/api/v1/engine/reload",
  tag = "engine",
  responses(
    (status = 200, description = "Engine rules reloaded", body = crate::http::openapi::ReloadResponseDoc),
    (status = 500, description = "Failed to reload engine rules", body = crate::http::openapi::ErrorResponse)
  )
)]
pub async fn reload(
  State(state): State<AppState>,
) -> Result<Json<ReloadResponse>, (StatusCode, Json<crate::http::openapi::ErrorResponse>)> {
  if let Err(err) = state.reload_rules().await {
    error!(target: "BANNER", %err, operation = "reload", "failed to refresh engine ruleset");
    return Err((
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(crate::http::openapi::ErrorResponse {
        code: "internal_error".to_owned(),
        message: "failed to refresh engine ruleset".to_owned(),
        validation: None,
      }),
    ));
  }

  Ok(Json(ReloadResponse { status: "ok", message: "engine rules reloaded" }))
}
