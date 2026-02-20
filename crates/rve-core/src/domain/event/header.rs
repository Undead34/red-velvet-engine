use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::common::{Channel, EventId, EventSource, Instrument};

/// Transport and identity metadata for an event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
  pub timestamp: DateTime<Utc>,
  pub source: EventSource,
  pub event_id: Option<EventId>,
  pub instrument: Option<Instrument>,
  pub channel: Option<Channel>,
}
