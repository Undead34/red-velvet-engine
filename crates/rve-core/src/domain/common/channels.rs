use std::fmt;

use serde::{Deserialize, Serialize};

use super::validation::is_valid_identifier;
use crate::domain::{DomainError, DomainResult};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(try_from = "String", into = "String")]
pub struct EventSource(String);

impl EventSource {
  pub fn new(value: impl Into<String>) -> DomainResult<Self> {
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
  fn try_from(value: String) -> DomainResult<Self> {
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
  pub fn new(value: impl Into<String>) -> DomainResult<Self> {
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
  fn try_from(value: String) -> DomainResult<Self> {
    Self::new(value)
  }
}

impl From<Instrument> for String {
  fn from(value: Instrument) -> Self {
    value.0
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(try_from = "String", into = "String")]
pub enum Channel {
  Web,
  Mobile,
  Api,
  Branch,
  CallCenter,
  Pos,
  Atm,
  Backoffice,
  Batch,
  Partner,
  Custom(String),
}

impl Channel {
  pub fn new(value: impl Into<String>) -> DomainResult<Self> {
    let value = value.into();
    match value.as_str() {
      "web" => Ok(Self::Web),
      "mobile" => Ok(Self::Mobile),
      "api" => Ok(Self::Api),
      "branch" => Ok(Self::Branch),
      "call_center" => Ok(Self::CallCenter),
      "pos" => Ok(Self::Pos),
      "atm" => Ok(Self::Atm),
      "backoffice" => Ok(Self::Backoffice),
      "batch" => Ok(Self::Batch),
      "partner" => Ok(Self::Partner),
      _ if is_valid_identifier(&value) => Ok(Self::Custom(value)),
      _ => Err(DomainError::InvalidChannel(value)),
    }
  }

  #[must_use]
  pub fn as_str(&self) -> &str {
    match self {
      Self::Web => "web",
      Self::Mobile => "mobile",
      Self::Api => "api",
      Self::Branch => "branch",
      Self::CallCenter => "call_center",
      Self::Pos => "pos",
      Self::Atm => "atm",
      Self::Backoffice => "backoffice",
      Self::Batch => "batch",
      Self::Partner => "partner",
      Self::Custom(value) => value,
    }
  }

  #[must_use]
  pub const fn known_values() -> &'static [&'static str] {
    &[
      "web",
      "mobile",
      "api",
      "branch",
      "call_center",
      "pos",
      "atm",
      "backoffice",
      "batch",
      "partner",
    ]
  }
}

impl TryFrom<String> for Channel {
  type Error = DomainError;
  fn try_from(value: String) -> DomainResult<Self> {
    Self::new(value)
  }
}

impl From<Channel> for String {
  fn from(value: Channel) -> Self {
    value.as_str().to_owned()
  }
}

impl fmt::Display for Channel {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

#[cfg(test)]
mod tests {
  use super::Channel;

  #[test]
  fn parses_known_channels_into_enum_variants() {
    assert!(matches!(Channel::new("web").unwrap(), Channel::Web));
    assert!(matches!(Channel::new("call_center").unwrap(), Channel::CallCenter));
  }

  #[test]
  fn preserves_valid_custom_channels() {
    let channel = Channel::new("partner_latam").unwrap();
    assert!(matches!(channel, Channel::Custom(_)));
    assert_eq!(channel.to_string(), "partner_latam");
  }

  #[test]
  fn rejects_invalid_channel_identifiers() {
    assert!(Channel::new("web checkout").is_err());
    assert!(Channel::new(" ").is_err());
  }
}
