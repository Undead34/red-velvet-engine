pub mod context;
pub mod signals;

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::{
  common::{BankRef, CountryCode, EntityType, Flag, KycLevel, Money},
  event::{context::Context, signals::Signals},
};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Event {
  pub header: Header,
  pub context: Context,
  pub signals: Signals,
  pub payload: Payload,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Header {
  pub timestamp: DateTime<Utc>,
  pub source: String,
  pub event_id: Option<String>,
  pub instrument: Option<String>,
  pub channel: Option<String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Payload {
  pub money: Money,
  pub parties: Parties,
  pub extensions: BTreeMap<String, Value>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Parties {
  pub originator: Party,
  pub beneficiary: Party,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Party {
  pub entity_type: EntityType,
  pub acct: String,
  pub country: Option<CountryCode>,
  pub bank: Option<BankRef>,
  pub kyc: Option<KycLevel>,
  pub watchlist: Flag,
  pub sanctions_score: Option<f32>,
}
