//! HTTP data transfer objects for the rules API.
//!
//! DTOs define the public wire contract at the adapter boundary. They are kept
//! separate from domain aggregates so handlers can validate transport payloads,
//! map them explicitly, and return stable API responses without exposing
//! persistence or domain internals.

pub mod request;
pub mod response;

pub use request::{Pagination, RuleDocumentRequest, RulePatchRequest};
pub use response::{RuleListResponse, RuleResponse};
