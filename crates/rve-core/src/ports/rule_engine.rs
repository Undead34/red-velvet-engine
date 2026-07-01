use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::{
  common::{RuleId, Severity},
  event::Event,
  rule::{Rule, RuleAction},
};

/// Compilation counters reported by the runtime.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct RuleCompileStats {
  pub total_rules: u32,
  pub compiled_rules: u32,
  pub failed_rules: u32,
}

/// Snapshot returned whenever a ruleset is published to the runtime.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RulesetSnapshot {
  pub version: u64,
  pub loaded_rules: u32,
  pub compile_stats: RuleCompileStats,
}

/// Single runtime hit emitted by a matching rule.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeHit {
  pub rule_id: RuleId,
  pub action: RuleAction,
  pub severity: Severity,
  pub score_delta: f32,
  pub explanation: Option<String>,
  pub tags: Vec<String>,
}

/// Raw runtime evaluation output.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeEvaluation {
  pub score: f32,
  pub hits: Vec<RuntimeHit>,
  pub evaluated_rules: u32,
  pub rollout_bucket: u8,
}

/// Runtime status exposed by outbound engines.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleEngineStatus {
  pub mode: String,
  pub ready: bool,
  pub version: u64,
  pub loaded_rules: u32,
  pub compile_stats: RuleCompileStats,
}

/// Single step in an execution trace.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleEngineTraceStep {
  pub workflow_id: String,
  pub rule_id: Option<String>,
  pub runtime_channel: Option<String>,
  pub task_id: Option<String>,
  pub result: String,
}

/// Trace emitted by a runtime evaluation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleEngineTrace {
  pub channel: Option<String>,
  pub steps: Vec<RuleEngineTraceStep>,
}

/// Raw runtime output containing both evaluation and trace information.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleEngineExecution {
  pub evaluation: RuntimeEvaluation,
  pub trace: RuleEngineTrace,
}

/// Runtime-level failures exposed to the application layer.
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum RuntimeEngineError {
  #[error("runtime compilation error for {rule_id:?}: {message}")]
  Compilation { rule_id: Option<RuleId>, message: String },
  #[error("runtime evaluation error for {rule_id:?}: {message}")]
  Evaluation { rule_id: Option<RuleId>, message: String },
  #[error("runtime configuration error: {message}")]
  Configuration { message: String },
  #[error("runtime engine not implemented: {message}")]
  NotImplemented { message: String },
  #[error("runtime internal error: {message}")]
  Internal { message: String },
}

/// Outbound port for rule evaluation runtimes.
#[async_trait]
pub trait RuleEnginePort: Send + Sync {
  /// Publishes a full ruleset snapshot to the runtime.
  async fn publish_rules(&self, rules: Vec<Rule>) -> Result<RulesetSnapshot, RuntimeEngineError>;

  /// Evaluates an event using the active ruleset.
  async fn evaluate(&self, event: &Event) -> Result<RuntimeEvaluation, RuntimeEngineError>;

  /// Evaluates an event and returns the underlying execution trace.
  async fn evaluate_with_trace(
    &self,
    event: &Event,
  ) -> Result<RuleEngineExecution, RuntimeEngineError>;

  /// Evaluates an event using an explicit runtime channel override.
  async fn evaluate_in_channel(
    &self,
    channel: &str,
    event: &Event,
  ) -> Result<RuntimeEvaluation, RuntimeEngineError>;

  /// Rebuilds the currently published ruleset.
  async fn reload(&self) -> Result<RulesetSnapshot, RuntimeEngineError>;

  /// Returns the current runtime status.
  fn status(&self) -> Result<RuleEngineStatus, RuntimeEngineError>;
}
