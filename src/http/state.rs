use std::sync::Arc;

use crate::bootstrap::AppContainer;

use rve_core::{RuleEnginePort, RuleRepositoryPort};

#[derive(Clone)]
pub struct AppState {
  pub rule_engine: Arc<dyn RuleEnginePort>,
  pub rule_repo: Arc<dyn RuleRepositoryPort>,
}

// Implementamos From para facilitar la conversión
impl From<AppContainer> for AppState {
  fn from(container: AppContainer) -> Self {
    Self { rule_engine: container.rule_engine, rule_repo: container.rule_repo }
  }
}
