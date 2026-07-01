use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;

use crate::ports::rule_engine::{
  RuleCompileStats, RuleEnginePort, RulesetSnapshot, RuntimeEngineError,
};
use crate::ports::rule_repository::{RuleRepositoryError, RuleRepositoryPort};

/// High-level runtime status returned by the application layer.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeControlOverview {
  /// Name of the runtime implementation.
  pub mode: String,
  /// Whether the runtime can serve evaluations.
  pub ready: bool,
  /// Number of rules currently stored in the repository.
  pub repository_rules: u32,
  /// Number of rules currently loaded into the runtime.
  pub loaded_rules: u32,
  /// Latest compile statistics reported by the runtime.
  pub compile_stats: RuleCompileStats,
}

/// Errors surfaced by [`RuntimeControlService`] use cases.
#[derive(Debug, Error)]
pub enum RuntimeControlError {
  /// Repository access failed while loading the ruleset.
  #[error(transparent)]
  Repository(#[from] RuleRepositoryError),
  /// Runtime access failed while reading status or publishing rules.
  #[error(transparent)]
  Runtime(#[from] RuntimeEngineError),
}

/// Input port for runtime lifecycle use cases.
#[async_trait]
pub trait RuntimeControlInputPort: Send + Sync {
  /// Returns the current runtime overview.
  async fn status(&self) -> Result<RuntimeControlOverview, RuntimeControlError>;

  /// Rebuilds the runtime snapshot from the repository source of truth.
  async fn reload_rules(&self) -> Result<RulesetSnapshot, RuntimeControlError>;
}

/// Default application service for runtime lifecycle operations.
#[derive(Clone)]
pub struct RuntimeControlService {
  repository: Arc<dyn RuleRepositoryPort>,
  runtime: Arc<dyn RuleEnginePort>,
}

impl RuntimeControlService {
  /// Creates a runtime control service backed by repository and runtime ports.
  #[must_use]
  pub fn new(repository: Arc<dyn RuleRepositoryPort>, runtime: Arc<dyn RuleEnginePort>) -> Self {
    Self { repository, runtime }
  }
}

#[async_trait]
impl RuntimeControlInputPort for RuntimeControlService {
  async fn status(&self) -> Result<RuntimeControlOverview, RuntimeControlError> {
    let repository_rules = self.repository.all().await?.len() as u32;
    let runtime_status = self.runtime.status()?;

    Ok(RuntimeControlOverview {
      mode: runtime_status.mode,
      ready: runtime_status.ready,
      repository_rules,
      loaded_rules: runtime_status.loaded_rules,
      compile_stats: runtime_status.compile_stats,
    })
  }

  async fn reload_rules(&self) -> Result<RulesetSnapshot, RuntimeControlError> {
    let rules = self.repository.all().await?;
    let snapshot = self.runtime.publish_rules(rules).await?;
    info!(
      version = snapshot.version,
      loaded_rules = snapshot.loaded_rules,
      total_rules = snapshot.compile_stats.total_rules,
      compiled_rules = snapshot.compile_stats.compiled_rules,
      failed_rules = snapshot.compile_stats.failed_rules,
      "runtime ruleset reloaded"
    );
    Ok(snapshot)
  }
}
