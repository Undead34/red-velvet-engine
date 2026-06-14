use axum::{
  Json,
  http::{HeaderMap, HeaderValue, StatusCode, header},
};
use tracing::{Span, info, warn};

use rve_core::{
  domain::event::Event,
  ports::rule_engine::RuntimeEngineError,
  services::engine::{Decision, DecisionService, DecisionServiceError, DecisionTrace},
};

use crate::http::contracts::{
  API_VERSION, DECISION_PAYLOAD_CANONICAL_VERSION, DECISION_PAYLOAD_LEGACY_SUNSET,
};
use crate::http::openapi::ErrorResponse;
use crate::http::state::AppState;

use super::payload::parse_event_payload;
use crate::http::api_v1::errors::map_engine_service_error;

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
  let parsed = parse_event_payload(request).map_err(|error| {
    (
      StatusCode::UNPROCESSABLE_ENTITY,
      Json(ErrorResponse {
        code: "unprocessable_entity".to_owned(),
        message: error.to_string(),
        validation: None,
      }),
    )
  })?;
  let event = parsed.event;
  let event_channel = event.header.channel.as_ref().map(ToString::to_string);
  let event_id = event.header.event_id.as_ref().map(ToString::to_string);
  let span = Span::current();
  span.record("event_id", tracing::field::display(event_id.as_deref().unwrap_or("")));
  span.record("event_channel", tracing::field::display(event_channel.as_deref().unwrap_or("")));
  span.record("legacy_payload_alias", parsed.used_legacy_payload_alias);

  let decision = match run_decision(state, &event, with_trace).await {
    Ok(decision) => decision,
    Err(DecisionServiceError::Runtime(RuntimeEngineError::Configuration { .. })) => {
      warn!(
        event_id = ?event_id,
        event_channel = ?event_channel,
        with_trace,
        "runtime not ready during decision; attempting repository reload"
      );
      DecisionService::reload_rules(state.rule_repo.as_ref(), state.rule_engine.as_ref())
        .await
        .map_err(map_engine_service_error)?;
      run_decision(state, &event, with_trace).await.map_err(map_engine_service_error)?
    }
    Err(error) => return Err(map_engine_service_error(error)),
  };

  let mut headers = HeaderMap::new();
  headers.insert("x-rve-api-version", HeaderValue::from_static(API_VERSION));
  headers.insert(
    "x-rve-decision-contract-version",
    HeaderValue::from_static(DECISION_PAYLOAD_CANONICAL_VERSION),
  );
  if parsed.used_legacy_payload_alias {
    headers.insert("deprecation", HeaderValue::from_static("true"));
    headers.insert("sunset", HeaderValue::from_static(DECISION_PAYLOAD_LEGACY_SUNSET));
    headers.insert(
      header::WARNING,
      HeaderValue::from_static(
        "299 - \"payload.money.value is deprecated; use payload.money.minor_units\"",
      ),
    );
  }

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
    DecisionService::decide_with_trace(state.rule_engine.as_ref(), event)
      .await
      .map(DecisionEnvelope::Trace)
  } else {
    DecisionService::decide(state.rule_engine.as_ref(), event).await.map(DecisionEnvelope::Decision)
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
