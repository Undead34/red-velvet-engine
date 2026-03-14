use std::sync::Arc;

use rve_core::ports::RuleRepositoryPort;

use crate::{engine::RVEngine, store::InMemoryRuleRepository};

#[derive(Clone)]
pub struct AppState {
  pub engine: Arc<RVEngine>,
  pub rule_repo: Arc<dyn RuleRepositoryPort>,
}

impl AppState {
  pub async fn new() -> Self {
    let repo = Arc::new(InMemoryRuleRepository::seeded());
    let engine = Arc::new(RVEngine::new());

    Self { engine, rule_repo: repo }
  }
}
