pub mod rule_repository;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::{
  common::{RuleId, Severity},
  event::Event,
  rule::{Rule, RuleAction},
};

pub use rule_repository::{RepositoryResult, RulePage, RuleRepositoryError, RuleRepositoryPort};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleCompileStats {
  pub total_rules: u32,
  pub compiled_rules: u32,
  pub failed_rules: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RulesetSnapshot {
  pub version: u64,
  pub loaded_rules: u32,
  pub compile_stats: RuleCompileStats,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeHit {
  pub rule_id: RuleId,
  pub action: RuleAction,
  pub severity: Severity,
  pub score_delta: f32,
  pub explanation: Option<String>,
  pub tags: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeEvaluation {
  pub score: f32,
  pub hits: Vec<RuntimeHit>,
  pub evaluated_rules: u32,
  pub rollout_bucket: u8,
}

#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum RuntimeEngineError {
  #[error("runtime compilation error for {rule_id:?}: {message}")]
  Compilation { rule_id: Option<RuleId>, message: String },
  #[error("runtime evaluation error for {rule_id:?}: {message}")]
  Evaluation { rule_id: Option<RuleId>, message: String },
  #[error("runtime configuration error: {message}")]
  Configuration { message: String },
  #[error("runtime internal error: {message}")]
  Internal { message: String },
}

#[async_trait]
pub trait RuntimeEnginePort: Send + Sync {
  async fn publish_rules(&self, rules: Vec<Rule>) -> Result<RulesetSnapshot, RuntimeEngineError>;

  async fn evaluate(&self, event: &Event) -> Result<RuntimeEvaluation, RuntimeEngineError>;
}
