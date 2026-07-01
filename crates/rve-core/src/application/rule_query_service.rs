use std::sync::Arc;

use async_trait::async_trait;
use thiserror::Error;

use crate::domain::{common::RuleId, rule::Rule};
use crate::ports::rule_repository::{RulePage, RuleRepositoryError, RuleRepositoryPort};

/// Errors surfaced by [`RuleQueryService`] use cases.
#[derive(Debug, Error)]
pub enum RuleQueryServiceError {
  /// Repository access failed while reading rules.
  #[error(transparent)]
  Repository(#[from] RuleRepositoryError),
}

/// Input port for rule query use cases.
#[async_trait]
pub trait RuleQueryInputPort: Send + Sync {
  /// Returns a paginated view of stored rules.
  async fn list_rules(&self, page: u32, limit: u32) -> Result<RulePage, RuleQueryServiceError>;

  /// Returns a single rule by identifier.
  async fn get_rule(&self, id: &RuleId) -> Result<Option<Rule>, RuleQueryServiceError>;
}

/// Default application service for rule queries.
#[derive(Clone)]
pub struct RuleQueryService {
  repository: Arc<dyn RuleRepositoryPort>,
}

impl RuleQueryService {
  /// Creates a query service backed by a repository port.
  #[must_use]
  pub fn new(repository: Arc<dyn RuleRepositoryPort>) -> Self {
    Self { repository }
  }
}

#[async_trait]
impl RuleQueryInputPort for RuleQueryService {
  async fn list_rules(&self, page: u32, limit: u32) -> Result<RulePage, RuleQueryServiceError> {
    self.repository.list(page, limit).await.map_err(RuleQueryServiceError::from)
  }

  async fn get_rule(&self, id: &RuleId) -> Result<Option<Rule>, RuleQueryServiceError> {
    self.repository.get(id).await.map_err(RuleQueryServiceError::from)
  }
}
