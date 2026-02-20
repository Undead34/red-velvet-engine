use std::fmt;

use serde::{Deserialize, Serialize};

use super::validation::is_valid_identifier;
use crate::domain::DomainError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(try_from = "String", into = "String")]
pub struct EventSource(String);

impl EventSource {
  pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
    let value = value.into();
    if is_valid_identifier(&value) {
      Ok(Self(value))
    } else {
      Err(DomainError::InvalidEventSource(value))
    }
  }
}

impl TryFrom<String> for EventSource {
  type Error = DomainError;
  fn try_from(value: String) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}

impl From<EventSource> for String {
  fn from(value: EventSource) -> Self {
    value.0
  }
}

impl fmt::Display for EventSource {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(try_from = "String", into = "String")]
pub struct Instrument(String);

impl Instrument {
  pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
    let value = value.into();
    if is_valid_identifier(&value) {
      Ok(Self(value))
    } else {
      Err(DomainError::InvalidInstrument(value))
    }
  }
}

impl TryFrom<String> for Instrument {
  type Error = DomainError;
  fn try_from(value: String) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}

impl From<Instrument> for String {
  fn from(value: Instrument) -> Self {
    value.0
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(try_from = "String", into = "String")]
pub struct Channel(String);

impl Channel {
  pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
    let value = value.into();
    if is_valid_identifier(&value) {
      Ok(Self(value))
    } else {
      Err(DomainError::InvalidChannel(value))
    }
  }
}

impl TryFrom<String> for Channel {
  type Error = DomainError;
  fn try_from(value: String) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}

impl From<Channel> for String {
  fn from(value: Channel) -> Self {
    value.0
  }
}
