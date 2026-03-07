use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use tracing::error;

use crate::http::state::AppState;

#[derive(Serialize)]
pub struct ReloadResponse {
  pub status: &'static str,
  pub message: &'static str,
}

#[derive(Serialize)]
pub struct EngineStatusResponse {
  pub ruleset_version: u64,
  pub loaded_rules: u32,
  pub repository_rules: u32,
  pub last_reload_at_ms: Option<u64>,
  pub last_reload_error: Option<String>,
}

#[utoipa::path(
  get,
  path = "/api/v1/engine/status",
  tag = "engine",
  responses(
    (status = 200, description = "Current engine runtime status", body = crate::http::openapi::EngineStatusResponseDoc),
    (status = 500, description = "Failed to read runtime status", body = crate::http::openapi::ErrorResponse)
  )
)]
pub async fn status(
  State(state): State<AppState>,
) -> Result<Json<EngineStatusResponse>, (StatusCode, Json<crate::http::openapi::ErrorResponse>)> {
  let repository_rules = match state.rule_repo.all().await {
    Ok(rules) => rules.len() as u32,
    Err(err) => {
      error!(target: "BANNER", %err, operation = "status", "failed to read repository rules");
      return Err((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(crate::http::openapi::ErrorResponse {
          code: "internal_error".to_owned(),
          message: "failed to read runtime status".to_owned(),
          validation: None,
        }),
      ));
    }
  };

  let runtime = state.engine_runtime_status().await;

  Ok(Json(EngineStatusResponse {
    ruleset_version: runtime.ruleset_version,
    loaded_rules: runtime.loaded_rules,
    repository_rules,
    last_reload_at_ms: runtime.last_reload_at_ms,
    last_reload_error: runtime.last_reload_error,
  }))
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
