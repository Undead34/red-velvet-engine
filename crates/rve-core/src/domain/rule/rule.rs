use serde::{Deserialize, Serialize};

use crate::domain::common::RuleId;

use super::{RolloutPolicy, RuleEnforcement, RuleEvaluation, RuleMeta, RuleSchedule, RuleState};

/// Aggregate root for a fraud rule definition.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rule {
  /// Technical unique identifier.
  pub id: RuleId,
  /// Human identity and version metadata.
  pub meta: RuleMeta,
  /// Operational state and audit information.
  pub state: RuleState,
  /// Time window where this rule is eligible.
  pub schedule: RuleSchedule,
  /// Gradual traffic exposure policy.
  pub rollout: RolloutPolicy,
  /// Predicate and rule logic expressions.
  pub evaluation: RuleEvaluation,
  /// Action and risk impact when triggered.
  pub enforcement: RuleEnforcement,
}
