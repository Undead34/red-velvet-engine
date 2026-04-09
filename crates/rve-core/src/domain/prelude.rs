//! Domain prelude with frequently used primitives.
//!
//! Importing `rve_core::domain::prelude::*` gives downstream crates a concise
//! way to access the most common risk engine types without reaching into deep
//! module hierarchies. This follows the [`proj-prelude-module`] guideline from
//! the Rust Best Practices reference.
//!
//! [`proj-prelude-module`]: https://rust-lang.github.io/api-guidelines/naming.html#c-prelude

pub use crate::domain::common::{
  AccountId, BankRef, Channel, CountryCode, Currency, CurrencyCode, DeviceId, EventId, EventSource,
  Flag, Instrument, KycLevel, LocaleTag, Money, MoneyError, RuleId, Score, ScoreError, SessionId,
  Severity, SeverityError, TimestampMs, TimestampMsError, TimezoneName, UserAgent,
};
pub use crate::domain::event::{
  Context, EnvironmentContext, Features, FinancialFeatures, GeoContext, Header, NetworkContext,
  Parties, Party, Payload, Signals, ValueTransfer,
};
pub use crate::domain::rule::{
  FunctionKind, JSONLOGIC_ROOT_VARS, RolloutPolicy, RuleAction, RuleAudit, RuleDecision,
  RuleDefinition, RuleEnforcement, RuleEvaluation, RuleExpression, RuleFunctionSpec, RuleIdentity,
  RuleMode, RulePolicy, RuleSchedule, RuleScope, RuleState,
};
pub use crate::domain::{DomainError, Event, Rule};
