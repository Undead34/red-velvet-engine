pub mod context;
pub mod signals;

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::{
  common::{
    AccountId, BankRef, Channel, CountryCode, EntityType, EventId, EventSource, Flag, Instrument,
    KycLevel, Money,
  },
  event::{context::Context, signals::Signals},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
  pub header: Header,
  pub context: Context,
  pub signals: Signals,
  pub payload: Payload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
  pub timestamp: DateTime<Utc>,
  pub source: EventSource,
  pub event_id: Option<EventId>,
  pub instrument: Option<Instrument>,
  pub channel: Option<Channel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payload {
  pub money: Money,
  pub parties: Parties,
  pub extensions: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parties {
  pub originator: Party,
  pub beneficiary: Party,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Party {
  pub entity_type: EntityType,
  pub acct: AccountId,
  pub country: Option<CountryCode>,
  pub bank: Option<BankRef>,
  pub kyc: Option<KycLevel>,
  pub watchlist: Flag,
  pub sanctions_score: Option<f32>,
}
