use datalogic_rs::DataLogic;
use rve_core::domain::{
  rule::{RuleEvaluation, RuleExpression},
  DomainError,
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

#[cfg(test)]
mod tests {
  use serde_json::json;

  use rve_core::domain::rule::{RuleEvaluation, RuleExpression};

  use super::validate_rule_evaluation;
  use crate::http::api_v1::rules::errors::ApiError;

  #[test]
  fn rejects_disallowed_var_roots() {
    let evaluation = RuleEvaluation {
      condition: RuleExpression::new(json!(true)).expect("valid condition"),
      logic: RuleExpression::new(json!({">": [{"var": "config.latam_countries"}, 0]}))
        .expect("valid logic"),
    };

    let result = validate_rule_evaluation(&evaluation);
    match result {
      Err(ApiError::Unprocessable(report)) => {
        assert!(!report.errors.is_empty());
      }
      _ => panic!("expected unprocessable error for invalid var root"),
    }
  }
}
