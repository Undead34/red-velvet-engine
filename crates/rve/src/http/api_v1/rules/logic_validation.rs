use datalogic_rs::DataLogic;
use rve_core::domain::rule::RuleEvaluation;
use serde_json::Value;

use super::errors::{ApiError, ApiResult};

const ALLOWED_ROOTS: [&str; 7] =
  ["event", "payload", "context", "signals", "extensions", "transaction", "device"];

pub(super) fn validate_rule_evaluation(evaluation: &RuleEvaluation) -> ApiResult<()> {
  validate_vars("evaluation.condition", &evaluation.condition)?;
  validate_vars("evaluation.logic", &evaluation.logic)?;

  let logic = DataLogic::new();
  logic.compile(&evaluation.condition).map_err(|err| {
    ApiError::validation("evaluation.condition", format!("invalid expression: {err}"))
  })?;
  logic.compile(&evaluation.logic).map_err(|err| {
    ApiError::validation("evaluation.logic", format!("invalid expression: {err}"))
  })?;

  Ok(())
}

fn validate_vars(field: &'static str, value: &Value) -> ApiResult<()> {
  let mut vars = Vec::new();
  collect_var_paths(value, &mut vars);

  for path in vars {
    let root = path.split('.').next().unwrap_or_default();
    if !ALLOWED_ROOTS.contains(&root) {
      return Err(ApiError::validation(
        field,
        format!("var path '{path}' is not allowed; expected roots: {}", ALLOWED_ROOTS.join(", ")),
      ));
    }
  }

  Ok(())
}

fn collect_var_paths<'a>(value: &'a Value, vars: &mut Vec<&'a str>) {
  match value {
    Value::Object(map) => {
      if let Some(var_value) = map.get("var") {
        match var_value {
          Value::String(path) => vars.push(path.as_str()),
          Value::Array(items) => {
            if let Some(Value::String(path)) = items.first() {
              vars.push(path.as_str());
            }
          }
          _ => {}
        }
      }

      for nested in map.values() {
        collect_var_paths(nested, vars);
      }
    }
    Value::Array(values) => {
      for nested in values {
        collect_var_paths(nested, vars);
      }
    }
    _ => {}
  }
}

#[cfg(test)]
mod tests {
  use serde_json::json;

  use rve_core::domain::rule::RuleEvaluation;

  use super::validate_rule_evaluation;
  use crate::http::api_v1::rules::errors::ApiError;

  #[test]
  fn rejects_disallowed_var_roots() {
    let evaluation = RuleEvaluation {
      condition: json!(true),
      logic: json!({">": [{"var": "config.latam_countries"}, 0]}),
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
