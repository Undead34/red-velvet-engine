use axum::{Json, http::StatusCode};
use serde_json::Value;

use crate::http::openapi::ErrorResponse;

#[utoipa::path(
  post,
  path = "/api/v1/decisions",
  tag = "decisions",
  request_body(
    content = Value,
    description = "Placeholder JSON body. The final event contract will be published when the runtime engine is implemented."
  ),
  responses(
    (status = 501, description = "Decision runtime is not implemented yet", body = ErrorResponse)
  )
)]
pub async fn create_decision(Json(_request): Json<Value>) -> (StatusCode, Json<ErrorResponse>) {
  (
    StatusCode::NOT_IMPLEMENTED,
    Json(ErrorResponse {
      code: "not_implemented".to_owned(),
      message: "decision runtime is not implemented yet".to_owned(),
      validation: None,
    }),
  )
}
