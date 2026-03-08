use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::common::{Channel, EventId, EventSource, Instrument};

/// Transport and identity metadata for an event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
  /// Event timestamp as provided by the producer.
  pub timestamp: DateTime<Utc>,
  /// Source system identifier.
  pub source: EventSource,
  /// Optional event identifier.
  pub event_id: Option<EventId>,
  /// Optional payment instrument.
  pub instrument: Option<Instrument>,
  /// Optional entry channel.
  pub channel: Option<Channel>,
}
