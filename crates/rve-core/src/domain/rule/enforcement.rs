use serde::{Deserialize, Serialize};

use crate::domain::common::{Score, Severity};

use super::RuleAction;

/// Outcome of a rule trigger.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleEnforcement {
  /// Score contribution added to the decision score.
  pub score_impact: Score,
  /// Recommended action for the caller.
  pub action: RuleAction,
  /// Operational criticality for dashboards/alerting.
  pub severity: Severity,
  /// Labels used for grouping and analytics.
  pub tags: Vec<String>,
  /// Optional cooldown to avoid repeated hits.
  pub cooldown_ms: Option<u64>,
}
