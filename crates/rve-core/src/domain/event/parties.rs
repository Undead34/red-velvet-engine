use serde::{Deserialize, Serialize};

use super::Party;
use super::error::EventPartyError;

/// Pair of involved parties in a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parties {
  pub originator: Party,
  pub beneficiary: Party,
}

impl Parties {
  pub fn validate(&self) -> Result<(), EventPartyError> {
    self.originator.validate()?;
    self.beneficiary.validate()?;
    Ok(())
  }
}
