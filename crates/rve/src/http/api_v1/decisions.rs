use std::{collections::BTreeMap, collections::HashSet, net::IpAddr};

use axum::{Json, extract::State};
use chrono::{DateTime, Utc};
use rve_core::{
  domain::{
    DomainError,
    common::{
      AccountId, BankRef, Channel, CountryCode, Currency, DeviceId, EventId, EventSource,
      Instrument, KycLevel, LocaleTag, SessionId, TimezoneName, UserAgent,
    },
    event::{
      Context, EnvironmentContext, Event, EventError, EventFeaturesError, EventGeoError,
      EventPartyError, Features, FinancialFeatures, GeoContext, Header, NetworkContext, Parties,
      Party, Payload, Signals, signals::Signal,
    },
  },
  services::engine::{Decision, DecisionService},
};
use serde::Deserialize;
use serde_json::Value;

use crate::http::state::AppState;

use super::rules::errors::{ApiError, ApiResult};

#[utoipa::path(
  post,
  path = "/api/v1/decisions",
  tag = "decisions",
  request_body(
    content = crate::http::openapi::DecisionRequestDoc,
    description = "Direct EventInput body (no `event` wrapper). `header.event_id` is optional but must be UUID when present. `features.fin` is currently strict (full object required). `payload.parties.originator` and `payload.parties.beneficiary` are required."
  ),
  responses(
    (status = 200, description = "Decision evaluated. If no rules match, response is `outcome=allow` with empty `hits`.", body = crate::http::openapi::DecisionResponseDoc),
    (status = 422, description = "Invalid event payload", body = crate::http::openapi::ErrorResponse),
    (status = 500, description = "Decision engine evaluation failed", body = crate::http::openapi::ErrorResponse)
  )
)]
pub async fn create_decision(
  State(state): State<AppState>,
  Json(request): Json<EventInput>,
) -> ApiResult<Json<Decision>> {
  let event = request.into_domain()?;
  let decision = DecisionService::decide(state.engine.as_ref(), &event)
    .await
    .map_err(|error| ApiError::Internal(format!("decision engine error: {error}")))?;

  Ok(Json(decision))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventInput {
  pub header: HeaderInput,
  pub context: ContextInput,
  pub features: FeaturesInput,
  pub signals: SignalsInput,
  pub payload: PayloadInput,
}

impl EventInput {
  fn into_domain(self) -> ApiResult<Event> {
    Event::new(
      self.header.into_domain()?,
      self.context.into_domain()?,
      self.features.into_domain(),
      self.signals.into_domain(),
      self.payload.into_domain()?,
    )
    .map_err(map_event_domain_error)
  }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeaderInput {
  pub timestamp: DateTime<Utc>,
  pub source: String,
  pub event_id: Option<String>,
  pub instrument: Option<String>,
  pub channel: Option<String>,
}

impl HeaderInput {
  fn into_domain(self) -> ApiResult<Header> {
    Ok(Header {
      timestamp: self.timestamp,
      source: EventSource::new(self.source)
        .map_err(|error| map_domain_error("header.source", error))?,
      event_id: self
        .event_id
        .map(EventId::try_from)
        .transpose()
        .map_err(|error| map_domain_error("header.event_id", error))?,
      instrument: self
        .instrument
        .map(Instrument::new)
        .transpose()
        .map_err(|error| map_domain_error("header.instrument", error))?,
      channel: self
        .channel
        .map(Channel::new)
        .transpose()
        .map_err(|error| map_domain_error("header.channel", error))?,
    })
  }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextInput {
  pub geo: GeoContextInput,
  pub net: NetworkContextInput,
  pub env: EnvironmentContextInput,
}

impl ContextInput {
  fn into_domain(self) -> ApiResult<Context> {
    Ok(Context {
      geo: self.geo.into_domain()?,
      net: self.net.into_domain(),
      env: self.env.into_domain()?,
    })
  }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeoContextInput {
  pub address: Option<String>,
  pub city: Option<String>,
  pub region: Option<String>,
  pub country: Option<String>,
  pub postal_code: Option<String>,
  pub lon: Option<f64>,
  pub lat: Option<f64>,
}

impl GeoContextInput {
  fn into_domain(self) -> ApiResult<GeoContext> {
    Ok(GeoContext {
      address: self.address,
      city: self.city,
      region: self.region,
      country: self
        .country
        .map(CountryCode::new)
        .transpose()
        .map_err(|error| map_domain_error("context.geo.country", error))?,
      postal_code: self.postal_code,
      lon: self.lon,
      lat: self.lat,
    })
  }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkContextInput {
  pub source_ip: Option<IpAddr>,
  pub destination_ip: Option<IpAddr>,
  pub hop_count: Option<u8>,
  pub asn: Option<u32>,
  pub isp: Option<String>,
}

impl NetworkContextInput {
  fn into_domain(self) -> NetworkContext {
    NetworkContext {
      source_ip: self.source_ip,
      destination_ip: self.destination_ip,
      hop_count: self.hop_count,
      asn: self.asn,
      isp: self.isp,
    }
  }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentContextInput {
  pub user_agent: Option<String>,
  pub locale: Option<String>,
  pub timezone: Option<String>,
  pub device_id: Option<String>,
  pub session_id: Option<String>,
}

impl EnvironmentContextInput {
  fn into_domain(self) -> ApiResult<EnvironmentContext> {
    Ok(EnvironmentContext {
      user_agent: self
        .user_agent
        .map(UserAgent::new)
        .transpose()
        .map_err(|error| map_domain_error("context.env.user_agent", error))?,
      locale: self
        .locale
        .map(LocaleTag::new)
        .transpose()
        .map_err(|error| map_domain_error("context.env.locale", error))?,
      timezone: self
        .timezone
        .map(TimezoneName::new)
        .transpose()
        .map_err(|error| map_domain_error("context.env.timezone", error))?,
      device_id: self
        .device_id
        .map(DeviceId::new)
        .transpose()
        .map_err(|error| map_domain_error("context.env.device_id", error))?,
      session_id: self
        .session_id
        .map(SessionId::new)
        .transpose()
        .map_err(|error| map_domain_error("context.env.session_id", error))?,
    })
  }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeaturesInput {
  pub fin: FinancialFeaturesInput,
}

impl FeaturesInput {
  fn into_domain(self) -> Features {
    Features { fin: self.fin.into_domain() }
  }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FinancialFeaturesInput {
  pub first_seen_at: u64,
  pub last_seen_at: u64,
  pub last_declined_at: Option<u64>,
  pub total_successful_txns: u64,
  pub total_declined_txns: u64,
  pub total_amount_spent: u64,
  pub max_ticket_ever: u64,
  pub consecutive_failed_logins: u32,
  pub consecutive_declines: u32,
  pub current_hour_count: u32,
  pub current_hour_amount: u64,
  pub current_day_count: u32,
  pub current_day_amount: u64,
  #[serde(default)]
  pub known_ips: HashSet<String>,
  #[serde(default)]
  pub known_devices: HashSet<String>,
}

impl FinancialFeaturesInput {
  fn into_domain(self) -> FinancialFeatures {
    FinancialFeatures {
      first_seen_at: self.first_seen_at,
      last_seen_at: self.last_seen_at,
      last_declined_at: self.last_declined_at,
      total_successful_txns: self.total_successful_txns,
      total_declined_txns: self.total_declined_txns,
      total_amount_spent: self.total_amount_spent,
      max_ticket_ever: self.max_ticket_ever,
      consecutive_failed_logins: self.consecutive_failed_logins,
      consecutive_declines: self.consecutive_declines,
      current_hour_count: self.current_hour_count,
      current_hour_amount: self.current_hour_amount,
      current_day_count: self.current_day_count,
      current_day_amount: self.current_day_amount,
      known_ips: self.known_ips,
      known_devices: self.known_devices,
    }
  }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignalsInput {
  #[serde(default)]
  pub flags: BTreeMap<Signal, rve_core::domain::common::Flag>,
}

impl SignalsInput {
  fn into_domain(self) -> Signals {
    Signals { flags: self.flags }
  }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PayloadInput {
  pub money: MoneyInput,
  pub parties: PartiesInput,
  #[serde(default)]
  pub extensions: BTreeMap<String, Value>,
}

impl PayloadInput {
  fn into_domain(self) -> ApiResult<Payload> {
    Ok(Payload {
      money: self.money.into_domain()?,
      parties: self.parties.into_domain()?,
      extensions: self.extensions,
    })
  }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MoneyInput {
  pub value: f64,
  pub ccy: String,
}

impl MoneyInput {
  fn into_domain(self) -> ApiResult<rve_core::domain::common::Money> {
    let ccy =
      Currency::new(self.ccy).map_err(|error| map_domain_error("payload.money.ccy", error))?;
    rve_core::domain::common::Money::new(self.value, ccy)
      .map_err(|error| map_domain_error("payload.money.value", error))
  }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartiesInput {
  pub originator: PartyInput,
  pub beneficiary: PartyInput,
}

impl PartiesInput {
  fn into_domain(self) -> ApiResult<Parties> {
    Ok(Parties {
      originator: self.originator.into_domain("payload.parties.originator")?,
      beneficiary: self.beneficiary.into_domain("payload.parties.beneficiary")?,
    })
  }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartyInput {
  pub entity_type: rve_core::domain::common::EntityType,
  pub acct: String,
  pub country: Option<String>,
  pub bank: Option<String>,
  pub kyc: Option<String>,
  pub watchlist: rve_core::domain::common::Flag,
  pub sanctions_score: Option<f32>,
}

impl PartyInput {
  fn into_domain(self, base_path: &str) -> ApiResult<Party> {
    Party::new(
      self.entity_type,
      AccountId::new(self.acct)
        .map_err(|error| map_domain_error(&format!("{base_path}.acct"), error))?,
      self
        .country
        .map(CountryCode::new)
        .transpose()
        .map_err(|error| map_domain_error(&format!("{base_path}.country"), error))?,
      self
        .bank
        .map(BankRef::new)
        .transpose()
        .map_err(|error| map_domain_error(&format!("{base_path}.bank"), error))?,
      self
        .kyc
        .map(KycLevel::new)
        .transpose()
        .map_err(|error| map_domain_error(&format!("{base_path}.kyc"), error))?,
      self.watchlist,
      self.sanctions_score,
    )
    .map_err(|error| map_party_error(base_path, error))
  }
}

fn map_domain_error(field: &str, error: DomainError) -> ApiError {
  ApiError::validation(field, error.to_string())
}

fn map_event_domain_error(error: DomainError) -> ApiError {
  match error {
    DomainError::Event(EventError::Geo(EventGeoError::InvalidLatitude { .. })) => {
      ApiError::validation("context.geo.lat", error.to_string())
    }
    DomainError::Event(EventError::Geo(EventGeoError::InvalidLongitude { .. })) => {
      ApiError::validation("context.geo.lon", error.to_string())
    }
    DomainError::Event(EventError::Features(EventFeaturesError::InvalidSeenChronology {
      ..
    })) => ApiError::validation("features.fin.last_seen_at", error.to_string()),
    DomainError::Event(EventError::Features(
      EventFeaturesError::InvalidLastDeclinedChronology { .. },
    )) => ApiError::validation("features.fin.last_declined_at", error.to_string()),
    DomainError::Event(EventError::Party(EventPartyError::InvalidSanctionsScore { .. })) => {
      ApiError::validation("payload.parties.*.sanctions_score", error.to_string())
    }
    _ => ApiError::validation("event", error.to_string()),
  }
}

fn map_party_error(base_path: &str, error: EventPartyError) -> ApiError {
  match error {
    EventPartyError::InvalidSanctionsScore { .. } => {
      ApiError::validation(format!("{base_path}.sanctions_score"), error.to_string())
    }
  }
}

#[cfg(test)]
mod tests {
  use serde_json::json;

  use super::EventInput;
  use crate::http::api_v1::rules::errors::ApiError;

  fn valid_event_payload() -> serde_json::Value {
    json!({
      "header": {
        "timestamp": "2026-03-01T00:00:00Z",
        "source": "api_gateway",
        "event_id": "0195d80e-4f96-7a4b-a8e0-3c5a3f0e7b21",
        "instrument": "card",
        "channel": "web"
      },
      "context": {
        "geo": {
          "country": "US",
          "lon": -74.0,
          "lat": 40.7
        },
        "net": {
          "source_ip": "1.1.1.1"
        },
        "env": {
          "locale": "en-US",
          "timezone": "UTC",
          "device_id": "dev_001",
          "session_id": "sess_001"
        }
      },
      "features": {
        "fin": {
          "first_seen_at": 1730000000000u64,
          "last_seen_at": 1730000001000u64,
          "last_declined_at": 1730000000500u64,
          "total_successful_txns": 10u64,
          "total_declined_txns": 1u64,
          "total_amount_spent": 500000u64,
          "max_ticket_ever": 120000u64,
          "consecutive_failed_logins": 0,
          "consecutive_declines": 0,
          "current_hour_count": 1,
          "current_hour_amount": 1000u64,
          "current_day_count": 2,
          "current_day_amount": 2000u64,
          "known_ips": ["1.1.1.1"],
          "known_devices": ["dev_001"]
        }
      },
      "signals": {
        "flags": {
          "vpn": "no"
        }
      },
      "payload": {
        "money": {
          "value": 100.5,
          "ccy": "USD"
        },
        "parties": {
          "originator": {
            "entity_type": "individual",
            "acct": "acct_001",
            "country": "US",
            "watchlist": "no"
          },
          "beneficiary": {
            "entity_type": "business",
            "acct": "acct_002",
            "country": "US",
            "watchlist": "unknown"
          }
        },
        "extensions": {}
      }
    })
  }

  #[test]
  fn accepts_direct_event_body() {
    let payload = valid_event_payload();
    let parsed: EventInput = serde_json::from_value(payload).expect("payload parses");
    let result = parsed.into_domain();
    assert!(result.is_ok(), "expected valid direct body: {result:?}");
  }

  #[test]
  fn rejects_old_wrapper_shape() {
    let payload = json!({
      "event": valid_event_payload()
    });

    let parsed = serde_json::from_value::<EventInput>(payload);
    assert!(parsed.is_err(), "old wrapper must not deserialize into EventInput");
  }

  #[test]
  fn rejects_invalid_geo_latitude_with_422_mapping() {
    let mut payload = valid_event_payload();
    payload["context"]["geo"]["lat"] = json!(123.0);

    let parsed: EventInput = serde_json::from_value(payload).expect("payload parses");
    let result = parsed.into_domain();

    match result {
      Err(ApiError::Unprocessable(report)) => {
        assert!(!report.errors.is_empty());
        assert_eq!(report.errors[0].path, "context.geo.lat");
      }
      _ => panic!("expected unprocessable error"),
    }
  }
}
