//! Core types and application boundaries for Red Velvet Engine.
//!
//! The crate is intentionally split into three layers:
//!
//! - [`domain`]: aggregates, value objects, and invariants.
//! - [`ports`]: outbound interfaces required by the core.
//! - [`application`]: input ports and orchestration services consumed by adapters.

pub mod application;
pub mod domain;
pub mod ports;

pub use application::{
  Decision, DecisionHit, DecisionInputPort, DecisionOutcome, DecisionService, DecisionServiceError,
  DecisionTrace, RuleCommandInputPort, RuleCommandService, RuleCommandServiceError,
  RuleQueryInputPort, RuleQueryService, RuleQueryServiceError, RuntimeControlError,
  RuntimeControlInputPort, RuntimeControlOverview, RuntimeControlService,
};
pub use ports::{RuleEnginePort, RuleRepositoryPort};

// Engine Edition
pub const ENGINE_NAME: &str = "Red Velvet Engine";
pub const ENGINE_CODENAME: &str = "Black Cherry";
pub const ENGINE_EMOJI: &str = "🍒";

pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PKG_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// "Red Velvet Engine — Black Cherry 🍒"
pub const ENGINE_EDITION: &str = concat!("Red Velvet Engine", " — ", "Black Cherry", " ", "🍒");
