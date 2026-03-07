use std::sync::Arc;

use rve_core::ports::{RuleRepositoryError, RuleRepositoryPort};
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::error;

use crate::{
  engine::{EngineError, RVEngine},
  store::InMemoryRuleRepository,
};

#[derive(Clone)]
pub struct AppState {
  pub engine: Arc<RVEngine>,
  pub rule_repo: Arc<dyn RuleRepositoryPort>,
  engine_runtime: Arc<RwLock<EngineRuntimeStatus>>,
}

#[derive(Clone, Debug)]
pub struct EngineRuntimeStatus {
  pub ruleset_version: u64,
  pub loaded_rules: u32,
  pub last_reload_at_ms: Option<u64>,
  pub last_reload_error: Option<String>,
}

impl AppState {
  pub async fn new() -> Self {
    let repo = Arc::new(InMemoryRuleRepository::seeded());
    let engine = Arc::new(RVEngine::new());
    let mut runtime = EngineRuntimeStatus {
      ruleset_version: 0,
      loaded_rules: 0,
      last_reload_at_ms: None,
      last_reload_error: None,
    };

    if let Ok(rules) = repo.all().await {
      runtime.loaded_rules = rules.len() as u32;
      if let Err(err) = engine.publish_rules(rules) {
        error!(target: "BANNER", %err, "failed to compile initial ruleset");
        runtime.last_reload_error = Some(err.to_string());
      } else {
        runtime.ruleset_version = 1;
        runtime.last_reload_at_ms = Some(current_time_ms());
      }
    }

    Self { engine, rule_repo: repo, engine_runtime: Arc::new(RwLock::new(runtime)) }
  }

  pub async fn reload_rules(&self) -> Result<(), EngineSyncError> {
    let rules = self.rule_repo.all().await.map_err(EngineSyncError::Repository)?;
    let loaded_rules = rules.len() as u32;

    match self.engine.publish_rules(rules) {
      Ok(()) => {
        let mut runtime = self.engine_runtime.write().await;
        runtime.ruleset_version = runtime.ruleset_version.saturating_add(1);
        runtime.loaded_rules = loaded_rules;
        runtime.last_reload_at_ms = Some(current_time_ms());
        runtime.last_reload_error = None;
        Ok(())
      }
      Err(err) => {
        let mut runtime = self.engine_runtime.write().await;
        runtime.last_reload_error = Some(err.to_string());
        Err(EngineSyncError::Engine(err))
      }
    }
  }

  pub async fn engine_runtime_status(&self) -> EngineRuntimeStatus {
    self.engine_runtime.read().await.clone()
  }
}

fn current_time_ms() -> u64 {
  use std::time::{SystemTime, UNIX_EPOCH};

  SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

#[derive(Debug, Error)]
pub enum EngineSyncError {
  #[error(transparent)]
  Repository(#[from] RuleRepositoryError),
  #[error(transparent)]
  Engine(#[from] EngineError),
}
