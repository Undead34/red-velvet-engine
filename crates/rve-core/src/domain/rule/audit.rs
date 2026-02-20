use serde::{Deserialize, Serialize};

use crate::domain::common::TimestampMs;

/// Audit trail fields for rule lifecycle changes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleAudit {
  pub created_at_ms: TimestampMs,
  pub updated_at_ms: TimestampMs,
  pub created_by: Option<String>,
  pub updated_by: Option<String>,
}
