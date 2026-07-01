use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{info, instrument, warn};

use crate::domain::{
  common::{RuleId, Severity},
  event::Event,
  rule::RuleAction,
};
use crate::ports::rule_engine::{
  RuleEnginePort, RuleEngineTrace, RulesetSnapshot, RuntimeEngineError, RuntimeEvaluation,
  RuntimeHit,
};
use crate::ports::rule_repository::{RuleRepositoryError, RuleRepositoryPort};

/// Decision payload returned by the application layer.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Decision {
  /// Final aggregated risk score.
  pub score: f32,
  /// Final enforcement outcome derived from the strongest hit.
  pub outcome: DecisionOutcome,
  /// Rules that matched during evaluation.
  pub hits: Vec<DecisionHit>,
  /// Number of rules evaluated by the runtime.
  pub evaluated_rules: u32,
  /// Number of rules that emitted a hit.
  pub executed_rules: u32,
  /// Rollout bucket selected for the event.
  pub rollout_bucket: u8,
}

/// Decision response enriched with runtime trace data.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DecisionTrace {
  /// Business decision derived from the runtime evaluation.
  pub decision: Decision,
  /// Low-level runtime trace for troubleshooting.
  pub trace: RuleEngineTrace,
}

/// Rule hit included in a [`Decision`].
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DecisionHit {
  /// Identifier of the matching rule.
  pub rule_id: RuleId,
  /// Enforcement action declared by the rule.
  pub action: RuleAction,
  /// Severity declared by the rule.
  pub severity: Severity,
  /// Score contribution applied by the rule.
  pub score_delta: f32,
  /// Optional human-readable explanation.
  pub explanation: Option<String>,
  /// Tags emitted by the rule.
  pub tags: Vec<String>,
}

/// Final outcome returned by decision use cases.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum DecisionOutcome {
  /// The event is allowed.
  Allow,
  /// The event should be manually reviewed.
  Review,
  /// The event should be blocked.
  Block,
  /// The event only produces tags.
  TagOnly,
  /// The outcome could not be derived.
  Unknown,
}

impl From<RuleAction> for DecisionOutcome {
  fn from(value: RuleAction) -> Self {
    match value {
      RuleAction::Allow => DecisionOutcome::Allow,
      RuleAction::Review => DecisionOutcome::Review,
      RuleAction::Block => DecisionOutcome::Block,
      RuleAction::TagOnly => DecisionOutcome::TagOnly,
    }
  }
}

impl From<RuntimeHit> for DecisionHit {
  fn from(value: RuntimeHit) -> Self {
    Self {
      rule_id: value.rule_id,
      action: value.action,
      severity: value.severity,
      score_delta: value.score_delta,
      explanation: value.explanation,
      tags: value.tags,
    }
  }
}

impl Decision {
  /// Builds a decision from pre-computed scores and hits.
  pub fn with_scores(
    score: f32,
    hits: Vec<DecisionHit>,
    evaluated_rules: u32,
    rollout_bucket: u8,
  ) -> Self {
    let executed_rules = hits.len() as u32;
    let outcome = match best_action(&hits) {
      Some(action) => DecisionOutcome::from(action),
      None => DecisionOutcome::Allow,
    };

    Self { score, outcome, hits, evaluated_rules, executed_rules, rollout_bucket }
  }

  /// Builds a decision from a runtime evaluation.
  pub fn from_runtime(result: RuntimeEvaluation) -> Self {
    let hits = result.hits.into_iter().map(DecisionHit::from).collect::<Vec<_>>();
    Self::with_scores(result.score, hits, result.evaluated_rules, result.rollout_bucket)
  }
}

impl DecisionTrace {
  /// Builds a traced decision from a runtime execution result.
  #[must_use]
  pub fn from_runtime(execution: crate::ports::rule_engine::RuleEngineExecution) -> Self {
    Self { decision: Decision::from_runtime(execution.evaluation), trace: execution.trace }
  }
}

fn best_action(hits: &[DecisionHit]) -> Option<RuleAction> {
  let mut best = None::<RuleAction>;
  for hit in hits {
    best = Some(match best {
      None => hit.action,
      Some(current) => {
        let current_weight = action_weight(&current);
        let candidate = action_weight(&hit.action);
        if candidate > current_weight { hit.action } else { current }
      }
    });
  }
  best
}

fn action_weight(action: &RuleAction) -> u8 {
  match action {
    RuleAction::Allow => 0,
    RuleAction::TagOnly => 1,
    RuleAction::Review => 2,
    RuleAction::Block => 3,
  }
}

