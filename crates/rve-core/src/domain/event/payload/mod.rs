use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::common::Money;

use super::{Parties, error::EventPartyError};

mod value_transfer;

pub use value_transfer::ValueTransfer;

/// Event business payload.
///
/// The model is represented as an internally tagged enum to support multiple
/// operation types while keeping a stable envelope
/// (`header/context/features/signals`).
///
/// Serialized form includes a `type` discriminator. Current payloads must use
/// `"type": "value_transfer"`.
///
/// Current supported variant:
/// - [`Payload::ValueTransfer`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
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

  /// Creates a validated value-transfer payload.
  ///
  /// # Errors
  ///
  /// Returns [`EventPartyError`] when party data is invalid.
  pub fn try_value_transfer(
    money: Money,
    parties: Parties,
    extensions: BTreeMap<String, Value>,
  ) -> Result<Self, EventPartyError> {
    Ok(Self::ValueTransfer(ValueTransfer::try_new(money, parties, extensions)?))
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
      Self::ValueTransfer(payload) => payload.validate(),
    }
  }
}
