use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::common::Money;

use super::{Parties, error::EventPartyError};

mod value_transfer;

pub use value_transfer::ValueTransfer;

/// Event business payload.
///
/// The model is represented as an enum to support multiple operation types
/// while keeping a stable envelope (`header/context/features/signals`).
///
/// Current supported variant:
/// - [`Payload::ValueTransfer`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Payload {
  /// Movement of monetary value between two parties.
  ValueTransfer(ValueTransfer),
}

impl Payload {
  /// Creates a value-transfer payload.
  #[must_use]
  pub fn value_transfer(
    money: Money,
    parties: Parties,
    extensions: BTreeMap<String, Value>,
  ) -> Self {
    Self::ValueTransfer(ValueTransfer::new(money, parties, extensions))
  }

  /// Returns the value-transfer payload when available.
  #[must_use]
  pub fn as_value_transfer(&self) -> Option<&ValueTransfer> {
    match self {
      Self::ValueTransfer(payload) => Some(payload),
    }
  }

  /// Returns mutable access to the value-transfer payload when available.
  pub fn as_value_transfer_mut(&mut self) -> Option<&mut ValueTransfer> {
    match self {
      Self::ValueTransfer(payload) => Some(payload),
    }
  }

  /// Validates payload-level invariants.
  ///
  /// # Errors
  ///
  /// Returns [`EventPartyError`] when party data is invalid.
  pub fn validate(&self) -> Result<(), EventPartyError> {
    match self {
      Self::ValueTransfer(payload) => payload.parties.validate(),
    }
  }
}
