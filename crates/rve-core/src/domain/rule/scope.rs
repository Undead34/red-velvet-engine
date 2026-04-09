use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::common::Channel;

/// Errors that can occur when validating [`RuleScope`].
#[derive(Debug, Clone, Serialize, Deserialize, Error, PartialEq, Eq)]
pub enum RuleScopeError {
  /// The scope was configured with an empty channel list.
  #[error("invalid rule scope: channels must contain at least one value when provided")]
  EmptyChannels,
}

/// Channel-based applicability for a rule.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RuleScope {
  /// Optional channel allowlist. When absent, the rule applies to all channels.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub channels: Option<BTreeSet<Channel>>,
}

impl RuleScope {
  #[must_use]
  pub fn all() -> Self {
    Self::default()
  }

  /// Creates a scope restricted to the provided channels.
  ///
  /// # Errors
  ///
  /// Returns [`RuleScopeError::EmptyChannels`] when the resulting set is empty.
  pub fn only<I>(channels: I) -> Result<Self, RuleScopeError>
  where
    I: IntoIterator<Item = Channel>,
  {
    let channels = channels.into_iter().collect::<BTreeSet<_>>();
    let scope = Self { channels: Some(channels) };
    scope.validate()?;
    Ok(scope)
  }

  /// Validates the scope configuration.
  ///
  /// # Errors
  ///
  /// Returns [`RuleScopeError::EmptyChannels`] when `channels` is present but empty.
  pub fn validate(&self) -> Result<(), RuleScopeError> {
    if self.channels.as_ref().is_some_and(BTreeSet::is_empty) {
      return Err(RuleScopeError::EmptyChannels);
    }
    Ok(())
  }

  #[must_use]
  pub fn applies_to(&self, channel: Option<&Channel>) -> bool {
    match (&self.channels, channel) {
      (None, _) => true,
      (Some(channels), Some(channel)) => channels.contains(channel),
      (Some(_), None) => false,
    }
  }

  #[must_use]
  pub fn channels(&self) -> Option<&BTreeSet<Channel>> {
    self.channels.as_ref()
  }
}

impl TryFrom<Option<Vec<Channel>>> for RuleScope {
  type Error = RuleScopeError;

  fn try_from(value: Option<Vec<Channel>>) -> Result<Self, Self::Error> {
    match value {
      None => Ok(Self::all()),
      Some(channels) => Self::only(channels),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::{RuleScope, RuleScopeError};
  use crate::domain::common::Channel;

  #[test]
  fn all_scope_matches_any_event_channel() {
    let scope = RuleScope::all();
    assert!(scope.applies_to(None));
    assert!(scope.applies_to(Some(&Channel::Web)));
  }

  #[test]
  fn scoped_rules_require_matching_channel() {
    let scope = RuleScope::only(vec![Channel::Web, Channel::Mobile]).unwrap();
    assert!(scope.applies_to(Some(&Channel::Web)));
    assert!(!scope.applies_to(Some(&Channel::Api)));
    assert!(!scope.applies_to(None));
  }

  #[test]
  fn rejects_empty_channel_scope() {
    assert!(matches!(RuleScope::only(Vec::new()), Err(RuleScopeError::EmptyChannels)));
  }
}
