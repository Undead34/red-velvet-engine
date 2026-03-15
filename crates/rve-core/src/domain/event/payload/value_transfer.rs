use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::common::Money;
use crate::domain::event::EventPartyError;

use crate::domain::event::Parties;

/// Payload for value-transfer operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueTransfer {
  /// Transaction amount and currency.
  pub money: Money,
  /// Originator/beneficiary data.
  pub parties: Parties,
  /// Free-form extensions passed to rules.
  pub extensions: BTreeMap<String, Value>,
}

impl ValueTransfer {
  /// Creates a value-transfer payload without validation.
  ///
  /// Prefer [`ValueTransfer::try_new`] when constructing from external input.
  #[must_use]
  pub fn new(money: Money, parties: Parties, extensions: BTreeMap<String, Value>) -> Self {
    Self { money, parties, extensions }
  }

  /// Creates a validated value-transfer payload.
  ///
  /// # Errors
  ///
  /// Returns [`EventPartyError`] when `parties` violate party constraints.
  pub fn try_new(
    money: Money,
    parties: Parties,
    extensions: BTreeMap<String, Value>,
  ) -> Result<Self, EventPartyError> {
    parties.validate()?;
    Ok(Self { money, parties, extensions })
  }

  /// Validates payload invariants.
  ///
  /// # Errors
  ///
  /// Returns [`EventPartyError`] when party data is invalid.
  pub fn validate(&self) -> Result<(), EventPartyError> {
    self.parties.validate()
  }
}
