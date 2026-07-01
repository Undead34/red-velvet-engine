use axum::{
  Json,
  http::{HeaderMap, HeaderValue, StatusCode},
};
use tracing::{Span, info, warn};

use rve_core::{
  application::{Decision, DecisionServiceError, DecisionTrace},
  domain::event::Event,
};

use crate::interfaces::http::contracts::{API_VERSION, DECISION_PAYLOAD_CANONICAL_VERSION};
use crate::interfaces::http::openapi::ErrorResponse;
use crate::interfaces::http::state::AppState;

use super::mapper::event_from_request;
use crate::interfaces::http::api_v1::errors::map_decision_service_error;

pub enum DecisionEnvelope {
  Decision(Decision),
  Trace(DecisionTrace),
}

/// Parse the request body, run the engine, and build response headers.
pub async fn evaluate_request(
  state: &AppState,
  request: serde_json::Value,
  with_trace: bool,
) -> Result<(HeaderMap, DecisionEnvelope), (StatusCode, Json<ErrorResponse>)> {
  let event = event_from_request(request).map_err(|error| {
    (
      StatusCode::UNPROCESSABLE_ENTITY,
      Json(ErrorResponse {
        code: "unprocessable_entity".to_owned(),
        message: error.to_string(),
        validation: None,
      }),
    )
  })?;
  let event_channel = event.header.channel.as_ref().map(ToString::to_string);
  let event_id = event.header.event_id.as_ref().map(ToString::to_string);
  let span = Span::current();
  span.record("event_id", tracing::field::display(event_id.as_deref().unwrap_or("")));
  span.record("event_channel", tracing::field::display(event_channel.as_deref().unwrap_or("")));

  let decision = run_decision(state, &event, with_trace).await.map_err(|error| {
    warn!(
      event_id = ?event_id,
      event_channel = ?event_channel,
      with_trace,
      error = %error,
      "decision evaluation failed"
    );
    map_decision_service_error(error)
  })?;

  let mut headers = HeaderMap::new();
  headers.insert("x-rve-api-version", HeaderValue::from_static(API_VERSION));
  headers.insert(
    "x-rve-decision-contract-version",
    HeaderValue::from_static(DECISION_PAYLOAD_CANONICAL_VERSION),
  );

  log_decision_observation(&decision, event_id.as_deref(), event_channel.as_deref(), with_trace);

  Ok((headers, decision))
}

/// Execute the engine for a single event.
pub async fn run_decision(
  state: &AppState,
  event: &Event,
  with_trace: bool,
) -> Result<DecisionEnvelope, DecisionServiceError> {
  if with_trace {
    state.decision_service.decide_with_trace(event).await.map(DecisionEnvelope::Trace)
  } else {
    state.decision_service.decide(event).await.map(DecisionEnvelope::Decision)
  }
}

fn log_decision_observation(
  decision: &DecisionEnvelope,
  event_id: Option<&str>,
  event_channel: Option<&str>,
  with_trace: bool,
) {
  match decision {
    DecisionEnvelope::Decision(decision) => {
      info!(
        event_id,
        event_channel,
        with_trace,
        outcome = ?decision.outcome,
        score = decision.score,
        evaluated_rules = decision.evaluated_rules,
        executed_rules = decision.executed_rules,
        hit_count = decision.hits.len(),
        "decision computed"
      );
    }
    DecisionEnvelope::Trace(trace) => {
      info!(
        event_id,
        event_channel,
        with_trace,
        outcome = ?trace.decision.outcome,
        score = trace.decision.score,
        evaluated_rules = trace.decision.evaluated_rules,
        executed_rules = trace.decision.executed_rules,
        hit_count = trace.decision.hits.len(),
        trace_steps = trace.trace.steps.len(),
        trace_channel = ?trace.trace.channel,
        "decision trace computed"
      );
    }
  }
}
