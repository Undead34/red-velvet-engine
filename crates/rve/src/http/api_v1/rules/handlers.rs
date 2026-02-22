use axum::{
  Json,
  extract::{Path, Query, State, rejection::JsonRejection},
  http::StatusCode,
  response::IntoResponse,
};
use rve_core::domain::rule::Rule;
use serde_json::Value;
use tracing::warn;

use crate::http::state::AppState;

use super::{
  errors::{
    ApiError, ApiResult, map_json_rejection, map_repository_error, parse_rule_id,
  },
  patch::apply_patch,
  types::{
    Pagination, PaginationMeta, RuleDocumentInput, RuleListResponse, collect_rule_warnings,
    validate_rule,
  },
};

/// List rules
///
/// Returns a paginated list of existing rules in the system.
#[utoipa::path(
  get,
  path = "/api/v1/rules",
  tag = "rules",
  params(Pagination),
  responses(
    (status = 200, description = "Paginated rules", body = crate::http::openapi::RuleListResponseDoc),
    (status = 500, description = "Repository error", body = crate::http::openapi::ErrorResponse)
  )
)]
pub async fn list_rules(
  State(state): State<AppState>,
  Query(pagination): Query<Pagination>,
) -> ApiResult<Json<RuleListResponse>> {
  let page = pagination.page.filter(|p| *p > 0).unwrap_or(1);
  let limit = pagination.limit.filter(|l| *l > 0).unwrap_or(20).min(100);

  let page_data = state.rule_repo.list(page, limit).await.map_err(map_repository_error)?;

  Ok(Json(RuleListResponse {
    data: page_data.items,
    pagination: PaginationMeta { page, limit, total: page_data.total },
  }))
}

/// Create a new rule
///
/// Parses and validates the provided payload, persists the new rule in the repository,
  /// The engine is not reloaded automatically; trigger `/api/v1/engine/reload` explicitly.
/// Non-fatal validation warnings are logged but will not prevent creation.
#[utoipa::path(
  post,
  path = "/api/v1/rules",
  tag = "rules",
  request_body(
      content = crate::http::openapi::RuleDocumentInputDoc,
      description = "Rule configuration payload containing conditions and actions"
  ),
  responses(
    (status = 201, description = "Rule successfully created", body = crate::http::openapi::RuleDoc),
    (status = 409, description = "A rule with the same identifier already exists", body = crate::http::openapi::ErrorResponse),
    (status = 422, description = "Validation failed for the provided payload", body = crate::http::openapi::ErrorResponse),
    (status = 500, description = "Internal server error during repository save", body = crate::http::openapi::ErrorResponse)
  )
)]
pub async fn create_rule(
  State(state): State<AppState>,
  payload: Result<Json<RuleDocumentInput>, JsonRejection>,
) -> ApiResult<impl IntoResponse> {
  let payload = payload.map_err(map_json_rejection)?.0;
  let rule = payload.into_rule(None)?;

  for warning in collect_rule_warnings(&rule) {
    warn!(target: "BANNER", path = %warning.path, message = %warning.message, "rule validation warning");
  }

  let created = state.rule_repo.create(rule).await.map_err(map_repository_error)?;

  Ok((StatusCode::CREATED, Json(created)))
}

/// Get a rule by ID
///
/// Retrieves the details of a specific rule using its unique identifier.
/// If the provided ID format is invalid or the rule does not exist, an appropriate error is returned.
#[utoipa::path(
  get,
  path = "/api/v1/rules/{id}",
  tag = "rules",
  params(
    ("id" = String, Path, description = "Unique identifier of the rule (UUID)", example = "019c7d7b-dc31-7fa9-8b1b-10e4fe820cb8")
  ),
  responses(
    (status = 200, description = "Rule successfully retrieved", body = crate::http::openapi::RuleDoc),
    (status = 400, description = "Invalid rule ID format provided", body = crate::http::openapi::ErrorResponse),
    (status = 404, description = "Rule not found", body = crate::http::openapi::ErrorResponse),
    (status = 500, description = "Internal server error", body = crate::http::openapi::ErrorResponse)
  )
)]
pub async fn get_rule(
  State(state): State<AppState>,
  Path(id): Path<String>,
) -> ApiResult<Json<Rule>> {
  let id = parse_rule_id(id)?;

  state
    .rule_repo
    .get(&id)
    .await
    .map_err(map_repository_error)?
    .map(Json)
    .ok_or_else(|| ApiError::NotFound("rule not found".to_owned()))
}

