use std::collections::BTreeMap;

use rve_core::domain::event::{Context, Features, Header, Parties, Signals};
use serde::Deserialize;
use serde_json::Value;

/// Canonical decision request accepted by `POST /api/v1/decisions`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionRequest {
  pub header: Header,
  pub context: Context,
  pub features: Features,
  pub signals: Signals,
  pub payload: DecisionPayloadRequest,
}

/// Canonical business payload accepted by decision endpoints.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum DecisionPayloadRequest {
  ValueTransfer(ValueTransferRequest),
}

/// Value-transfer payload accepted by decision endpoints.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValueTransferRequest {
  pub money: MoneyRequest,
  pub parties: Parties,
  #[serde(default)]
  pub extensions: BTreeMap<String, Value>,
}

/// Money payload accepted by the public API.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MoneyRequest {
  pub minor_units: i64,
  pub ccy: String,
}
