use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::error::EventFeaturesError;

/// Historical and derived features used by fraud rules.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Features {
  pub fin: FinancialFeatures,
}

/// Financial behavior features.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct FinancialFeatures {
  pub first_seen_at: u64,
  pub last_seen_at: u64,
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
  pub fn validate(&self) -> Result<(), EventFeaturesError> {
    self.fin.validate()
  }
}

impl FinancialFeatures {
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