/// Replace an existing rule
///
/// Fully replaces the configuration of a rule identified by its UUID.
/// Validates the new payload and updates the repository.
/// The engine is not reloaded automatically; trigger `/api/v1/engine/reload` explicitly.
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
    description = "Complete rule configuration payload to replace the existing one"
  ),
  responses(
    (status = 200, description = "Rule successfully replaced", body = crate::http::openapi::RuleDoc),
    (status = 400, description = "Invalid rule ID format provided", body = crate::http::openapi::ErrorResponse),
    (status = 404, description = "Rule not found", body = crate::http::openapi::ErrorResponse),
    (status = 422, description = "Validation failed for the provided payload", body = crate::http::openapi::ErrorResponse),
    (status = 500, description = "Internal server error during repository update", body = crate::http::openapi::ErrorResponse)
  )
)]
pub async fn update_rule(
  State(state): State<AppState>,
  Path(id): Path<String>,
  payload: Result<Json<RuleDocumentInput>, JsonRejection>,
) -> ApiResult<Json<Rule>> {
  let payload = payload.map_err(map_json_rejection)?.0;
  let id = parse_rule_id(id)?;
  let rule = payload.into_rule(Some(id))?;

  for warning in collect_rule_warnings(&rule) {
    warn!(target: "BANNER", path = %warning.path, message = %warning.message, "rule validation warning");
  }

  let updated = state.rule_repo.replace(rule).await.map_err(map_repository_error)?;

  Ok(Json(updated))
}

/// Delete a rule
///
/// Permanently removes a rule from the system using its unique identifier.
/// The engine is not reloaded automatically; trigger `/api/v1/engine/reload` explicitly.
#[utoipa::path(
  delete,
  path = "/api/v1/rules/{id}",
  tag = "rules",
  params(
    ("id" = String, Path, description = "Unique identifier of the rule (UUID)", example = "550e8400-e29b-41d4-a716-446655440000")
  ),
  responses(
    (status = 204, description = "Rule successfully deleted (no content returned)"),
    (status = 400, description = "Invalid rule ID format provided", body = crate::http::openapi::ErrorResponse),
    (status = 404, description = "Rule not found", body = crate::http::openapi::ErrorResponse),
    (status = 500, description = "Internal server error during repository deletion", body = crate::http::openapi::ErrorResponse)
  )
)]
pub async fn delete_rule(
  State(state): State<AppState>,
  Path(id): Path<String>,
) -> ApiResult<StatusCode> {
  let id = parse_rule_id(id)?;
  state.rule_repo.delete(&id).await.map_err(map_repository_error)?;
  Ok(StatusCode::NO_CONTENT)
}

/// Partially update a rule
///
/// Applies a partial update or JSON patch to an existing rule identified by its UUID.
/// The system fetches the current rule, applies the changes, and validates the final state.
/// If valid, the repository is updated.
/// The engine is not reloaded automatically; trigger `/api/v1/engine/reload` explicitly.
/// Non-fatal validation warnings are logged but will not prevent the patch.
#[utoipa::path(
  patch,
  path = "/api/v1/rules/{id}",
  tag = "rules",
  params(
    ("id" = String, Path, description = "Unique identifier of the rule (UUID)", example = "019c7d7b-dc31-7fa9-8b1b-10e4fe820cb8")
  ),
  request_body(
    content = serde_json::Value,
    description = "Partial rule configuration payload containing only the fields to be updated"
  ),
  responses(
    (status = 200, description = "Rule successfully patched", body = crate::http::openapi::RuleDoc),
    (status = 400, description = "Invalid rule ID format or malformed patch payload provided", body = crate::http::openapi::ErrorResponse),
    (status = 404, description = "Rule not found", body = crate::http::openapi::ErrorResponse),
    (status = 422, description = "Validation failed for the rule's final state after applying the patch", body = crate::http::openapi::ErrorResponse),
    (status = 500, description = "Internal server error during repository update", body = crate::http::openapi::ErrorResponse)
  )
)]
pub async fn patch_rule(
  State(state): State<AppState>,
  Path(id): Path<String>,
  payload: Result<Json<Value>, JsonRejection>,
) -> ApiResult<Json<Rule>> {
  let payload = payload.map_err(map_json_rejection)?.0;
  let id = parse_rule_id(id)?;

  let mut rule = state
    .rule_repo
    .get(&id)
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| ApiError::NotFound("rule not found".to_owned()))?;

  apply_patch(&mut rule, payload)?;
  validate_rule(&rule)?;

  for warning in collect_rule_warnings(&rule) {
    warn!(target: "BANNER", path = %warning.path, message = %warning.message, "rule validation warning");
  }

  let saved = state.rule_repo.replace(rule).await.map_err(map_repository_error)?;

  Ok(Json(saved))
}
