use rve_core::domain::{
  common::{Currency, Money},
  event::Event,
};
use serde_json::{Value, json};

use super::errors::DecisionPayloadError;

/// Wrapper around a parsed event carrying a flag indicating
/// whether any legacy field was normalised during parsing.
pub struct ParsedEventPayload {
  pub event: Event,
  pub used_legacy_payload_alias: bool,
}

/// Parse an incoming decision request body, applying legacy
/// normalisation when the payload uses the deprecated
/// `payload.money.value` field instead of `payload.money.minor_units`.
pub fn parse_event_payload(request: Value) -> Result<ParsedEventPayload, DecisionPayloadError> {
  if let Ok(event) = serde_json::from_value::<Event>(request.clone()) {
    return Ok(ParsedEventPayload { event, used_legacy_payload_alias: false });
  }

  let (normalized, used_legacy_payload_alias) = normalize_legacy_money_payload(request)?;
  serde_json::from_value::<Event>(normalized)
    .map(|event| ParsedEventPayload { event, used_legacy_payload_alias })
    .map_err(|err| DecisionPayloadError::Invalid(format!("invalid event payload: {err}")))
}

/// Normalise a request body that still uses the deprecated
/// `payload.money.value` (string | number) into the canonical
/// `payload.money.minor_units` (integer) representation.
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

  let currency = Currency::new(ccy).map_err(|error| DecisionPayloadError::Domain(error.into()))?;
  let amount = Money::parse(&amount_text, currency)
    .map_err(|err| DecisionPayloadError::Invalid(format!("invalid event payload: {err}")))?;

  money.remove("value");
  money.insert("minor_units".to_owned(), json!(amount.amount().units()));

  Ok((request, true))
}
