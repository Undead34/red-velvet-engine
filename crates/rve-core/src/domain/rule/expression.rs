use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::{DomainError, DomainResult};

/// Maximum nesting depth accepted by the validator.
const MAX_EXPRESSION_DEPTH: usize = 20;
/// Maximum array length accepted by the validator.
const MAX_ARRAY_LEN: usize = 128;
/// Maximum number of object nodes accepted by the validator.
const MAX_NODE_COUNT: usize = 1_024;
/// Maximum length for `var` paths.
const MAX_STRING_VAR_LEN: usize = 512;

/// Allowed root namespaces for `var` paths.
pub const JSONLOGIC_ROOT_VARS: [&str; 8] =
  ["event", "payload", "context", "features", "signals", "extensions", "transaction", "device"];

// ── Operator groups ───────────────────────────────────────────────────────────
// Each group documents a semantic category so the team knows exactly where to
// add operators without reading the whole allowlist.

/// Core standard operators natively supported by the logic engine.
const OP_CORE: &[&str] = &[
  "var",
  "val",
  "if",
  "?:",
  "missing",
  "missing_some",
  "try",
  "throw",
  "type",
  "exists",
  "switch",
  "??",
];

/// Comparison and mathematical operators.
const OP_MATH_CMP: &[&str] = &[
  "==", "===", "!=", "!==", ">", ">=", "<", "<=", "+", "-", "*", "/", "%", "max", "min", "abs",
  "ceil", "floor",
];

/// Logical operators.
const OP_LOGIC: &[&str] = &["!", "!!", "and", "or"];

/// String manipulation operators.
const OP_STRING: &[&str] =
  &["cat", "substr", "starts_with", "ends_with", "upper", "lower", "trim", "split", "match"];

/// Array and collection operators.
const OP_ARRAY: &[&str] = &[
  "in", "merge", "filter", "map", "reduce", "all", "some", "none", "sort", "slice", "length",
  "preserve",
];

/// Temporal operators (dates, timestamps, recency checks).
const OP_TEMPORAL: &[&str] = &[
  "datetime",
  "timestamp",
  "parse_date",
  "format_date",
  "date_diff",
  "now",
  // Recency helpers (runtime must register these functions).
  "time_since",
  "is_recent",
];

/// Fraud-domain specific operators (lists, networks, etc.).
const OP_DOMAIN: &[&str] = &["in_list", "not_in_list", "ip_in_range"];

/// A validated JSONLogic expression.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RuleExpression {
  value: Value,
}

impl RuleExpression {
  /// Returns `true` if `op` is recognised by any operator group.
  pub fn is_allowed_operator(op: &str) -> bool {
    OP_CORE.contains(&op)
      || OP_MATH_CMP.contains(&op)
      || OP_LOGIC.contains(&op)
      || OP_STRING.contains(&op)
      || OP_ARRAY.contains(&op)
      || OP_TEMPORAL.contains(&op)
      || OP_DOMAIN.contains(&op)
  }

