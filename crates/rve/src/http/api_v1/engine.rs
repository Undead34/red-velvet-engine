use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use tracing::error;

use crate::http::openapi::{EngineStatusResponseDoc, ErrorResponse};
use crate::http::state::AppState;

#[derive(Serialize)]
pub struct EngineStatusResponse {
  pub mode: &'static str,
  pub ready: bool,
  pub repository_rules: u32,
  pub loaded_rules: u32,
  pub message: &'static str,
}

#[utoipa::path(
  get,
  path = "/api/v1/engine/status",
  tag = "engine",
  responses(
    (status = 200, description = "Current placeholder runtime status", body = EngineStatusResponseDoc),
    (status = 500, description = "Failed to read runtime status", body = ErrorResponse)
  )
)]
pub async fn status(
  State(state): State<AppState>,
) -> Result<Json<EngineStatusResponse>, (StatusCode, Json<ErrorResponse>)> {
  let repository_rules = match state.rule_repo.all().await {
    Ok(rules) => rules.len() as u32,
    Err(err) => {
      error!(target: "BANNER", %err, operation = "status", "failed to read repository rules");
      return Err((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
          code: "internal_error".to_owned(),
          message: "failed to read runtime status".to_owned(),
          validation: None,
        }),
      ));
    }
  };

  Ok(Json(EngineStatusResponse {
    mode: "placeholder",
    ready: false,
    repository_rules,
    loaded_rules: 0,
    message: "runtime engine is not implemented",
  }))
}

#[utoipa::path(
  post,
  path = "/api/v1/engine/reload",
  tag = "engine",
  responses(
    (status = 501, description = "Runtime reload is not implemented yet", body = ErrorResponse)
  )
)]
pub async fn reload() -> (StatusCode, Json<ErrorResponse>) {
  (
    StatusCode::NOT_IMPLEMENTED,
    Json(ErrorResponse {
      code: "not_implemented".to_owned(),
      message: "runtime reload is not implemented yet".to_owned(),
      validation: None,
    }),
  )
}
