//! Fraud detection rule engine domain model.
//!
//! This module provides the core primitives for defining, validating, and
//! evaluating fraud rules. It follows a structured, hierarchical model
//! designed for high-performance execution and strict data sandboxing.
//!
//! # Architecture Overview
//!
//! The domain is organized around the [`Rule`] aggregate root, which coordinates
//! four primary functional areas:
//!
//! 1.  **Identity**: Metadata and tracking ([`RuleIdentity`]).
//! 2.  **Policy**: Lifecycle, scheduling, and rollout ([`RulePolicy`]).
//! 3.  **Definition**: The logical "If" criteria ([`RuleDefinition`]).
//! 4.  **Decision**: The resulting "Then" actions ([`RuleDecision`]).
//!
//! # Execution Flow
//!
//! Before evaluating the heavy business logic, the engine performs a
//! short-circuit check using the rule's policy:
//!
//! * **Mode**: Is the rule [`RuleMode::Active`]?
//! * **Schedule**: Is the current time within the [`RuleSchedule`]?
//! * **Rollout**: Does the event fall within the [`RolloutPolicy`] bucket?
//!
//! Only if these conditions pass, the [`RuleEvaluation`] logic is executed
//! against the event payload.

mod action;
mod audit;
mod decision;
mod definition;
mod enforcement;
mod evaluation;
mod expression;
mod function;
mod meta;
pub mod mode;
mod policy;
mod rollout;
mod rule;
mod schedule;
mod state;

pub use action::RuleAction;
pub use audit::{RuleAudit, RuleAuditError};
pub use decision::RuleDecision;
pub use definition::RuleDefinition;
pub use enforcement::RuleEnforcement;
pub use evaluation::RuleEvaluation;
pub use expression::{JSONLOGIC_ROOT_VARS, RuleExpression};
pub use function::{FunctionKind, RuleFunctionSpec};
pub use meta::RuleIdentity;
pub use mode::RuleMode;
pub use policy::{RulePolicy, RulePolicyError};
pub use rollout::{RolloutPolicy, RuleRolloutError};
pub use rule::Rule;
pub use schedule::{RuleSchedule, RuleScheduleError};
pub use state::{RuleState, RuleStateError};
