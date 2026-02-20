use serde::{Deserialize, Serialize};

use crate::domain::common::{AccountId, BankRef, CountryCode, EntityType, Flag, KycLevel};

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
