use serde::{Deserialize, Serialize};

use crate::domain::DomainError;

use super::{
  Header, Payload, context::Context, error::EventError, features::Features, signals::Signals,
};

/// Validated event aggregate consumed by the engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
  /// Transport and identity metadata.
  pub header: Header,
  /// Request-time context snapshot.
  pub context: Context,
  /// Historical and derived features used in evaluation.
  pub features: Features,
  /// Detection signals mapped as flags.
  pub signals: Signals,
  /// Business payload and extensions.
  pub payload: Payload,
}

impl Event {
  /// Creates a new event and validates domain invariants.
  ///
  /// This method exposes the local event boundary and returns [`EventError`]
  /// without promoting errors to the root domain type.
  ///
  /// # Errors
  ///
  /// Returns [`EventError`] if any event component is invalid.
  pub fn try_new(
    header: Header,
    context: Context,
    features: Features,
    signals: Signals,
    payload: Payload,
  ) -> Result<Self, EventError> {
    let event = Self { header, context, features, signals, payload };
    event.validate_event()?;
    Ok(event)
  }

  /// Creates a new event and validates domain invariants.
  ///
  /// # Errors
  ///
  /// Returns [`DomainError`] if any component violates event constraints.
  pub fn new(
    header: Header,
    context: Context,
    features: Features,
    signals: Signals,
    payload: Payload,
  ) -> Result<Self, DomainError> {
    Self::try_new(header, context, features, signals, payload).map_err(DomainError::from)
  }

  /// Validates this event against event-domain invariants.
  ///
  /// # Errors
  ///
  /// Returns [`EventError`] for invalid geo coordinates, party constraints,
  /// or feature chronology.
  pub fn validate_event(&self) -> Result<(), EventError> {
    self.context.geo.validate().map_err(EventError::Geo)?;
    self.payload.validate().map_err(EventError::Party)?;
    self.features.validate().map_err(EventError::Features)?;
    Ok(())
  }

  /// Validates this event.
  ///
  /// # Errors
  ///
  /// Returns [`DomainError`] for invalid geo coordinates, party constraints,
  /// or feature chronology.
  pub fn validate(&self) -> Result<(), DomainError> {
    self.validate_event().map_err(DomainError::from)
  }
}
