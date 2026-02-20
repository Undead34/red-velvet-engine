use serde::{Deserialize, Serialize};

use crate::domain::common::TimestampMs;

/// Optional activation window for scheduled campaigns/hotfixes.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RuleSchedule {
  pub active_from_ms: Option<TimestampMs>,
  pub active_until_ms: Option<TimestampMs>,
}

impl RuleSchedule {
  /// Returns true when `now_ms` falls inside the configured window.
  pub fn is_within_window(&self, now_ms: u64) -> bool {
    if let Some(from) = self.active_from_ms {
      if now_ms < from.as_u64() {
        return false;
      }
    }

    if let Some(until) = self.active_until_ms {
      if now_ms >= until.as_u64() {
        return false;
      }
    }

    true
  }
}
