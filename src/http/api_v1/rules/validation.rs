use datalogic_rs::DataLogic;
use rve_core::domain::{
  DomainError,
  rule::{Rule, RuleEvaluation, RuleExpression},
};
use serde_json::Value;

use super::errors::{ApiError, ApiResult, ValidationIssue};

pub fn validate_rule(rule: &Rule) -> ApiResult<()> {
  rule
    .policy()
    .state()
    .validate()
    .map_err(|error| ApiError::validation("state", error.to_string()))?;
  rule
    .policy()
    .schedule()
    .validate()
    .map_err(|error| ApiError::validation("schedule", error.to_string()))?;
  rule
    .policy()
    .rollout()
    .validate()
    .map_err(|error| ApiError::validation("rollout", error.to_string()))?;
  validate_rule_evaluation(rule.evaluation())?;
  Ok(())
}

pub fn collect_rule_warnings(rule: &Rule) -> Vec<ValidationIssue> {
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

pub fn validate_rule_evaluation(evaluation: &RuleEvaluation) -> ApiResult<()> {
  validate_vars("evaluation.condition", evaluation.condition.as_value())?;
  validate_vars("evaluation.logic", evaluation.logic.as_value())?;

  let logic = DataLogic::new();
  logic.compile(evaluation.condition.as_value()).map_err(|err| {
    ApiError::validation("evaluation.condition", format!("invalid expression: {err}"))
  })?;
  logic.compile(evaluation.logic.as_value()).map_err(|err| {
    ApiError::validation("evaluation.logic", format!("invalid expression: {err}"))
  })?;

  Ok(())
}

fn validate_vars(field: &'static str, value: &Value) -> ApiResult<()> {
  match RuleExpression::new(value.clone()) {
    Ok(expression) => {
      expression
        .validate_vars()
        .map_err(|error: DomainError| ApiError::validation(field, error.to_string()))?;
    }
    Err(error) => return Err(ApiError::validation(field, error.to_string())),
  }

  Ok(())
}
