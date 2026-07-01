use axum::{
  Json,
  extract::{Path, State, rejection::JsonRejection},
  http::HeaderMap,
  response::IntoResponse,
};
use serde_json::Value;
use tracing::{info, instrument, warn};
use validator::Validate;

use crate::interfaces::http::state::AppState;

use super::{
  dto::{RulePatchRequest, RuleResponse, request::map_validation_errors},
  errors::{
    ApiError, ApiResult, map_json_rejection, map_rule_command_error, map_rule_query_error,
    parse_rule_id,
  },
  validation::{collect_rule_warnings, validate_rule as validate_rule_fn},
  versioning::{assert_if_match, response_version_headers, rule_version},
};

/// Partially update a rule
///
/// Applies a partial update or JSON patch to an existing rule identified by its UUID.
/// The system fetches the current rule, applies the changes, and validates the final state.
/// If valid, the repository is updated.
/// Runtime execution is currently unavailable; writes affect the repository only.
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
    (status = 200, description = "Rule successfully patched", body = crate::interfaces::http::openapi::RuleDoc),
    (status = 400, description = "Invalid rule ID format or malformed patch payload provided", body = crate::interfaces::http::openapi::ErrorResponse),
    (status = 409, description = "Version conflict when If-Match does not match current rule", body = crate::interfaces::http::openapi::ErrorResponse),
    (status = 404, description = "Rule not found", body = crate::interfaces::http::openapi::ErrorResponse),
    (status = 422, description = "Validation failed for the rule's final state after applying the patch", body = crate::interfaces::http::openapi::ErrorResponse),
    (status = 500, description = "Internal server error during repository update", body = crate::interfaces::http::openapi::ErrorResponse)
  )
)]
#[instrument(name = "http.rules.patch", skip(state, headers, payload), fields(rule_id = %id))]
pub async fn patch_rule(
  State(state): State<AppState>,
  headers: HeaderMap,
  Path(id): Path<String>,
  payload: Result<Json<Value>, JsonRejection>,
) -> ApiResult<impl IntoResponse> {
  let payload = payload.map_err(map_json_rejection)?.0;
  let id = parse_rule_id(id)?;

  let mut rule = state
    .rule_query_service
    .get_rule(&id)
    .await
    .map_err(map_rule_query_error)?
    .ok_or_else(|| ApiError::NotFound("rule not found".to_owned()))?;

  let current_version = rule_version(&rule)?;
  assert_if_match(&headers, &current_version)?;

  apply_patch(&mut rule, payload)?;
  validate_rule_fn(&rule)?;

  for warning in collect_rule_warnings(&rule) {
    warn!(path = %warning.path, message = %warning.message, "rule validation warning");
  }

  let saved =
    state.rule_command_service.replace_rule(rule).await.map_err(map_rule_command_error)?;
  let version = rule_version(&saved)?;
  let response_headers = response_version_headers(&version)?;
  info!(rule_id = %saved.id, version, "rule patched");

  Ok((response_headers, Json(RuleResponse::from(&saved))))
}

pub(super) fn apply_patch(rule: &mut rve_core::domain::rule::Rule, patch: Value) -> ApiResult<()> {
  let patch: RulePatchRequest = serde_json::from_value(patch)
    .map_err(|err| ApiError::validation("request", err.to_string()))?;

  patch.validate().map_err(map_validation_errors)?;

  if patch.state.is_none() && patch.rollout.is_none() && patch.schedule.is_none() {
    return Err(ApiError::validation("request", "patch does not contain supported changes"));
  }

  let mut next_policy = rule.policy().clone();
  let mut changed = false;

  if let Some(state_patch) = patch.state {
    if let Some(mode) = state_patch.mode {
      next_policy
        .state
        .transition_to(mode)
        .map_err(|err| ApiError::validation("state.mode", err.to_string()))?;
      changed = true;
    }

    if let Some(audit_patch) = state_patch.audit {
      if let Some(updated_at_ms) = audit_patch.updated_at_ms {
        next_policy.state.audit.updated_at_ms = updated_at_ms;
        changed = true;
      }
      if let Some(updated_by) = audit_patch.updated_by {
        next_policy.state.audit.updated_by = Some(updated_by);
        changed = true;
      }
    }
  }

  if let Some(rollout_patch) = patch.rollout
    && let Some(percent) = rollout_patch.percent
  {
    next_policy.rollout.percent = percent;
    changed = true;
  }

  if let Some(schedule_patch) = patch.schedule
    && (schedule_patch.active_from_ms.is_some() || schedule_patch.active_until_ms.is_some())
  {
    next_policy.schedule.active_from_ms = schedule_patch.active_from_ms;
    next_policy.schedule.active_until_ms = schedule_patch.active_until_ms;
    changed = true;
  }

  if !changed {
    return Err(ApiError::validation("request", "patch does not contain supported changes"));
  }

  rule.set_policy(next_policy).map_err(|err| ApiError::validation("rule", err.to_string()))?;

  Ok(())
}
