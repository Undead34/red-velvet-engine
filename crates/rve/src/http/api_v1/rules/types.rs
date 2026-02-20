use rve_core::domain::{
  common::{Score, RuleId},
  rule::{
    Rule, RuleAction, RuleAudit, RuleEnforcement, RuleEvaluation, RuleMeta, RuleSchedule,
    RuleState, RolloutPolicy, mode::RuleMode,
  },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::{ApiError, ApiResult};

#[derive(Deserialize)]
pub struct Pagination {
  pub page: Option<u32>,
  pub limit: Option<u32>,
}

#[derive(Serialize)]
pub struct RuleListResponse {
  pub data: Vec<Rule>,
  pub pagination: PaginationMeta,
}

#[derive(Serialize)]
pub struct PaginationMeta {
  pub page: u32,
  pub limit: u32,
  pub total: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuleDocumentInput {
  #[serde(default)]
  pub id: Option<RuleId>,
  pub meta: RuleMetaInput,
  pub state: RuleStateInput,
  pub schedule: RuleScheduleInput,
  pub rollout: RolloutPolicyInput,
  pub evaluation: RuleEvaluationInput,
  pub enforcement: RuleEnforcementInput,
}

impl RuleDocumentInput {
  pub(super) fn into_rule(self, override_id: Option<RuleId>) -> ApiResult<Rule> {
    let rule = Rule {
      id: override_id.or(self.id).unwrap_or_else(RuleId::new_v7),
      meta: self.meta.into_domain(),
      state: self.state.into_domain(),
      schedule: self.schedule.into_domain(),
      rollout: self.rollout.into_domain(),
      evaluation: self.evaluation.into_domain(),
      enforcement: self.enforcement.into_domain()?,
    };

    validate_rule(&rule)?;
    Ok(rule)
  }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleMetaInput {
  pub name: String,
  pub description: Option<String>,
  pub version: semver::Version,
  pub autor: String,
  pub tags: Option<Vec<String>>,
}

impl RuleMetaInput {
  fn into_domain(self) -> RuleMeta {
    RuleMeta {
      name: self.name,
      description: self.description,
      version: self.version,
      autor: self.autor,
      tags: self.tags,
    }
  }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleStateInput {
  pub mode: RuleMode,
  pub audit: RuleAuditInput,
}

impl RuleStateInput {
  fn into_domain(self) -> RuleState {
    RuleState { mode: self.mode, audit: self.audit.into_domain() }
  }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleAuditInput {
  pub created_at_ms: rve_core::domain::common::TimestampMs,
  pub updated_at_ms: rve_core::domain::common::TimestampMs,
  pub created_by: Option<String>,
  pub updated_by: Option<String>,
}

impl RuleAuditInput {
  fn into_domain(self) -> RuleAudit {
    RuleAudit {
      created_at_ms: self.created_at_ms,
      updated_at_ms: self.updated_at_ms,
      created_by: self.created_by,
      updated_by: self.updated_by,
    }
  }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleScheduleInput {
  pub active_from_ms: Option<rve_core::domain::common::TimestampMs>,
  pub active_until_ms: Option<rve_core::domain::common::TimestampMs>,
}

impl RuleScheduleInput {
  fn into_domain(self) -> RuleSchedule {
    RuleSchedule { active_from_ms: self.active_from_ms, active_until_ms: self.active_until_ms }
  }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RolloutPolicyInput {
  pub percent: u8,
}

impl RolloutPolicyInput {
  fn into_domain(self) -> RolloutPolicy {
    RolloutPolicy { percent: self.percent }
  }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleEvaluationInput {
  pub condition: Value,
  pub logic: Value,
}

impl RuleEvaluationInput {
  fn into_domain(self) -> RuleEvaluation {
    RuleEvaluation { condition: self.condition, logic: self.logic }
  }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleEnforcementInput {
  pub score_impact: f32,
  pub action: RuleAction,
  pub severity: rve_core::domain::common::Severity,
  pub tags: Vec<String>,
  pub cooldown_ms: Option<u64>,
}

impl RuleEnforcementInput {
  fn into_domain(self) -> ApiResult<RuleEnforcement> {
    let score_impact = Score::new(self.score_impact).ok_or_else(|| {
      ApiError::validation("enforcement.score_impact", "must be between 1.0 and 10.0")
    })?;

    Ok(RuleEnforcement {
      score_impact,
      action: self.action,
      severity: self.severity,
      tags: self.tags,
      cooldown_ms: self.cooldown_ms,
    })
  }
}

pub(super) fn validate_rule(rule: &Rule) -> ApiResult<()> {
  if rule.rollout.percent > 100 {
    return Err(ApiError::validation("rollout.percent", "must be between 0 and 100"));
  }

  if let (Some(from), Some(until)) = (rule.schedule.active_from_ms, rule.schedule.active_until_ms)
    && until.as_u64() <= from.as_u64()
  {
    return Err(ApiError::validation(
      "schedule.active_until_ms",
      "must be greater than schedule.active_from_ms",
    ));
  }

  if rule.state.audit.updated_at_ms.as_u64() < rule.state.audit.created_at_ms.as_u64() {
    return Err(ApiError::validation(
      "state.audit.updated_at_ms",
      "must be greater than or equal to state.audit.created_at_ms",
    ));
  }

  Ok(())
}

pub(super) fn parse_patch_value<T>(field: &'static str, value: &Value) -> ApiResult<T>
where
  T: serde::de::DeserializeOwned,
{
  serde_json::from_value(value.clone())
    .map_err(|_| ApiError::validation(field, "invalid type or value"))
}
