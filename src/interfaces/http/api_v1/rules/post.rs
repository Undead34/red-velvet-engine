use axum::{
  Json,
  extract::{State, rejection::JsonRejection},
  http::StatusCode,
  response::IntoResponse,
};
use tracing::{info, instrument, warn};

use crate::interfaces::http::state::AppState;

use super::{
  dto::{RuleDocumentRequest, RuleResponse},
  errors::{ApiResult, map_json_rejection, map_rule_command_error},
  mapper::rule_from_document,
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
      content = crate::interfaces::http::openapi::RuleDocumentInputDoc,
      description = "Rule configuration payload. `meta.author` is required, `meta.autor` is rejected. `enforcement.score_impact` must be in `1.0..=10.0`."
  ),
  responses(
    (status = 201, description = "Rule successfully created", body = crate::interfaces::http::openapi::RuleDoc),
    (status = 409, description = "A rule with the same identifier already exists", body = crate::interfaces::http::openapi::ErrorResponse),
    (status = 422, description = "Validation failed for the provided payload", body = crate::interfaces::http::openapi::ErrorResponse),
    (status = 500, description = "Internal server error during repository save", body = crate::interfaces::http::openapi::ErrorResponse)
  )
)]
#[instrument(name = "http.rules.create", skip(state, payload))]
pub async fn create_rule(
  State(state): State<AppState>,
  payload: Result<Json<RuleDocumentRequest>, JsonRejection>,
) -> ApiResult<impl IntoResponse> {
  let payload = payload.map_err(map_json_rejection)?.0;
  let rule = rule_from_document(payload, None)?;

  for warning in collect_rule_warnings(&rule) {
    warn!(path = %warning.path, message = %warning.message, "rule validation warning");
  }

  let created =
    state.rule_command_service.create_rule(rule).await.map_err(map_rule_command_error)?;
  let version = rule_version(&created)?;
  let headers = response_version_headers(&version)?;
  info!(rule_id = %created.id, version, "rule created");

  Ok((StatusCode::CREATED, headers, Json(RuleResponse::from(&created))))
}
