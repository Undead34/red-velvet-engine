//! Mapping between decision HTTP DTOs and domain events.

use rve_core::domain::{
  common::{Amount, Currency, Money},
  event::{Event, Payload},
};
use serde_json::Value;

use super::{
  dto::request::{DecisionPayloadRequest, DecisionRequest, MoneyRequest, ValueTransferRequest},
  errors::DecisionPayloadError,
};

/// Parses an incoming decision request body using the canonical event contract.
pub fn event_from_request(request: Value) -> Result<Event, DecisionPayloadError> {
  let request = serde_json::from_value::<DecisionRequest>(request)
    .map_err(|err| DecisionPayloadError::Invalid(format!("invalid event payload: {err}")))?;

  map_decision_request(request)
}

fn map_decision_request(request: DecisionRequest) -> Result<Event, DecisionPayloadError> {
  let payload = map_payload(request.payload)?;
  Event::try_new(request.header, request.context, request.features, request.signals, payload)
    .map_err(|err| DecisionPayloadError::Invalid(format!("invalid event payload: {err}")))
}

fn map_payload(payload: DecisionPayloadRequest) -> Result<Payload, DecisionPayloadError> {
  match payload {
    DecisionPayloadRequest::ValueTransfer(payload) => map_value_transfer(payload),
  }
}

fn map_value_transfer(payload: ValueTransferRequest) -> Result<Payload, DecisionPayloadError> {
  let money = map_money(payload.money)?;
  Payload::try_value_transfer(money, payload.parties, payload.extensions)
    .map_err(|err| DecisionPayloadError::Invalid(format!("invalid payload.parties: {err}")))
}

fn map_money(money: MoneyRequest) -> Result<Money, DecisionPayloadError> {
  let currency = Currency::new(&money.ccy)
    .map_err(|err| DecisionPayloadError::Invalid(format!("invalid payload.money.ccy: {err}")))?;

  Ok(Money::new(Amount::new(i128::from(money.minor_units)), currency))
}
