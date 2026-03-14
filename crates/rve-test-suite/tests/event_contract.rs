use std::collections::{BTreeMap, HashSet};

use chrono::Utc;
use rve_core::domain::{
  DomainError,
  common::{
    AccountId, Currency, DeviceId, EventSource, Flag, LocaleTag, SessionId, Severity, TimezoneName,
    UserAgent,
  },
  event::{
    Context, EnvironmentContext, Event, EventError, EventFeaturesError, EventGeoError,
    EventPartyError, Features, FinancialFeatures, GeoContext, Header, NetworkContext, Parties,
    Party, Payload, Signals, signals::Signal,
  },
};

fn valid_event() -> Event {
  let header = Header {
    timestamp: Utc::now(),
    source: EventSource::new("api_gateway").unwrap(),
    event_id: None,
    instrument: None,
    channel: None,
  };

  let context = Context {
    geo: GeoContext {
      address: None,
      city: Some("NYC".to_owned()),
      region: Some("NY".to_owned()),
      country: None,
      postal_code: None,
      lon: Some(-74.0),
      lat: Some(40.7),
    },
    net: NetworkContext {
      source_ip: None,
      destination_ip: None,
      hop_count: None,
      asn: None,
      isp: None,
    },
    env: EnvironmentContext {
      user_agent: Some(UserAgent::new("Mozilla/5.0").unwrap()),
      locale: Some(LocaleTag::new("en-US").unwrap()),
      timezone: Some(TimezoneName::new("UTC").unwrap()),
      device_id: Some(DeviceId::new("dev_001").unwrap()),
      session_id: Some(SessionId::new("sess_001").unwrap()),
    },
  };

  let features = Features {
    fin: FinancialFeatures {
      first_seen_at: 1_730_000_000_000,
      last_seen_at: 1_730_000_000_100,
      last_declined_at: Some(1_730_000_000_050),
      total_successful_txns: 10,
      total_declined_txns: 1,
      total_amount_spent: 1_000_000,
      max_ticket_ever: 100_000,
      consecutive_failed_logins: 0,
      consecutive_declines: 0,
      current_hour_count: 1,
      current_hour_amount: 500,
      current_day_count: 2,
      current_day_amount: 700,
      known_ips: HashSet::from([String::from("1.1.1.1")]),
      known_devices: HashSet::from([String::from("dev_001")]),
    },
  };

  let signals = Signals { flags: BTreeMap::from([(Signal::Vpn, Flag::No)]) };

  let payload = Payload {
    money: rve_core::domain::common::Money::from_major_str("100.5", Currency::new("USD").unwrap())
      .unwrap(),
    parties: Parties {
      originator: Party::new(
        rve_core::domain::common::EntityType::Individual,
        AccountId::new("acct_001").unwrap(),
        None,
        None,
        None,
        Flag::No,
        Some(0.2),
      )
      .unwrap(),
      beneficiary: Party::new(
        rve_core::domain::common::EntityType::Business,
        AccountId::new("acct_002").unwrap(),
        None,
        None,
        None,
        Flag::Unknown,
        Some(0.5),
      )
      .unwrap(),
    },
    extensions: BTreeMap::new(),
  };

  Event::new(header, context, features, signals, payload).unwrap()
}

#[test]
fn creates_valid_event() {
  let event = valid_event();
  assert_eq!(event.payload.money.ccy().as_str(), "USD");
}

#[test]
fn rejects_invalid_geo_ranges() {
  let mut event = valid_event();
  event.context.geo.lat = Some(200.0);
  let result = event.validate();
  assert!(matches!(
    result,
    Err(DomainError::Event(EventError::Geo(EventGeoError::InvalidLatitude { .. })))
  ));
}

#[test]
fn rejects_invalid_feature_chronology() {
  let mut event = valid_event();
  event.features.fin.first_seen_at = 200;
  event.features.fin.last_seen_at = 100;
  let result = event.validate();
  assert!(matches!(
    result,
    Err(DomainError::Event(EventError::Features(EventFeaturesError::InvalidSeenChronology { .. })))
  ));
}

#[test]
fn rejects_invalid_sanctions_score() {
  let mut event = valid_event();
  event.payload.parties.originator.sanctions_score = Some(1.5);
  let result = event.validate();
  assert!(matches!(
    result,
    Err(DomainError::Event(EventError::Party(EventPartyError::InvalidSanctionsScore { .. })))
  ));
}

#[test]
fn decision_types_still_compile_with_event_changes() {
  let _ = Severity::High;
  let _ = valid_event();
}
