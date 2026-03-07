use rve_core::domain::{
  DomainError,
  common::{RuleId, Score, Severity, TimestampMs},
  rule::{
    RolloutPolicy, Rule, RuleAction, RuleAudit, RuleDecision, RuleDefinition, RuleEnforcement,
    RuleEvaluation, RuleExpression, RuleIdentity, RulePolicy, RuleSchedule, RuleState,
    mode::RuleMode,
  },
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use utoipa::IntoParams;
use validator::{Validate, ValidationError, ValidationErrors, ValidationErrorsKind};

use super::errors::{ApiError, ApiResult, ValidationIssue, ValidationReport};
use super::logic_validation::validate_rule_evaluation;

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct Pagination {
  /// Page number (1-based).
  #[param(default = 1, minimum = 1)]
  pub page: Option<u32>,

  /// Number of items per page. Max 100.
  #[param(default = 20, minimum = 1, maximum = 100)]
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
pub struct RuleDocumentInput {
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

    let identity = self.meta.into_domain();
    let policy = RulePolicy::new(
      self.state.into_domain()?,
      self.schedule.into_domain()?,
      self.rollout.into_domain()?,
    )
    .map_err(|error| map_domain_error("policy", error.into()))?;
    let definition = RuleDefinition::new(self.evaluation.into_domain()?)
      .map_err(|error| map_domain_error("definition", error))?;
    let outcome = RuleDecision::new(self.enforcement.into_domain()?);

    let rule = Rule::new(
      override_id.or(self.id).unwrap_or_else(RuleId::new_v7),
      identity,
      policy,
      definition,
      outcome,
    )
    .map_err(|error| map_domain_error("rule", error))?;

    validate_rule(&rule)?;
    Ok(rule)
  }
}

#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct RuleMetaInput {
  #[validate(length(min = 3, max = 80))]
  pub code: Option<String>,
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
  fn into_domain(self) -> RuleIdentity {
    RuleIdentity {
      code: self.code,
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
  fn into_domain(self) -> ApiResult<RuleState> {
    RuleState::new(self.mode, self.audit.into_domain())
      .map_err(|error| map_domain_error("state", error.into()))
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
  fn into_domain(self) -> ApiResult<RuleSchedule> {
    RuleSchedule::new(self.active_from_ms, self.active_until_ms)
      .map_err(|error| map_domain_error("schedule", error.into()))
  }
}

#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct RolloutPolicyInput {
  #[validate(range(max = 100))]
  pub percent: u8,
}

impl RolloutPolicyInput {
  fn into_domain(self) -> ApiResult<RolloutPolicy> {
    RolloutPolicy::new(self.percent).map_err(|error| map_domain_error("rollout", error.into()))
  }
}

#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct RuleEvaluationInput {
  pub condition: Value,
  pub logic: Value,
}

impl RuleEvaluationInput {
  fn into_domain(self) -> ApiResult<RuleEvaluation> {
    let condition = RuleExpression::new(normalize_expression_aliases(self.condition))
      .map_err(|error| map_domain_error("evaluation.condition", error))?;
    let logic = RuleExpression::new(normalize_expression_aliases(self.logic))
      .map_err(|error| map_domain_error("evaluation.logic", error))?;

    Ok(RuleEvaluation { condition, logic })
  }
}

fn normalize_expression_aliases(value: Value) -> Value {
  match value {
    Value::Object(values) => {
      let mut normalized = Map::new();

      for (op, arg) in values {
        let normalized_arg = normalize_expression_aliases(arg);
        match op.as_str() {
          "=" => {
            normalized.insert("==".to_owned(), normalized_arg);
          }
          "not_in" => {
            normalized.insert("!".to_owned(), json!({ "in": normalized_arg }));
          }
          _ => {
            normalized.insert(op, normalized_arg);
          }
        }
      }

      Value::Object(normalized)
    }
    Value::Array(values) => {
      Value::Array(values.into_iter().map(normalize_expression_aliases).collect())
    }
    _ => value,
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
  rule.policy().state().validate().map_err(|error| map_domain_error("state", error.into()))?;
  rule
    .policy()
    .schedule()
    .validate()
    .map_err(|error| map_domain_error("schedule", error.into()))?;
  rule.policy().rollout().validate().map_err(|error| map_domain_error("rollout", error.into()))?;
  validate_rule_evaluation(rule.evaluation())?;
  Ok(())
}

pub(super) fn collect_rule_warnings(rule: &Rule) -> Vec<ValidationIssue> {
  let mut warnings = Vec::new();

  if matches!(rule.evaluation().condition.as_value(), Value::Bool(true)) {
    warnings.push(ValidationIssue {
      path: "evaluation.condition".to_owned(),
      message: "condition is always true; rule always evaluates logic".to_owned(),
    });
  }

  if rule.enforcement().tags.is_empty() {
    warnings.push(ValidationIssue {
      path: "enforcement.tags".to_owned(),
      message: "empty tags reduce observability in dashboards".to_owned(),
    });
  }

  warnings
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
  let mut issues = Vec::new();
  collect_validation_messages("", &errors, &mut issues);

  let errors = if issues.is_empty() {
    vec![ValidationIssue {
      path: "request".to_owned(),
      message: "invalid request payload".to_owned(),
    }]
  } else {
    issues
  };

  ApiError::Unprocessable(ValidationReport { errors, warnings: Vec::new() })
}

fn collect_validation_messages(
  prefix: &str,
  errors: &ValidationErrors,
  out: &mut Vec<ValidationIssue>,
) {
  for (field, kind) in errors.errors() {
    let path = if prefix.is_empty() { field.to_string() } else { format!("{prefix}.{field}") };

    match kind {
      ValidationErrorsKind::Field(field_errors) => {
        for field_error in field_errors {
          let message = field_error
            .message
            .clone()
            .unwrap_or_else(|| field_error.code.to_string().into())
            .to_string();
          out.push(ValidationIssue { path: path.clone(), message });
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

fn map_domain_error(field: &str, error: DomainError) -> ApiError {
  ApiError::validation(field, error.to_string())
}

#[cfg(test)]
mod tests {
  use serde_json::json;

  use super::RuleDocumentInput;
  use crate::http::api_v1::rules::errors::ApiError;

  fn valid_rule_payload() -> serde_json::Value {
    json!({
      "meta": {
        "code": "RL01",
        "name": "High Value Payment",
        "description": "flags high value transaction",
        "version": "1.0.0",
        "autor": "RiskOps",
        "tags": ["high_value", "payments"]
      },
      "state": {
        "mode": "active",
        "audit": {
          "created_at_ms": 1730000000000u64,
          "updated_at_ms": 1730000001000u64,
          "created_by": "alice",
          "updated_by": "alice"
        }
      },
      "schedule": {
        "active_from_ms": 1730000000000u64,
        "active_until_ms": 1731000000000u64
      },
      "rollout": { "percent": 50 },
      "evaluation": {
        "condition": true,
        "logic": {">": [{"var": "payload.money.value"}, 1000]}
      },
      "enforcement": {
        "score_impact": 6.5,
        "action": "review",
        "severity": "high",
        "tags": ["financial_fraud"],
        "cooldown_ms": 60000
      }
    })
  }

  #[test]
  fn rejects_unknown_fields_in_payload() {
    let mut payload = valid_rule_payload();
    payload["unknown"] = json!(true);

    let parsed = serde_json::from_value::<RuleDocumentInput>(payload);
    assert!(parsed.is_err());
  }

  #[test]
  fn returns_unprocessable_for_invalid_schedule() {
    let mut payload = valid_rule_payload();
    payload["schedule"]["active_until_ms"] = json!(1720000000000u64);

    let parsed: RuleDocumentInput = serde_json::from_value(payload).expect("payload parses");
    let result = parsed.into_rule(None);

    match result {
      Err(ApiError::Unprocessable(report)) => {
        assert!(!report.errors.is_empty());
      }
      _ => panic!("expected unprocessable validation error"),
    }
  }

  #[test]
  fn normalizes_legacy_expression_operators() {
    let payload = json!({
      "meta": {
        "code": "RL01",
        "name": "High Value Payment",
        "description": "legacy operators are normalized",
        "version": "1.0.0",
        "autor": "RiskOps"
      },
      "state": {
        "mode": "active",
        "audit": {
          "created_at_ms": 1730000000000u64,
          "updated_at_ms": 1730000001000u64,
          "created_by": "alice",
          "updated_by": "alice"
        }
      },
      "schedule": {
        "active_from_ms": 1730000000000u64
      },
      "rollout": { "percent": 50 },
      "evaluation": {
        "condition": {"=": [{"var": "payload.money.value"}, 1000]},
        "logic": {
          "and": [
            {"not_in": [{"var": "payload.money.ccy"}, ["USD", "EUR"]]},
            {"=": [{"var": "context.fin.current_hour_count"}, 0]}
          ]
        }
      },
      "enforcement": {
        "score_impact": 6.5,
        "action": "review",
        "severity": "high",
        "tags": ["financial_fraud"],
        "cooldown_ms": 60000
      }
    });

    let parsed: RuleDocumentInput = serde_json::from_value(payload).expect("payload parses");
    let result = parsed.into_rule(None);
    assert!(result.is_ok(), "legacy operators must be normalized before validation: {result:?}");
  }
}
