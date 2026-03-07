use serde::{Deserialize, Serialize};

use crate::domain::common::TimestampMs;
use thiserror::Error;

/// The error type returned when rule audit metadata validation fails.
#[derive(Debug, Clone, Serialize, Deserialize, Error, PartialEq, Eq)]
pub enum RuleAuditError {
  /// The update timestamp is earlier than the creation timestamp.
  #[error(
    "audit timestamps invalid: updated_at_ms ({updated_at_ms}) must be >= created_at_ms ({created_at_ms})"
  )]
  InvalidTimestampOrder {
    /// The timestamp representing the initial creation.
    created_at_ms: u64,
    /// The invalid timestamp representing the modification.
    updated_at_ms: u64,
  },
}

/// Chronological tracking and attribution metadata for a rule.
///
/// `RuleAudit` maintains the absolute timestamps and optional identities
/// associated with creation and subsequent modifications.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleAudit {
  /// The absolute timestamp of creation.
  pub created_at_ms: TimestampMs,

  /// The absolute timestamp of the most recent modification.
  pub updated_at_ms: TimestampMs,

  /// The identity of the entity that created the rule.
  pub created_by: Option<String>,

  /// The identity of the entity that last modified the rule.
  pub updated_by: Option<String>,
}

impl RuleAudit {
  /// Validates the chronological invariants of the audit trail.
  ///
  /// Returns [`RuleAuditError::InvalidTimestampOrder`] when the timeline is
  /// invalid (`updated_at_ms < created_at_ms`).
  pub fn validate(&self) -> Result<(), RuleAuditError> {
    if self.updated_at_ms.as_u64() < self.created_at_ms.as_u64() {
      return Err(RuleAuditError::InvalidTimestampOrder {
        created_at_ms: self.created_at_ms.as_u64(),
        updated_at_ms: self.updated_at_ms.as_u64(),
      });
    }
    Ok(())
  }
}
