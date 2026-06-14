use axum::{
  Json,
  extract::{State, rejection::JsonRejection},
  http::StatusCode,
  response::IntoResponse,
};
use tracing::{info, instrument, warn};

use crate::http::state::AppState;

use super::{
  errors::{ApiResult, map_json_rejection, map_repository_error},
  requests::RuleDocumentInput,
  validation::collect_rule_warnings,
  versioning::{response_version_headers, rule_version},
};

/// Create a new rule
///
/// Parses and validates the provided payload, persists the new rule in the repository,
/// Runtime execution is currently unavailable; writes affect the repository only.
/// Non-fatal validation warnings are logged but will not prevent creation.
#[utoipa::path(
  post,
  path = "/api/v1/rules",
  tag = "rules",
  request_body(
      content = crate::http::openapi::RuleDocumentInputDoc,
      description = "Rule configuration payload. `meta.author` is required, `meta.autor` is rejected. `enforcement.score_impact` must be in `1.0..=10.0`."
  ),
  responses(
    (status = 201, description = "Rule successfully created", body = crate::http::openapi::RuleDoc),
    (status = 409, description = "A rule with the same identifier already exists", body = crate::http::openapi::ErrorResponse),
    (status = 422, description = "Validation failed for the provided payload", body = crate::http::openapi::ErrorResponse),
    (status = 500, description = "Internal server error during repository save", body = crate::http::openapi::ErrorResponse)
  )
)]
#[instrument(name = "http.rules.create", skip(state, payload))]
pub async fn create_rule(
  State(state): State<AppState>,
  payload: Result<Json<RuleDocumentInput>, JsonRejection>,
) -> ApiResult<impl IntoResponse> {
  let payload = payload.map_err(map_json_rejection)?.0;
  let rule = payload.into_rule(None)?;

  for warning in collect_rule_warnings(&rule) {
    warn!(path = %warning.path, message = %warning.message, "rule validation warning");
  }

  let created = state.rule_repo.create(rule).await.map_err(map_repository_error)?;
  let version = rule_version(&created)?;
  let headers = response_version_headers(&version)?;
  info!(rule_id = %created.id, version, "rule created");

  Ok((StatusCode::CREATED, headers, Json(created)))
}
