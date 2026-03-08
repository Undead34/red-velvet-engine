use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
  domain::{
    common::{RuleId, Severity},
    event::Event,
    rule::RuleAction,
  },
  ports::{
    RuleRepositoryError, RuleRepositoryPort, RulesetSnapshot, RuntimeEngineError, RuntimeEnginePort,
    RuntimeEvaluation, RuntimeHit,
  },
};

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

#[derive(Debug, Error)]
pub enum DecisionServiceError {
  #[error(transparent)]
  Repository(#[from] RuleRepositoryError),
  #[error(transparent)]
  Runtime(#[from] RuntimeEngineError),
}

pub struct DecisionService;

impl DecisionService {
  pub async fn reload_rules<R, E>(
    repository: &R,
    runtime: &E,
  ) -> Result<RulesetSnapshot, DecisionServiceError>
  where
    R: RuleRepositoryPort + ?Sized,
    E: RuntimeEnginePort + ?Sized,
  {
    let rules = repository.all().await?;
    runtime.publish_rules(rules).await.map_err(DecisionServiceError::from)
  }

  pub async fn decide<E>(runtime: &E, event: &Event) -> Result<Decision, DecisionServiceError>
  where
    E: RuntimeEnginePort + ?Sized,
  {
    let evaluation = runtime.evaluate(event).await?;
    Ok(Decision::from_runtime(evaluation))
  }
}

pub type EngineResult = Decision;
pub type RuleHit = DecisionHit;
