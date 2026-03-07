use rve_core::domain::rule::Rule;
use serde_json::Value;

use super::{
  errors::{ApiError, ApiResult},
  types::parse_patch_value,
};

pub(super) fn apply_patch(rule: &mut Rule, patch: Value) -> ApiResult<()> {
  let patch = patch
    .as_object()
    .ok_or_else(|| ApiError::validation("request", "patch payload must be a JSON object"))?;
  let mut changed = false;

  for field in patch.keys() {
    if !matches!(field.as_str(), "state" | "rollout" | "schedule") {
      return Err(ApiError::validation(field, "field is not patchable"));
    }
  }

  let mut next_policy = rule.policy().clone();

  if let Some(state) = patch.get("state") {
    let state =
      state.as_object().ok_or_else(|| ApiError::validation("state", "must be an object"))?;

    for field in state.keys() {
      if !matches!(field.as_str(), "mode" | "audit") {
        return Err(ApiError::validation(format!("state.{field}"), "field is not patchable"));
      }
    }

    if let Some(mode_value) = state.get("mode") {
      let mode: rve_core::domain::rule::mode::RuleMode =
        parse_patch_value("state.mode", mode_value)?;

      next_policy
        .state
        .transition_to(mode)
        .map_err(|error| ApiError::validation("state.mode", error.to_string()))?;
      changed = true;
    }

    if let Some(audit) = state.get("audit") {
      let audit = audit
        .as_object()
        .ok_or_else(|| ApiError::validation("state.audit", "must be an object"))?;

      for field in audit.keys() {
        if !matches!(field.as_str(), "updated_by" | "updated_at_ms") {
          return Err(ApiError::validation(
            format!("state.audit.{field}"),
            "field is not patchable",
          ));
        }
      }

      if let Some(updated_by) = audit.get("updated_by") {
        let mut state = next_policy.state.clone();
        state.audit.updated_by = parse_patch_value("state.audit.updated_by", updated_by)?;
        next_policy.state = state;
        changed = true;
      }
      if let Some(updated_at) = audit.get("updated_at_ms") {
        let mut state = next_policy.state.clone();
        state.audit.updated_at_ms = parse_patch_value("state.audit.updated_at_ms", updated_at)?;
        next_policy.state = state;
        changed = true;
      }
    }
  }

  if let Some(rollout) = patch.get("rollout") {
    let rollout =
      rollout.as_object().ok_or_else(|| ApiError::validation("rollout", "must be an object"))?;

    for field in rollout.keys() {
      if field != "percent" {
        return Err(ApiError::validation(format!("rollout.{field}"), "field is not patchable"));
      }
    }

    if let Some(percent) = rollout.get("percent") {
      next_policy.rollout.percent = parse_patch_value("rollout.percent", percent)?;
      changed = true;
    }
  }

  if let Some(schedule) = patch.get("schedule") {
    let schedule =
      schedule.as_object().ok_or_else(|| ApiError::validation("schedule", "must be an object"))?;

    for field in schedule.keys() {
      if !matches!(field.as_str(), "active_until_ms" | "active_from_ms") {
        return Err(ApiError::validation(format!("schedule.{field}"), "field is not patchable"));
      }
    }

    if let Some(active_until_ms) = schedule.get("active_until_ms") {
      next_policy.schedule.active_until_ms =
        parse_patch_value("schedule.active_until_ms", active_until_ms)?;
      changed = true;
    }
    if let Some(active_from_ms) = schedule.get("active_from_ms") {
      next_policy.schedule.active_from_ms =
        parse_patch_value("schedule.active_from_ms", active_from_ms)?;
      changed = true;
    }
  }

  if !changed {
    return Err(ApiError::validation("request", "patch does not contain supported changes"));
  }

  rule.set_policy(next_policy).map_err(|error| ApiError::validation("rule", error.to_string()))?;

  Ok(())
}
