use std::time::Instant;

use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use tracing::error;

use rve_core::{
  domain::{common::Severity, event::Event, rule::RuleAction},
  services::engine::RuleHit,
};

use crate::http::state::AppState;

pub async fn create_decision(
  State(state): State<AppState>,
  Json(request): Json<DecisionRequest>,
) -> Result<Json<DecisionResponse>, StatusCode> {
  let started = Instant::now();
  let result = state.engine.evaluate(&request.event).map_err(|err| {
    error!(target: "BANNER", %err, "engine evaluation failed");
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  let hits: Vec<DecisionHit> = result.hits.into_iter().map(DecisionHit::from).collect();
  let latency_ms = started.elapsed().as_millis().min(u128::from(u32::MAX)) as u32;

  Ok(Json(DecisionResponse {
    decision: DecisionPayload {
      score: result.score,
      hits,
      metadata: DecisionMetadata {
        evaluated_rules: result.evaluated_rules,
        latency_ms,
        rollout_bucket: result.rollout_bucket,
      },
    },
  }))
}

#[derive(Deserialize)]
pub struct DecisionRequest {
  pub event: Event,
}

#[derive(Serialize)]
pub struct DecisionResponse {
  pub decision: DecisionPayload,
}

#[derive(Serialize)]
pub struct DecisionPayload {
  pub score: f32,
  pub hits: Vec<DecisionHit>,
  pub metadata: DecisionMetadata,
}

#[derive(Serialize)]
pub struct DecisionHit {
  pub rule_id: String,
  pub action: RuleAction,
  pub severity: Severity,
  pub explanation: String,
}

impl From<RuleHit> for DecisionHit {
  fn from(hit: RuleHit) -> Self {
    let explanation =
      hit.explanation.filter(|s| !s.is_empty()).unwrap_or_else(|| "Rule triggered".into());
    Self { rule_id: hit.rule_id, action: hit.action, severity: hit.severity, explanation }
  }
}

#[derive(Serialize)]
pub struct DecisionMetadata {
  pub evaluated_rules: u32,
  pub latency_ms: u32,
  pub rollout_bucket: u8,
}
