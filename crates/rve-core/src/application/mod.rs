//! Application services and input ports for the hexagonal core.
//!
//! Inbound adapters should depend on the traits defined in this module rather
//! than talking directly to repositories or runtime engines. The concrete
//! services coordinate domain objects through outbound [`crate::ports`].

mod decision_service;
mod rule_command_service;
mod rule_query_service;
mod runtime_control_service;

pub use decision_service::{
  Decision, DecisionHit, DecisionInputPort, DecisionOutcome, DecisionService, DecisionServiceError,
  DecisionTrace,
};
pub use rule_command_service::{RuleCommandInputPort, RuleCommandService, RuleCommandServiceError};
pub use rule_query_service::{RuleQueryInputPort, RuleQueryService, RuleQueryServiceError};
pub use runtime_control_service::{
  RuntimeControlError, RuntimeControlInputPort, RuntimeControlOverview, RuntimeControlService,
};
