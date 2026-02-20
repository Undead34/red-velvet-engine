use serde::{Deserialize, Serialize};

use super::{Header, Payload, context::Context, signals::Signals};

/// Full decision input event consumed by the engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
  pub header: Header,
  pub context: Context,
  pub signals: Signals,
  pub payload: Payload,
}
