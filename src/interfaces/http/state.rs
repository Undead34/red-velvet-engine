use std::sync::Arc;

use crate::bootstrap::AppContainer;

use rve_core::application::{
  DecisionInputPort, DecisionService, RuleCommandInputPort, RuleCommandService, RuleQueryInputPort,
  RuleQueryService, RuntimeControlInputPort, RuntimeControlService,
};
use rve_core::{RuleEnginePort, RuleRepositoryPort};

/// Application-facing state injected into HTTP handlers.
#[derive(Clone)]
pub struct AppState {
  pub decision_service: Arc<dyn DecisionInputPort>,
  pub rule_command_service: Arc<dyn RuleCommandInputPort>,
  pub rule_query_service: Arc<dyn RuleQueryInputPort>,
  pub runtime_control_service: Arc<dyn RuntimeControlInputPort>,
}

impl AppState {
  /// Creates an HTTP state value from outbound ports.
  #[must_use]
  pub fn new(rule_repo: Arc<dyn RuleRepositoryPort>, rule_engine: Arc<dyn RuleEnginePort>) -> Self {
    Self {
      decision_service: Arc::new(DecisionService::new(rule_repo.clone(), rule_engine.clone())),
      rule_command_service: Arc::new(RuleCommandService::new(rule_repo.clone())),
      rule_query_service: Arc::new(RuleQueryService::new(rule_repo.clone())),
      runtime_control_service: Arc::new(RuntimeControlService::new(rule_repo, rule_engine)),
    }
  }
}

impl From<AppContainer> for AppState {
  fn from(container: AppContainer) -> Self {
    Self {
      decision_service: container.decision_service,
      rule_command_service: container.rule_command_service,
      rule_query_service: container.rule_query_service,
      runtime_control_service: container.runtime_control_service,
    }
  }
}
