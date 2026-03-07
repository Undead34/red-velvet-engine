use serde::{Deserialize, Serialize};

use super::RuleAction;
use crate::domain::common::{Score, Severity};

/// The concrete impact and operational directives of a triggered rule.
///
/// `RuleEnforcement` defines how the system should react to a positive rule match.
/// It combines quantitative risk adjustments ([`Score`]), qualitative severity levels,
/// and explicit operational commands ([`RuleAction`]).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleEnforcement {
  /// The numerical weight added to the aggregate risk score.
  pub score_impact: Score,

  /// The explicit operational directive to be executed by the consumer.
  pub action: RuleAction,

  /// The classification of the rule's criticality for monitoring and escalation.
  pub severity: Severity,

  /// Categorical metadata used for downstream telemetry, analytics, and grouping.
  pub tags: Vec<String>,

  /// An optional suppression window (in milliseconds) to prevent redundant triggers.
  pub cooldown_ms: Option<u64>,
}
