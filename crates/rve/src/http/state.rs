use std::sync::Arc;

use rve_core::{
  ports::RuleRepositoryPort,
  services::engine::{DecisionService, DecisionServiceError},
};
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::error;

use crate::{engine::RVEngine, store::InMemoryRuleRepository};

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

    match DecisionService::reload_rules(repo.as_ref(), engine.as_ref()).await {
      Ok(snapshot) => {
        runtime.ruleset_version = snapshot.version;
        runtime.loaded_rules = snapshot.loaded_rules;
        runtime.last_reload_at_ms = Some(current_time_ms());
      }
      Err(err) => {
        error!(target: "BANNER", %err, "failed to compile initial ruleset");
        runtime.last_reload_error = Some(err.to_string());
      }
    }

    Self { engine, rule_repo: repo, engine_runtime: Arc::new(RwLock::new(runtime)) }
  }

  pub async fn reload_rules(&self) -> Result<(), EngineSyncError> {
    match DecisionService::reload_rules(self.rule_repo.as_ref(), self.engine.as_ref()).await {
      Ok(snapshot) => {
        let mut runtime = self.engine_runtime.write().await;
        runtime.ruleset_version = snapshot.version;
        runtime.loaded_rules = snapshot.loaded_rules;
        runtime.last_reload_at_ms = Some(current_time_ms());
        runtime.last_reload_error = None;
        Ok(())
      }
      Err(err) => {
        let mut runtime = self.engine_runtime.write().await;
        runtime.last_reload_error = Some(err.to_string());
        Err(EngineSyncError::Decision(err))
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
  Decision(#[from] DecisionServiceError),
}
