//! Outbound adapters used by the runtime crate.
//!
//! Infrastructure modules implement the outbound ports declared in `rve-core`
//! and remain outside the application core.

pub mod persistence;
pub mod runtime;
