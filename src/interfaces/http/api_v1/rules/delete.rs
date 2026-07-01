use axum::{
  extract::{Path, State},
  http::{HeaderMap, StatusCode},
};
use tracing::{info, instrument};

use crate::interfaces::http::state::AppState;

use super::{
  errors::{ApiError, ApiResult, map_rule_command_error, map_rule_query_error, parse_rule_id},
  versioning::{assert_if_match, rule_version},
};

/// Delete a rule
///
/// Permanently removes a rule from the system using its unique identifier.
/// Runtime execution is currently unavailable; writes affect the repository only.
#[utoipa::path(
  delete,
  path = "/api/v1/rules/{id}",
  tag = "rules",
  params(
    ("id" = String, Path, description = "Unique identifier of the rule (UUID)", example = "550e8400-e29b-41d4-a716-446655440000")
  ),
  responses(
    (status = 204, description = "Rule successfully deleted (no content returned)"),
    (status = 400, description = "Invalid rule ID format provided", body = crate::interfaces::http::openapi::ErrorResponse),
    (status = 409, description = "Version conflict when If-Match does not match current rule", body = crate::interfaces::http::openapi::ErrorResponse),
    (status = 404, description = "Rule not found", body = crate::interfaces::http::openapi::ErrorResponse),
    (status = 500, description = "Internal server error during repository deletion", body = crate::interfaces::http::openapi::ErrorResponse)
  )
)]
#[instrument(name = "http.rules.delete", skip(state, headers), fields(rule_id = %id))]
pub async fn delete_rule(
  State(state): State<AppState>,
  headers: HeaderMap,
  Path(id): Path<String>,
) -> ApiResult<StatusCode> {
  let id = parse_rule_id(id)?;

  let current = state
    .rule_query_service
    .get_rule(&id)
    .await
    .map_err(map_rule_query_error)?
    .ok_or_else(|| ApiError::NotFound("rule not found".to_owned()))?;

  let current_version = rule_version(&current)?;
  assert_if_match(&headers, &current_version)?;

  state.rule_command_service.delete_rule(&id).await.map_err(map_rule_command_error)?;
  info!(rule_id = %id, "rule deleted");

  Ok(StatusCode::NO_CONTENT)
}
