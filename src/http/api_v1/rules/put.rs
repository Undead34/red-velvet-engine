use axum::{
  Json,
  extract::{Path, State, rejection::JsonRejection},
  http::HeaderMap,
  response::IntoResponse,
};
use tracing::{info, instrument, warn};

use crate::http::state::AppState;

use super::{
  errors::{ApiError, ApiResult, map_json_rejection, map_repository_error, parse_rule_id},
  requests::RuleDocumentInput,
  validation::collect_rule_warnings,
  versioning::{assert_if_match, response_version_headers, rule_version},
};

/// Replace an existing rule
///
/// Fully replaces the configuration of a rule identified by its UUID.
/// Validates the new payload and updates the repository.
/// Runtime execution is currently unavailable; writes affect the repository only.
/// Non-fatal validation warnings are logged but will not prevent the update.
#[utoipa::path(
  put,
  path = "/api/v1/rules/{id}",
  tag = "rules",
  params(
    ("id" = String, Path, description = "Unique identifier of the rule (UUID)", example = "019c7d7b-dc31-7fa9-8b1b-10e4fe820cb8")
  ),
  request_body(
    content = crate::http::openapi::RuleDocumentInputDoc,
    description = "Complete rule configuration payload to replace the existing one. `enforcement.score_impact` must be in `1.0..=10.0`."
  ),
  responses(
    (status = 200, description = "Rule successfully replaced", body = crate::http::openapi::RuleDoc),
    (status = 400, description = "Invalid rule ID format provided", body = crate::http::openapi::ErrorResponse),
    (status = 409, description = "Version conflict when If-Match does not match current rule", body = crate::http::openapi::ErrorResponse),
    (status = 404, description = "Rule not found", body = crate::http::openapi::ErrorResponse),
    (status = 422, description = "Validation failed for the provided payload", body = crate::http::openapi::ErrorResponse),
    (status = 500, description = "Internal server error during repository update", body = crate::http::openapi::ErrorResponse)
  )
)]
#[instrument(name = "http.rules.update", skip(state, headers, payload), fields(rule_id = %id))]
pub async fn update_rule(
  State(state): State<AppState>,
  headers: HeaderMap,
  Path(id): Path<String>,
  payload: Result<Json<RuleDocumentInput>, JsonRejection>,
) -> ApiResult<impl IntoResponse> {
  let payload = payload.map_err(map_json_rejection)?.0;
  let id = parse_rule_id(id)?;

  let current = state
    .rule_repo
    .get(&id)
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| ApiError::NotFound("rule not found".to_owned()))?;

  let current_version = rule_version(&current)?;
  assert_if_match(&headers, &current_version)?;

  let rule = payload.into_rule(Some(id))?;

  for warning in collect_rule_warnings(&rule) {
    warn!(path = %warning.path, message = %warning.message, "rule validation warning");
  }

  let updated = state.rule_repo.replace(rule).await.map_err(map_repository_error)?;
  let version = rule_version(&updated)?;
  let response_headers = response_version_headers(&version)?;
  info!(rule_id = %updated.id, version, "rule replaced");

  Ok((response_headers, Json(updated)))
}
