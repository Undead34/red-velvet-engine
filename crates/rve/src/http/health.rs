use axum::{http::HeaderMap, response::IntoResponse};
use serde_json::json;

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
pub async fn handler() -> impl IntoResponse {
  let mut headers = HeaderMap::new();
  headers.insert("X-Miku", "39".parse().unwrap()); // Miku says: thank you

  let body = axum::Json(json!({ "status": "ok" }));

  (headers, body)
}
