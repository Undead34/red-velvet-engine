use axum::{
  Json,
  extract::State,
  http::StatusCode,
};
use rve_core::{
  domain::{
    common::{Currency, Money},
    event::Event,
  },
  ports::RuntimeEngineError,
  services::engine::{Decision, DecisionService, DecisionServiceError},
};
use serde_json::{Value, json};

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
pub async fn create_decision(
  State(state): State<AppState>,
  Json(request): Json<Value>,
) -> Result<Json<Decision>, (StatusCode, Json<ErrorResponse>)> {
  let event = parse_event_payload(request).map_err(|message| {
    (
      StatusCode::UNPROCESSABLE_ENTITY,
      Json(ErrorResponse {
        code: "unprocessable_entity".to_owned(),
        message,
        validation: None,
      }),
    )
  })?;

  let decision = match DecisionService::decide(state.engine.as_ref(), &event).await {
    Ok(decision) => decision,
    Err(DecisionServiceError::Runtime(RuntimeEngineError::Configuration { .. })) => {
      DecisionService::reload_rules(state.rule_repo.as_ref(), state.engine.as_ref())
        .await
        .map_err(map_decision_error)?;
      DecisionService::decide(state.engine.as_ref(), &event)
        .await
        .map_err(map_decision_error)?
    }
    Err(error) => return Err(map_decision_error(error)),
  };

  Ok(Json(decision))
}

fn parse_event_payload(request: Value) -> Result<Event, String> {
  if let Ok(event) = serde_json::from_value::<Event>(request.clone()) {
    return Ok(event);
  }

  let normalized = normalize_legacy_money_payload(request)?;
  serde_json::from_value::<Event>(normalized)
    .map_err(|err| format!("invalid event payload: {err}"))
}

fn normalize_legacy_money_payload(mut request: Value) -> Result<Value, String> {
  let Some(payload) = request.get_mut("payload").and_then(Value::as_object_mut) else {
    return Ok(request);
  };

  if payload.get("type").is_none() {
    payload.insert("type".to_owned(), json!("value_transfer"));
  }

  let Some(money) = payload
    .get_mut("money")
    .and_then(Value::as_object_mut)
  else {
    return Ok(request);
  };

  if money.get("minor_units").is_some() {
    return Ok(request);
  }

  let Some(ccy) = money.get("ccy").and_then(Value::as_str) else {
    return Ok(request);
  };
  let Some(raw_value) = money.get("value") else {
    return Ok(request);
  };

  let amount_text = match raw_value {
    Value::String(text) => text.clone(),
    Value::Number(number) => number.to_string(),
    _ => return Ok(request),
  };

  let currency = Currency::new(ccy).map_err(|err| format!("invalid event payload: {err}"))?;
  let amount =
    Money::from_major_str(&amount_text, currency).map_err(|err| format!("invalid event payload: {err}"))?;

  money.remove("value");
  money.insert("minor_units".to_owned(), json!(amount.minor_units()));

  Ok(request)
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
