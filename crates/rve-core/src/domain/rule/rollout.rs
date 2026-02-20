use serde::{Deserialize, Serialize};

/// Gradual release control for a rule.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RolloutPolicy {
  /// Percentage of traffic where this rule can run (0..=100).
  pub percent: u8,
}

impl RolloutPolicy {
  /// Returns true when the event bucket is allowed by rollout percentage.
  pub fn is_allowed(&self, bucket_0_99: u8) -> bool {
    bucket_0_99 < self.percent.min(100)
  }
}
