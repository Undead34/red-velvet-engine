use serde::{Deserialize, Serialize};

use crate::domain::common::{AccountId, BankRef, CountryCode, EntityType, Flag, KycLevel};

use super::error::EventPartyError;

/// Identity and compliance attributes for one party.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Party {
  pub entity_type: EntityType,
  pub acct: AccountId,
  pub country: Option<CountryCode>,
  pub bank: Option<BankRef>,
  pub kyc: Option<KycLevel>,
  pub watchlist: Flag,
  pub sanctions_score: Option<f32>,
}

impl Party {
  pub fn new(
    entity_type: EntityType,
    acct: AccountId,
    country: Option<CountryCode>,
    bank: Option<BankRef>,
    kyc: Option<KycLevel>,
    watchlist: Flag,
    sanctions_score: Option<f32>,
  ) -> Result<Self, EventPartyError> {
    let party = Self { entity_type, acct, country, bank, kyc, watchlist, sanctions_score };
    party.validate()?;
    Ok(party)
  }

  pub fn validate(&self) -> Result<(), EventPartyError> {
    if let Some(score) = self.sanctions_score
      && (!score.is_finite() || !(0.0..=1.0).contains(&score))
    {
      return Err(EventPartyError::InvalidSanctionsScore { value: score.to_string() });
    }
    Ok(())
  }
}
