use axum::{
  Json,
  extract::State,
  http::{HeaderMap, HeaderValue, StatusCode, header},
};
use serde_json::{Value, json};
use thiserror::Error;
use tracing::{Span, info, instrument, warn};

use rve_core::{
  domain::{
    common::{Currency, Money},
    event::Event,
  },
  ports::rule_engine::RuntimeEngineError,
  services::engine::{Decision, DecisionService, DecisionServiceError, DecisionTrace},
};

use crate::http::contracts::{
  API_VERSION, DECISION_PAYLOAD_CANONICAL_VERSION, DECISION_PAYLOAD_LEGACY_SUNSET,
};
use crate::http::openapi::ErrorResponse;
use crate::http::state::AppState;

#[utoipa::path(
  post,
  path = "/api/v1/decisions",
  tag = "decisions",
  request_body(
    content = crate::http::openapi::DecisionRequestDoc,
    description = "Fraud event payload evaluated against active rules."
  ),
  responses(
    (status = 200, description = "Decision computed successfully", body = crate::http::openapi::DecisionResponseDoc),
    (status = 422, description = "Invalid event payload", body = ErrorResponse),
    (status = 500, description = "Decision runtime error", body = ErrorResponse)
  )
)]
#[instrument(
  name = "http.decisions.create",
  skip(state, request),
  fields(event_id = tracing::field::Empty, event_channel = tracing::field::Empty, legacy_payload_alias = tracing::field::Empty)
)]
pub async fn create_decision(
  State(state): State<AppState>,
  Json(request): Json<Value>,
) -> Result<(HeaderMap, Json<Decision>), (StatusCode, Json<ErrorResponse>)> {
  let (headers, decision) = evaluate_request(&state, request, false).await?;
  Ok((
    headers,
    Json(match decision {
      DecisionEnvelope::Decision(decision) => decision,
      DecisionEnvelope::Trace(_) => unreachable!("trace response requested on decision endpoint"),
    }),
  ))
}

#[utoipa::path(
  post,
  path = "/api/v1/decisions/trace",
  tag = "decisions",
  request_body(
    content = crate::http::openapi::DecisionRequestDoc,
    description = "Fraud event payload evaluated against active rules with execution trace."
  ),
  responses(
    (status = 200, description = "Decision trace computed successfully", body = crate::http::openapi::DecisionTraceResponseDoc),
    (status = 422, description = "Invalid event payload", body = ErrorResponse),
    (status = 500, description = "Decision runtime error", body = ErrorResponse)
  )
)]
#[instrument(
  name = "http.decisions.trace",
  skip(state, request),
  fields(event_id = tracing::field::Empty, event_channel = tracing::field::Empty, legacy_payload_alias = tracing::field::Empty)
)]
pub async fn create_decision_trace(
  State(state): State<AppState>,
  Json(request): Json<Value>,
) -> Result<(HeaderMap, Json<DecisionTrace>), (StatusCode, Json<ErrorResponse>)> {
  let (headers, decision) = evaluate_request(&state, request, true).await?;
  Ok((
    headers,
    Json(match decision {
      DecisionEnvelope::Trace(trace) => trace,
      DecisionEnvelope::Decision(_) => {
        unreachable!("decision response requested on trace endpoint")
      }
    }),
  ))
}

enum DecisionEnvelope {
  Decision(Decision),
  Trace(DecisionTrace),
}

async fn evaluate_request(
  state: &AppState,
  request: Value,
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
        .map_err(map_decision_error)?;
      run_decision(state, &event, with_trace).await.map_err(map_decision_error)?
    }
    Err(error) => return Err(map_decision_error(error)),
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

async fn run_decision(
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

struct ParsedEventPayload {
  event: Event,
  used_legacy_payload_alias: bool,
}

fn parse_event_payload(request: Value) -> Result<ParsedEventPayload, DecisionPayloadError> {
  if let Ok(event) = serde_json::from_value::<Event>(request.clone()) {
    return Ok(ParsedEventPayload { event, used_legacy_payload_alias: false });
  }

  let (normalized, used_legacy_payload_alias) = normalize_legacy_money_payload(request)?;
  serde_json::from_value::<Event>(normalized)
    .map(|event| ParsedEventPayload { event, used_legacy_payload_alias })
    .map_err(|err| DecisionPayloadError::Invalid(format!("invalid event payload: {err}")))
}

fn normalize_legacy_money_payload(
  mut request: Value,
) -> Result<(Value, bool), DecisionPayloadError> {
  let Some(payload) = request.get_mut("payload").and_then(Value::as_object_mut) else {
    return Ok((request, false));
  };

  if payload.get("type").is_none() {
    payload.insert("type".to_owned(), json!("value_transfer"));
  }

  let Some(money) = payload.get_mut("money").and_then(Value::as_object_mut) else {
    return Ok((request, false));
  };

  if money.get("minor_units").is_some() {
    return Ok((request, false));
  }

  let Some(ccy) = money.get("ccy").and_then(Value::as_str) else {
    return Ok((request, false));
  };
  let Some(raw_value) = money.get("value") else {
    return Ok((request, false));
  };

  let amount_text = match raw_value {
    Value::String(text) => text.clone(),
    Value::Number(number) => number.to_string(),
    _ => return Ok((request, false)),
  };

  let currency = Currency::new(ccy).map_err(DecisionPayloadError::from)?;
  let amount = Money::from_major_str(&amount_text, currency)
    .map_err(|err| DecisionPayloadError::Invalid(format!("invalid event payload: {err}")))?;

  money.remove("value");
  money.insert("minor_units".to_owned(), json!(amount.minor_units()));

  Ok((request, true))
}

#[derive(Debug, Error)]
enum DecisionPayloadError {
  #[error("{0}")]
  Invalid(String),
  #[error(transparent)]
  Domain(#[from] rve_core::domain::DomainError),
}

fn map_decision_error(error: DecisionServiceError) -> (StatusCode, Json<ErrorResponse>) {
  let (status, code) = match &error {
    DecisionServiceError::Runtime(RuntimeEngineError::Configuration { .. }) => {
      (StatusCode::SERVICE_UNAVAILABLE, "runtime_configuration")
    }
    DecisionServiceError::Runtime(RuntimeEngineError::Compilation { .. }) => {
      (StatusCode::INTERNAL_SERVER_ERROR, "runtime_compilation")
    }
    DecisionServiceError::Runtime(RuntimeEngineError::Evaluation { .. }) => {
      (StatusCode::INTERNAL_SERVER_ERROR, "runtime_evaluation")
    }
    DecisionServiceError::Runtime(RuntimeEngineError::NotImplemented { .. }) => {
      (StatusCode::NOT_IMPLEMENTED, "not_implemented")
    }
    DecisionServiceError::Runtime(RuntimeEngineError::Internal { .. }) => {
      (StatusCode::INTERNAL_SERVER_ERROR, "runtime_internal")
    }
    DecisionServiceError::Repository(_) => (StatusCode::INTERNAL_SERVER_ERROR, "repository_error"),
  };

  (
    status,
    Json(ErrorResponse { code: code.to_owned(), message: error.to_string(), validation: None }),
  )
}
