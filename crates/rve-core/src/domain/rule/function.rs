use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::DomainError;

/// Logical function categories used by fraud rule outcomes.
///
/// These kinds are domain-level abstractions. Adapters can map them to
/// concrete runtime implementations (e.g. dataflow task function names).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FunctionKind {
  Parse,
  Validate,
  Filter,
  Map,
  Enrich,
  Publish,
  Custom,
}

impl FunctionKind {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Parse => "parse",
      Self::Validate => "validate",
      Self::Filter => "filter",
      Self::Map => "map",
      Self::Enrich => "enrich",
      Self::Publish => "publish",
      Self::Custom => "custom",
    }
  }
}

/// Runtime function specification attached to a rule outcome.
///
/// The `config` payload remains flexible by design, but must be a JSON object
/// and satisfy minimum domain-level invariants.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleFunctionSpec {
  pub kind: FunctionKind,
  pub config: Value,
}

impl RuleFunctionSpec {
  /// Builds a validated function specification.
  ///
  /// # Errors
  ///
  /// Returns [`DomainError::InvalidRuleFunctionConfig`] when `config` does not
  /// satisfy minimal shape and kind-specific invariants.
  pub fn new(kind: FunctionKind, config: Value) -> Result<Self, DomainError> {
    validate_function_config(&kind, &config)?;
    Ok(Self { kind, config })
  }

  /// Validates an existing function specification.
  ///
  /// # Errors
  ///
  /// Returns [`DomainError::InvalidRuleFunctionConfig`] for invalid configs.
  pub fn validate(&self) -> Result<(), DomainError> {
    validate_function_config(&self.kind, &self.config)
  }
}

fn validate_function_config(kind: &FunctionKind, config: &Value) -> Result<(), DomainError> {
  let object = config.as_object().ok_or_else(|| DomainError::InvalidRuleFunctionConfig {
    kind: kind.as_str().to_owned(),
    reason: "config must be a JSON object".to_owned(),
  })?;

  if matches!(kind, FunctionKind::Custom) {
    let has_name = object.get("name").and_then(Value::as_str).is_some_and(|name| !name.is_empty());
    if !has_name {
      return Err(DomainError::InvalidRuleFunctionConfig {
        kind: kind.as_str().to_owned(),
        reason: "custom functions require non-empty `config.name`".to_owned(),
      });
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use serde_json::json;

  use crate::domain::{
    DomainError,
    rule::{FunctionKind, RuleFunctionSpec},
  };

  #[test]
  fn accepts_known_kind_with_object_config() {
    let spec = RuleFunctionSpec::new(
      FunctionKind::Validate,
      json!({
        "rules": [{"logic": true, "message": "ok"}]
      }),
    );
    assert!(spec.is_ok());
  }

  #[test]
  fn rejects_non_object_config() {
    let error = RuleFunctionSpec::new(FunctionKind::Map, json!(true)).expect_err("must fail");
    assert!(matches!(error, DomainError::InvalidRuleFunctionConfig { .. }));
  }

  #[test]
  fn rejects_custom_without_name() {
    let error = RuleFunctionSpec::new(FunctionKind::Custom, json!({"input": {"x": 1}}))
      .expect_err("must fail");
    assert!(matches!(error, DomainError::InvalidRuleFunctionConfig { .. }));
  }
}
