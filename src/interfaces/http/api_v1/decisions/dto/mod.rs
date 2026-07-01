//! HTTP data transfer objects for decision evaluation.
//!
//! The request DTO preserves the public API contract and maps into the event
//! domain model at the adapter boundary. Response DTOs keep application return
//! types from becoming the serialized API surface.

pub mod request;
pub mod response;

pub use response::{DecisionResponse, DecisionTraceResponse};
