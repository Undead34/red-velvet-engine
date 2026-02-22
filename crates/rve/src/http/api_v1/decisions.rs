use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;

use crate::http::state::AppState;

#[utoipa::path(
  post,
  path = "/api/v1/decisions",
  tag = "decisions",
  request_body = crate::http::openapi::DecisionRequestDoc,
  responses(
    (status = 501, description = "Decision API skeleton endpoint", body = crate::http::openapi::ErrorResponse)
  )
)]
pub async fn create_decision(
  State(_state): State<AppState>,
  Json(_request): Json<DecisionRequest>,
) -> (StatusCode, Json<crate::http::openapi::ErrorResponse>) {
  (
    StatusCode::NOT_IMPLEMENTED,
    Json(crate::http::openapi::ErrorResponse {
      code: "NOT_IMPLEMENTED".into(),
      message: "Decision API is currently a skeleton endpoint".into(),
      validation: None,
    }),
  )
}

#[derive(Deserialize)]
pub struct DecisionRequest {
  pub event: serde_json::Value,
}
