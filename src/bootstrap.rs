use anyhow::{Context, anyhow};
use std::sync::Arc;
use tracing::info;

use rve_core::application::{
  DecisionInputPort, DecisionService, RuleCommandInputPort, RuleCommandService, RuleQueryInputPort,
  RuleQueryService, RuntimeControlInputPort, RuntimeControlService,
};
use rve_core::ports::{RuleEnginePort, RuleRepositoryPort};

use crate::{
  error::AppError, infrastructure::persistence::RedisRuleRepository,
  infrastructure::runtime::DataflowRuleEngine,
};

/// Container responsible for wiring application services to outbound adapters.
pub struct AppContainer {
  pub decision_service: Arc<dyn DecisionInputPort>,
  pub rule_command_service: Arc<dyn RuleCommandInputPort>,
  pub rule_query_service: Arc<dyn RuleQueryInputPort>,
  pub runtime_control_service: Arc<dyn RuntimeControlInputPort>,
}

impl AppContainer {
  /// Builds the production wiring graph.
  pub async fn bootstrap() -> Result<Self, AppError> {
    let rule_engine = build_rule_engine().await.map_err(AppError::from)?;
    let rule_repo = build_repository().await.map_err(AppError::from)?;

    Ok(Self::from_ports(rule_repo, rule_engine))
  }

  /// Builds the container from already constructed outbound ports.
  #[must_use]
  pub fn from_ports(
    rule_repo: Arc<dyn RuleRepositoryPort>,
    rule_engine: Arc<dyn RuleEnginePort>,
  ) -> Self {
    Self {
      decision_service: Arc::new(DecisionService::new(rule_repo.clone(), rule_engine.clone())),
      rule_command_service: Arc::new(RuleCommandService::new(rule_repo.clone())),
      rule_query_service: Arc::new(RuleQueryService::new(rule_repo.clone())),
      runtime_control_service: Arc::new(RuntimeControlService::new(rule_repo, rule_engine)),
    }
  }
}

/// Builds the runtime engine adapter.
pub async fn build_rule_engine() -> anyhow::Result<Arc<dyn RuleEnginePort>> {
  let adapter = Arc::new(DataflowRuleEngine::new());

  Ok(adapter)
}

/// Builds the rule repository adapter.
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
