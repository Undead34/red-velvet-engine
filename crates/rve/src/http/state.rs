use std::sync::Arc;

use rve_core::ports::RuleRepositoryPort;
use tracing::{info, warn};

use crate::{
  engine::RVEngine,
  store::{InMemoryRuleRepository, RedisRuleRepository, seed_rules},
};

#[derive(Clone)]
pub struct AppState {
  pub engine: Arc<RVEngine>,
  pub rule_repo: Arc<dyn RuleRepositoryPort>,
}

impl AppState {
  pub async fn new() -> Self {
    let repo: Arc<dyn RuleRepositoryPort> = match build_repository().await {
      Ok(repo) => repo,
      Err(message) => {
        warn!(target: "BANNER", %message, "falling back to in-memory rule repository");
        Arc::new(InMemoryRuleRepository::seeded())
      }
    };

    if let Err(err) = seed_if_empty(repo.as_ref()).await {
      warn!(target: "BANNER", %err, "failed to seed repository");
    }

    let engine = Arc::new(RVEngine::new());

    Self { engine, rule_repo: repo }
  }
}

async fn build_repository() -> Result<Arc<dyn RuleRepositoryPort>, String> {
  let redis_url = std::env::var("RVE_REDIS_URL").ok();
  let Some(redis_url) = redis_url.filter(|v| !v.trim().is_empty()) else {
    info!(target: "BANNER", "using in-memory rule repository");
    return Ok(Arc::new(InMemoryRuleRepository::seeded()));
  };

  let prefix = std::env::var("RVE_REDIS_PREFIX").unwrap_or_else(|_| "rve".to_owned());
  let repo = RedisRuleRepository::new(&redis_url, prefix)
    .map_err(|err| format!("redis repository init failed: {err}"))?;

  if let Err(err) = repo.all().await {
    return Err(format!("redis health check failed: {err}"));
  }

  info!(target: "BANNER", "using redis-backed rule repository");
  Ok(Arc::new(repo))
}

async fn seed_if_empty(repo: &dyn RuleRepositoryPort) -> Result<(), String> {
  let existing = repo.all().await.map_err(|err| err.to_string())?;
  if !existing.is_empty() {
    return Ok(());
  }

  for rule in seed_rules() {
    repo.create(rule).await.map_err(|err| err.to_string())?;
  }

  Ok(())
}
