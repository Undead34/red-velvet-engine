use axum::{extract::State, Json};
use rve_core::{domain::event::Event, services::engine::Decision};
use serde::Deserialize;
use serde_json::Value;

use crate::http::state::AppState;

use super::rules::errors::{ApiError, ApiResult};

#[utoipa::path(
  post,
  path = "/api/v1/decisions",
  tag = "decisions",
  request_body = crate::http::openapi::DecisionRequestDoc,
  responses(
    (status = 200, description = "Decision evaluated", body = crate::http::openapi::DecisionResponseDoc),
    (status = 422, description = "Invalid event payload", body = crate::http::openapi::ErrorResponse),
    (status = 500, description = "Decision engine evaluation failed", body = crate::http::openapi::ErrorResponse)
  )
)]
pub async fn create_decision(
  State(state): State<AppState>,
  Json(request): Json<DecisionRequest>,
) -> ApiResult<Json<Decision>> {
  let event: Event = serde_json::from_value(request.event).map_err(|error| {
    ApiError::validation("event", format!("invalid event payload: {error}"))
  })?;

  let decision = state
    .engine
    .evaluate(&event)
    .map_err(|error| ApiError::Internal(format!("decision engine error: {error}")))?;

  Ok(Json(decision))
}

#[derive(Deserialize)]
pub struct DecisionRequest {
  pub event: Value,
}
