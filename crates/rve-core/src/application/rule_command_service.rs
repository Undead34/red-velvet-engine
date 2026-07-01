use std::sync::Arc;

use async_trait::async_trait;
use thiserror::Error;

use crate::domain::{common::RuleId, rule::Rule};
use crate::ports::rule_repository::{RuleRepositoryError, RuleRepositoryPort};

/// Errors surfaced by [`RuleCommandService`] use cases.
#[derive(Debug, Error)]
pub enum RuleCommandServiceError {
  /// Repository access failed while mutating rules.
  #[error(transparent)]
  Repository(#[from] RuleRepositoryError),
}

/// Input port for rule mutation use cases.
#[async_trait]
pub trait RuleCommandInputPort: Send + Sync {
  /// Persists a new rule.
  async fn create_rule(&self, rule: Rule) -> Result<Rule, RuleCommandServiceError>;

  /// Replaces an existing rule.
  async fn replace_rule(&self, rule: Rule) -> Result<Rule, RuleCommandServiceError>;

  /// Deletes a rule by identifier.
  async fn delete_rule(&self, id: &RuleId) -> Result<(), RuleCommandServiceError>;
}

/// Default application service for rule mutations.
#[derive(Clone)]
pub struct RuleCommandService {
  repository: Arc<dyn RuleRepositoryPort>,
}

impl RuleCommandService {
  /// Creates a command service backed by a repository port.
  #[must_use]
  pub fn new(repository: Arc<dyn RuleRepositoryPort>) -> Self {
    Self { repository }
  }
}

#[async_trait]
impl RuleCommandInputPort for RuleCommandService {
  async fn create_rule(&self, rule: Rule) -> Result<Rule, RuleCommandServiceError> {
    self.repository.create(rule).await.map_err(RuleCommandServiceError::from)
  }

  async fn replace_rule(&self, rule: Rule) -> Result<Rule, RuleCommandServiceError> {
    self.repository.replace(rule).await.map_err(RuleCommandServiceError::from)
  }

  async fn delete_rule(&self, id: &RuleId) -> Result<(), RuleCommandServiceError> {
    self.repository.delete(id).await.map_err(RuleCommandServiceError::from)
  }
}
