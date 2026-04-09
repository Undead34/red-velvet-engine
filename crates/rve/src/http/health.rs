use axum::{
  http::{HeaderMap, HeaderValue},
  response::IntoResponse,
};
use serde_json::json;
use tracing::instrument;

/// Health check
///
/// Verifies that the service is running and responsive.
/// Returns a simple health status along with a special custom header.
#[utoipa::path(
  get,
  path = "/health",
  tag = "health",
  responses(
    (
      status = 200,
      description = "Service is healthy and ready to accept requests",
      body = crate::http::openapi::HealthResponse,
      headers(
        ("X-Miku" = String, description = "Miku says: thank you (39)")
      )
    )
  )
)]
#[instrument(name = "http.health")]
pub async fn handler() -> impl IntoResponse {
  let mut headers = HeaderMap::new();
  headers.insert("X-Miku", HeaderValue::from_static("39")); // Miku says: thank you

  let body = axum::Json(json!({ "status": "ok" }));

  (headers, body)
}
