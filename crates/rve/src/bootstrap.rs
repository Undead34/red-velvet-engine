use anyhow::{Context, anyhow};
use std::sync::Arc;
use tracing::info;

use rve_core::ports::{RuleEnginePort, RuleRepositoryPort};

use crate::{engine::DataflowRuleEngine, error::AppError, store::RedisRuleRepository};

pub struct AppContainer {
  pub rule_engine: Arc<dyn RuleEnginePort>,
  pub rule_repo: Arc<dyn RuleRepositoryPort>,
}

impl AppContainer {
  pub async fn bootstrap() -> Result<Self, AppError> {
    let rule_engine = build_rule_engine().await.map_err(AppError::from)?;
    let rule_repo = build_repository().await.map_err(AppError::from)?;

    Ok(Self { rule_engine, rule_repo })
  }
}

pub async fn build_rule_engine() -> anyhow::Result<Arc<dyn RuleEnginePort>> {
  let adapter = Arc::new(DataflowRuleEngine::new());

  Ok(adapter)
}

async fn build_repository() -> anyhow::Result<Arc<dyn RuleRepositoryPort>> {
  let redis_url = std::env::var("RVE_REDIS_URL").context("RVE_REDIS_URL must be set")?;
  let redis_url = redis_url.trim().to_owned();
  if redis_url.is_empty() {
    return Err(anyhow!("RVE_REDIS_URL cannot be empty"));
  }

  let prefix = std::env::var("RVE_REDIS_PREFIX").unwrap_or_else(|_| "rve".to_owned());
  let repo = RedisRuleRepository::new(&redis_url, prefix)
    .map_err(|err| anyhow!("redis repository init failed: {err}"))?;

  repo.all().await.map_err(|err| anyhow!("redis health check failed: {err}"))?;

  info!(target: "BANNER", "using redis-backed rule repository");
  Ok(Arc::new(repo))
}
