use serde::{Deserialize, Serialize};

use crate::domain::DomainError;

use super::{
  Header, Payload,
  context::Context,
  error::EventError,
  features::Features,
  signals::Signals,
};

/// Full decision input event consumed by the engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
  pub header: Header,
  pub context: Context,
  pub features: Features,
  pub signals: Signals,
  pub payload: Payload,
}

impl Event {
  pub fn new(
    header: Header,
    context: Context,
    features: Features,
    signals: Signals,
    payload: Payload,
  ) -> Result<Self, DomainError> {
    let event = Self { header, context, features, signals, payload };
    event.validate()?;
    Ok(event)
  }

  pub fn validate(&self) -> Result<(), DomainError> {
    self.context.geo.validate().map_err(EventError::Geo)?;
    self.payload.parties.validate().map_err(EventError::Party)?;
    self.features.validate().map_err(EventError::Features)?;
    Ok(())
  }
}
