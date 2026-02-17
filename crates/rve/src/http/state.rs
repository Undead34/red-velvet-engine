use std::sync::Arc;

use rve_core::ports::{RuleRepositoryError, RuleRepositoryPort};
use thiserror::Error;
use tracing::error;

use crate::{
  engine::{EngineError, RVEngine},
  store::InMemoryRuleRepository,
};

#[derive(Clone)]
pub struct AppState {
  pub engine: Arc<RVEngine>,
  pub rule_repo: Arc<dyn RuleRepositoryPort>,
}

impl AppState {
  pub async fn new() -> Self {
    let repo = Arc::new(InMemoryRuleRepository::seeded());
    let engine = Arc::new(RVEngine::new());

    if let Ok(rules) = repo.all().await {
      if let Err(err) = engine.publish_rules(rules) {
        error!(target: "BANNER", %err, "failed to compile initial ruleset");
      }
    }

    Self { engine, rule_repo: repo }
  }

  pub async fn reload_rules(&self) -> Result<(), EngineSyncError> {
    let rules = self.rule_repo.all().await.map_err(EngineSyncError::Repository)?;
    self.engine.publish_rules(rules).map_err(EngineSyncError::Engine)
  }
}

#[derive(Debug, Error)]
pub enum EngineSyncError {
  #[error(transparent)]
  Repository(#[from] RuleRepositoryError),
  #[error(transparent)]
  Engine(#[from] EngineError),
}
