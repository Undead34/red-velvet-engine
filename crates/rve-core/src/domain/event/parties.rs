use serde::{Deserialize, Serialize};

use super::Party;
use super::error::EventPartyError;

/// Originator and beneficiary pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parties {
  /// Party sending or initiating the operation.
  pub originator: Party,
  /// Party receiving the operation.
  pub beneficiary: Party,
}

impl Parties {
  /// Validates both parties.
  ///
  /// # Errors
  ///
  /// Returns [`EventPartyError`] if either party is invalid.
  pub fn validate(&self) -> Result<(), EventPartyError> {
    self.originator.validate()?;
    self.beneficiary.validate()?;
    Ok(())
  }
}
