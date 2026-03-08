use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::error::EventFeaturesError;

/// Historical and derived features used by rules.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Features {
  /// Financial behavior features.
  pub fin: FinancialFeatures,
}

/// Financial counters and timelines.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct FinancialFeatures {
  /// First known timestamp for this identity.
  pub first_seen_at: u64,
  /// Most recent known timestamp for this identity.
  pub last_seen_at: u64,
  /// Most recent declined timestamp, if any.
  pub last_declined_at: Option<u64>,
  pub total_successful_txns: u64,
  pub total_declined_txns: u64,
  pub total_amount_spent: u64,
  pub max_ticket_ever: u64,
  pub consecutive_failed_logins: u32,
  pub consecutive_declines: u32,
  pub current_hour_count: u32,
  pub current_hour_amount: u64,
  pub current_day_count: u32,
  pub current_day_amount: u64,
  pub known_ips: HashSet<String>,
  pub known_devices: HashSet<String>,
}

impl Features {
  /// Validates all feature groups.
  ///
  /// # Errors
  ///
  /// Returns [`EventFeaturesError`] if chronology is invalid.
  pub fn validate(&self) -> Result<(), EventFeaturesError> {
    self.fin.validate()
  }
}

impl FinancialFeatures {
  /// Validates feature chronology constraints.
  ///
  /// # Errors
  ///
  /// Returns [`EventFeaturesError::InvalidSeenChronology`] when
  /// `first_seen_at > last_seen_at`.
  ///
  /// Returns [`EventFeaturesError::InvalidLastDeclinedChronology`] when
  /// `last_declined_at < first_seen_at`.
  pub fn validate(&self) -> Result<(), EventFeaturesError> {
    if self.first_seen_at > self.last_seen_at {
      return Err(EventFeaturesError::InvalidSeenChronology {
        first_seen_at: self.first_seen_at,
        last_seen_at: self.last_seen_at,
      });
    }

    if let Some(last_declined_at) = self.last_declined_at
      && last_declined_at < self.first_seen_at
    {
      return Err(EventFeaturesError::InvalidLastDeclinedChronology {
        first_seen_at: self.first_seen_at,
        last_declined_at,
      });
    }

    Ok(())
  }
}
