use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{info, instrument};

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Decision {
  pub score: f32,
  pub outcome: DecisionOutcome,
  pub hits: Vec<DecisionHit>,
  pub evaluated_rules: u32,
  pub executed_rules: u32,
  pub rollout_bucket: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DecisionTrace {
  pub decision: Decision,
  pub trace: RuleEngineTrace,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DecisionHit {
  pub rule_id: RuleId,
  pub action: RuleAction,
  pub severity: Severity,
  pub score_delta: f32,
  pub explanation: Option<String>,
  pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum DecisionOutcome {
  Allow,
  Review,
  Block,
  TagOnly,
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

  pub fn from_runtime(result: RuntimeEvaluation) -> Self {
    let hits = result.hits.into_iter().map(DecisionHit::from).collect::<Vec<_>>();
    Self::with_scores(result.score, hits, result.evaluated_rules, result.rollout_bucket)
  }
}

impl DecisionTrace {
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

/// Errors surfaced by [`DecisionService`] orchestration routines.
#[derive(Debug, Error)]
pub enum DecisionServiceError {
  #[error(transparent)]
  Repository(#[from] RuleRepositoryError),
  #[error(transparent)]
  Runtime(#[from] RuntimeEngineError),
}

pub struct DecisionService;

impl DecisionService {
  /// Reloads all rules from the repository into the execution runtime.
  ///
  /// # Errors
  ///
  /// Returns [`DecisionServiceError::Repository`] when the repository cannot
  /// return the current rules, or [`DecisionServiceError::Runtime`] when the
  /// runtime fails to compile/publish the ruleset.
  #[must_use]
  #[instrument(
    name = "decision_service.reload_rules",
    skip(repository, runtime),
    fields(repository_rules = tracing::field::Empty, loaded_rules = tracing::field::Empty, version = tracing::field::Empty)
  )]
  pub async fn reload_rules<R, E>(
    repository: &R,
    runtime: &E,
  ) -> Result<RulesetSnapshot, DecisionServiceError>
  where
    R: RuleRepositoryPort + ?Sized,
    E: RuleEnginePort + ?Sized,
  {
    let rules = repository.all().await?;
    tracing::Span::current().record("repository_rules", rules.len());
    let snapshot = runtime.publish_rules(rules).await.map_err(DecisionServiceError::from)?;
    tracing::Span::current().record("loaded_rules", snapshot.loaded_rules);
    tracing::Span::current().record("version", snapshot.version);
    info!("ruleset published to runtime");
    Ok(snapshot)
  }

  /// Evaluates an [`Event`] with the active runtime ruleset.
  ///
  /// # Errors
  ///
  /// Returns [`DecisionServiceError::Runtime`] when the runtime backend fails
  /// to evaluate the event successfully.
  #[must_use]
  #[instrument(
    name = "decision_service.decide",
    skip(runtime, event),
    fields(event_id = tracing::field::Empty, event_channel = tracing::field::Empty, evaluated_rules = tracing::field::Empty, hit_count = tracing::field::Empty, score = tracing::field::Empty)
  )]
  pub async fn decide<E>(runtime: &E, event: &Event) -> Result<Decision, DecisionServiceError>
  where
    E: RuleEnginePort + ?Sized,
  {
    if let Some(event_id) = event.header.event_id.as_ref() {
      tracing::Span::current().record("event_id", event_id.to_string());
    }
    if let Some(channel) = event.header.channel.as_ref() {
      tracing::Span::current().record("event_channel", channel.to_string());
    }
    let evaluation = runtime.evaluate(event).await?;
    tracing::Span::current().record("evaluated_rules", evaluation.evaluated_rules);
    tracing::Span::current().record("hit_count", evaluation.hits.len());
    tracing::Span::current().record("score", evaluation.score);
    Ok(Decision::from_runtime(evaluation))
  }

  /// Evaluates an [`Event`] and returns the runtime trace.
  ///
  /// # Errors
  ///
  /// Returns [`DecisionServiceError::Runtime`] when the runtime backend fails
  /// to evaluate the event successfully.
  #[must_use]
  #[instrument(
    name = "decision_service.decide_with_trace",
    skip(runtime, event),
    fields(event_id = tracing::field::Empty, event_channel = tracing::field::Empty, evaluated_rules = tracing::field::Empty, hit_count = tracing::field::Empty, score = tracing::field::Empty, trace_steps = tracing::field::Empty)
  )]
  pub async fn decide_with_trace<E>(
    runtime: &E,
    event: &Event,
  ) -> Result<DecisionTrace, DecisionServiceError>
  where
    E: RuleEnginePort + ?Sized,
  {
    if let Some(event_id) = event.header.event_id.as_ref() {
      tracing::Span::current().record("event_id", event_id.to_string());
    }
    if let Some(channel) = event.header.channel.as_ref() {
      tracing::Span::current().record("event_channel", channel.to_string());
    }
    let execution = runtime.evaluate_with_trace(event).await?;
    tracing::Span::current().record("evaluated_rules", execution.evaluation.evaluated_rules);
    tracing::Span::current().record("hit_count", execution.evaluation.hits.len());
    tracing::Span::current().record("score", execution.evaluation.score);
    tracing::Span::current().record("trace_steps", execution.trace.steps.len());
    Ok(DecisionTrace::from_runtime(execution))
  }
}

pub type EngineResult = Decision;
pub type RuleHit = DecisionHit;
