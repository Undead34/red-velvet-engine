//! Event domain model used by the decision engine.
//!
//! The aggregate is split into:
//! - [`Event`]: validated input passed to the engine.
//! - [`Context`]: request-time snapshot (`geo`, `net`, `env`).
//! - [`Features`]: historical and derived counters used by fraud rules.
//! - [`Payload`]: business data and custom extensions.
//!
//! This module defines domain-side validation boundaries. Adapters are expected
//! to map transport payloads into these types before evaluation.

pub mod context;
mod error;
mod event;
mod features;
mod header;
mod parties;
mod party;
mod payload;
pub mod signals;

pub use context::{Context, EnvironmentContext, GeoContext, NetworkContext};
pub use error::{EventError, EventFeaturesError, EventGeoError, EventPartyError};
pub use event::Event;
pub use features::{Features, FinancialFeatures};
pub use header::Header;
pub use parties::Parties;
pub use party::Party;
pub use payload::Payload;
pub use signals::Signals;