/// Errors surfaced by [`DecisionService`] use cases.
#[derive(Debug, Error)]
pub enum DecisionServiceError {
  /// Repository access failed while attempting to synchronize the runtime.
  #[error(transparent)]
  Repository(#[from] RuleRepositoryError),
  /// Runtime compilation or evaluation failed.
  #[error(transparent)]
  Runtime(#[from] RuntimeEngineError),
}

/// Input port for decision evaluation use cases.
#[async_trait]
pub trait DecisionInputPort: Send + Sync {
  /// Evaluates a domain event and returns the resulting decision.
  async fn decide(&self, event: &Event) -> Result<Decision, DecisionServiceError>;

  /// Evaluates a domain event and returns the runtime trace.
  async fn decide_with_trace(&self, event: &Event) -> Result<DecisionTrace, DecisionServiceError>;
}

/// Default application service for decision evaluation.
#[derive(Clone)]
pub struct DecisionService {
  repository: Arc<dyn RuleRepositoryPort>,
  runtime: Arc<dyn RuleEnginePort>,
}

impl DecisionService {
  /// Creates a decision service backed by repository and runtime ports.
  #[must_use]
  pub fn new(repository: Arc<dyn RuleRepositoryPort>, runtime: Arc<dyn RuleEnginePort>) -> Self {
    Self { repository, runtime }
  }

  #[instrument(
    name = "decision_service.reload_runtime",
    skip(self),
    fields(repository_rules = tracing::field::Empty, loaded_rules = tracing::field::Empty, version = tracing::field::Empty)
  )]
  async fn reload_runtime(&self) -> Result<RulesetSnapshot, DecisionServiceError> {
    let rules = self.repository.all().await?;
    tracing::Span::current().record("repository_rules", rules.len());
    let snapshot = self.runtime.publish_rules(rules).await?;
    tracing::Span::current().record("loaded_rules", snapshot.loaded_rules);
    tracing::Span::current().record("version", snapshot.version);
    info!("ruleset published to runtime");
    Ok(snapshot)
  }
}

#[async_trait]
impl DecisionInputPort for DecisionService {
  #[instrument(
    name = "decision_service.decide",
    skip(self, event),
    fields(event_id = tracing::field::Empty, event_channel = tracing::field::Empty, evaluated_rules = tracing::field::Empty, hit_count = tracing::field::Empty, score = tracing::field::Empty)
  )]
  async fn decide(&self, event: &Event) -> Result<Decision, DecisionServiceError> {
    if let Some(event_id) = event.header.event_id.as_ref() {
      tracing::Span::current().record("event_id", event_id.to_string());
    }
    if let Some(channel) = event.header.channel.as_ref() {
      tracing::Span::current().record("event_channel", channel.to_string());
    }

    let evaluation = match self.runtime.evaluate(event).await {
      Ok(result) => result,
      Err(RuntimeEngineError::Configuration { .. }) => {
        warn!("runtime not ready during decision; attempting repository reload");
        self.reload_runtime().await?;
        self.runtime.evaluate(event).await?
      }
      Err(error) => return Err(DecisionServiceError::Runtime(error)),
    };

    tracing::Span::current().record("evaluated_rules", evaluation.evaluated_rules);
    tracing::Span::current().record("hit_count", evaluation.hits.len());
    tracing::Span::current().record("score", evaluation.score);
    Ok(Decision::from_runtime(evaluation))
  }

  #[instrument(
    name = "decision_service.decide_with_trace",
    skip(self, event),
    fields(event_id = tracing::field::Empty, event_channel = tracing::field::Empty, evaluated_rules = tracing::field::Empty, hit_count = tracing::field::Empty, score = tracing::field::Empty, trace_steps = tracing::field::Empty)
  )]
  async fn decide_with_trace(&self, event: &Event) -> Result<DecisionTrace, DecisionServiceError> {
    if let Some(event_id) = event.header.event_id.as_ref() {
      tracing::Span::current().record("event_id", event_id.to_string());
    }
    if let Some(channel) = event.header.channel.as_ref() {
      tracing::Span::current().record("event_channel", channel.to_string());
    }

    let execution = match self.runtime.evaluate_with_trace(event).await {
      Ok(result) => result,
      Err(RuntimeEngineError::Configuration { .. }) => {
        warn!("runtime not ready during decision trace; attempting repository reload");
        self.reload_runtime().await?;
        self.runtime.evaluate_with_trace(event).await?
      }
      Err(error) => return Err(DecisionServiceError::Runtime(error)),
    };

    tracing::Span::current().record("evaluated_rules", execution.evaluation.evaluated_rules);
    tracing::Span::current().record("hit_count", execution.evaluation.hits.len());
    tracing::Span::current().record("score", execution.evaluation.score);
    tracing::Span::current().record("trace_steps", execution.trace.steps.len());
    Ok(DecisionTrace::from_runtime(execution))
  }
}
