use rve_core::domain::{
  common::{RuleId, Score, Severity, TimestampMs},
  rule::{
    Rule, RuleAction, RuleAudit, RuleEnforcement, RuleEvaluation, RuleMeta, RuleSchedule,
    RuleState, RolloutPolicy, mode::RuleMode,
  },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use validator::{Validate, ValidationError, ValidationErrors, ValidationErrorsKind};

use super::errors::{ApiError, ApiResult};
use super::logic_validation::validate_rule_evaluation;

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

#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuleDocumentInput {
  #[serde(default)]
  pub id: Option<RuleId>,
  #[validate(nested)]
  pub meta: RuleMetaInput,
  #[validate(nested)]
  pub state: RuleStateInput,
  #[validate(nested)]
  pub schedule: RuleScheduleInput,
  #[validate(nested)]
  pub rollout: RolloutPolicyInput,
  #[validate(nested)]
  pub evaluation: RuleEvaluationInput,
  #[validate(nested)]
  pub enforcement: RuleEnforcementInput,
}

impl RuleDocumentInput {
  pub(super) fn into_rule(self, override_id: Option<RuleId>) -> ApiResult<Rule> {
    self.validate().map_err(map_validation_errors)?;

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

#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct RuleMetaInput {
  #[validate(length(min = 1, max = 120))]
  pub name: String,
  #[validate(length(max = 1000))]
  pub description: Option<String>,
  pub version: semver::Version,
  #[validate(length(min = 1, max = 120))]
  pub autor: String,
  #[validate(length(max = 32))]
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

#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct RuleStateInput {
  pub mode: RuleMode,
  #[validate(nested)]
  pub audit: RuleAuditInput,
}

impl RuleStateInput {
  fn into_domain(self) -> RuleState {
    RuleState { mode: self.mode, audit: self.audit.into_domain() }
  }
}

#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
#[validate(schema(function = "validate_audit_input"))]
pub struct RuleAuditInput {
  pub created_at_ms: TimestampMs,
  pub updated_at_ms: TimestampMs,
  #[validate(length(min = 1, max = 120))]
  pub created_by: Option<String>,
  #[validate(length(min = 1, max = 120))]
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

#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
#[validate(schema(function = "validate_schedule_input"))]
pub struct RuleScheduleInput {
  pub active_from_ms: Option<TimestampMs>,
  pub active_until_ms: Option<TimestampMs>,
}

impl RuleScheduleInput {
  fn into_domain(self) -> RuleSchedule {
    RuleSchedule { active_from_ms: self.active_from_ms, active_until_ms: self.active_until_ms }
  }
}

#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct RolloutPolicyInput {
  #[validate(range(max = 100))]
  pub percent: u8,
}

impl RolloutPolicyInput {
  fn into_domain(self) -> RolloutPolicy {
    RolloutPolicy { percent: self.percent }
  }
}

#[derive(Debug, Deserialize, Validate)]
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

#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct RuleEnforcementInput {
  #[validate(range(min = 1.0, max = 10.0))]
  pub score_impact: f32,
  pub action: RuleAction,
  pub severity: Severity,
  #[validate(length(min = 1, max = 64))]
  pub tags: Vec<String>,
  #[validate(range(min = 1, max = 86_400_000))]
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
  validate_rule_evaluation(&rule.evaluation)?;
  Ok(())
}

pub(super) fn parse_patch_value<T>(field: &str, value: &Value) -> ApiResult<T>
where
  T: serde::de::DeserializeOwned,
{
  serde_json::from_value(value.clone())
    .map_err(|_| ApiError::validation(field, "invalid type or value"))
}

fn validate_schedule_input(schedule: &RuleScheduleInput) -> Result<(), ValidationError> {
  if let (Some(from), Some(until)) = (schedule.active_from_ms, schedule.active_until_ms)
    && until.as_u64() <= from.as_u64()
  {
    return Err(ValidationError::new("active_until_must_be_greater_than_active_from"));
  }
  Ok(())
}

fn validate_audit_input(audit: &RuleAuditInput) -> Result<(), ValidationError> {
  if audit.updated_at_ms.as_u64() < audit.created_at_ms.as_u64() {
    return Err(ValidationError::new("updated_at_must_be_greater_or_equal_created_at"));
  }
  Ok(())
}

fn map_validation_errors(errors: ValidationErrors) -> ApiError {
  let mut messages = Vec::new();
  collect_validation_messages("", &errors, &mut messages);

  if let Some((field, message)) = messages.into_iter().next() {
    ApiError::validation(field, message)
  } else {
    ApiError::validation("request", "invalid request payload")
  }
}

fn collect_validation_messages(
  prefix: &str,
  errors: &ValidationErrors,
  out: &mut Vec<(String, String)>,
) {
  for (field, kind) in errors.errors() {
    let path = if prefix.is_empty() {
      field.to_string()
    } else {
      format!("{prefix}.{field}")
    };

    match kind {
      ValidationErrorsKind::Field(field_errors) => {
        for field_error in field_errors {
          let message = field_error
            .message
            .clone()
            .unwrap_or_else(|| field_error.code.to_string().into())
            .to_string();
          out.push((path.clone(), message));
        }
      }
      ValidationErrorsKind::Struct(struct_errors) => {
        collect_validation_messages(&path, struct_errors, out);
      }
      ValidationErrorsKind::List(list_errors) => {
        for (index, nested) in list_errors {
          let indexed = format!("{path}[{index}]");
          collect_validation_messages(&indexed, nested, out);
        }
      }
    }
  }
}