  /// Returns all operator groups with their names and operator lists.
  ///
  /// Useful for UIs that need to render operator palettes grouped by category.
  pub fn operator_groups() -> Vec<(&'static str, &'static [&'static str])> {
    vec![
      ("core", OP_CORE),
      ("math_cmp", OP_MATH_CMP),
      ("logic", OP_LOGIC),
      ("string", OP_STRING),
      ("array", OP_ARRAY),
      ("temporal", OP_TEMPORAL),
      ("domain", OP_DOMAIN),
    ]
  }

  /// Creates an expression after structural validation.
  ///
  /// # Errors
  ///
  /// Returns [`DomainError::InvalidRuleExpression`] if the expression exceeds
  /// configured limits or uses unsupported operators.
  pub fn new(value: Value) -> DomainResult<Self> {
    validate_expression(&value)?;
    Ok(Self { value })
  }

  /// Returns a reference to the underlying JSON value.
  pub fn as_value(&self) -> &Value {
    &self.value
  }

  /// Consumes the expression and returns the underlying JSON value.
  pub fn into_value(self) -> Value {
    self.value
  }

  /// Returns `true` if `path` starts with an allowed root namespace.
  pub fn is_root_var(path: &str) -> bool {
    let root = path.split('.').next().unwrap_or_default();
    JSONLOGIC_ROOT_VARS.contains(&root)
  }

  /// Validates all `var` paths in the expression.
  ///
  /// # Errors
  ///
  /// Returns an error if any `var` path has a disallowed root.
  pub fn validate_vars(&self) -> DomainResult<()> {
    let mut vars = Vec::new();
    collect_var_paths(&self.value, &mut vars);
    for path in vars {
      if !Self::is_root_var(path) {
        return Err(DomainError::InvalidRuleExpression(format!(
          "disallowed var root in '{path}'; expected one of {}",
          JSONLOGIC_ROOT_VARS.join(", ")
        )));
      }
    }
    Ok(())
  }
}

/// Validates the expression tree.
fn validate_expression(value: &Value) -> DomainResult<()> {
  let mut nodes = 0usize;
  validate_node(value, 0, &mut nodes)
}

/// Validates a single node in the expression tree.
fn validate_node(value: &Value, depth: usize, nodes: &mut usize) -> DomainResult<()> {
  if depth > MAX_EXPRESSION_DEPTH {
    return Err(DomainError::InvalidRuleExpression("expression exceeds max depth".to_owned()));
  }

  match value {
    Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => Ok(()),
    Value::Array(items) => {
      if items.is_empty() {
        return Ok(());
      }
      if items.len() > MAX_ARRAY_LEN {
        return Err(DomainError::InvalidRuleExpression(
          "expression array is too large for safe evaluation".to_owned(),
        ));
      }
      for item in items {
        validate_node(item, depth + 1, nodes)?;
      }
      Ok(())
    }
    Value::Object(map) => {
      *nodes += 1;
      if *nodes > MAX_NODE_COUNT {
        return Err(DomainError::InvalidRuleExpression(
          "expression has too many nodes for safe evaluation".to_owned(),
        ));
      }

      if map.is_empty() {
        return Err(DomainError::InvalidRuleExpression(
          "empty object expression is invalid".to_owned(),
        ));
      }

      if map.len() > 4 {
        return Err(DomainError::InvalidRuleExpression(
          "too many keys in expression object".to_owned(),
        ));
      }

      let has_var = map.contains_key("var");
      for (op, nested) in map {
        if *op == "var" {
          match nested {
            Value::String(path) => {
              if path.is_empty() || path.len() > MAX_STRING_VAR_LEN {
                return Err(DomainError::InvalidRuleExpression(
                  "var path is empty or too long".to_owned(),
                ));
              }
            }
            Value::Array(values) => {
              let Some(Value::String(path)) = values.first() else {
                return Err(DomainError::InvalidRuleExpression(
                  "var must be a string path".to_owned(),
                ));
              };
              if path.is_empty() || path.len() > MAX_STRING_VAR_LEN {
                return Err(DomainError::InvalidRuleExpression(
                  "var path is empty or too long".to_owned(),
                ));
              }
            }
            _ => {
              return Err(DomainError::InvalidRuleExpression("var must be string path".to_owned()));
            }
          }
          continue;
        }

        if !RuleExpression::is_allowed_operator(op.as_str()) {
          return Err(DomainError::InvalidRuleExpression(format!("unsupported operator '{op}'")));
        }

        validate_node(nested, depth + 1, nodes)?;
      }

      if has_var && map.len() > 1 {
        return Err(DomainError::InvalidRuleExpression(
          "var expression object cannot contain operator keys".to_owned(),
        ));
      }

      Ok(())
    }
  }
}

/// Collects all `var` paths from the expression tree.
fn collect_var_paths<'a>(value: &'a Value, vars: &mut Vec<&'a str>) {
  match value {
    Value::Object(map) => {
      if let Some(Value::String(path)) = map.get("var") {
        vars.push(path.as_str());
      } else if let Some(Value::Array(items)) = map.get("var")
        && let Some(Value::String(path)) = items.first()
      {
        vars.push(path.as_str());
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
  use super::*;
  use serde_json::json;

  // ── Operator coverage ──────────────────────────────────────────────────────

  #[test]
  fn is_allowed_operator_accepts_all_core() {
    for op in OP_CORE {
      assert!(RuleExpression::is_allowed_operator(op), "core operator '{op}' not recognised");
    }
  }

  #[test]
  fn is_allowed_operator_accepts_all_math_cmp() {
    for op in OP_MATH_CMP {
      assert!(RuleExpression::is_allowed_operator(op), "math/cmp operator '{op}' not recognised");
    }
  }

  #[test]
  fn is_allowed_operator_accepts_all_logic() {
    for op in OP_LOGIC {
      assert!(RuleExpression::is_allowed_operator(op), "logic operator '{op}' not recognised");
    }
  }

  #[test]
  fn is_allowed_operator_accepts_all_string() {
    for op in OP_STRING {
      assert!(RuleExpression::is_allowed_operator(op), "string operator '{op}' not recognised");
    }
  }

  #[test]
  fn is_allowed_operator_accepts_all_array() {
    for op in OP_ARRAY {
      assert!(RuleExpression::is_allowed_operator(op), "array operator '{op}' not recognised");
    }
  }

  #[test]
  fn is_allowed_operator_accepts_all_temporal() {
    for op in OP_TEMPORAL {
      assert!(RuleExpression::is_allowed_operator(op), "temporal operator '{op}' not recognised");
    }
  }

  #[test]
  fn is_allowed_operator_accepts_all_domain() {
    for op in OP_DOMAIN {
      assert!(RuleExpression::is_allowed_operator(op), "domain operator '{op}' not recognised");
    }
  }

  #[test]
  fn is_allowed_operator_rejects_legacy_operators() {
    assert!(!RuleExpression::is_allowed_operator("="));
    assert!(!RuleExpression::is_allowed_operator("not_in"));
    assert!(!RuleExpression::is_allowed_operator("not"));
  }

  #[test]
  fn is_allowed_operator_rejects_garbage() {
    assert!(!RuleExpression::is_allowed_operator(""));
    assert!(!RuleExpression::is_allowed_operator("  "));
    assert!(!RuleExpression::is_allowed_operator("foo"));
    assert!(!RuleExpression::is_allowed_operator("eval"));
  }

  // ── Expression validation ──────────────────────────────────────────────────

  #[test]
  fn valid_simple_expression() {
    let val = json!({ "==": [1, 1] });
    assert!(RuleExpression::new(val).is_ok());
  }

  #[test]
  fn rejects_empty_object() {
    let val = json!({});
    let err = RuleExpression::new(val).unwrap_err();
    assert!(err.to_string().contains("empty object"));
  }

  #[test]
  fn rejects_unsupported_operator() {
    let val = json!({ "eval": "dangerous" });
    let err = RuleExpression::new(val).unwrap_err();
    assert!(err.to_string().contains("unsupported operator"));
  }

  #[test]
  fn rejects_too_many_keys() {
    let val = json!({ "a": 1, "b": 2, "c": 3, "d": 4, "e": 5 });
    let err = RuleExpression::new(val).unwrap_err();
    assert!(err.to_string().contains("too many keys"));
  }

  #[test]
  fn rejects_var_mixed_with_other_keys() {
    let val = json!({ "var": "payload.foo", "and": true });
    let err = RuleExpression::new(val).unwrap_err();
    assert!(err.to_string().contains("cannot contain operator keys"));
  }

  #[test]
  fn rejects_empty_var_path() {
    let val = json!({ "var": "" });
    let err = RuleExpression::new(val).unwrap_err();
    assert!(err.to_string().contains("var path is empty"));
  }

  #[test]
  fn rejects_too_deep_expression() {
    let mut deep = json!(null);
    for _ in 0..=MAX_EXPRESSION_DEPTH {
      deep = json!({ "and": [deep] });
    }
    let err = RuleExpression::new(deep).unwrap_err();
    assert!(err.to_string().contains("max depth"));
  }

  #[test]
  fn rejects_array_too_large() {
    let val = json!({ "and": (0..=MAX_ARRAY_LEN).map(|i| json!(i)).collect::<Vec<_>>() });
    let err = RuleExpression::new(val).unwrap_err();
    assert!(err.to_string().contains("array is too large"));
  }

  #[test]
  fn rejects_disallowed_var_root() {
    let expr = RuleExpression::new(json!({ "var": "custom.root" })).unwrap();
    let err = expr.validate_vars().unwrap_err();
    assert!(err.to_string().contains("disallowed var root"));
  }

  #[test]
  fn accepts_valid_var_root() {
    for root in &JSONLOGIC_ROOT_VARS {
      let path = format!("{root}.something.nested");
      let expr = RuleExpression::new(json!({ "var": path })).unwrap();
      assert!(expr.validate_vars().is_ok(), "root '{root}' should be allowed");
    }
  }

  // ── New domain operators ───────────────────────────────────────────────────

  #[test]
  fn accepts_new_temporal_operators() {
    for op in &["time_since", "is_recent"] {
      let val = json!({ *op: [{"var": "features.fin.last_seen_at"}] });
      assert!(RuleExpression::new(val).is_ok(), "new temporal operator '{op}' rejected");
    }
  }

  #[test]
  fn accepts_new_domain_operators() {
    for op in &["in_list", "not_in_list", "ip_in_range"] {
      let val = json!({ *op: [{"var": "payload.parties.originator.email"}, "global_blacklist"] });
      assert!(RuleExpression::new(val).is_ok(), "domain operator '{op}' rejected");
    }
  }
}
