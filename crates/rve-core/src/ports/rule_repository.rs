use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::domain::{common::RuleId, rule::Rule};

/// Result type returned by repository operations.
pub type RepositoryResult<T> = Result<T, RuleRepositoryError>;

/// Repository-level failures exposed to the application layer.
#[derive(Debug, thiserror::Error, Clone, Serialize, Deserialize)]
pub enum RuleRepositoryError {
  #[error("rule already exists: {0}")]
  AlreadyExists(RuleId),
  #[error("rule not found: {0}")]
  NotFound(RuleId),
  #[error("rule repository error: {0}")]
  Storage(String),
}

/// Paginated rule listing returned by [`RuleRepositoryPort::list`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RulePage {
  pub items: Vec<Rule>,
  pub total: u32,
}

/// Outbound port for rule persistence.
#[async_trait]
pub trait RuleRepositoryPort: Send + Sync {
  /// Returns a paginated list of rules.
  async fn list(&self, page: u32, limit: u32) -> RepositoryResult<RulePage>;

  /// Returns a single rule when present.
  async fn get(&self, id: &RuleId) -> RepositoryResult<Option<Rule>>;

  /// Returns all stored rules.
  async fn all(&self) -> RepositoryResult<Vec<Rule>>;

  /// Persists a new rule.
  async fn create(&self, rule: Rule) -> RepositoryResult<Rule>;

  /// Replaces an existing rule.
  async fn replace(&self, rule: Rule) -> RepositoryResult<Rule>;

  /// Deletes a rule.
  async fn delete(&self, id: &RuleId) -> RepositoryResult<()>;
}
