use datalogic_rs::DataLogic;
use rve_core::domain::{
  DomainError,
  rule::{RuleEvaluation, RuleExpression},
};
use serde_json::Value;

use super::errors::{ApiError, ApiResult};

pub(super) fn validate_rule_evaluation(evaluation: &RuleEvaluation) -> ApiResult<()> {
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
