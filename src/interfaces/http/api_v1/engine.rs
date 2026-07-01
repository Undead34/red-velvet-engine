use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use tracing::instrument;

use rve_core::ports::rule_engine::RuleCompileStats;

use crate::interfaces::http::openapi::{
  EngineReloadResponseDoc, EngineStatusResponseDoc, ErrorResponse,
};
use crate::interfaces::http::state::AppState;

use super::errors::map_runtime_control_error;

#[derive(Serialize)]
pub struct EngineStatusResponse {
  pub mode: String,
  pub ready: bool,
  pub repository_rules: u32,
  pub loaded_rules: u32,
  pub message: String,
}

#[derive(Serialize)]
pub struct EngineReloadResponse {
  pub version: u64,
  pub loaded_rules: u32,
  pub compile_stats: RuleCompileStats,
}

#[utoipa::path(
  get,
  path = "/api/v1/engine/status",
  tag = "engine",
  responses(
    (status = 200, description = "Current runtime status", body = EngineStatusResponseDoc),
    (status = 500, description = "Failed to read runtime status", body = ErrorResponse)
  )
)]
#[instrument(name = "http.engine.status", skip(state))]
pub async fn status(
  State(state): State<AppState>,
) -> Result<Json<EngineStatusResponse>, (StatusCode, Json<ErrorResponse>)> {
  let runtime_status =
    state.runtime_control_service.status().await.map_err(map_runtime_control_error)?;

  let message =
    if runtime_status.ready { "runtime ready".to_owned() } else { "runtime not ready".to_owned() };

  Ok(Json(EngineStatusResponse {
    mode: runtime_status.mode,
    ready: runtime_status.ready,
    repository_rules: runtime_status.repository_rules,
    loaded_rules: runtime_status.loaded_rules,
    message,
  }))
}

#[utoipa::path(
  post,
  path = "/api/v1/engine/reload",
  tag = "engine",
  responses(
    (status = 200, description = "Runtime ruleset reloaded", body = EngineReloadResponseDoc),
    (status = 500, description = "Failed to reload runtime", body = ErrorResponse)
  )
)]
#[instrument(name = "http.engine.reload", skip(state))]
pub async fn reload(
  State(state): State<AppState>,
) -> Result<Json<EngineReloadResponse>, (StatusCode, Json<ErrorResponse>)> {
  let snapshot =
    state.runtime_control_service.reload_rules().await.map_err(map_runtime_control_error)?;

  Ok(Json(EngineReloadResponse {
    version: snapshot.version,
    loaded_rules: snapshot.loaded_rules,
    compile_stats: snapshot.compile_stats,
  }))
}
