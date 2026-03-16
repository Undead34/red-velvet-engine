use axum::{Json, extract::State, http::StatusCode};
use rve_core::{
  ports::RuntimeEngineError,
  services::engine::{DecisionService, DecisionServiceError},
};
use serde::Serialize;
use tracing::error;

use crate::http::openapi::{EngineStatusResponseDoc, ErrorResponse};
use crate::http::state::AppState;

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
  pub compile_stats: rve_core::ports::RuleCompileStats,
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

  let runtime_status = state.engine.status().map_err(|err| {
    error!(target: "BANNER", %err, operation = "status", "failed to read runtime status");
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(ErrorResponse {
        code: "internal_error".to_owned(),
        message: "failed to read runtime status".to_owned(),
        validation: None,
      }),
    )
  })?;

  let message = if runtime_status.ready {
    "runtime ready".to_owned()
  } else {
    "runtime not ready".to_owned()
  };

  Ok(Json(EngineStatusResponse {
    mode: runtime_status.mode,
    ready: runtime_status.ready,
    repository_rules,
    loaded_rules: runtime_status.loaded_rules,
    message,
  }))
}

#[utoipa::path(
  post,
  path = "/api/v1/engine/reload",
  tag = "engine",
  responses(
    (status = 200, description = "Runtime ruleset reloaded", body = serde_json::Value),
    (status = 500, description = "Failed to reload runtime", body = ErrorResponse)
  )
)]
pub async fn reload(
  State(state): State<AppState>,
) -> Result<Json<EngineReloadResponse>, (StatusCode, Json<ErrorResponse>)> {
  let snapshot = DecisionService::reload_rules(state.rule_repo.as_ref(), state.engine.as_ref())
    .await
    .map_err(map_engine_error)?;

  Ok(Json(EngineReloadResponse {
    version: snapshot.version,
    loaded_rules: snapshot.loaded_rules,
    compile_stats: snapshot.compile_stats,
  }))
}

fn map_engine_error(error: DecisionServiceError) -> (StatusCode, Json<ErrorResponse>) {
  let (status, code) = match &error {
    DecisionServiceError::Runtime(RuntimeEngineError::Configuration { .. }) => {
      (StatusCode::SERVICE_UNAVAILABLE, "runtime_configuration")
    }
    DecisionServiceError::Runtime(RuntimeEngineError::Compilation { .. }) => {
      (StatusCode::INTERNAL_SERVER_ERROR, "runtime_compilation")
    }
    DecisionServiceError::Runtime(RuntimeEngineError::Evaluation { .. }) => {
      (StatusCode::INTERNAL_SERVER_ERROR, "runtime_evaluation")
    }
    DecisionServiceError::Runtime(RuntimeEngineError::NotImplemented { .. }) => {
      (StatusCode::NOT_IMPLEMENTED, "not_implemented")
    }
    DecisionServiceError::Runtime(RuntimeEngineError::Internal { .. }) => {
      (StatusCode::INTERNAL_SERVER_ERROR, "runtime_internal")
    }
    DecisionServiceError::Repository(_) => (StatusCode::INTERNAL_SERVER_ERROR, "repository_error"),
  };

  (
    status,
    Json(ErrorResponse { code: code.to_owned(), message: error.to_string(), validation: None }),
  )
}
