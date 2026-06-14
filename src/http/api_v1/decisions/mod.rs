mod errors;
mod evaluate;
mod payload;

use axum::{
  Json,
  extract::State,
  http::{HeaderMap, StatusCode},
};
use tracing::instrument;

use rve_core::services::engine::{Decision, DecisionTrace};

use crate::http::openapi::ErrorResponse;
use crate::http::state::AppState;

use evaluate::{DecisionEnvelope, evaluate_request};

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
  Json(request): Json<serde_json::Value>,
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
  Json(request): Json<serde_json::Value>,
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
