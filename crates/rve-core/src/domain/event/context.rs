use std::net::IpAddr;

use serde::{Deserialize, Serialize};

use crate::domain::common::{CountryCode, DeviceId, LocaleTag, SessionId, TimezoneName, UserAgent};

use super::error::EventGeoError;

/// Snapshot context attached to an event.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Context {
  pub geo: GeoContext,
  pub net: NetworkContext,
  pub env: EnvironmentContext,
}

/// Geographic snapshot.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct GeoContext {
  pub address: Option<String>,
  pub city: Option<String>,
  pub region: Option<String>,
  pub country: Option<CountryCode>,
  pub postal_code: Option<String>,
  pub lon: Option<f64>,
  pub lat: Option<f64>,
}

impl GeoContext {
  pub fn validate(&self) -> Result<(), EventGeoError> {
    if let Some(lat) = self.lat
      && (!lat.is_finite() || !(-90.0..=90.0).contains(&lat))
    {
      return Err(EventGeoError::InvalidLatitude { value: lat.to_string() });
    }

    if let Some(lon) = self.lon
      && (!lon.is_finite() || !(-180.0..=180.0).contains(&lon))
    {
      return Err(EventGeoError::InvalidLongitude { value: lon.to_string() });
    }

    Ok(())
  }
}

/// Network snapshot.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct NetworkContext {
  pub source_ip: Option<IpAddr>,
  pub destination_ip: Option<IpAddr>,
  pub hop_count: Option<u8>,
  pub asn: Option<u32>,
  pub isp: Option<String>,
}

/// Device and runtime snapshot.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentContext {
  pub user_agent: Option<UserAgent>,
  pub locale: Option<LocaleTag>,
  pub timezone: Option<TimezoneName>,
  pub device_id: Option<DeviceId>,
  pub session_id: Option<SessionId>,
}
