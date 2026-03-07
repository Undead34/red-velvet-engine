use thiserror::Error;

use crate::domain::rule::{RulePolicyError, RuleRolloutError, RuleScheduleError, RuleStateError};

/// Error boundary for the domain.
///
/// `DomainError` is the public aggregate error surface for use across domains
/// and adapters. Sub-components expose more granular errors (`RulePolicyError`,
/// `RuleStateError`, etc.), which are mapped into this type at aggregate
/// boundaries.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DomainError {
  #[error("invalid country code: {0}")]
  InvalidCountryCode(String),
  #[error("invalid currency code: {0}")]
  InvalidCurrencyCode(String),
  #[error("invalid account id: {0}")]
  InvalidAccountId(String),
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
  #[error("invalid device id: {0}")]
  InvalidDeviceId(String),
  #[error("invalid session id: {0}")]
  InvalidSessionId(String),
  #[error("invalid kyc level: {0}")]
  InvalidKycLevel(String),
  #[error("invalid locale tag: {0}")]
  InvalidLocaleTag(String),
  #[error("invalid timezone name: {0}")]
  InvalidTimezoneName(String),
  #[error("invalid user agent: {0}")]
  InvalidUserAgent(String),
  #[error("invalid rule id: {0}")]
  InvalidRuleId(String),
  #[error("invalid rule expression: {0}")]
  InvalidRuleExpression(String),
  /// Consolidated rule-policy error used by the aggregate boundary.
  #[error(transparent)]
  RulePolicy(#[from] RulePolicyError),
  #[deprecated(note = "legacy variant; prefer RulePolicy")]
  #[error("invalid rule schedule: {0}")]
  InvalidRuleSchedule(String),
  #[deprecated(note = "legacy variant; prefer RulePolicy")]
  #[error("invalid rule rollout: {0}")]
  InvalidRuleRollout(String),
  #[deprecated(note = "legacy variant; prefer RulePolicy")]
  #[error("invalid rule audit: {0}")]
  InvalidRuleAudit(String),
  #[deprecated(note = "legacy variant; prefer RulePolicy")]
  #[error("invalid rule state transition: {0}")]
  InvalidRuleStateTransition(String),
  #[deprecated(note = "legacy variant; prefer RulePolicy")]
  #[error("invalid rule evaluation: {0}")]
  InvalidRuleEvaluation(String),
  #[error("invalid money amount: {0}")]
  InvalidMoneyAmount(String),
  #[error("invalid timestamp (ms): {0}")]
  InvalidTimestampMs(u64),
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
    Self::RulePolicy(error.into())
  }
}
