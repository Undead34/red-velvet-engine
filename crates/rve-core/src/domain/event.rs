use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

use crate::domain::{
    context::Context,
    signals::Signals,
    types::{BankRef, CountryCode, EntityType, Flag, KycLevel, Money},
};

#[derive(Default, Debug, Clone, Serialize)]
pub struct Event {
    pub header: Header,
    pub context: Context,
    pub signals: Signals,
    pub payload: Payload,
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct Header {
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub event_id: Option<String>,
    pub instrument: Option<String>,
    pub channel: Option<String>,
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct Payload {
    pub money: Money,
    pub parties: Parties,
    pub extensions: BTreeMap<String, Value>,
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct Parties {
    pub originator: Party,
    pub beneficiary: Party,
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct Party {
    pub entity_type: EntityType,
    pub acct: String,
    pub country: Option<CountryCode>,
    pub bank: Option<BankRef>,
    pub kyc: Option<KycLevel>,
    pub watchlist: Flag,
    pub sanctions_score: Option<f32>,
}
