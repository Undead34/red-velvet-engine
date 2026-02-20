use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Logic payload evaluated by the rules engine.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleEvaluation {
  /// Fast guard expression checked before full logic.
  pub condition: Value,
  /// Main expression executed when condition passes.
  pub logic: Value,
}
