use axum::{
  http::{HeaderMap, HeaderValue},
  response::IntoResponse,
};
use tracing::instrument;

use crate::interfaces::http::openapi::{HealthResponse, HealthVersionResponse};
use crate::version::version_metadata;
use rve_core::ENGINE_NAME;

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
      body = crate::interfaces::http::openapi::HealthResponse,
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

  let version = version_metadata();
  let body = axum::Json(HealthResponse {
    status: "healthy".to_owned(),
    engine: ENGINE_NAME.to_owned(),
    version: HealthVersionResponse {
      semver: version.semver().to_owned(),
      release: version.calver().to_owned(),
      commit: version.commit().to_owned(),
      branch: version.branch().to_owned(),
      build: version.build_timestamp().to_owned(),
      dirty: version.is_dirty(),
    },
  });

  (headers, body)
}
