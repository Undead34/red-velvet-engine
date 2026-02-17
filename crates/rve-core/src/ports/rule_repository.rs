use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::domain::rule::{Rule, RuleId};

pub type RepositoryResult<T> = Result<T, RuleRepositoryError>;

#[derive(Debug, thiserror::Error, Clone, Serialize, Deserialize)]
pub enum RuleRepositoryError {
  #[error("rule already exists: {0}")]
  AlreadyExists(RuleId),
  #[error("rule not found: {0}")]
  NotFound(RuleId),
  #[error("rule repository error: {0}")]
  Storage(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RulePage {
  pub items: Vec<Rule>,
  pub total: u32,
}

#[async_trait]
pub trait RuleRepositoryPort: Send + Sync {
  async fn list(&self, page: u32, limit: u32) -> RepositoryResult<RulePage>;

  async fn get(&self, id: &RuleId) -> RepositoryResult<Option<Rule>>;

  async fn all(&self) -> RepositoryResult<Vec<Rule>>;

  async fn create(&self, rule: Rule) -> RepositoryResult<Rule>;

  async fn replace(&self, rule: Rule) -> RepositoryResult<Rule>;

  async fn delete(&self, id: &RuleId) -> RepositoryResult<()>;
}
