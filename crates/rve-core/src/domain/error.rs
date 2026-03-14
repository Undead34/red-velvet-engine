use crate::domain::common::{MoneyError, TimestampMsError};
use crate::domain::event::{EventError, EventFeaturesError, EventGeoError, EventPartyError};
use crate::domain::rule::{RulePolicyError, RuleRolloutError, RuleScheduleError, RuleStateError};
use thiserror::Error;

/// Top-level error type for the domain layer.
///
/// Component-level errors are mapped into this type at aggregate boundaries.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DomainError {
  /// Error from rule policy validation or transition rules.
  #[error(transparent)]
  RulePolicy(#[from] RulePolicyError),
  /// Error from event domain validation.
  #[error(transparent)]
  Event(#[from] EventError),

  /// Rule expression is syntactically invalid or uses unsupported constructs.
  #[error("invalid rule expression: {0}")]
  InvalidRuleExpression(String),

  /// Rule function config is invalid for the selected function kind.
  #[error("invalid rule function config for `{kind}`: {reason}")]
  InvalidRuleFunctionConfig { kind: String, reason: String },

  /// Rule identifier is invalid.
  #[error("invalid rule id: {0}")]
  InvalidRuleId(String),

  #[error("invalid account id: {0}")]
  InvalidAccountId(String),

  #[error("invalid device id: {0}")]
  InvalidDeviceId(String),

  #[error("invalid session id: {0}")]
  InvalidSessionId(String),

  #[error("invalid kyc level: {0}")]
  InvalidKycLevel(String),

  #[error("invalid country code: {0}")]
  InvalidCountryCode(String),

  #[error("invalid currency code: {0}")]
  InvalidCurrencyCode(String),

  #[error("invalid locale tag: {0}")]
  InvalidLocaleTag(String),

  #[error("invalid timezone name: {0}")]
  InvalidTimezoneName(String),

  #[error("invalid bank reference: {0}")]
  InvalidBankRef(String),

  #[error("invalid event source: {0}")]
  InvalidEventSource(String),

  #[error("invalid event id: {0}")]
  InvalidEventId(String),

  #[error("invalid instrument: {0}")]
  InvalidInstrument(String),

  #[error("invalid channel: {0}")]
  InvalidChannel(String),

  #[error(transparent)]
  Money(#[from] MoneyError),

  #[error(transparent)]
  TimestampMs(#[from] TimestampMsError),

  #[error("invalid user agent: {0}")]
  InvalidUserAgent(String),
}

impl From<RuleScheduleError> for DomainError {
  fn from(error: RuleScheduleError) -> Self {
    Self::RulePolicy(RulePolicyError::Schedule(error))
  }
}

impl From<RuleRolloutError> for DomainError {
  fn from(error: RuleRolloutError) -> Self {
    Self::RulePolicy(RulePolicyError::Rollout(error))
  }
}

impl From<RuleStateError> for DomainError {
  fn from(error: RuleStateError) -> Self {
    Self::RulePolicy(RulePolicyError::State(error))
  }
}

impl From<EventGeoError> for DomainError {
  fn from(error: EventGeoError) -> Self {
    Self::Event(EventError::Geo(error))
  }
}

impl From<EventPartyError> for DomainError {
  fn from(error: EventPartyError) -> Self {
    Self::Event(EventError::Party(error))
  }
}

impl From<EventFeaturesError> for DomainError {
  fn from(error: EventFeaturesError) -> Self {
    Self::Event(EventError::Features(error))
  }
}
