use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::DomainError;

/// Maximum allowable depth for nested logical expressions to prevent stack overflow.
const MAX_EXPRESSION_DEPTH: usize = 20;
/// Maximum length for arrays within an expression to prevent excessive iteration.
const MAX_ARRAY_LEN: usize = 128;
/// Maximum total number of nodes allowed in a single expression tree.
const MAX_NODE_COUNT: usize = 1_024;
/// Maximum character length for variable path strings.
const MAX_STRING_VAR_LEN: usize = 512;

/// Allowed root namespaces for variable resolution.
///
/// Expressions are restricted to these roots to ensure strict data sandboxing.
pub const JSONLOGIC_ROOT_VARS: [&str; 7] =
  ["event", "payload", "context", "signals", "extensions", "transaction", "device"];

/// Valid operators supported by the rule engine runtime.
///
/// This list is aligned with the `datalogic-rs` (v4.0.x) specification.
pub const ALLOWED_OPERATORS: [&str; 61] = [
  "var",
  "val",
  "==",
  "===",
  "!=",
  "!==",
  ">",
  ">=",
  "<",
  "<=",
  "!",
  "!!",
  "and",
  "or",
  "if",
  "?:",
  "+",
  "-",
  "*",
  "/",
  "%",
  "max",
  "min",
  "cat",
  "substr",
  "in",
  "merge",
  "filter",
  "map",
  "reduce",
  "all",
  "some",
  "none",
  "sort",
  "slice",
  "missing",
  "missing_some",
  "try",
  "throw",
  "type",
  "datetime",
  "timestamp",
  "parse_date",
  "format_date",
  "date_diff",
  "now",
  "abs",
  "ceil",
  "floor",
  "length",
  "preserve",
  "starts_with",
  "ends_with",
  "upper",
  "lower",
  "trim",
  "split",
  "exists",
  "switch",
  "match",
  "??",
];

/// A validated logical expression tree.
///
/// `RuleExpression` wraps a JSON-based abstract syntax tree (AST). Upon creation,
/// the expression is statically analyzed to ensure it adheres to safety limits,
/// uses only authorized operators, and accesses permitted data namespaces.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RuleExpression {
  value: Value,
}

impl RuleExpression {
  /// Creates a new `RuleExpression` after performing deep structural validation.
  ///
  /// # Errors
  ///
  /// Returns a [`DomainError::InvalidRuleExpression`] if the AST exceeds depth/node limits
  /// or contains unauthorized operators.
  pub fn new(value: Value) -> Result<Self, DomainError> {
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

  /// Returns `true` if the given path starts with a permitted root namespace.
  pub fn is_root_var(path: &str) -> bool {
    let root = path.split('.').next().unwrap_or_default();
    JSONLOGIC_ROOT_VARS.contains(&root)
  }

  /// Performs a static analysis of all variable paths within the expression.
  ///
  /// # Errors
  ///
  /// Returns an error if any path references a root not defined in [`JSONLOGIC_ROOT_VARS`].
  pub fn validate_vars(&self) -> Result<(), DomainError> {
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

/// Recursively validates the AST structure against safety constraints.
fn validate_expression(value: &Value) -> Result<(), DomainError> {
  let mut nodes = 0usize;
  validate_node(value, 0, &mut nodes)
}

/// Validates a specific node within the expression tree.
///
/// Checks for:
/// - Maximum recursion depth ([`MAX_EXPRESSION_DEPTH`]).
/// - Total node count ([`MAX_NODE_COUNT`]).
/// - Operator whitelist ([`ALLOWED_OPERATORS`]).
/// - Variable path integrity.
fn validate_node(value: &Value, depth: usize, nodes: &mut usize) -> Result<(), DomainError> {
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

        if !ALLOWED_OPERATORS.contains(&op.as_str()) {
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

/// Traverses the AST to collect all variable access paths.
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
