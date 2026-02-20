use rve_core::domain::rule::Rule;
use serde_json::Value;

use super::{errors::ApiResult, types::parse_patch_value};

pub(super) fn apply_patch(rule: &mut Rule, patch: Value) -> ApiResult<()> {
  if let Some(state) = patch.get("state") {
    if let Some(mode_value) = state.get("mode") {
      rule.state.mode = parse_patch_value("state.mode", mode_value)?;
    }
    if let Some(audit) = state.get("audit") {
      if let Some(updated_by) = audit.get("updated_by") {
        rule.state.audit.updated_by = parse_patch_value("state.audit.updated_by", updated_by)?;
      }
      if let Some(updated_at) = audit.get("updated_at_ms") {
        rule.state.audit.updated_at_ms =
          parse_patch_value("state.audit.updated_at_ms", updated_at)?;
      }
    }
  }

  if let Some(rollout) = patch.get("rollout") {
    if let Some(percent) = rollout.get("percent") {
      rule.rollout.percent = parse_patch_value("rollout.percent", percent)?;
    }
  }

  if let Some(schedule) = patch.get("schedule") {
    if let Some(active_until_ms) = schedule.get("active_until_ms") {
      rule.schedule.active_until_ms =
        parse_patch_value("schedule.active_until_ms", active_until_ms)?;
    }
    if let Some(active_from_ms) = schedule.get("active_from_ms") {
      rule.schedule.active_from_ms = parse_patch_value("schedule.active_from_ms", active_from_ms)?;
    }
  }

  Ok(())
}
